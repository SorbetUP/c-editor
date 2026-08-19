use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone, Debug)]
pub struct FsEvent {
  pub path: String,
  pub kind: String,
  pub is_directory: bool,
}

const IGNORE_NEXT_TTL: Duration = Duration::from_secs(30);
const IGNORE_NEXT_MAX: usize = 1024;

#[derive(Clone)]
struct IgnoreEntry {
  path: String,
  registered_at: Instant,
}

#[derive(Clone)]
pub struct IgnoreNext {
  entries: Arc<std::sync::Mutex<Vec<IgnoreEntry>>>,
}

impl Default for IgnoreNext {
  fn default() -> Self {
    IgnoreNext {
      entries: Arc::new(std::sync::Mutex::new(Vec::new())),
    }
  }
}

impl IgnoreNext {
  fn register(&self, path: &Path) -> Result<(), String> {
    let mut entries = self.entries.lock().map_err(|error| error.to_string())?;
    let now = Instant::now();
    prune_ignore_entries(&mut entries, now);
    let normalized = normalize_event_path(path);
    entries.retain(|entry| entry.path != normalized);
    entries.push(IgnoreEntry {
      path: normalized,
      registered_at: now,
    });
    if entries.len() > IGNORE_NEXT_MAX {
      entries.remove(0);
    }
    Ok(())
  }

  fn consume(&self, path: &Path) -> Result<bool, String> {
    let mut entries = self.entries.lock().map_err(|error| error.to_string())?;
    let now = Instant::now();
    prune_ignore_entries(&mut entries, now);
    let normalized = normalize_event_path(path);
    if let Some(index) = entries.iter().position(|entry| entry.path == normalized) {
      entries.remove(index);
      return Ok(true);
    }
    Ok(false)
  }

  #[cfg(test)]
  fn len(&self) -> usize {
    self.entries.lock().unwrap().len()
  }
}

fn prune_ignore_entries(entries: &mut Vec<IgnoreEntry>, now: Instant) {
  entries.retain(|entry| now.duration_since(entry.registered_at) < IGNORE_NEXT_TTL);
}

pub struct WatcherState {
  pub watchers: std::sync::Mutex<HashMap<String, RecommendedWatcher>>,
  pub ignore_next: IgnoreNext,
}

impl WatcherState {
  pub fn new() -> Self {
    WatcherState {
      watchers: std::sync::Mutex::new(HashMap::new()),
      ignore_next: IgnoreNext::default(),
    }
  }
}

type R<T> = Result<T, String>;

fn kind_str(kind: &EventKind) -> &'static str {
  match kind {
    EventKind::Create(_) => "create",
    EventKind::Modify(_) => "modify",
    EventKind::Remove(_) => "remove",
    EventKind::Access(_) => "access",
    EventKind::Any => "any",
    _ => "other",
  }
}

pub fn normalize_event_path(path: &Path) -> String {
  let raw = path.to_string_lossy().replace('\\', "/");
  let prefix = if raw.len() >= 2 && raw.as_bytes()[1] == b':' {
    raw[..2].to_string()
  } else if raw.starts_with('/') {
    "/".to_string()
  } else {
    String::new()
  };
  let mut components = Vec::new();
  for component in raw[prefix.len()..].split('/') {
    match component {
      "" | "." => {}
      ".." => {
        if components.last().is_some_and(|last| *last != "..") {
          components.pop();
        } else if prefix.is_empty() {
          components.push("..".to_string());
        }
      }
      value => components.push(value.to_string()),
    }
  }
  let suffix = components.join("/");
  if prefix == "/" {
    format!("/{suffix}")
  } else if prefix.is_empty() {
    suffix
  } else if suffix.is_empty() {
    format!("{prefix}/")
  } else {
    format!("{prefix}/{suffix}")
  }
}

fn is_ignored_path(path: &Path) -> bool {
  let normalized = normalize_event_path(path);
  let components: Vec<_> = normalized.split('/').collect();
  components
    .iter()
    .any(|component| component.starts_with('.') && *component != "." && *component != "..")
    || components
      .iter()
      .any(|component| matches!(*component, ".elephantnote" | ".git" | "node_modules"))
    || normalized.ends_with(".tmp")
}

fn assert_target_inside_root(root: &Path, target: &Path) -> R<PathBuf> {
  let root = std::fs::canonicalize(root)
    .map_err(|error| format!("active vault is unavailable: {error}"))?;
  let target = std::fs::canonicalize(target)
    .map_err(|error| format!("watch target is unavailable: {error}"))?;
  if !target.starts_with(&root) {
    return Err(format!(
      "Refusing to watch a path outside the active vault: {}",
      target.display()
    ));
  }
  Ok(target)
}

fn resolve_watch_target(app: &AppHandle, path: &str) -> R<PathBuf> {
  let vault = crate::vault::config::get_active_vault(app)?;
  assert_target_inside_root(Path::new(&vault.path), Path::new(path))
}

fn watcher_key(window_id: &str, target: &Path) -> String {
  format!("{window_id}:{}", normalize_event_path(target))
}

#[tauri::command]
pub fn tauri_watcher_watch_file(
  app: AppHandle,
  state: State<'_, WatcherState>,
  window_id: String,
  path: String,
) -> R<()> {
  watch_target(&app, &state, &window_id, &path, RecursiveMode::NonRecursive)
}

#[tauri::command]
pub fn tauri_watcher_watch_directory(
  app: AppHandle,
  state: State<'_, WatcherState>,
  window_id: String,
  path: String,
) -> R<()> {
  watch_target(&app, &state, &window_id, &path, RecursiveMode::Recursive)
}

fn watch_target(
  app: &AppHandle,
  state: &State<'_, WatcherState>,
  window_id: &str,
  path: &str,
  mode: RecursiveMode,
) -> R<()> {
  let target = resolve_watch_target(app, path)?;
  let app_handle = app.clone();
  let ignore_next = state.ignore_next.clone();
  let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
  let mut watcher = notify::recommended_watcher(tx).map_err(|e| e.to_string())?;
  watcher.watch(&target, mode).map_err(|e| e.to_string())?;

  let wid = window_id.to_string();
  thread::spawn(move || {
    let mut last_emit: HashMap<String, Instant> = HashMap::new();
    let debounce = Duration::from_millis(300);
    for event in rx {
      match event {
        Ok(ev) => {
          for path in &ev.paths {
            if is_ignored_path(path) {
              continue;
            }
            match ignore_next.consume(path) {
              Ok(true) => continue,
              Ok(false) => {}
              Err(error) => {
                eprintln!("[watcher] ignore-next error: {error}");
              }
            }
            let normalized_path = normalize_event_path(path);
            let now = Instant::now();
            if let Some(prev) = last_emit.get(&normalized_path) {
              if now.duration_since(*prev) < debounce {
                continue;
              }
            }
            last_emit.insert(normalized_path.clone(), now);
            let payload = FsEvent {
              path: normalized_path.clone(),
              kind: kind_str(&ev.kind).to_string(),
              is_directory: path.is_dir(),
            };
            if let Err(error) = app_handle.emit(&format!("elephantnote:fs:changed:{wid}"), payload)
            {
              eprintln!("[watcher] emit error for {normalized_path}: {error}");
            }
          }
        }
        Err(e) => {
          eprintln!("[watcher] error: {e}");
        }
      }
    }
  });

  let mut guard = state.watchers.lock().map_err(|e| e.to_string())?;
  guard.insert(watcher_key(window_id, &target), watcher);
  Ok(())
}

#[tauri::command]
pub fn tauri_watcher_unwatch_file(
  app: AppHandle,
  state: State<'_, WatcherState>,
  window_id: String,
  path: String,
) -> R<()> {
  let target = resolve_watch_target(&app, &path)?;
  remove_watcher(&state, &watcher_key(&window_id, &target))
}

#[tauri::command]
pub fn tauri_watcher_unwatch_directory(
  app: AppHandle,
  state: State<'_, WatcherState>,
  window_id: String,
  path: String,
) -> R<()> {
  let target = resolve_watch_target(&app, &path)?;
  remove_watcher(&state, &watcher_key(&window_id, &target))
}

#[tauri::command]
pub fn tauri_watcher_unwatch_all(state: State<'_, WatcherState>, window_id: String) -> R<()> {
  let mut guard = state.watchers.lock().map_err(|e| e.to_string())?;
  let prefix = format!("{window_id}:");
  let keys: Vec<String> = guard
    .keys()
    .filter(|k| k.starts_with(&prefix))
    .cloned()
    .collect();
  for k in keys {
    guard.remove(&k);
  }
  Ok(())
}

#[tauri::command]
pub fn tauri_watcher_ignore_next(
  app: AppHandle,
  state: State<'_, WatcherState>,
  path: String,
) -> R<()> {
  let target = resolve_watch_target(&app, &path)?;
  state.ignore_next.register(&target)
}

fn remove_watcher(state: &State<'_, WatcherState>, key: &str) -> R<()> {
  let mut guard = state.watchers.lock().map_err(|e| e.to_string())?;
  guard.remove(key);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::PathBuf;

  #[test]
  fn ignored_paths_include_hidden_vault_metadata() {
    assert!(is_ignored_path(&PathBuf::from(
      "/tmp/v/.elephantnote/index/index.json"
    )));
    assert!(is_ignored_path(&PathBuf::from("/tmp/v/.git/HEAD")));
    assert!(is_ignored_path(&PathBuf::from(r"C:\tmp\v\.hidden\note.md")));
  }

  #[test]
  fn visible_files_are_not_ignored() {
    assert!(!is_ignored_path(&PathBuf::from("/tmp/v/note.md")));
    assert!(!is_ignored_path(&PathBuf::from("/tmp/v/sub/note.md")));
  }

  #[test]
  fn temp_files_are_ignored() {
    assert!(is_ignored_path(&PathBuf::from("/tmp/v/note.json.tmp")));
  }

  #[test]
  fn kind_str_maps_event_kinds() {
    assert_eq!(
      kind_str(&EventKind::Create(notify::event::CreateKind::File)),
      "create"
    );
    assert_eq!(
      kind_str(&EventKind::Modify(notify::event::ModifyKind::Any)),
      "modify"
    );
    assert_eq!(
      kind_str(&EventKind::Remove(notify::event::RemoveKind::File)),
      "remove"
    );
  }

  #[test]
  fn fs_event_serializes_to_json() {
    let ev = FsEvent {
      path: "/tmp/note.md".into(),
      kind: "create".into(),
      is_directory: false,
    };
    let json = serde_json::to_value(&ev).unwrap();
    assert_eq!(json.get("kind").and_then(|v| v.as_str()), Some("create"));
  }

  #[test]
  fn ignore_next_holds_paths() {
    let ignore = IgnoreNext::default();
    ignore.register(Path::new("/tmp/a.md")).unwrap();
    assert_eq!(ignore.len(), 1);
  }

  #[test]
  fn fs_event_with_directory_flag() {
    let ev = FsEvent {
      path: "/tmp/dir".into(),
      kind: "modify".into(),
      is_directory: true,
    };
    let json = serde_json::to_value(&ev).unwrap();
    assert_eq!(
      json.get("is_directory").and_then(|v| v.as_bool()),
      Some(true)
    );
  }

  #[test]
  fn event_paths_are_normalized_across_platform_separators() {
    assert_eq!(
      normalize_event_path(Path::new(r"C:\vault\folder\note.md")),
      "C:/vault/folder/note.md"
    );
    assert_eq!(
      normalize_event_path(Path::new("/vault/./folder/../note.md")),
      "/vault/note.md"
    );
  }

  #[test]
  fn ignore_next_is_consumed_once_and_does_not_grow_without_bound() {
    let ignore = IgnoreNext::default();
    ignore.register(Path::new(r"C:\vault\note.md")).unwrap();
    assert_eq!(ignore.len(), 1);
    assert!(ignore.consume(Path::new("C:/vault/note.md")).unwrap());
    assert!(!ignore.consume(Path::new("C:/vault/note.md")).unwrap());

    for index in 0..(IGNORE_NEXT_MAX + 1) {
      ignore
        .register(&PathBuf::from(format!("/vault/{index}.md")))
        .unwrap();
    }
    assert_eq!(ignore.len(), IGNORE_NEXT_MAX);
  }

  #[test]
  fn watcher_target_must_stay_inside_canonical_vault() {
    let root = std::env::temp_dir().join(format!("elephant-watch-root-{}", std::process::id()));
    let outside = std::env::temp_dir().join(format!("elephant-watch-outside-{}", std::process::id()));
    fs::create_dir_all(root.join("Folder")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    assert!(assert_target_inside_root(&root, &root.join("Folder")).is_ok());
    assert!(assert_target_inside_root(&root, &outside).is_err());
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(outside).ok();
  }

  #[test]
  fn watcher_key_is_identical_for_watch_and_unwatch_after_canonicalization() {
    let root = std::env::temp_dir().join(format!("elephant-watch-key-{}", std::process::id()));
    let folder = root.join("Folder");
    fs::create_dir_all(&folder).unwrap();
    let canonical = assert_target_inside_root(&root, &folder).unwrap();
    assert_eq!(watcher_key("main", &canonical), watcher_key("main", &canonical));
    fs::remove_dir_all(root).ok();
  }
}
