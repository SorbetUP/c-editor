//! Persistent Freya vault registry using the same desktop contract as Tauri.
//!
//! Freya is still a development shell, but it must not create a second source
//! of truth for vault identity. The canonical file is therefore the Tauri
//! `tauri-vaults.json` registry. Older Freya `elephantnote.json` registries are
//! migrated once, atomically, and then ignored.

use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::vault_adapter::types::{next_vault_id, VaultConfig, VaultDescriptor, VAULT_SCHEMA_VERSION};

const CONFIG_FILE: &str = "tauri-vaults.json";
const LEGACY_FREYA_CONFIG_FILE: &str = "elephantnote.json";
const PROFILE_OVERRIDE_ENV: &str = "ELEPHANT_FREYA_PROFILE";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedRegistry {
    #[serde(default)]
    vaults: Vec<VaultDescriptor>,
    active_vault_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VaultRegistry {
    pub(crate) vaults: Vec<VaultDescriptor>,
    pub(crate) active_vault_id: Option<String>,
}

impl VaultRegistry {
    pub(crate) fn load() -> Result<Self, String> {
        let canonical = config_path()?;
        if canonical.exists() {
            return read_registry(&canonical);
        }

        let legacy = canonical.with_file_name(LEGACY_FREYA_CONFIG_FILE);
        if !legacy.exists() {
            return Ok(Self::default());
        }

        let registry = read_registry(&legacy)?;
        registry.persist()?;
        eprintln!(
            "[freya][vault-registry] action=migrate source={} target={}",
            legacy.display(),
            canonical.display()
        );
        Ok(registry)
    }

    pub(crate) fn persist(&self) -> Result<(), String> {
        let path = config_path()?;
        let parent = path
            .parent()
            .ok_or_else(|| format!("vault registry path has no parent: {}", path.display()))?;
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create vault registry directory {}: {error}",
                parent.display()
            )
        })?;

        let config = VaultConfig {
            vaults: self.vaults.clone(),
            active_vault_id: self.active_vault_id.clone(),
        };
        let mut value = serde_json::to_value(config)
            .map_err(|error| format!("encode vault registry: {error}"))?;
        value["schemaVersion"] = serde_json::json!(VAULT_SCHEMA_VERSION);
        let encoded = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
        write_atomic(&path, &encoded)?;
        eprintln!(
            "[freya][vault-registry] action=persist path={} count={}",
            path.display(),
            self.vaults.len()
        );
        Ok(())
    }

    pub(crate) fn active(&self) -> Option<&VaultDescriptor> {
        self.active_vault_id.as_ref().and_then(|id| {
            self.vaults
                .iter()
                .find(|vault| vault.id == *id && vault.enabled)
        })
    }

    pub(crate) fn add_or_activate(&mut self, root: &Path) -> Result<VaultDescriptor, String> {
        let canonical = fs::canonicalize(root)
            .map_err(|error| format!("vault root is not accessible {}: {error}", root.display()))?;
        if !canonical.is_dir() {
            return Err(format!(
                "vault root is not a directory: {}",
                canonical.display()
            ));
        }
        let path = normalize_path(&canonical);
        let descriptor =
            if let Some(vault) = self.vaults.iter_mut().find(|vault| vault.path == path) {
                vault.enabled = true;
                vault.last_opened_at = now_string();
                vault.clone()
            } else {
                let name = canonical
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Personal")
                    .to_owned();
                let descriptor = VaultDescriptor {
                    id: next_vault_id(&self.vaults, &name),
                    name,
                    path,
                    icon: String::new(),
                    last_opened_at: now_string(),
                    enabled: true,
                };
                self.vaults.push(descriptor.clone());
                descriptor
            };
        self.active_vault_id = Some(descriptor.id.clone());
        Ok(descriptor)
    }

    pub(crate) fn activate(&mut self, id: &str) -> Result<VaultDescriptor, String> {
        let Some(vault) = self.vaults.iter_mut().find(|vault| vault.id == id) else {
            return Err(format!("Unknown vault ID: {id}"));
        };
        vault.enabled = true;
        vault.last_opened_at = now_string();
        self.active_vault_id = Some(id.to_owned());
        Ok(vault.clone())
    }

    pub(crate) fn remove(&mut self, id: &str) -> Result<(), String> {
        if !self.vaults.iter().any(|vault| vault.id == id) {
            return Err(format!("Unknown vault ID: {id}"));
        }
        self.vaults.retain(|vault| vault.id != id);
        if self.active_vault_id.as_deref() == Some(id) {
            self.active_vault_id = self
                .vaults
                .iter()
                .find(|vault| vault.enabled)
                .map(|vault| vault.id.clone());
        }
        Ok(())
    }

    pub(crate) fn set_name(&mut self, id: &str, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Vault name cannot be empty.".to_owned());
        }
        let Some(vault) = self.vaults.iter_mut().find(|vault| vault.id == id) else {
            return Err(format!("Unknown vault ID: {id}"));
        };
        vault.name = name.to_owned();
        Ok(())
    }

    pub(crate) fn set_icon(&mut self, id: &str, icon: &str) -> Result<(), String> {
        let Some(vault) = self.vaults.iter_mut().find(|vault| vault.id == id) else {
            return Err(format!("Unknown vault ID: {id}"));
        };
        vault.icon = icon.to_owned();
        Ok(())
    }
}

fn read_registry(path: &Path) -> Result<VaultRegistry, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("read vault registry {}: {error}", path.display()))?;
    let persisted: PersistedRegistry = serde_json::from_str(&raw)
        .map_err(|error| format!("parse vault registry {}: {error}", path.display()))?;
    let mut registry = VaultRegistry {
        vaults: persisted.vaults,
        active_vault_id: persisted.active_vault_id,
    };
    if registry.active().is_none() {
        registry.active_vault_id = registry
            .vaults
            .iter()
            .find(|vault| vault.enabled)
            .map(|vault| vault.id.clone());
    }
    Ok(registry)
}

fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

fn config_dir() -> Result<PathBuf, String> {
    if let Some(profile) = env::var_os(PROFILE_OVERRIDE_ENV) {
        return Ok(PathBuf::from(profile));
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.elephantnote.app"));
    }
    #[cfg(target_os = "windows")]
    if let Some(app_data) = env::var_os("APPDATA") {
        return Ok(PathBuf::from(app_data).join("com.elephantnote.app"));
    }
    let root = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(root.join("com.elephantnote.app"))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("write vault registry {}: {error}", temporary.display()))?;
    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("replace vault registry {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("install vault registry {}: {error}", path.display()))?;
    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_add_activate_remove_and_persist_preserves_real_paths() {
        let profile = std::env::temp_dir().join(format!(
            "elephant-freya-registry-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first = profile.join("First Vault");
        let second = profile.join("Second Vault");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let previous = env::var_os(PROFILE_OVERRIDE_ENV);
        env::set_var(PROFILE_OVERRIDE_ENV, &profile);

        let mut registry = VaultRegistry::default();
        let first_record = registry.add_or_activate(&first).unwrap();
        let second_record = registry.add_or_activate(&second).unwrap();
        assert_eq!(registry.active().unwrap().id, second_record.id);
        registry.activate(&first_record.id).unwrap();
        registry.persist().unwrap();
        assert!(profile.join(CONFIG_FILE).is_file());
        let restored = VaultRegistry::load().unwrap();
        assert_eq!(restored.active().unwrap().name, "First Vault");
        assert_eq!(restored.vaults.len(), 2);
        let second_id = restored
            .vaults
            .iter()
            .find(|vault| vault.name == "Second Vault")
            .unwrap()
            .id
            .clone();
        let mut restored = restored;
        restored.remove(&second_id).unwrap();
        assert_eq!(restored.vaults.len(), 1);

        match previous {
            Some(value) => env::set_var(PROFILE_OVERRIDE_ENV, value),
            None => env::remove_var(PROFILE_OVERRIDE_ENV),
        }
        let _ = fs::remove_dir_all(profile);
    }

    #[test]
    fn legacy_freya_registry_is_migrated_to_tauri_filename() {
        let profile = std::env::temp_dir().join(format!(
            "elephant-freya-registry-migrate-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&profile).unwrap();
        let previous = env::var_os(PROFILE_OVERRIDE_ENV);
        env::set_var(PROFILE_OVERRIDE_ENV, &profile);
        let legacy = profile.join(LEGACY_FREYA_CONFIG_FILE);
        fs::write(
            &legacy,
            serde_json::json!({
                "schemaVersion": 1,
                "vaults": [],
                "activeVaultId": null
            })
            .to_string(),
        )
        .unwrap();

        let registry = VaultRegistry::load().unwrap();
        assert!(registry.vaults.is_empty());
        assert!(profile.join(CONFIG_FILE).is_file());

        match previous {
            Some(value) => env::set_var(PROFILE_OVERRIDE_ENV, value),
            None => env::remove_var(PROFILE_OVERRIDE_ENV),
        }
        let _ = fs::remove_dir_all(profile);
    }
}