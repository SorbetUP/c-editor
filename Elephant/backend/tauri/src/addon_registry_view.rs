use serde_json::{Map, Value};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, State};

use crate::addons::{self, AddonState};
use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

fn packages_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  Ok(vault_layout::addons_dir(&vault.path).join("packages"))
}

fn physical_manifest(app: &AppHandle, addon_id: &str) -> R<Map<String, Value>> {
  let path = packages_root(app)?.join(addon_id).join("manifest.json");
  let raw = fs::read_to_string(&path)
    .map_err(|error| format!("Failed to read installed addon manifest for {addon_id}: {error}"))?;
  let manifest: Value = serde_json::from_str(&raw)
    .map_err(|error| format!("Invalid installed addon manifest for {addon_id}: {error}"))?;
  let object = manifest
    .as_object()
    .cloned()
    .ok_or_else(|| format!("Installed addon manifest must be an object: {addon_id}"))?;
  if object.get("id").and_then(Value::as_str) != Some(addon_id) {
    return Err(format!("Installed addon manifest id mismatch: {addon_id}"));
  }
  Ok(object)
}

#[tauri::command]
pub fn tauri_addons_list_full(
  app: AppHandle,
  state: State<'_, AddonState>,
) -> R<Vec<Value>> {
  let records = addons::tauri_addons_list(app.clone(), state)?;
  let mut output = Vec::with_capacity(records.len());

  for record in records {
    let addon_id = record.manifest.id.clone();
    let mut value = serde_json::to_value(&record).map_err(|error| error.to_string())?;
    let record_object = value
      .as_object_mut()
      .ok_or_else(|| format!("Installed addon record must be an object: {addon_id}"))?;
    record_object.insert("manifest".to_string(), Value::Object(physical_manifest(&app, &addon_id)?));
    output.push(value);
  }

  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn manifest_merge_preserves_package_only_fields() {
    let mut manifest = Map::new();
    manifest.insert("requires".to_string(), serde_json::json!({ "elephant.ai": ">=2.0.0" }));
    manifest.insert("native".to_string(), serde_json::json!({ "protocol": "elephant-addon-sidecar-v1" }));
    assert!(manifest.contains_key("requires"));
    assert!(manifest.contains_key("native"));
  }
}
