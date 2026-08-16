//! Small polling boundary for external vault changes.
//!
//! The watcher never owns library state or renders a replacement list.  It
//! only produces a deterministic directory fingerprint and asks `ShellState`
//! to reload the current page when the real filesystem changes.

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::Duration,
};

use freya::{prelude::*, sdk::use_timeout};

use super::ShellState;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DirectoryFingerprint {
    pub(super) digest: u64,
    pub(super) entries: usize,
}

pub(super) fn fingerprint(
    root: &Path,
    relative_path: &str,
) -> Result<DirectoryFingerprint, String> {
    let directory = safe_directory(root, relative_path)?;
    let mut records = Vec::new();
    for entry in fs::read_dir(&directory).map_err(|error| {
        format!(
            "Read watched vault directory {}: {error}",
            directory.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("Read watched vault entry: {error}"))?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("Read watched vault metadata: {error}"))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|value| (value.as_secs(), value.subsec_nanos()))
            .unwrap_or_default();
        records.push((
            name.to_string_lossy().into_owned(),
            metadata.is_dir(),
            metadata.len(),
            modified,
        ));
    }
    records.sort();
    let mut hasher = DefaultHasher::new();
    records.hash(&mut hasher);
    Ok(DirectoryFingerprint {
        digest: hasher.finish(),
        entries: records.len(),
    })
}

fn safe_directory(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|error| format!("Vault is unavailable: {error}"))?;
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "Watched directory escapes the vault: {relative_path}"
        ));
    }
    let directory = root.join(relative);
    let canonical = fs::canonicalize(&directory)
        .map_err(|error| format!("Watched directory is unavailable: {error}"))?;
    canonical
        .starts_with(&root)
        .then_some(canonical)
        .ok_or_else(|| format!("Watched directory escapes the vault: {relative_path}"))
}

#[derive(PartialEq)]
pub(super) struct VaultWatcherHost {
    pub(super) state: State<ShellState>,
}

impl Component for VaultWatcherHost {
    fn render(&self) -> impl IntoElement {
        let snapshot = self.state.read().clone();
        let scope = snapshot
            .vault
            .as_ref()
            .map(|vault| {
                format!(
                    "{}:{}",
                    vault.root().display(),
                    snapshot.library.current_path.as_str()
                )
            })
            .unwrap_or_default();
        let root = snapshot
            .vault
            .as_ref()
            .map(|vault| vault.root().to_path_buf());
        let relative_path = snapshot.library.current_path.as_str().to_owned();
        let initial_fingerprint = root
            .as_ref()
            .and_then(|root| fingerprint(root, &relative_path).ok());
        let fingerprint_state = use_state(move || initial_fingerprint);
        let initial_scope = scope.clone();
        let scope_state = use_state(move || initial_scope);
        let timeout = use_timeout(|| POLL_INTERVAL);

        let mut scope_state_for_effect = scope_state;
        let mut fingerprint_for_scope = fingerprint_state;
        let current_scope = scope.clone();
        use_side_effect_with_deps(&scope, move |_| {
            if scope_state_for_effect.peek().clone() != current_scope {
                scope_state_for_effect.set(current_scope.clone());
                fingerprint_for_scope.set(None);
            }
        });

        let mut timeout_for_poll = timeout;
        let mut fingerprint_for_poll = fingerprint_state;
        let mut poll_state = self.state;
        use_side_effect(move || {
            if !timeout_for_poll.elapsed() {
                return;
            }
            timeout_for_poll.reset();
            let Some(root) = root.as_ref() else {
                return;
            };
            let next = match fingerprint(root, &relative_path) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("[freya][vault-watch] action=poll-failure error={error}");
                    return;
                }
            };
            let previous = fingerprint_for_poll.peek().clone();
            fingerprint_for_poll.set(Some(next.clone()));
            if previous.is_some() && previous.as_ref() != Some(&next) {
                poll_state.write().refresh_current_directory_from_external();
                eprintln!(
                    "[freya][vault-watch] action=refresh path={} entries={}",
                    relative_path, next.entries
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
    use std::{fs, time::SystemTime};

    #[test]
    fn fingerprint_changes_for_real_file_addition_and_content_change() {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-watch-{}",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Alpha.md"), "alpha").unwrap();
        let first = fingerprint(&root, "").unwrap();
        fs::write(root.join("Beta.md"), "beta").unwrap();
        let second = fingerprint(&root, "").unwrap();
        assert_ne!(first, second);
        fs::write(root.join("Beta.md"), "changed").unwrap();
        let third = fingerprint(&root, "").unwrap();
        assert_ne!(second, third);
        let _ = fs::remove_dir_all(root);
    }
}
