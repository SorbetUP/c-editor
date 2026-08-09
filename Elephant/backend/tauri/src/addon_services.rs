use serde::Serialize;
use serde_json::{json, Value};
#[cfg(mobile)]
use std::collections::HashSet;
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, State};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::{timeout, Duration},
};

#[cfg(mobile)]
use crate::embedded_addon_services;
use crate::{addon_runtime_access, vault::config as vault_config, vault_layout};

type R<T> = Result<T, String>;

const SERVICE_PROTOCOL: &str = "elephant-addon-service-v1";
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const MAX_TIMEOUT_MS: u64 = 30 * 60 * 1_000;
const START_TIMEOUT_MS: u64 = 20_000;
const STOP_TIMEOUT_MS: u64 = 5_000;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

struct ServiceProcess {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    platform: String,
    relative_path: String,
}

impl ServiceProcess {
    fn running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

pub struct AddonServiceState {
    services: Mutex<HashMap<String, ServiceProcess>>,
    #[cfg(mobile)]
    embedded: Mutex<HashSet<String>>,
    next_id: AtomicU64,
}

impl AddonServiceState {
    pub fn new() -> Self {
        Self {
            services: Mutex::new(HashMap::new()),
            #[cfg(mobile)]
            embedded: Mutex::new(HashSet::new()),
            next_id: AtomicU64::new(1),
        }
    }

    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for AddonServiceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonServiceStatus {
    running: bool,
    addon_id: String,
    platform: String,
    relative_path: String,
    protocol: &'static str,
    error: Option<String>,
}

struct ResolvedService {
    executable: PathBuf,
    package_dir: PathBuf,
    data_dir: PathBuf,
    vault_dir: PathBuf,
    platform: String,
    relative_path: String,
}

#[cfg(mobile)]
#[derive(Debug, Clone)]
struct ResolvedEmbeddedService {
    host: String,
    vault_dir: PathBuf,
    platform: String,
    relative_path: String,
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

fn services_supported_on(os: &str) -> bool {
    !matches!(os, "android" | "ios")
}

fn safe_relative_path(value: &str) -> R<PathBuf> {
    let input = Path::new(value.trim());
    if value.trim().is_empty() || input.is_absolute() {
        return Err("A relative addon service path is required".to_string());
    }
    let mut result = PathBuf::new();
    for component in input.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Path traversal is not allowed: {value}"));
            }
        }
    }
    if result.as_os_str().is_empty() {
        return Err("A relative addon service path is required".to_string());
    }
    Ok(result)
}

fn installed_manifest(package_dir: &Path, addon_id: &str) -> R<Value> {
    let raw = fs::read_to_string(package_dir.join("manifest.json")).map_err(|error| {
        format!("Failed to read installed addon manifest for {addon_id}: {error}")
    })?;
    let manifest: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Invalid installed addon manifest for {addon_id}: {error}"))?;
    if manifest.get("id").and_then(Value::as_str) != Some(addon_id) {
        return Err(format!("Installed addon manifest id mismatch: {addon_id}"));
    }
    Ok(manifest)
}

fn embedded_mobile_descriptor(manifest: &Value, os: &str) -> R<(String, String)> {
    let descriptor = manifest
        .pointer(&format!("/native/mobile/{os}"))
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Addon has no embedded service declaration for {os}"))?;
    if descriptor.get("supported").and_then(Value::as_bool) != Some(true) {
        return Err(format!("Addon embedded service is disabled for {os}"));
    }
    if descriptor.get("runner").and_then(Value::as_str) != Some("embedded-rust") {
        return Err(format!(
            "Addon does not declare the embedded Rust runner for {os}"
        ));
    }
    let host = descriptor
        .get("host")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Addon embedded service host is missing for {os}"))?;
    Ok((host.to_string(), format!("embedded://{host}")))
}

#[cfg(mobile)]
fn resolve_embedded_service(app: &AppHandle, addon_id: &str) -> R<ResolvedEmbeddedService> {
    let record = addon_runtime_access::read_enabled_addon(app, addon_id)?;
    if record.source != "official" {
        return Err(format!(
            "Only an official addon may use a statically embedded service host: {addon_id}"
        ));
    }

    let vault = vault_config::get_active_vault(app)?;
    let vault_dir = fs::canonicalize(&vault.path)
        .map_err(|error| format!("Active vault is unavailable: {error}"))?;
    let package_dir = vault_layout::addons_dir(&vault.path)
        .join("packages")
        .join(addon_id);
    let manifest = installed_manifest(&package_dir, addon_id)?;
    if manifest
        .pointer("/permissions/native")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err(format!(
            "Addon native permission was not granted: {addon_id}"
        ));
    }
    if manifest.pointer("/native/runner").and_then(Value::as_str) != Some("service") {
        return Err(format!(
            "Addon does not declare the service runner: {addon_id}"
        ));
    }

    let (host, relative_path) = embedded_mobile_descriptor(&manifest, std::env::consts::OS)?;
    if !embedded_addon_services::supports(&host) {
        return Err(format!(
            "This application build does not provide embedded host {host}"
        ));
    }
    Ok(ResolvedEmbeddedService {
        host,
        vault_dir,
        platform: platform_key(),
        relative_path,
    })
}

fn resolve_service(app: &AppHandle, addon_id: &str) -> R<ResolvedService> {
    if !services_supported_on(std::env::consts::OS) {
        return Err("Persistent process services require a desktop addon package".to_string());
    }
    addon_runtime_access::read_enabled_addon(app, addon_id)?;

    let vault = vault_config::get_active_vault(app)?;
    let vault_dir = fs::canonicalize(&vault.path)
        .map_err(|error| format!("Active vault is unavailable: {error}"))?;
    let addons_dir = vault_layout::addons_dir(&vault.path);
    let package_dir = addons_dir.join("packages").join(addon_id);
    let data_dir = addons_dir.join("data").join(addon_id);
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;

    let manifest = installed_manifest(&package_dir, addon_id)?;
    let native_permission = manifest
        .pointer("/permissions/native")
        .and_then(Value::as_bool)
        == Some(true);
    if !native_permission {
        return Err(format!(
            "Addon native permission was not granted: {addon_id}"
        ));
    }

    let native = manifest
        .get("native")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Addon does not declare a native runtime: {addon_id}"))?;
    if native.get("runner").and_then(Value::as_str) != Some("service") {
        return Err(format!(
            "Addon does not declare the persistent service runner: {addon_id}"
        ));
    }

    let platform = platform_key();
    let relative = native
        .get("sidecars")
        .and_then(Value::as_object)
        .and_then(|sidecars| {
            sidecars
                .get(&platform)
                .or_else(|| sidecars.get(std::env::consts::OS))
        })
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Addon has no service executable for {platform}: {addon_id}"))?;
    let relative_path = safe_relative_path(relative)?;
    let canonical_package = fs::canonicalize(&package_dir).map_err(|error| {
        format!("Addon package directory is unavailable for {addon_id}: {error}")
    })?;
    let executable_path = package_dir.join(&relative_path);
    let executable = fs::canonicalize(&executable_path).map_err(|error| {
        format!(
            "Addon service executable is unavailable for {addon_id}: {} ({error})",
            executable_path.display()
        )
    })?;
    if !executable.starts_with(&canonical_package) {
        return Err(format!(
            "Addon service executable escapes its package: {addon_id}"
        ));
    }
    let metadata = fs::metadata(&executable).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err(format!(
            "Addon service executable is not a file: {addon_id}"
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o700);
        fs::set_permissions(&executable, permissions)
            .map_err(|error| format!("Failed to mark addon service executable: {error}"))?;
    }

    Ok(ResolvedService {
        executable,
        package_dir: canonical_package,
        data_dir,
        vault_dir,
        platform,
        relative_path: relative_path.to_string_lossy().replace('\\', "/"),
    })
}

async fn exchange(
    process: &mut ServiceProcess,
    addon_id: &str,
    id: u64,
    method: &str,
    params: Value,
    timeout_ms: u64,
) -> R<Value> {
    let mut request = serde_json::to_vec(&json!({
      "protocol": SERVICE_PROTOCOL,
      "id": id,
      "addonId": addon_id,
      "method": method,
      "params": params,
    }))
    .map_err(|error| error.to_string())?;
    request.push(b'\n');
    process
        .stdin
        .write_all(&request)
        .await
        .map_err(|error| error.to_string())?;
    process
        .stdin
        .flush()
        .await
        .map_err(|error| error.to_string())?;

    let wait_ms = timeout_ms.clamp(1, MAX_TIMEOUT_MS);
    let mut line = String::new();
    let bytes = timeout(
        Duration::from_millis(wait_ms),
        process.stdout.read_line(&mut line),
    )
    .await
    .map_err(|_| format!("Addon service timed out after {wait_ms} ms"))?
    .map_err(|error| error.to_string())?;
    if bytes == 0 {
        return Err("Addon service stopped before responding".to_string());
    }
    if line.len() > MAX_RESPONSE_BYTES {
        return Err("Addon service response exceeded 16 MiB".to_string());
    }

    let response: Value = serde_json::from_str(line.trim_end())
        .map_err(|error| format!("Addon service returned invalid JSON: {error}"))?;
    if response.get("protocol").and_then(Value::as_str) != Some(SERVICE_PROTOCOL)
        || response.get("id").and_then(Value::as_u64) != Some(id)
    {
        return Err("Addon service returned an invalid response envelope".to_string());
    }
    if response.get("ok").and_then(Value::as_bool) == Some(true) {
        return Ok(response.get("result").cloned().unwrap_or(Value::Null));
    }
    Err(response
        .pointer("/error/message")
        .or_else(|| response.get("error"))
        .and_then(Value::as_str)
        .unwrap_or("Addon service returned an unknown error")
        .to_string())
}

#[cfg(mobile)]
async fn exchange_embedded(
    resolved: ResolvedEmbeddedService,
    method: String,
    params: Value,
    timeout_ms: u64,
) -> R<Value> {
    let wait_ms = timeout_ms.clamp(1, MAX_TIMEOUT_MS);
    let host = resolved.host;
    let vault_dir = resolved.vault_dir;
    let task = tokio::task::spawn_blocking(move || {
        embedded_addon_services::call(&host, &vault_dir, &method, params)
    });
    timeout(Duration::from_millis(wait_ms), task)
        .await
        .map_err(|_| format!("Embedded addon service timed out after {wait_ms} ms"))?
        .map_err(|error| format!("Embedded addon service task failed: {error}"))?
}

async fn spawn_service(app: &AppHandle, addon_id: &str, id: u64) -> R<ServiceProcess> {
    let resolved = resolve_service(app, addon_id)?;
    let mut child = Command::new(&resolved.executable)
        .current_dir(&resolved.package_dir)
        .env("ELEPHANT_ADDON_ID", addon_id)
        .env("ELEPHANT_ADDON_PACKAGE_DIR", &resolved.package_dir)
        .env("ELEPHANT_ADDON_DATA_DIR", &resolved.data_dir)
        .env("ELEPHANT_VAULT_DIR", &resolved.vault_dir)
        .env("ELEPHANT_ADDON_SERVICE_PROTOCOL", SERVICE_PROTOCOL)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("Failed to start addon service: {error}"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Addon service stdin is unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Addon service stdout is unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Addon service stderr is unavailable".to_string())?;
    spawn_service_log_reader(addon_id, stderr);
    let mut process = ServiceProcess {
        child,
        stdin: BufWriter::new(stdin),
        stdout: BufReader::new(stdout),
        platform: resolved.platform,
        relative_path: resolved.relative_path,
    };
    exchange(
        &mut process,
        addon_id,
        id,
        "service.start",
        json!({}),
        START_TIMEOUT_MS,
    )
    .await?;
    Ok(process)
}

fn spawn_service_log_reader(addon_id: &str, stderr: ChildStderr) {
    let addon_id = addon_id.to_string();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => eprintln!("[addon-service:{addon_id}] {line}"),
                Ok(None) => break,
                Err(error) => {
                    eprintln!("[addon-service:{addon_id}] stderr read failed: {error}");
                    break;
                }
            }
        }
    });
}

#[tauri::command]
pub async fn tauri_addons_service_status(
    app: AppHandle,
    state: State<'_, AddonServiceState>,
    addon_id: String,
) -> R<AddonServiceStatus> {
    #[cfg(mobile)]
    {
        return Ok(match resolve_embedded_service(&app, &addon_id) {
            Ok(resolved) => AddonServiceStatus {
                running: state.embedded.lock().await.contains(&addon_id),
                addon_id,
                platform: resolved.platform,
                relative_path: resolved.relative_path,
                protocol: SERVICE_PROTOCOL,
                error: None,
            },
            Err(error) => AddonServiceStatus {
                running: false,
                addon_id,
                platform: platform_key(),
                relative_path: String::new(),
                protocol: SERVICE_PROTOCOL,
                error: Some(error),
            },
        });
    }

    #[cfg(not(mobile))]
    {
        let mut services = state.services.lock().await;
        if let Some(process) = services.get_mut(&addon_id) {
            if process.running() {
                return Ok(AddonServiceStatus {
                    running: true,
                    addon_id,
                    platform: process.platform.clone(),
                    relative_path: process.relative_path.clone(),
                    protocol: SERVICE_PROTOCOL,
                    error: None,
                });
            }
            services.remove(&addon_id);
        }

        Ok(match resolve_service(&app, &addon_id) {
            Ok(resolved) => AddonServiceStatus {
                running: false,
                addon_id,
                platform: resolved.platform,
                relative_path: resolved.relative_path,
                protocol: SERVICE_PROTOCOL,
                error: None,
            },
            Err(error) => AddonServiceStatus {
                running: false,
                addon_id,
                platform: platform_key(),
                relative_path: String::new(),
                protocol: SERVICE_PROTOCOL,
                error: Some(error),
            },
        })
    }
}

#[tauri::command]
pub async fn tauri_addons_service_start(
    app: AppHandle,
    state: State<'_, AddonServiceState>,
    addon_id: String,
) -> R<Value> {
    #[cfg(mobile)]
    {
        let resolved = resolve_embedded_service(&app, &addon_id)?;
        exchange_embedded(
            resolved.clone(),
            "service.start".to_string(),
            json!({}),
            START_TIMEOUT_MS,
        )
        .await?;
        state.embedded.lock().await.insert(addon_id);
        return Ok(json!({
          "running": true,
          "platform": resolved.platform,
          "relativePath": resolved.relative_path,
          "protocol": SERVICE_PROTOCOL,
          "mode": "embedded-rust",
        }));
    }

    #[cfg(not(mobile))]
    {
        let mut services = state.services.lock().await;
        if let Some(process) = services.get_mut(&addon_id) {
            if process.running() {
                return Ok(json!({
                  "running": true,
                  "platform": process.platform.clone(),
                  "relativePath": process.relative_path.clone(),
                  "protocol": SERVICE_PROTOCOL,
                }));
            }
            services.remove(&addon_id);
        }

        let process = spawn_service(&app, &addon_id, state.request_id()).await?;
        let result = json!({
          "running": true,
          "platform": process.platform.clone(),
          "relativePath": process.relative_path.clone(),
          "protocol": SERVICE_PROTOCOL,
        });
        services.insert(addon_id, process);
        Ok(result)
    }
}

#[tauri::command]
pub async fn tauri_addons_service_call(
    app: AppHandle,
    state: State<'_, AddonServiceState>,
    addon_id: String,
    method: String,
    params: Option<Value>,
    timeout_ms: Option<u64>,
) -> R<Value> {
    let method = method.trim().to_string();
    if method.is_empty() || method.len() > 128 || method.starts_with("service.") {
        return Err("A valid addon service method is required".to_string());
    }

    #[cfg(mobile)]
    {
        let resolved = resolve_embedded_service(&app, &addon_id)?;
        if !state.embedded.lock().await.contains(&addon_id) {
            exchange_embedded(
                resolved.clone(),
                "service.start".to_string(),
                json!({}),
                START_TIMEOUT_MS,
            )
            .await?;
            state.embedded.lock().await.insert(addon_id.clone());
        }
        return exchange_embedded(
            resolved,
            method,
            params.unwrap_or_else(|| json!({})),
            timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
        )
        .await;
    }

    #[cfg(not(mobile))]
    {
        let mut services = state.services.lock().await;
        let restart = services
            .get_mut(&addon_id)
            .map(|process| !process.running())
            .unwrap_or(true);
        if restart {
            services.remove(&addon_id);
            let process = spawn_service(&app, &addon_id, state.request_id()).await?;
            services.insert(addon_id.clone(), process);
        }

        let result = {
            let process = services
                .get_mut(&addon_id)
                .ok_or_else(|| "Addon service failed to start".to_string())?;
            exchange(
                process,
                &addon_id,
                state.request_id(),
                &method,
                params.unwrap_or_else(|| json!({})),
                timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
            )
            .await
        };
        if services
            .get_mut(&addon_id)
            .map(|process| !process.running())
            .unwrap_or(true)
        {
            services.remove(&addon_id);
        }
        result
    }
}

#[tauri::command]
pub async fn tauri_addons_service_stop(
    state: State<'_, AddonServiceState>,
    addon_id: String,
) -> R<Value> {
    #[cfg(mobile)]
    {
        let stopped = state.embedded.lock().await.remove(&addon_id);
        return Ok(json!({ "running": false, "stopped": stopped, "mode": "embedded-rust" }));
    }

    #[cfg(not(mobile))]
    {
        let process = state.services.lock().await.remove(&addon_id);
        let Some(mut process) = process else {
            return Ok(json!({ "running": false, "stopped": false }));
        };

        let _ = exchange(
            &mut process,
            &addon_id,
            state.request_id(),
            "service.stop",
            json!({}),
            STOP_TIMEOUT_MS,
        )
        .await;
        if !matches!(
            timeout(Duration::from_millis(STOP_TIMEOUT_MS), process.child.wait()).await,
            Ok(Ok(_))
        ) {
            let _ = process.child.kill().await;
            let _ = process.child.wait().await;
        }
        Ok(json!({ "running": false, "stopped": true }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_paths_reject_traversal() {
        assert!(safe_relative_path("../outside").is_err());
        assert!(safe_relative_path("native/linux-x86_64/service").is_ok());
    }

    #[test]
    fn process_services_are_desktop_only() {
        assert!(services_supported_on("linux"));
        assert!(services_supported_on("macos"));
        assert!(services_supported_on("windows"));
        assert!(!services_supported_on("android"));
        assert!(!services_supported_on("ios"));
    }

    #[test]
    fn embedded_mobile_service_declarations_are_explicit_and_versioned() {
        let manifest = json!({
          "native": {
            "mobile": {
              "android": {
                "supported": true,
                "runner": "embedded-rust",
                "host": "elephant-knowledge-v1"
              }
            }
          }
        });
        assert_eq!(
            embedded_mobile_descriptor(&manifest, "android").unwrap(),
            (
                "elephant-knowledge-v1".to_string(),
                "embedded://elephant-knowledge-v1".to_string()
            )
        );
        assert!(embedded_mobile_descriptor(&manifest, "ios").is_err());
    }

    #[test]
    fn protocol_and_limits_are_versioned() {
        assert_eq!(SERVICE_PROTOCOL, "elephant-addon-service-v1");
        assert_eq!(MAX_RESPONSE_BYTES, 16 * 1024 * 1024);
        assert_eq!(MAX_TIMEOUT_MS, 30 * 60 * 1_000);
    }
}
