use serde_json::{Map, Value};
use std::{fs, io, path::PathBuf};
use tauri::AppHandle;

use crate::infra::{read_json_or, write_json_atomically};

const SERVICE_NAME: &str = "elephantnote";
const ENCRYPT_KEYS: &[&str] = &["githubToken"];
const MAX_SECRET_NAME_BYTES: usize = 128;
const ADDON_SECRET_PREFIX: &str = "addon:";

pub struct DataCenter {
  path: PathBuf,
  data: Map<String, Value>,
}

impl DataCenter {
  fn path_of(app: &AppHandle) -> Option<PathBuf> {
    crate::acceptance_profile::app_data_dir(app).ok().map(|dir| dir.join("userData.json"))
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

  pub fn secret_get(&self, name: &str) -> io::Result<Option<String>> {
    validate_legacy_secret_name(name)?;
    secure_read(name)
  }

  pub fn secret_set(&self, name: &str, value: &str) -> io::Result<()> {
    validate_legacy_secret_name(name)?;
    secure_write(name, value)
  }

  pub fn secret_delete(&self, name: &str) -> io::Result<()> {
    validate_legacy_secret_name(name)?;
    secure_delete(name)
  }

  pub fn addon_secret_get(&self, addon_id: &str, name: &str) -> io::Result<Option<String>> {
    secure_read(&namespaced_addon_secret_key(addon_id, name)?)
  }

  pub fn addon_secret_set(&self, addon_id: &str, name: &str, value: &str) -> io::Result<()> {
    secure_write(&namespaced_addon_secret_key(addon_id, name)?, value)
  }

  pub fn addon_secret_delete(&self, addon_id: &str, name: &str) -> io::Result<()> {
    secure_delete(&namespaced_addon_secret_key(addon_id, name)?)
  }

  pub fn validate_secret_name(name: &str) -> io::Result<()> {
    validate_secret_name(name)
  }

  pub fn namespaced_addon_secret_key(addon_id: &str, name: &str) -> io::Result<String> {
    namespaced_addon_secret_key(addon_id, name)
  }
}

fn ensure_defaults(data: &mut Map<String, Value>, app: &AppHandle) {
  let user_data = crate::acceptance_profile::app_data_dir(app).ok();
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
  secure_read(key).ok().flatten()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn write_secret(key: &str, value: &str) -> std::io::Result<()> {
  secure_write(key, value)
}

fn validate_secret_name(name: &str) -> io::Result<()> {
  let bytes = name.as_bytes();
  if bytes.is_empty()
    || bytes.len() > MAX_SECRET_NAME_BYTES
    || !bytes[0].is_ascii_lowercase()
    || !bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
  {
    return Err(io::Error::new(
      io::ErrorKind::InvalidInput,
      "Secret name must start with a lowercase ASCII letter and contain only lowercase letters, numbers, dots, dashes or underscores",
    ));
  }
  Ok(())
}

fn validate_legacy_secret_name(name: &str) -> io::Result<()> {
  let bytes = name.as_bytes();
  if bytes.is_empty()
    || bytes.len() > MAX_SECRET_NAME_BYTES
    || !bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
  {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid secret name"));
  }
  Ok(())
}

fn valid_addon_id(addon_id: &str) -> bool {
  let bytes = addon_id.as_bytes();
  !bytes.is_empty()
    && bytes.len() <= MAX_SECRET_NAME_BYTES
    && bytes[0].is_ascii_lowercase()
    && bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
}

fn namespaced_addon_secret_key(addon_id: &str, name: &str) -> io::Result<String> {
  if !valid_addon_id(addon_id) {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid addon id for secret namespace"));
  }
  validate_secret_name(name)?;
  Ok(format!("{ADDON_SECRET_PREFIX}{addon_id}:{name}"))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn secure_read(key: &str) -> io::Result<Option<String>> {
  let entry = keyring::Entry::new(SERVICE_NAME, key).map_err(keyring_error)?;
  match entry.get_password() {
    Ok(value) => Ok(Some(value)),
    Err(keyring::Error::NoEntry) => Ok(None),
    Err(error) => Err(keyring_error(error)),
  }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn secure_read(_key: &str) -> io::Result<Option<String>> {
  Err(secure_storage_unavailable())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn secure_write(key: &str, value: &str) -> io::Result<()> {
  keyring::Entry::new(SERVICE_NAME, key)
    .map_err(keyring_error)?
    .set_password(value)
    .map_err(keyring_error)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn secure_write(_key: &str, _value: &str) -> io::Result<()> {
  Err(secure_storage_unavailable())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn secure_delete(key: &str) -> io::Result<()> {
  let entry = keyring::Entry::new(SERVICE_NAME, key).map_err(keyring_error)?;
  match entry.delete_credential() {
    Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
    Err(error) => Err(keyring_error(error)),
  }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn secure_delete(_key: &str) -> io::Result<()> {
  Err(secure_storage_unavailable())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn keyring_error(error: keyring::Error) -> io::Error {
  io::Error::new(io::ErrorKind::Other, format!("keyring secure storage error: {error}"))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn secure_storage_unavailable() -> io::Error {
  io::Error::new(
    io::ErrorKind::Unsupported,
    "Secure secret storage is unavailable on this platform because keyring is not supported",
  )
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

  #[test]
  fn validates_secret_names_and_namespaces_addon_entries() {
    assert!(DataCenter::validate_secret_name("openai").is_ok());
    assert!(DataCenter::validate_secret_name("OpenAI").is_err());
    assert!(DataCenter::validate_secret_name(" openai").is_err());
    assert!(DataCenter::validate_secret_name("../token").is_err());
    assert!(validate_legacy_secret_name("githubToken").is_ok());
    assert_eq!(
      DataCenter::namespaced_addon_secret_key("com.example.ai", "openai").unwrap(),
      "addon:com.example.ai:openai"
    );
    assert!(DataCenter::namespaced_addon_secret_key("../escape", "openai").is_err());
  }
}
