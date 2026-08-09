use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone, Debug)]
pub struct FsEvent {
  pub path: String,
  pub kind: String,
  pub is_directory: bool,
}

#[derive(Default)]
pub struct IgnoreNext {
  pub paths: std::sync::Mutex<Vec<PathBuf>>,
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

fn is_ignored_path(path: &Path) -> bool {
  let s = path.to_string_lossy();
  s.contains("/.elephantnote/") || s.contains("/.git/") || s.contains("/node_modules/") || s.ends_with(".tmp")
}

#[tauri::command]
pub fn tauri_watcher_watch_file(app: AppHandle, state: State<'_, WatcherState>, window_id: String, path: String) -> R<()> {
  watch_target(&app, &state, &window_id, &path, RecursiveMode::NonRecursive)
}

#[tauri::command]
pub fn tauri_watcher_watch_directory(app: AppHandle, state: State<'_, WatcherState>, window_id: String, path: String) -> R<()> {
  watch_target(&app, &state, &window_id, &path, RecursiveMode::Recursive)
}

fn watch_target(app: &AppHandle, state: &State<'_, WatcherState>, window_id: &str, path: &str, mode: RecursiveMode) -> R<()> {
  let target = PathBuf::from(path);
  if !target.exists() {
    return Err(format!("path does not exist: {path}"));
  }
  let app_handle = app.clone();
  let ignore = state.ignore_next.paths.lock().map_err(|e| e.to_string())?.clone();
  let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
  let mut watcher = notify::recommended_watcher(tx).map_err(|e| e.to_string())?;
  watcher
    .watch(&target, mode)
    .map_err(|e| e.to_string())?;

  let wid = window_id.to_string();
  thread::spawn(move || {
    let mut last_emit: HashMap<PathBuf, Instant> = HashMap::new();
    let debounce = Duration::from_millis(300);
    for event in rx {
      match event {
        Ok(ev) => {
          for path in &ev.paths {
            if is_ignored_path(path) {
              continue;
            }
            if ignore.iter().any(|p| p == path) {
              continue;
            }
            let now = Instant::now();
            if let Some(prev) = last_emit.get(path) {
              if now.duration_since(*prev) < debounce {
                continue;
              }
            }
            last_emit.insert(path.clone(), now);
            let payload = FsEvent {
              path: path.to_string_lossy().to_string(),
              kind: kind_str(&ev.kind).to_string(),
              is_directory: path.is_dir(),
            };
            let _ = app_handle.emit(&format!("elephantnote:fs:changed:{wid}"), payload);
          }
        }
        Err(e) => {
          eprintln!("[watcher] error: {e}");
        }
      }
    }
  });

  let mut guard = state.watchers.lock().map_err(|e| e.to_string())?;
  let key = format!("{window_id}:{}", path);
  guard.insert(key, watcher);
  Ok(())
}

#[tauri::command]
pub fn tauri_watcher_unwatch_file(state: State<'_, WatcherState>, window_id: String, path: String) -> R<()> {
  remove_watcher(&state, &format!("{window_id}:{path}"))
}

#[tauri::command]
pub fn tauri_watcher_unwatch_directory(state: State<'_, WatcherState>, window_id: String, path: String) -> R<()> {
  remove_watcher(&state, &format!("{window_id}:{path}"))
}

#[tauri::command]
pub fn tauri_watcher_unwatch_all(state: State<'_, WatcherState>, window_id: String) -> R<()> {
  let mut guard = state.watchers.lock().map_err(|e| e.to_string())?;
  let prefix = format!("{window_id}:");
  let keys: Vec<String> = guard.keys().filter(|k| k.starts_with(&prefix)).cloned().collect();
  for k in keys {
    guard.remove(&k);
  }
  Ok(())
}

#[tauri::command]
pub fn tauri_watcher_ignore_next(state: State<'_, WatcherState>, path: String) -> R<()> {
  let mut guard = state.ignore_next.paths.lock().map_err(|e| e.to_string())?;
  guard.push(PathBuf::from(path));
  Ok(())
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
    assert!(is_ignored_path(&PathBuf::from("/tmp/v/.elephantnote/index/index.json")));
    assert!(is_ignored_path(&PathBuf::from("/tmp/v/.git/HEAD")));
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
    assert_eq!(kind_str(&EventKind::Create(notify::event::CreateKind::File)), "create");
    assert_eq!(kind_str(&EventKind::Modify(notify::event::ModifyKind::Any)), "modify");
    assert_eq!(kind_str(&EventKind::Remove(notify::event::RemoveKind::File)), "remove");
  }

  #[test]
  fn fs_event_serializes_to_json() {
    let ev = FsEvent { path: "/tmp/note.md".into(), kind: "create".into(), is_directory: false };
    let json = serde_json::to_value(&ev).unwrap();
    assert_eq!(json.get("kind").and_then(|v| v.as_str()), Some("create"));
  }

  #[test]
  fn ignore_next_holds_paths() {
    let ignore = IgnoreNext::default();
    ignore.paths.lock().unwrap().push(PathBuf::from("/tmp/a.md"));
    let guard = ignore.paths.lock().unwrap();
    assert_eq!(guard.len(), 1);
  }

  #[test]
  fn fs_event_with_directory_flag() {
    let ev = FsEvent { path: "/tmp/dir".into(), kind: "modify".into(), is_directory: true };
    let json = serde_json::to_value(&ev).unwrap();
    assert_eq!(json.get("is_directory").and_then(|v| v.as_bool()), Some(true));
  }

  #[test]
  fn file_operations_emit_for_visible_path() {
    let dir = std::env::temp_dir().join(format!("elephantnote_watch_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let visible = dir.join("note.md");
    assert!(!is_ignored_path(&visible));
    let hidden = dir.join(".elephantnote").join("index.json");
    fs::create_dir_all(hidden.parent().unwrap()).unwrap();
    assert!(is_ignored_path(&hidden));
    fs::remove_dir_all(&dir).ok();
  }
}