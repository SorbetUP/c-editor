use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

const SERVICE_NAME: &str = "elephantnote";
const ENCRYPT_KEYS: &[&str] = &["githubToken"];

pub struct DataCenter {
  path: PathBuf,
  data: Map<String, Value>,
}

impl DataCenter {
  fn path_of(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join("userData.json"))
  }

  pub fn load(app: &AppHandle) -> Option<Self> {
    let path = Self::path_of(app)?;
    fs::create_dir_all(path.parent()?).ok();
    let mut data: Map<String, Value> = read_json_or(&path, Map::new());
    ensure_defaults(&mut data, app);
    Some(DataCenter { path, data })
  }

  pub fn get_all_sync(&self) -> Value {
    let mut copy = self.data.clone();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    for key in ENCRYPT_KEYS {
      if let Some(secret) = read_secret(key) {
        copy.insert((*key).to_string(), Value::String(secret));
      }
    }
    Value::Object(copy)
  }

  pub fn get(&self, key: &str) -> Value {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if ENCRYPT_KEYS.contains(&key) {
      if let Some(secret) = read_secret(key) {
        return Value::String(secret);
      }
      return Value::Null;
    }
    self.data.get(key).cloned().unwrap_or(Value::Null)
  }

  pub fn set(&mut self, key: &str, value: Value) -> std::io::Result<Value> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if ENCRYPT_KEYS.contains(&key) {
      write_secret(key, value.as_str().unwrap_or(""))?;
      return Ok(value);
    }
    let prev = self.data.insert(key.to_string(), value.clone()).unwrap_or(Value::Null);
    write_json_atomically(&self.path, &Value::Object(self.data.clone()))?;
    let _ = prev;
    Ok(value)
  }

  pub fn set_many(&mut self, items: Value) -> std::io::Result<()> {
    if let Value::Object(map) = items {
      for (k, v) in map {
        self.set(&k, v)?;
      }
    }
    Ok(())
  }
}

fn ensure_defaults(data: &mut Map<String, Value>, app: &AppHandle) {
  let user_data = app.path().app_data_dir().ok();
  if let Some(dir) = user_data {
    let images = dir.join("images").to_string_lossy().to_string();
    let screenshot = dir.join("screenshot").to_string_lossy().to_string();
    data.entry("imageFolderPath").or_insert(Value::String(images));
    data.entry("screenshotFolderPath").or_insert(Value::String(screenshot));
  }
  let _ = fs::create_dir_all(data.get("screenshotFolderPath").and_then(|v| v.as_str()).unwrap_or(""));
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_secret(key: &str) -> Option<String> {
  keyring::Entry::new(SERVICE_NAME, key)
    .ok()
    .and_then(|entry| entry.get_password().ok())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn write_secret(key: &str, value: &str) -> std::io::Result<()> {
  keyring::Entry::new(SERVICE_NAME, key)
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?
    .set_password(value)
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encrypt_keys_contains_github_token() {
    assert!(ENCRYPT_KEYS.contains(&"githubToken"));
  }

  #[test]
  fn service_name_is_stable() {
    assert_eq!(SERVICE_NAME, "elephantnote");
  }
}