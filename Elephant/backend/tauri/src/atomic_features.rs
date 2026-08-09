use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

const ATOMIC_FEATURES_FILE: &str = "atomic-features.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(transparent)]
pub struct AtomicFeatures(pub Map<String, Value>);

pub struct AtomicFeatureStore {
  path: PathBuf,
  data: Map<String, Value>,
}

impl AtomicFeatureStore {
  pub fn load(app: &AppHandle) -> Option<Self> {
    let dir = app.path().app_data_dir().ok()?;
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join(ATOMIC_FEATURES_FILE);
    let data: Map<String, Value> = read_json_or(&path, Map::new());
    Some(AtomicFeatureStore { path, data })
  }

  pub fn list(&self) -> AtomicFeatures {
    AtomicFeatures(self.data.clone())
  }

  pub fn get(&self, feature: &str) -> Value {
    self.data.get(feature).cloned().unwrap_or(Value::Bool(false))
  }

  pub fn toggle(&mut self, feature: &str) -> std::io::Result<bool> {
    let current = self.get(feature).as_bool().unwrap_or(false);
    let new = !current;
    self.data.insert(feature.to_string(), Value::Bool(new));
    write_json_atomically(&self.path, &Value::Object(self.data.clone()))?;
    Ok(new)
  }

  pub fn set_enabled(&mut self, feature: &str, enabled: bool) -> std::io::Result<()> {
    self.data.insert(feature.to_string(), Value::Bool(enabled));
    write_json_atomically(&self.path, &Value::Object(self.data.clone()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn toggle_flips_value_and_persists() {
    let dir = std::env::temp_dir().join(format!("elephantnote_atomic_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(ATOMIC_FEATURES_FILE);
    let mut store = AtomicFeatureStore { path, data: Map::new() };
    assert!(!store.get("rag").as_bool().unwrap_or(false));
    assert!(store.toggle("rag").unwrap());
    assert!(store.get("rag").as_bool().unwrap_or(false));
    assert!(!store.toggle("rag").unwrap());
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn list_returns_all_features() {
    let mut data = Map::new();
    data.insert("alpha".into(), Value::Bool(true));
    data.insert("beta".into(), Value::Bool(false));
    let store = AtomicFeatureStore { path: PathBuf::from("/tmp/x"), data };
    let list = store.list();
    assert_eq!(list.0.len(), 2);
  }

  #[test]
  fn set_enabled_overwrites_value() {
    let dir = std::env::temp_dir().join(format!("elephantnote_atomic_set_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(ATOMIC_FEATURES_FILE);
    let mut store = AtomicFeatureStore { path, data: Map::new() };
    store.set_enabled("llm_full", true).unwrap();
    assert_eq!(store.get("llm_full"), Value::Bool(true));
    store.set_enabled("llm_full", false).unwrap();
    assert_eq!(store.get("llm_full"), Value::Bool(false));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn default_returns_false_when_missing() {
    let store = AtomicFeatureStore { path: PathBuf::from("/tmp/nx.json"), data: Map::new() };
    assert_eq!(store.get("unknown"), Value::Bool(false));
  }

  #[test]
  fn json_shape_preserves_bool() {
    let dir = std::env::temp_dir().join(format!("elephantnote_atomic_json_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(ATOMIC_FEATURES_FILE);
    let mut store = AtomicFeatureStore { path: path.clone(), data: Map::new() };
    store.set_enabled("x", true).unwrap();
    let on_disk: Map<String, Value> = read_json_or(&path, Map::new());
    assert_eq!(on_disk.get("x").and_then(|v| v.as_bool()), Some(true));
    fs::remove_dir_all(&dir).ok();
  }
}