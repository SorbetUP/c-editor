use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use super::metadata::write_atomic;
use super::types::{active_vault, next_vault_id, VaultConfig, VaultDescriptor};
use super::types::VAULT_SCHEMA_VERSION;

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

pub fn canonicalize_vault_root(path: impl AsRef<str>) -> R<PathBuf> {
  let path = path.as_ref().trim();
  if path.is_empty() {
    return Err("A vault root is required.".to_string());
  }
  if path.contains('\0') {
    return Err("A vault root cannot contain a NUL byte.".to_string());
  }
  let path = PathBuf::from(path);
  if !path.is_absolute() {
    return Err(format!("Vault root must be absolute: {}", path.to_string_lossy()));
  }
  let canonical = fs::canonicalize(&path).map_err(|error| format!("Vault root is not accessible: {error}"))?;
  if !canonical.is_dir() {
    return Err(format!("Vault root is not a directory: {}", canonical.to_string_lossy()));
  }
  Ok(canonical)
}

pub fn config_path(app: &AppHandle) -> R<PathBuf> {
  let dir = crate::acceptance_profile::app_config_dir(app).map_err(|e| e.to_string())?;
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
    enabled: true,
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

  if config.active_vault_id.as_deref().is_none() || active_vault(&config).is_none() {
    config.active_vault_id = config.vaults.iter().find(|vault| vault.enabled).map(|vault| vault.id.clone());
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
    serde_json::from_str(&raw).map_err(|error| format!("Invalid vault configuration: {error}"))?
  };

  let fallback = fallback_vault_path(app);
  if let Some(path) = fallback.as_ref() {
    fs::create_dir_all(path).map_err(|e| e.to_string())?;
  }
  let mut normalized = with_fallback_vault(config.clone(), fallback);
  let mut changed = normalized.active_vault_id != config.active_vault_id;
  for vault in &mut normalized.vaults {
    let configured_path = Path::new(&vault.path);
    if configured_path.is_absolute() && configured_path.is_dir() {
      let canonical = normalize_absolute_path(canonicalize_vault_root(&vault.path)?.to_string_lossy());
      if vault.path != canonical {
        vault.path = canonical;
        changed = true;
      }
    } else {
      // Keep missing vaults in the registry so the UI can offer relinking
      // instead of silently discarding a user's workspace.
      let normalized_path = normalize_absolute_path(&vault.path);
      if vault.path != normalized_path {
        vault.path = normalized_path;
        changed = true;
      }
    }
  }
  if normalized.active_vault_id.is_some() && active_vault(&normalized).is_none() {
    normalized.active_vault_id = normalized.vaults.iter().find(|vault| vault.enabled).map(|vault| vault.id.clone());
    changed = true;
  }
  if changed {
    let mut value = serde_json::to_value(&normalized).map_err(|e| e.to_string())?;
    value["schemaVersion"] = serde_json::json!(VAULT_SCHEMA_VERSION);
    write_atomic(&path, &serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?)?;
  }
  Ok(normalized)
}

pub fn write_config(app: &AppHandle, config: &VaultConfig) -> R<()> {
  let mut value = serde_json::to_value(config).map_err(|e| e.to_string())?;
  value["schemaVersion"] = serde_json::json!(VAULT_SCHEMA_VERSION);
  write_atomic(&config_path(app)?, &serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?)
}

pub fn get_active_vault(app: &AppHandle) -> R<VaultDescriptor> {
  active_vault(&read_config(app)?).ok_or_else(|| "No active ElephantNote vault.".to_string())
}

pub fn upsert_vault(config: &mut VaultConfig, vault_path: String) -> R<VaultDescriptor> {
  let normalized_path = normalize_absolute_path(canonicalize_vault_root(&vault_path)?.to_string_lossy());
  if let Some(index) = config.vaults.iter().position(|vault| vault.path == normalized_path) {
    config.vaults[index].last_opened_at = now_string();
    config.vaults[index].enabled = true;
    config.active_vault_id = Some(config.vaults[index].id.clone());
    return Ok(config.vaults[index].clone());
  }

  let name = basename(Path::new(&normalized_path));
  let id = next_vault_id(&config.vaults, &name);
  let vault = VaultDescriptor {
    id: id.clone(),
    name,
    path: normalized_path,
    icon: String::new(),
    last_opened_at: now_string(),
    enabled: true,
  };
  config.active_vault_id = Some(id);
  config.vaults.push(vault.clone());
  Ok(vault)
}

pub fn set_active_vault(config: &mut VaultConfig, vault_id: String) -> R<()> {
  let Some(vault) = config.vaults.iter_mut().find(|vault| vault.id == vault_id) else {
    return Err(format!("Unknown vault ID: {vault_id}"));
  };
  vault.enabled = true;
  vault.last_opened_at = now_string();
  config.active_vault_id = Some(vault_id);
  Ok(())
}

pub fn set_vault_enabled(config: &mut VaultConfig, vault_id: &str, enabled: bool) -> R<()> {
  let Some(index) = config.vaults.iter().position(|vault| vault.id == vault_id) else {
    return Err(format!("Unknown vault ID: {vault_id}"));
  };
  config.vaults[index].enabled = enabled;
  if enabled {
    if config.active_vault_id.is_none() {
      config.active_vault_id = Some(vault_id.to_string());
    }
  } else if config.active_vault_id.as_deref() == Some(vault_id) {
    config.active_vault_id = config
      .vaults
      .iter()
      .find(|candidate| candidate.enabled && candidate.id != vault_id)
      .map(|candidate| candidate.id.clone());
  }
  Ok(())
}

pub fn set_vault_icon(config: &mut VaultConfig, vault_id: &str, icon: String) -> R<()> {
  let mut found = false;
  for vault in &mut config.vaults {
    if vault.id == vault_id {
      vault.icon = icon.clone();
      found = true;
    }
  }
  if found { Ok(()) } else { Err(format!("Unknown vault ID: {vault_id}")) }
}

pub fn set_vault_name(config: &mut VaultConfig, vault_id: &str, name: String) -> R<()> {
  let name = name.trim().to_string();
  if name.is_empty() {
    return Err("Vault name cannot be empty.".to_string());
  }
  let mut found = false;
  for vault in &mut config.vaults {
    if vault.id == vault_id {
      vault.name = name.clone();
      found = true;
    }
  }
  if found { Ok(()) } else { Err(format!("Unknown vault ID: {vault_id}")) }
}

pub fn remove_vault(config: &mut VaultConfig, vault_id: &str) -> R<()> {
  if !config.vaults.iter().any(|vault| vault.id == vault_id) {
    return Err(format!("Unknown vault ID: {vault_id}"));
  }
  config.vaults.retain(|vault| vault.id != vault_id);
  if config.active_vault_id.as_deref() == Some(vault_id) {
    config.active_vault_id = config.vaults.iter().find(|vault| vault.enabled).map(|vault| vault.id.clone());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("elephant-config-{name}"));
    fs::create_dir_all(&root).unwrap();
    root
  }

  #[test]
  fn upserts_new_vault() {
    let mut config = VaultConfig::default();
    let root = test_root("work");
    let vault = upsert_vault(&mut config, root.to_string_lossy().to_string()).unwrap();
    assert_eq!(vault.name, "elephant-config-work");
    assert_eq!(config.active_vault_id, Some(vault.id));
    assert_eq!(config.vaults.len(), 1);
  }

  #[test]
  fn upsert_existing_vault_keeps_single_record() {
    let mut config = VaultConfig::default();
    let root = test_root("same");
    let first = upsert_vault(&mut config, root.to_string_lossy().to_string()).unwrap();
    let second = upsert_vault(&mut config, root.to_string_lossy().to_string()).unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(config.vaults.len(), 1);
  }

  #[test]
  fn removing_active_vault_selects_next_one() {
    let mut config = VaultConfig::default();
    let a = upsert_vault(&mut config, test_root("a").to_string_lossy().to_string()).unwrap();
    let b = upsert_vault(&mut config, test_root("b").to_string_lossy().to_string()).unwrap();
    set_active_vault(&mut config, a.id.clone()).unwrap();
    remove_vault(&mut config, &a.id).unwrap();
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
    let existing = upsert_vault(&mut config, test_root("existing").to_string_lossy().to_string()).unwrap();
    let next = with_fallback_vault(config, Some(PathBuf::from("/data/app/vaults/Personal")));
    assert_eq!(next.vaults.len(), 1);
    assert_eq!(next.active_vault_id, Some(existing.id));
  }

  #[test]
  fn rejects_unknown_active_vault_ids() {
    let mut config = VaultConfig::default();
    assert!(set_active_vault(&mut config, "missing".to_string()).is_err());
  }

  #[test]
  fn disabling_active_vault_selects_an_enabled_vault() {
    let mut config = VaultConfig::default();
    let active = upsert_vault(&mut config, test_root("enabled-active").to_string_lossy().to_string()).unwrap();
    let fallback = upsert_vault(&mut config, test_root("enabled-fallback").to_string_lossy().to_string()).unwrap();
    set_active_vault(&mut config, active.id.clone()).unwrap();

    set_vault_enabled(&mut config, &active.id, false).unwrap();

    assert_eq!(config.active_vault_id, Some(fallback.id));
    assert!(!config.vaults.iter().find(|vault| vault.id == active.id).unwrap().enabled);
  }

  #[test]
  fn enabling_a_vault_without_an_active_vault_selects_it() {
    let mut config = VaultConfig::default();
    let vault = upsert_vault(&mut config, test_root("enable-without-active").to_string_lossy().to_string()).unwrap();
    config.active_vault_id = None;
    config.vaults[0].enabled = false;

    set_vault_enabled(&mut config, &vault.id, true).unwrap();

    assert_eq!(config.active_vault_id, Some(vault.id));
  }
}
