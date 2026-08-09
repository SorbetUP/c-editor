use semver::{Version, VersionReq};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};
use tauri::{AppHandle, State};

use crate::addons::{self, AddonState, InstalledAddon};
use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

fn addons_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  Ok(vault_layout::addons_dir(&vault.path))
}

fn registry(app: &AppHandle) -> R<Value> {
  let path = addons_root(app)?.join("registry.json");
  if !path.exists() {
    return Ok(serde_json::json!({ "version": 1, "addons": {} }));
  }
  let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
  serde_json::from_str(&raw).map_err(|error| format!("Invalid addon registry: {error}"))
}

fn package_manifest(app: &AppHandle, addon_id: &str) -> R<Value> {
  let path = addons_root(app)?
    .join("packages")
    .join(addon_id)
    .join("manifest.json");
  let raw = fs::read_to_string(&path)
    .map_err(|error| format!("Failed to read manifest for {addon_id}: {error}"))?;
  let manifest: Value = serde_json::from_str(&raw)
    .map_err(|error| format!("Invalid manifest for {addon_id}: {error}"))?;
  if manifest.get("id").and_then(Value::as_str) != Some(addon_id) {
    return Err(format!("Installed addon manifest id mismatch: {addon_id}"));
  }
  Ok(manifest)
}

fn requirements(manifest: &Value) -> BTreeMap<String, String> {
  manifest
    .get("requires")
    .and_then(Value::as_object)
    .map(|entries| {
      entries
        .iter()
        .filter_map(|(id, value)| value.as_str().map(|requirement| (id.clone(), requirement.to_string())))
        .collect()
    })
    .unwrap_or_default()
}

fn records(registry: &Value) -> R<&serde_json::Map<String, Value>> {
  registry
    .get("addons")
    .and_then(Value::as_object)
    .ok_or_else(|| "Addon registry is missing its addons map".to_string())
}

fn record_version(record: &Value) -> R<Version> {
  let value = record
    .get("manifest")
    .and_then(|manifest| manifest.get("version"))
    .and_then(Value::as_str)
    .ok_or_else(|| "Installed addon version is missing".to_string())?;
  Version::parse(value).map_err(|error| format!("Invalid installed addon version {value}: {error}"))
}

pub(crate) fn validate_requirements(app: &AppHandle, registry: &Value, addon_id: &str) -> R<()> {
  let manifest = package_manifest(app, addon_id)?;
  let installed = records(registry)?;
  for (required_id, requirement) in requirements(&manifest) {
    let required = installed
      .get(&required_id)
      .ok_or_else(|| format!("{addon_id} requires missing addon {required_id} {requirement}"))?;
    if required.get("enabled").and_then(Value::as_bool) != Some(true) {
      return Err(format!("{addon_id} requires enabled addon {required_id}"));
    }
    let required_version = record_version(required)?;
    let requirement = VersionReq::parse(&requirement)
      .map_err(|error| format!("Invalid requirement declared by {addon_id} for {required_id}: {error}"))?;
    if !requirement.matches(&required_version) {
      return Err(format!(
        "{addon_id} requires {required_id} {requirement}, installed version is {required_version}"
      ));
    }
  }
  Ok(())
}

pub(crate) fn dependents(app: &AppHandle, registry: &Value, addon_id: &str, enabled_only: bool) -> R<Vec<String>> {
  let mut result = Vec::new();
  for (candidate_id, record) in records(registry)? {
    if candidate_id == addon_id {
      continue;
    }
    if enabled_only && record.get("enabled").and_then(Value::as_bool) != Some(true) {
      continue;
    }
    let manifest = package_manifest(app, candidate_id)?;
    if requirements(&manifest).contains_key(addon_id) {
      result.push(candidate_id.clone());
    }
  }
  result.sort();
  Ok(result)
}

#[tauri::command]
pub fn tauri_addons_set_enabled_checked(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
  enabled: bool,
) -> R<InstalledAddon> {
  let registry = registry(&app)?;
  if enabled {
    validate_requirements(&app, &registry, &addon_id)?;
  } else {
    let active_dependents = dependents(&app, &registry, &addon_id, true)?;
    if !active_dependents.is_empty() {
      return Err(format!(
        "Cannot disable {addon_id}; enabled dependents: {}",
        active_dependents.join(", ")
      ));
    }
  }
  addons::tauri_addons_set_enabled(app, state, addon_id, enabled)
}

#[tauri::command]
pub fn tauri_addons_uninstall_checked(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
) -> R<Value> {
  let registry = registry(&app)?;
  let installed_dependents = dependents(&app, &registry, &addon_id, false)?;
  if !installed_dependents.is_empty() {
    return Err(format!(
      "Cannot uninstall {addon_id}; installed dependents: {}",
      installed_dependents.join(", ")
    ));
  }
  addons::tauri_addons_uninstall(app, state, addon_id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_declared_requirements() {
    let manifest = serde_json::json!({
      "requires": {
        "elephant.ai": ">=2.0.0",
        "elephant.other": "^1.4"
      }
    });
    let parsed = requirements(&manifest);
    assert_eq!(parsed.get("elephant.ai").map(String::as_str), Some(">=2.0.0"));
    assert_eq!(parsed.len(), 2);
  }

  #[test]
  fn semver_requirements_are_strict() {
    let requirement = VersionReq::parse(">=2.0.0").unwrap();
    assert!(requirement.matches(&Version::parse("2.1.0").unwrap()));
    assert!(!requirement.matches(&Version::parse("1.9.9").unwrap()));
  }
}
