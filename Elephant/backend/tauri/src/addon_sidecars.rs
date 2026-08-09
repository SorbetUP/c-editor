use serde::Serialize;
use serde_json::{json, Value};
use std::{
  fs,
  path::{Component, Path, PathBuf},
  process::Stdio,
  sync::Mutex,
};
use tauri::{AppHandle, State};
use tokio::{
  io::AsyncWriteExt,
  process::Command,
  time::{timeout, Duration},
};

use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

const SIDECAR_PROTOCOL: &str = "elephant-addon-sidecar-v1";
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_TIMEOUT_MS: u64 = 120_000;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub struct AddonSidecarState {
  lock: Mutex<()>,
}

impl AddonSidecarState {
  pub fn new() -> Self {
    Self::default()
  }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonSidecarStatus {
  pub available: bool,
  pub addon_id: String,
  pub platform: String,
  pub relative_path: String,
  pub error: Option<String>,
}

fn addons_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  Ok(vault_layout::addons_dir(&vault.path))
}

fn registry_path(app: &AppHandle) -> R<PathBuf> {
  Ok(addons_root(app)?.join("registry.json"))
}

fn package_dir(app: &AppHandle, addon_id: &str) -> R<PathBuf> {
  Ok(addons_root(app)?.join("packages").join(addon_id))
}

fn data_dir(app: &AppHandle, addon_id: &str) -> R<PathBuf> {
  let path = addons_root(app)?.join("data").join(addon_id);
  fs::create_dir_all(&path).map_err(|error| error.to_string())?;
  Ok(path)
}

fn read_registry(app: &AppHandle) -> R<Value> {
  let path = registry_path(app)?;
  if !path.exists() {
    return Ok(json!({ "version": 1, "addons": {} }));
  }
  let raw = fs::read_to_string(path).map_err(|error| format!("Failed to read addon registry: {error}"))?;
  serde_json::from_str(&raw).map_err(|error| format!("Invalid addon registry: {error}"))
}

fn read_package_manifest(app: &AppHandle, addon_id: &str) -> R<Value> {
  let path = package_dir(app, addon_id)?.join("manifest.json");
  let raw = fs::read_to_string(&path)
    .map_err(|error| format!("Failed to read installed addon manifest for {addon_id}: {error}"))?;
  let manifest: Value = serde_json::from_str(&raw)
    .map_err(|error| format!("Invalid installed addon manifest for {addon_id}: {error}"))?;
  if manifest.get("id").and_then(Value::as_str) != Some(addon_id) {
    return Err(format!("Installed addon manifest id mismatch: {addon_id}"));
  }
  Ok(manifest)
}

fn safe_relative_path(value: &str) -> R<PathBuf> {
  let path = Path::new(value);
  if value.trim().is_empty() || path.is_absolute() {
    return Err("A safe relative sidecar path is required".to_string());
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
    return Err("A safe relative sidecar path is required".to_string());
  }
  Ok(normalized)
}

fn platform_key() -> String {
  let os = match std::env::consts::OS {
    "macos" => "macos",
    "windows" => "windows",
    "linux" => "linux",
    "android" => "android",
    "ios" => "ios",
    other => other,
  };
  let arch = match std::env::consts::ARCH {
    "aarch64" => "aarch64",
    "x86_64" => "x86_64",
    "arm" => "armv7",
    "x86" => "i686",
    other => other,
  };
  format!("{os}-{arch}")
}

fn process_sidecars_supported_on(os: &str) -> bool {
  !matches!(os, "android" | "ios")
}

fn ensure_process_sidecar_supported() -> R<()> {
  if process_sidecars_supported_on(std::env::consts::OS) {
    Ok(())
  } else {
    Err(format!(
      "Process addon sidecars are not supported on {}. This addon requires a package-owned Android or iOS host adapter.",
      std::env::consts::OS
    ))
  }
}

fn ensure_installed_and_enabled(registry: &Value, addon_id: &str) -> R<()> {
  let record = registry
    .get("addons")
    .and_then(Value::as_object)
    .and_then(|addons| addons.get(addon_id))
    .ok_or_else(|| format!("Unknown addon: {addon_id}"))?;
  if record.get("enabled").and_then(Value::as_bool) != Some(true) {
    return Err(format!("Addon is disabled: {addon_id}"));
  }
  Ok(())
}

fn resolve_sidecar(app: &AppHandle, addon_id: &str) -> R<(PathBuf, String, String)> {
  let registry = read_registry(app)?;
  ensure_installed_and_enabled(&registry, addon_id)?;
  let manifest = read_package_manifest(app, addon_id)?;
  let native_allowed = manifest
    .get("permissions")
    .and_then(|permissions| permissions.get("native"))
    .and_then(Value::as_bool)
    == Some(true);
  if !native_allowed {
    return Err(format!("Addon native permission was not granted: {addon_id}"));
  }
  ensure_process_sidecar_supported()?;

  let platform = platform_key();
  let sidecars = manifest
    .get("native")
    .and_then(|native| native.get("sidecars"))
    .and_then(Value::as_object)
    .ok_or_else(|| format!("Addon does not declare native sidecars: {addon_id}"))?;
  let relative = sidecars
    .get(&platform)
    .or_else(|| sidecars.get(std::env::consts::OS))
    .and_then(Value::as_str)
    .ok_or_else(|| format!("Addon has no sidecar for platform {platform}: {addon_id}"))?;
  let relative_path = safe_relative_path(relative)?;

  let root = package_dir(app, addon_id)?;
  let canonical_root = fs::canonicalize(&root)
    .map_err(|error| format!("Addon package directory is unavailable for {addon_id}: {error}"))?;
  let candidate = root.join(&relative_path);
  let canonical = fs::canonicalize(&candidate)
    .map_err(|error| format!("Addon sidecar is unavailable for {addon_id}: {error}"))?;
  if !canonical.starts_with(&canonical_root) {
    return Err(format!("Addon sidecar escapes its package directory: {addon_id}"));
  }
  let metadata = fs::metadata(&canonical).map_err(|error| error.to_string())?;
  if !metadata.is_file() {
    return Err(format!("Addon sidecar is not a regular file: {addon_id}"));
  }

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = metadata.permissions();
    let mode = permissions.mode();
    if mode & 0o100 == 0 {
      permissions.set_mode(mode | 0o700);
      fs::set_permissions(&canonical, permissions)
        .map_err(|error| format!("Failed to mark addon sidecar executable: {error}"))?;
    }
  }

  Ok((canonical, relative_path.to_string_lossy().replace('\\', "/"), platform))
}

#[tauri::command]
pub fn tauri_addons_sidecar_status(
  app: AppHandle,
  state: State<'_, AddonSidecarState>,
  addon_id: String,
) -> AddonSidecarStatus {
  let _guard = state.lock.lock().ok();
  match resolve_sidecar(&app, &addon_id) {
    Ok((_path, relative_path, platform)) => AddonSidecarStatus {
      available: true,
      addon_id,
      platform,
      relative_path,
      error: None,
    },
    Err(error) => AddonSidecarStatus {
      available: false,
      addon_id,
      platform: platform_key(),
      relative_path: String::new(),
      error: Some(error),
    },
  }
}

#[tauri::command]
pub async fn tauri_addons_sidecar_call(
  app: AppHandle,
  state: State<'_, AddonSidecarState>,
  addon_id: String,
  method: String,
  params: Option<Value>,
  timeout_ms: Option<u64>,
) -> R<Value> {
  if method.trim().is_empty() || method.len() > 128 {
    return Err("A valid addon sidecar method is required".to_string());
  }

  let (sidecar, _relative_path, platform) = {
    let _guard = state.lock.lock().map_err(|_| "Addon sidecar lock is poisoned".to_string())?;
    resolve_sidecar(&app, &addon_id)?
  };
  let package = package_dir(&app, &addon_id)?;
  let data = data_dir(&app, &addon_id)?;
  let request = json!({
    "protocol": SIDECAR_PROTOCOL,
    "addonId": addon_id,
    "platform": platform,
    "method": method,
    "params": params.unwrap_or_else(|| json!({}))
  });
  let request_bytes = serde_json::to_vec(&request).map_err(|error| error.to_string())?;

  let mut command = Command::new(&sidecar);
  command
    .current_dir(&package)
    .env("ELEPHANT_ADDON_ID", request.get("addonId").and_then(Value::as_str).unwrap_or(""))
    .env("ELEPHANT_ADDON_PACKAGE_DIR", &package)
    .env("ELEPHANT_ADDON_DATA_DIR", &data)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);

  let mut child = command.spawn().map_err(|error| format!("Failed to start addon sidecar: {error}"))?;
  if let Some(mut stdin) = child.stdin.take() {
    stdin.write_all(&request_bytes).await.map_err(|error| format!("Failed to write addon sidecar request: {error}"))?;
    stdin.shutdown().await.map_err(|error| format!("Failed to close addon sidecar stdin: {error}"))?;
  }

  let timeout_ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS).clamp(1, MAX_TIMEOUT_MS);
  let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
    .await
    .map_err(|_| format!("Addon sidecar timed out after {timeout_ms} ms"))?
    .map_err(|error| format!("Addon sidecar failed: {error}"))?;

  if output.stdout.len() > MAX_RESPONSE_BYTES || output.stderr.len() > MAX_RESPONSE_BYTES {
    return Err("Addon sidecar response exceeded the 16 MiB limit".to_string());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  if !output.status.success() {
    return Err(if stderr.is_empty() {
      format!("Addon sidecar exited with status {}", output.status)
    } else {
      format!("Addon sidecar failed: {stderr}")
    });
  }

  let response: Value = serde_json::from_slice(&output.stdout)
    .map_err(|error| format!("Addon sidecar returned invalid JSON: {error}; stderr={stderr}"))?;
  if response.get("protocol").and_then(Value::as_str) != Some(SIDECAR_PROTOCOL) {
    return Err("Addon sidecar returned an unsupported protocol".to_string());
  }
  if response.get("ok").and_then(Value::as_bool) == Some(true) {
    return Ok(response.get("result").cloned().unwrap_or(Value::Null));
  }
  let error = response
    .get("error")
    .and_then(|value| value.get("message").or(Some(value)))
    .and_then(Value::as_str)
    .unwrap_or("Addon sidecar returned an unknown error");
  Err(error.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_sidecar_path_traversal() {
    assert!(safe_relative_path("../outside").is_err());
    assert!(safe_relative_path("native/linux-x86_64/sidecar").is_ok());
  }

  #[test]
  fn process_sidecars_are_explicitly_desktop_only() {
    assert!(process_sidecars_supported_on("linux"));
    assert!(process_sidecars_supported_on("macos"));
    assert!(process_sidecars_supported_on("windows"));
    assert!(!process_sidecars_supported_on("android"));
    assert!(!process_sidecars_supported_on("ios"));
  }

  #[test]
  fn exposes_a_stable_platform_key() {
    let key = platform_key();
    assert!(key.contains('-'));
    assert!(!key.contains(' '));
  }
}
