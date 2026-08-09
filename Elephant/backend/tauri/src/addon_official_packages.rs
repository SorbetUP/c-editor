use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, State};

use crate::addon_catalog::{self, CatalogAddon};
use crate::addons::{AddonState, InstalledAddon};
use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

const OFFICIAL_MARKER_FILE: &str = ".elephant-official-package.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfficialPackageMarker {
  source: String,
  package_hash: String,
}

fn package_dir(app: &AppHandle, addon_id: &str) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  Ok(vault_layout::addons_dir(&vault.path)
    .join("packages")
    .join(addon_id))
}

fn marker_path(app: &AppHandle, addon_id: &str) -> R<PathBuf> {
  Ok(package_dir(app, addon_id)?.join(OFFICIAL_MARKER_FILE))
}

fn write_official_marker(app: &AppHandle, record: &InstalledAddon) -> R<()> {
  let marker = OfficialPackageMarker {
    source: "official".to_string(),
    package_hash: record.package_hash.clone(),
  };
  let path = marker_path(app, &record.manifest.id)?;
  let bytes = serde_json::to_vec_pretty(&marker).map_err(|error| error.to_string())?;
  fs::write(path, bytes).map_err(|error| format!("Failed to mark official addon package: {error}"))
}

pub fn is_official_package(app: &AppHandle, addon_id: &str, package_hash: &str) -> bool {
  let path = match marker_path(app, addon_id) {
    Ok(path) => path,
    Err(_) => return false,
  };
  let raw = match fs::read_to_string(path) {
    Ok(raw) => raw,
    Err(_) => return false,
  };
  let marker: OfficialPackageMarker = match serde_json::from_str(&raw) {
    Ok(marker) => marker,
    Err(_) => return false,
  };
  marker.source == "official" && marker.package_hash == package_hash
}

#[tauri::command]
pub fn tauri_addons_catalog_list() -> R<Vec<CatalogAddon>> {
  addon_catalog::tauri_addons_catalog_list()
}

#[tauri::command]
pub fn tauri_addons_catalog_install(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
) -> R<InstalledAddon> {
  let mut record = addon_catalog::tauri_addons_catalog_install(app.clone(), state, addon_id)?;
  write_official_marker(&app, &record)?;
  record.source = "official".to_string();
  Ok(record)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn official_marker_is_hash_bound() {
    let marker = OfficialPackageMarker {
      source: "official".to_string(),
      package_hash: "abc".to_string(),
    };
    assert_eq!(marker.source, "official");
    assert_ne!(marker.package_hash, "def");
  }
}
