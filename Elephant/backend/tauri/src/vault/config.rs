use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use super::types::{active_vault, next_vault_id, VaultConfig, VaultDescriptor};

pub const CONFIG_FILE: &str = "tauri-vaults.json";

const MOBILE_DEFAULT_VAULT_ID: &str = "mobile-personal";
const MOBILE_DEFAULT_VAULT_NAME: &str = "Personal";

fn is_acceptance_temp_vault(path: &str) -> bool {
  path.contains("/elephant-tauri-acceptance-") && path.ends_with("/vault")
}

type R<T> = Result<T, String>;

pub fn now_string() -> String {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs().to_string())
    .unwrap_or_else(|_| "0".to_string())
}

pub fn basename(path: &Path) -> String {
  path.file_name().and_then(|name| name.to_str()).unwrap_or("Personal").to_string()
}

pub fn normalize_absolute_path(path: impl AsRef<str>) -> String {
  path.as_ref().replace('\\', "/")
}

pub fn config_path(app: &AppHandle) -> R<PathBuf> {
  let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
  fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  Ok(dir.join(CONFIG_FILE))
}

fn mobile_default_vault(path: PathBuf) -> VaultDescriptor {
  let normalized_path = path.to_string_lossy();
  VaultDescriptor {
    id: MOBILE_DEFAULT_VAULT_ID.to_string(),
    name: MOBILE_DEFAULT_VAULT_NAME.to_string(),
    path: normalize_absolute_path(normalized_path.as_ref()),
    icon: String::new(),
    last_opened_at: now_string(),
  }
}

fn with_fallback_vault(mut config: VaultConfig, fallback_path: Option<PathBuf>) -> VaultConfig {
  if let Some(fallback_path) = fallback_path {
    if config.vaults.is_empty() {
      let vault = mobile_default_vault(fallback_path);
      config.active_vault_id = Some(vault.id.clone());
      config.vaults.push(vault);
      return config;
    }
  }

  if config.active_vault_id.as_deref().is_none()
    || active_vault(&config).is_none()
  {
    config.active_vault_id = config.vaults.first().map(|vault| vault.id.clone());
  }

  // Desktop acceptance runs use temporary vaults. They must never become the
  // user's next normal development vault after the test process exits.
  #[cfg(not(mobile))]
  if let Some(active) = active_vault(&config) {
    if is_acceptance_temp_vault(&active.path) {
      let replacement = config
        .vaults
        .iter()
        .filter(|vault| !is_acceptance_temp_vault(&vault.path))
        .filter(|vault| Path::new(&vault.path).is_dir())
        .max_by(|left, right| left.last_opened_at.cmp(&right.last_opened_at))
        .map(|vault| vault.id.clone());
      if replacement.is_some() {
        config.active_vault_id = replacement;
      }
    }
  }

  config
}

#[cfg(mobile)]
fn fallback_vault_path(app: &AppHandle) -> Option<PathBuf> {
  app
    .path()
    .app_data_dir()
    .ok()
    .map(|dir| dir.join("vaults").join(MOBILE_DEFAULT_VAULT_NAME))
}

#[cfg(not(mobile))]
fn fallback_vault_path(_app: &AppHandle) -> Option<PathBuf> {
  None
}

pub fn read_config(app: &AppHandle) -> R<VaultConfig> {
  let path = config_path(app)?;
  let config = if !path.exists() {
    VaultConfig::default()
  } else {
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).unwrap_or_default()
  };

  let fallback = fallback_vault_path(app);
  if let Some(path) = fallback.as_ref() {
    fs::create_dir_all(path).map_err(|e| e.to_string())?;
  }
  let normalized = with_fallback_vault(config.clone(), fallback);
  if normalized.active_vault_id != config.active_vault_id {
    let raw = serde_json::to_string_pretty(&normalized).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())?;
  }
  Ok(normalized)
}

pub fn write_config(app: &AppHandle, config: &VaultConfig) -> R<()> {
  let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
  fs::write(config_path(app)?, raw).map_err(|e| e.to_string())
}

pub fn get_active_vault(app: &AppHandle) -> R<VaultDescriptor> {
  active_vault(&read_config(app)?).ok_or_else(|| "No active ElephantNote vault.".to_string())
}

pub fn upsert_vault(config: &mut VaultConfig, vault_path: String) -> VaultDescriptor {
  let normalized_path = normalize_absolute_path(vault_path);
  if let Some(index) = config.vaults.iter().position(|vault| vault.path == normalized_path) {
    config.vaults[index].last_opened_at = now_string();
    config.active_vault_id = Some(config.vaults[index].id.clone());
    return config.vaults[index].clone();
  }

  let name = basename(Path::new(&normalized_path));
  let id = next_vault_id(&config.vaults, &name);
  let vault = VaultDescriptor {
    id: id.clone(),
    name,
    path: normalized_path,
    icon: String::new(),
    last_opened_at: now_string(),
  };
  config.active_vault_id = Some(id);
  config.vaults.push(vault.clone());
  vault
}

pub fn set_active_vault(config: &mut VaultConfig, vault_id: String) {
  config.active_vault_id = Some(vault_id);
}

pub fn set_vault_icon(config: &mut VaultConfig, vault_id: &str, icon: String) {
  for vault in &mut config.vaults {
    if vault.id == vault_id {
      vault.icon = icon.clone();
    }
  }
}

pub fn set_vault_name(config: &mut VaultConfig, vault_id: &str, name: String) {
  let name = name.trim().to_string();
  if name.is_empty() {
    return;
  }
  for vault in &mut config.vaults {
    if vault.id == vault_id {
      vault.name = name.clone();
    }
  }
}

pub fn remove_vault(config: &mut VaultConfig, vault_id: &str) {
  config.vaults.retain(|vault| vault.id != vault_id);
  if config.active_vault_id.as_deref() == Some(vault_id) {
    config.active_vault_id = config.vaults.first().map(|vault| vault.id.clone());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn upserts_new_vault() {
    let mut config = VaultConfig::default();
    let vault = upsert_vault(&mut config, "Work".to_string());
    assert_eq!(vault.name, "Work");
    assert_eq!(config.active_vault_id, Some(vault.id));
    assert_eq!(config.vaults.len(), 1);
  }

  #[test]
  fn upsert_existing_vault_keeps_single_record() {
    let mut config = VaultConfig::default();
    let first = upsert_vault(&mut config, "Work".to_string());
    let second = upsert_vault(&mut config, "Work".to_string());
    assert_eq!(first.id, second.id);
    assert_eq!(config.vaults.len(), 1);
  }

  #[test]
  fn removing_active_vault_selects_next_one() {
    let mut config = VaultConfig::default();
    let a = upsert_vault(&mut config, "A".to_string());
    let b = upsert_vault(&mut config, "B".to_string());
    set_active_vault(&mut config, a.id.clone());
    remove_vault(&mut config, &a.id);
    assert_eq!(config.active_vault_id, Some(b.id));
  }

  #[test]
  fn desktop_config_does_not_invent_a_vault() {
    let config = with_fallback_vault(VaultConfig::default(), None);
    assert!(config.vaults.is_empty());
    assert!(config.active_vault_id.is_none());
  }

  #[test]
  fn acceptance_temp_vaults_are_identified() {
    assert!(is_acceptance_temp_vault("/tmp/elephant-tauri-acceptance-abc/vault"));
    assert!(!is_acceptance_temp_vault("/Users/sorbet/Documents/Notes"));
  }

  #[test]
  fn mobile_fallback_adds_an_internal_personal_vault() {
    let config = with_fallback_vault(VaultConfig::default(), Some(PathBuf::from("/data/app/vaults/Personal")));
    assert_eq!(config.vaults.len(), 1);
    assert_eq!(config.vaults[0].id, MOBILE_DEFAULT_VAULT_ID);
    assert_eq!(config.vaults[0].name, MOBILE_DEFAULT_VAULT_NAME);
    assert_eq!(config.active_vault_id, Some(MOBILE_DEFAULT_VAULT_ID.to_string()));
  }

  #[test]
  fn mobile_default_vault_normalizes_pathbuf_lossy_text_explicitly() {
    let vault = mobile_default_vault(PathBuf::from("C:\\Users\\noam\\Vault"));
    assert_eq!(vault.path, "C:/Users/noam/Vault");
  }

  #[test]
  fn fallback_keeps_existing_desktop_vaults() {
    let mut config = VaultConfig::default();
    let existing = upsert_vault(&mut config, "/Users/noam/Documents/Vault".to_string());
    let next = with_fallback_vault(config, Some(PathBuf::from("/data/app/vaults/Personal")));
    assert_eq!(next.vaults.len(), 1);
    assert_eq!(next.active_vault_id, Some(existing.id));
  }
}
