use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

pub struct Preferences {
  path: PathBuf,
  data: Map<String, Value>,
}

impl Preferences {
  fn path_of(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("preferences.json"))
  }

  pub fn load(app: &AppHandle) -> Option<Self> {
    let path = Self::path_of(app)?;
    let data: Map<String, Value> = read_json_or(&path, Map::new());
    let data = merge_bundled_defaults(data, &path);
    Some(Preferences { path, data })
  }

  pub fn get_all(&self) -> Value {
    Value::Object(self.data.clone())
  }

  pub fn get(&self, key: &str) -> Value {
    self.data.get(key).cloned().unwrap_or(Value::Null)
  }

  pub fn set(&mut self, key: &str, value: Value) -> std::io::Result<Value> {
    self.data.insert(key.to_string(), value.clone());
    write_json_atomically(&self.path, &Value::Object(self.data.clone()))?;
    Ok(value)
  }

  pub fn set_many(&mut self, items: Value) -> std::io::Result<()> {
    if let Value::Object(map) = items {
      for (k, v) in map {
        self.data.insert(k, v);
      }
      write_json_atomically(&self.path, &Value::Object(self.data.clone()))?;
    }
    Ok(())
  }
}

fn merge_bundled_defaults(mut data: Map<String, Value>, _path: &Path) -> Map<String, Value> {
  let defaults: Map<String, Value> = bundled_defaults();
  for (k, v) in defaults {
    if !data.contains_key(&k) {
      data.insert(k, v);
    }
  }
  data
}

fn bundled_defaults() -> Map<String, Value> {
  let mut map = Map::new();
  map.insert("autoSave".into(), Value::Bool(false));
  map.insert("autoSaveDelay".into(), Value::Number(5000.into()));
  map.insert("titleBarStyle".into(), Value::String("custom".into()));
  map.insert("openFilesInNewWindow".into(), Value::Bool(false));
  map.insert("openFolderInNewWindow".into(), Value::Bool(false));
  map.insert("zoom".into(), Value::Number(serde_json::Number::from_f64(1.0).unwrap_or(1.into())));
  map.insert("hideScrollbar".into(), Value::Bool(false));
  map.insert("spellcheckerEnabled".into(), Value::Bool(true));
  map
}

pub fn prefs_path_for(app: &AppHandle) -> Option<PathBuf> {
  app.path().app_config_dir().ok().map(|d| d.join("preferences.json"))
}

pub fn ensure_config_dir(app: &AppHandle) -> Option<PathBuf> {
  let dir = app.path().app_config_dir().ok()?;
  fs::create_dir_all(&dir).ok()?;
  Some(dir)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn bundled_defaults_contains_required_keys() {
    let defaults = bundled_defaults();
    assert!(defaults.contains_key("autoSave"));
    assert!(defaults.contains_key("titleBarStyle"));
    assert_eq!(defaults.get("autoSaveDelay").and_then(|v| v.as_u64()), Some(5000));
  }

  #[test]
  fn merge_keeps_known_values_and_fills_missing() {
    let mut user = Map::new();
    user.insert("autoSave".into(), Value::Bool(true));
    let merged = merge_bundled_defaults(user, Path::new("/tmp/x.json"));
    assert_eq!(merged.get("autoSave").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(merged.get("titleBarStyle").and_then(|v| v.as_str()), Some("custom"));
  }

  #[test]
  fn set_and_persist_roundtrip_via_tmp_file() {
    let dir = std::env::temp_dir().join(format!("elephantnote_prefs_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("preferences.json");
    let mut data: Map<String, Value> = Map::new();
    data.insert("autoSave".into(), Value::Bool(true));
    crate::infra::write_json_atomically(&path, &Value::Object(data.clone())).unwrap();
    let on_disk: Map<String, Value> = crate::infra::read_json_or(&path, Map::new());
    assert_eq!(on_disk.get("autoSave").and_then(|v| v.as_bool()), Some(true));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn json_helper_equality() {
    let v = json!({ "a": 1, "b": [1, 2, 3] });
    let mut map = Map::new();
    map.insert("a".into(), Value::Number(1.into()));
    map.insert("b".into(), Value::Array(vec![1.into(), 2.into(), 3.into()]));
    assert_eq!(v, Value::Object(map));
  }
}