use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

const KEYBINDINGS_FILE: &str = "keybindings.json";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(transparent)]
pub struct Keybindings(pub Map<String, Value>);

fn macos_defaults() -> Map<String, Value> {
  let mut map = Map::new();
  map.insert("file.save".into(), Value::String("CmdOrCtrl+S".into()));
  map.insert("file.save-as".into(), Value::String("CmdOrCtrl+Shift+S".into()));
  map.insert("edit.undo".into(), Value::String("CmdOrCtrl+Z".into()));
  map.insert("edit.redo".into(), Value::String("CmdOrCtrl+Shift+Z".into()));
  map.insert("edit.find".into(), Value::String("CmdOrCtrl+F".into()));
  map.insert("file.quick-open".into(), Value::String("CmdOrCtrl+P".into()));
  map.insert("view.toggle-sidebar".into(), Value::String("CmdOrCtrl+J".into()));
  map.insert("app.preferences".into(), Value::String("CmdOrCtrl+,".into()));
  map
}

fn windows_defaults() -> Map<String, Value> {
  let mut map = Map::new();
  map.insert("file.save".into(), Value::String("CmdOrCtrl+S".into()));
  map.insert("file.save-as".into(), Value::String("CmdOrCtrl+Shift+S".into()));
  map.insert("edit.undo".into(), Value::String("CmdOrCtrl+Z".into()));
  map.insert("edit.redo".into(), Value::String("CmdOrCtrl+Y".into()));
  map.insert("edit.find".into(), Value::String("CmdOrCtrl+F".into()));
  map.insert("file.quick-open".into(), Value::String("CmdOrCtrl+P".into()));
  map.insert("view.toggle-sidebar".into(), Value::String("CmdOrCtrl+J".into()));
  map.insert("app.preferences".into(), Value::String("CmdOrCtrl+,".into()));
  map
}

fn linux_defaults() -> Map<String, Value> {
  windows_defaults()
}

pub fn os_defaults() -> Map<String, Value> {
  if cfg!(target_os = "macos") {
    macos_defaults()
  } else if cfg!(target_os = "windows") {
    windows_defaults()
  } else {
    linux_defaults()
  }
}

pub struct KeybindingsStore {
  path: PathBuf,
}

impl KeybindingsStore {
  pub fn load(app: &AppHandle) -> Option<Self> {
    let dir = app.path().app_config_dir().ok()?;
    fs::create_dir_all(&dir).ok()?;
    Some(KeybindingsStore { path: dir.join(KEYBINDINGS_FILE) })
  }

  pub fn get(&self) -> Keybindings {
    let merged = merge_with_defaults(read_json_or(&self.path, Keybindings::default()).0);
    Keybindings(merged)
  }

  pub fn save(&self, bindings: Keybindings) -> std::io::Result<()> {
    write_json_atomically(&self.path, &bindings)
  }
}

fn merge_with_defaults(user: Map<String, Value>) -> Map<String, Value> {
  let mut merged = os_defaults();
  for (k, v) in user {
    merged.insert(k, v);
  }
  merged
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn os_defaults_contains_required_keys() {
    let defaults = os_defaults();
    assert!(defaults.contains_key("file.save"));
    assert!(defaults.contains_key("edit.undo"));
    assert!(defaults.contains_key("file.quick-open"));
  }

  #[test]
  fn merge_user_overrides_default_keeping_unknown() {
    let mut user = Map::new();
    user.insert("file.save".into(), Value::String("Ctrl+S".into()));
    user.insert("custom.binding".into(), Value::String("Ctrl+Shift+X".into()));
    let merged = merge_with_defaults(user);
    assert_eq!(merged.get("file.save").and_then(|v| v.as_str()), Some("Ctrl+S"));
    assert_eq!(merged.get("edit.undo").and_then(|v| v.as_str()), Some("CmdOrCtrl+Z"));
    assert!(merged.contains_key("custom.binding"));
  }

  #[test]
  fn macos_uses_cmd_z_for_redo() {
    let defaults = macos_defaults();
    assert_eq!(defaults.get("edit.redo").and_then(|v| v.as_str()), Some("CmdOrCtrl+Shift+Z"));
  }

  #[test]
  fn windows_uses_ctrl_y_for_redo() {
    let defaults = windows_defaults();
    assert_eq!(defaults.get("edit.redo").and_then(|v| v.as_str()), Some("CmdOrCtrl+Y"));
  }

  #[test]
  fn save_and_load_roundtrip_uses_merge() {
    let dir = std::env::temp_dir().join(format!("elephantnote_kb_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("keybindings.json");
    let mut user = Map::new();
    user.insert("file.save".into(), Value::String("Ctrl+S".into()));
    write_json_atomically(&path, &Keybindings(user.clone())).unwrap();
    let loaded: Keybindings = read_json_or(&path, Keybindings::default());
    assert_eq!(loaded.0.get("file.save").and_then(|v| v.as_str()), Some("Ctrl+S"));
    fs::remove_dir_all(&dir).ok();
  }
}