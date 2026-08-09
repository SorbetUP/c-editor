use serde::Deserialize;
use std::{
  collections::BTreeMap,
  fs,
  path::{Component, Path, PathBuf},
};
use tauri::AppHandle;

use crate::{
  addons::InstalledAddon,
  vault::config as vault_config,
  vault_layout,
};

type R<T> = Result<T, String>;

const REGISTRY_VERSION: u32 = 1;
const PUBLIC_HTTPS_CAPABILITY: &str = "public-https";

#[derive(Debug, Deserialize)]
struct AddonRegistry {
  version: u32,
  addons: BTreeMap<String, InstalledAddon>,
}

pub fn normalize_relative_path(value: &str, empty_error: &str) -> R<String> {
  let value = value.trim();
  if value.is_empty() {
    return Err(empty_error.to_string());
  }
  let path = Path::new(value);
  if path.is_absolute() {
    return Err("Addon paths must be relative to the active vault".to_string());
  }

  let mut normalized = PathBuf::new();
  for component in path.components() {
    match component {
      Component::Normal(part) => normalized.push(part),
      Component::CurDir => {}
      Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
        return Err(format!("Path traversal is not allowed: {value}"));
      }
    }
  }
  if normalized.as_os_str().is_empty() {
    return Err(empty_error.to_string());
  }
  Ok(normalized.to_string_lossy().replace('\\', "/"))
}

pub fn scope_matches(scope: &str, relative_path: &str) -> bool {
  let scope = scope.trim().replace('\\', "/");
  if scope == "*" {
    return true;
  }
  if let Some(prefix) = scope.strip_suffix("/**") {
    let prefix = prefix.trim_end_matches('/');
    return relative_path == prefix || relative_path.starts_with(&format!("{prefix}/"));
  }
  relative_path == scope
}

pub fn host_matches(pattern: &str, host: &str) -> bool {
  let pattern = pattern.trim().to_ascii_lowercase();
  let host = host.to_ascii_lowercase();
  if pattern == PUBLIC_HTTPS_CAPABILITY {
    true
  } else if let Some(suffix) = pattern.strip_prefix("*.") {
    host != suffix && host.ends_with(&format!(".{suffix}"))
  } else {
    host == pattern
  }
}

pub fn read_enabled_addon(app: &AppHandle, addon_id: &str) -> R<InstalledAddon> {
  let vault = vault_config::get_active_vault(app)?;
  let path = vault_layout::addons_dir(&vault.path).join("registry.json");
  let raw = fs::read_to_string(&path).map_err(|error| format!("Failed to read addon registry: {error}"))?;
  let registry: AddonRegistry = serde_json::from_str(&raw).map_err(|error| format!("Invalid addon registry: {error}"))?;
  if registry.version != REGISTRY_VERSION {
    return Err(format!("Unsupported addon registry version {}", registry.version));
  }
  let record = registry
    .addons
    .get(addon_id)
    .cloned()
    .ok_or_else(|| format!("Unknown addon: {addon_id}"))?;
  if !record.enabled {
    return Err(format!("Addon is disabled: {addon_id}"));
  }
  Ok(record)
}

pub fn canonical_vault_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  let root = PathBuf::from(vault.path);
  fs::create_dir_all(&root).map_err(|error| error.to_string())?;
  fs::canonicalize(root).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_paths_reject_traversal() {
    assert!(normalize_relative_path("../Inbox", "required").is_err());
    assert_eq!(normalize_relative_path("Inbox/2026", "required").unwrap(), "Inbox/2026");
  }

  #[test]
  fn scopes_and_hosts_are_boundary_aware() {
    assert!(scope_matches("Inbox/**", "Inbox/note.md"));
    assert!(!scope_matches("Inbox/**", "Inbox-old/note.md"));
    assert!(host_matches("public-https", "example.com"));
    assert!(host_matches("*.example.com", "api.example.com"));
    assert!(!host_matches("*.example.com", "example.com"));
  }
}
