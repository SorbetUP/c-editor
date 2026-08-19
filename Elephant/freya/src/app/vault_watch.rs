//! Native filesystem watch boundary for Freya.
//!
//! The previous implementation fingerprinted the current directory every
//! 500 ms. That performed repeated `read_dir + metadata + hash` work while the
//! vault was idle and still missed useful recursive events. This bridge uses
//! the same `notify` family as the Tauri backend, watches the active vault
//! recursively, coalesces event bursts, and asks `ShellState` to refresh the
//! visible directory/open note from the real filesystem.

use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

use freya::{prelude::*, sdk::use_timeout};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use super::ShellState;

const UI_DRAIN_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
enum WatchMessage {
    Changed(PathBuf),
    Error(String),
}

#[derive(Default)]
struct WatchBridge {
    root: Option<PathBuf>,
    watcher: Option<RecommendedWatcher>,
    receiver: Option<Receiver<WatchMessage>>,
}

impl WatchBridge {
    fn configure(&mut self, root: Option<&Path>) -> Result<(), String> {
        let root = root
            .map(|root| {
                std::fs::canonicalize(root)
                    .map_err(|error| format!("Vault is unavailable: {error}"))
            })
            .transpose()?;
        if self.root == root {
            return Ok(());
        }

        self.watcher = None;
        self.receiver = None;
        self.root = root.clone();
        let Some(root) = root else {
            return Ok(());
        };

        let (sender, receiver) = mpsc::channel::<WatchMessage>();
        let root_for_callback = root.clone();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            match event {
                Ok(event) => {
                    for path in event.paths {
                        if should_ignore(&root_for_callback, &path) {
                            continue;
                        }
                        let _ = sender.send(WatchMessage::Changed(path));
                    }
                }
                Err(error) => {
                    let _ = sender.send(WatchMessage::Error(error.to_string()));
                }
            }
        })
        .map_err(|error| format!("Create vault watcher: {error}"))?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|error| format!("Watch vault {}: {error}", root.display()))?;
        self.watcher = Some(watcher);
        self.receiver = Some(receiver);
        eprintln!("[freya][vault-watch] action=watch-start root={}", root.display());
        Ok(())
    }

    fn drain(&mut self) -> DrainResult {
        let Some(receiver) = self.receiver.as_ref() else {
            return DrainResult::default();
        };
        let mut result = DrainResult::default();
        loop {
            match receiver.try_recv() {
                Ok(WatchMessage::Changed(path)) => {
                    result.changed = true;
                    result.events = result.events.saturating_add(1);
                    result.last_path = Some(path);
                }
                Ok(WatchMessage::Error(error)) => result.error = Some(error),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        result
    }
}

#[derive(Default)]
struct DrainResult {
    changed: bool,
    events: usize,
    last_path: Option<PathBuf>,
    error: Option<String>,
}

fn should_ignore(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        value.starts_with('.') || value == "node_modules"
    }) || path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with('~') || name.ends_with(".tmp"))
}

#[derive(PartialEq)]
pub(super) struct VaultWatcherHost {
    pub(super) state: State<ShellState>,
}

impl Component for VaultWatcherHost {
    fn render(&self) -> impl IntoElement {
        let root = self
            .state
            .read()
            .vault
            .as_ref()
            .map(|vault| vault.root().to_path_buf());
        let scope = root
            .as_ref()
            .map(|root| root.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bridge = use_state(WatchBridge::default);
        let timeout = use_timeout(|| UI_DRAIN_INTERVAL);

        let mut bridge_for_scope = bridge;
        let root_for_scope = root.clone();
        let mut state_for_scope = self.state;
        use_side_effect_with_deps(&scope, move |_| {
            if let Err(error) = bridge_for_scope.write().configure(root_for_scope.as_deref()) {
                eprintln!("[freya][vault-watch] action=watch-failure error={error}");
                state_for_scope.write().error = Some(error);
            }
        });

        let mut timeout_for_drain = timeout;
        let mut bridge_for_drain = bridge;
        let mut state_for_drain = self.state;
        use_side_effect(move || {
            if !timeout_for_drain.elapsed() {
                return;
            }
            timeout_for_drain.reset();
            let drained = bridge_for_drain.write().drain();
            if let Some(error) = drained.error {
                eprintln!("[freya][vault-watch] action=event-failure error={error}");
                state_for_drain.write().error = Some(format!("Vault watcher failed: {error}"));
            }
            if drained.changed {
                state_for_drain.write().refresh_current_directory_from_external();
                eprintln!(
                    "[freya][vault-watch] action=refresh events={} path={}",
                    drained.events,
                    drained
                        .last_path
                        .as_deref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_default()
                );
            }
        });

        rect()
            .width(Size::px(0.))
            .height(Size::px(0.))
            .interactive(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, thread, time::{SystemTime, UNIX_EPOCH}};

    fn temp_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-watch-{label}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn notify_bridge_observes_real_file_changes_without_directory_polling() {
        let root = temp_root("notify");
        let mut bridge = WatchBridge::default();
        bridge.configure(Some(&root)).unwrap();
        fs::write(root.join("Alpha.md"), "alpha").unwrap();
        let mut observed = false;
        for _ in 0..100 {
            let result = bridge.drain();
            if result.changed {
                observed = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(observed, "notify watcher must observe a physical note write");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hidden_and_temporary_paths_are_filtered() {
        let root = Path::new("/vault");
        assert!(should_ignore(root, Path::new("/vault/.elephantnote/index.sqlite")));
        assert!(should_ignore(root, Path::new("/vault/.assets/image.png")));
        assert!(should_ignore(root, Path::new("/vault/note.md.tmp")));
        assert!(!should_ignore(root, Path::new("/vault/Folder/note.md")));
    }
}