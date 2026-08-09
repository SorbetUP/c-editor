use serde::Serialize;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

use crate::addon_runtime_access::{
  canonical_vault_root,
  normalize_relative_path,
  read_enabled_addon,
  scope_matches,
};

type R<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonAssetDirectory {
  pub relative_path: String,
  pub path: String,
}

fn resolve_requested_directory(app: &AppHandle, addon_id: &str, requested: &str) -> R<(String, PathBuf)> {
  let record = read_enabled_addon(app, addon_id)?;
  let root = canonical_vault_root(app)?;
  let requested = requested.trim();

  if requested.is_empty() || requested == "." {
    if !record
      .manifest
      .permissions
      .notes
      .read
      .iter()
      .any(|scope| scope.trim() == "*")
    {
      return Err("Addon is not permitted to expose the vault root through the asset protocol".to_string());
    }
    return Ok((String::new(), root));
  }

  let relative_path = normalize_relative_path(requested, "An asset directory inside the active vault is required")?;
  if !record
    .manifest
    .permissions
    .notes
    .read
    .iter()
    .any(|scope| scope_matches(scope, &relative_path))
  {
    return Err(format!("Addon is not permitted to expose assets under {relative_path}"));
  }

  let canonical = fs::canonicalize(root.join(&relative_path))
    .map_err(|error| format!("Asset directory is unavailable: {error}"))?;
  if !canonical.starts_with(&root) || !canonical.is_dir() {
    return Err("Asset directory must stay inside the active vault".to_string());
  }
  Ok((relative_path, canonical))
}

#[tauri::command]
pub fn tauri_addons_assets_allow_directory(
  app: AppHandle,
  addon_id: String,
  relative_path: String,
) -> R<AddonAssetDirectory> {
  let (relative_path, directory) = resolve_requested_directory(&app, &addon_id, &relative_path)?;
  app
    .asset_protocol_scope()
    .allow_directory(&directory, true)
    .map_err(|error| format!("Failed to authorize addon asset directory: {error}"))?;

  Ok(AddonAssetDirectory {
    relative_path,
    path: directory.to_string_lossy().to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn asset_directory_payload_is_platform_neutral() {
    let payload = AddonAssetDirectory {
      relative_path: "Sites/demo".to_string(),
      path: "/vault/Sites/demo".to_string(),
    };
    let value = serde_json::to_value(payload).unwrap();
    assert_eq!(value.get("relativePath").and_then(|value| value.as_str()), Some("Sites/demo"));
    assert_eq!(value.get("path").and_then(|value| value.as_str()), Some("/vault/Sites/demo"));
  }
}
