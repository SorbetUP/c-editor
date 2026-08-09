use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
  collections::BTreeMap,
  fs,
  io::{self, Read, Write},
  path::{Component, Path, PathBuf},
  sync::Mutex,
  time::Duration,
};
use tauri::{AppHandle, State};
use url::Url;
use zip::ZipArchive;

use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

const ADDON_API_VERSION: u32 = 1;
const MAX_PACKAGE_BYTES: u64 = 25 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 100 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 5 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 512;
const MAX_HTTP_RESPONSE_BYTES: u64 = 5 * 1024 * 1024;
const REGISTRY_VERSION: u32 = 1;

#[derive(Default)]
pub struct AddonState {
  lock: Mutex<()>,
}

impl AddonState {
  pub fn new() -> Self {
    Self::default()
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonRuntime {
  #[serde(rename = "type")]
  pub kind: String,
  pub entry: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotePermissions {
  #[serde(default)]
  pub read: Vec<String>,
  #[serde(default)]
  pub write: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPermissions {
  #[serde(default)]
  pub hosts: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonPermissions {
  #[serde(default)]
  pub notes: NotePermissions,
  #[serde(default)]
  pub network: NetworkPermissions,
  #[serde(default)]
  pub storage: bool,
  #[serde(default)]
  pub commands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonManifest {
  pub id: String,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub author: String,
  #[serde(default = "default_api_version")]
  pub api_version: u32,
  #[serde(default)]
  pub min_app_version: String,
  pub runtime: AddonRuntime,
  #[serde(default)]
  pub permissions: AddonPermissions,
  #[serde(default = "empty_object")]
  pub contributes: Value,
  #[serde(default)]
  pub activation_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
  pub manifest: AddonManifest,
  pub enabled: bool,
  pub package_hash: String,
  pub installed_at: String,
  pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddonRegistry {
  version: u32,
  addons: BTreeMap<String, InstalledAddon>,
}

impl Default for AddonRegistry {
  fn default() -> Self {
    Self {
      version: REGISTRY_VERSION,
      addons: BTreeMap::new(),
    }
  }
}

fn default_api_version() -> u32 {
  ADDON_API_VERSION
}

fn empty_object() -> Value {
  Value::Object(Map::new())
}

fn now() -> String {
  chrono::Utc::now().to_rfc3339()
}

fn addons_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  let root = vault_layout::addons_dir(&vault.path);
  fs::create_dir_all(root.join("packages")).map_err(|error| error.to_string())?;
  fs::create_dir_all(root.join("data")).map_err(|error| error.to_string())?;
  Ok(root)
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

fn read_registry(app: &AppHandle) -> R<AddonRegistry> {
  let path = registry_path(app)?;
  if !path.exists() {
    return Ok(AddonRegistry::default());
  }
  let raw = fs::read_to_string(&path).map_err(|error| format!("Failed to read addon registry: {error}"))?;
  let registry: AddonRegistry = serde_json::from_str(&raw).map_err(|error| format!("Invalid addon registry: {error}"))?;
  if registry.version != REGISTRY_VERSION {
    return Err(format!("Unsupported addon registry version {}", registry.version));
  }
  Ok(registry)
}

fn write_registry(app: &AppHandle, registry: &AddonRegistry) -> R<()> {
  let path = registry_path(app)?;
  let temp = path.with_extension(format!("json.{}.tmp", chrono::Utc::now().timestamp_millis()));
  let raw = serde_json::to_vec_pretty(registry).map_err(|error| error.to_string())?;
  fs::write(&temp, raw).map_err(|error| format!("Failed to write addon registry: {error}"))?;
  fs::rename(&temp, &path).map_err(|error| format!("Failed to replace addon registry: {error}"))
}

fn valid_addon_id(id: &str) -> bool {
  !id.is_empty()
    && id.len() <= 128
    && id.as_bytes()[0].is_ascii_lowercase()
    && id.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
}

fn safe_relative_path(value: &str) -> R<PathBuf> {
  let path = Path::new(value);
  if value.trim().is_empty() || path.is_absolute() {
    return Err("A safe relative path is required".to_string());
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
    return Err("A safe relative path is required".to_string());
  }
  Ok(normalized)
}

fn normalize_note_path(value: &str) -> R<String> {
  let path = safe_relative_path(value)?;
  Ok(path.to_string_lossy().replace('\\', "/"))
}

fn validate_manifest(manifest: &AddonManifest) -> R<()> {
  if !valid_addon_id(&manifest.id) {
    return Err("Addon id must contain lowercase ASCII letters, numbers, dots, dashes or underscores".to_string());
  }
  if manifest.name.trim().is_empty() || manifest.version.trim().is_empty() {
    return Err("Addon name and version are required".to_string());
  }
  if manifest.api_version != ADDON_API_VERSION {
    return Err(format!("Unsupported addon apiVersion {}", manifest.api_version));
  }
  if manifest.runtime.kind != "javascript-worker" {
    return Err(format!("Unsupported addon runtime {}", manifest.runtime.kind));
  }
  let entry = safe_relative_path(&manifest.runtime.entry)?;
  if entry.extension().and_then(|value| value.to_str()) != Some("js") {
    return Err("javascript-worker entry must be a .js file".to_string());
  }
  if !manifest.min_app_version.trim().is_empty() {
    let required = semver::Version::parse(manifest.min_app_version.trim())
      .map_err(|error| format!("Invalid minAppVersion: {error}"))?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
    if current < required {
      return Err(format!("Addon requires ElephantNote {} or newer", required));
    }
  }
  for scope in manifest.permissions.notes.read.iter().chain(manifest.permissions.notes.write.iter()) {
    validate_scope(scope)?;
  }
  for host in &manifest.permissions.network.hosts {
    validate_host_pattern(host)?;
  }
  Ok(())
}

fn validate_scope(scope: &str) -> R<()> {
  let trimmed = scope.trim();
  if trimmed == "*" {
    return Ok(());
  }
  let base = trimmed.strip_suffix("/**").unwrap_or(trimmed);
  safe_relative_path(base).map(|_| ())
}

fn validate_host_pattern(host: &str) -> R<()> {
  let normalized = host.trim().to_ascii_lowercase();
  let candidate = normalized.strip_prefix("*.").unwrap_or(&normalized);
  if candidate.is_empty()
    || candidate.contains('/')
    || candidate.contains(':')
    || candidate.chars().any(|character| !(character.is_ascii_alphanumeric() || character == '.' || character == '-'))
  {
    return Err(format!("Invalid network host permission: {host}"));
  }
  Ok(())
}

fn scope_matches(scope: &str, relative_path: &str) -> bool {
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

fn require_note_scope(scopes: &[String], relative_path: &str, operation: &str) -> R<()> {
  if scopes.iter().any(|scope| scope_matches(scope, relative_path)) {
    Ok(())
  } else {
    Err(format!("Addon is not permitted to {operation} note {relative_path}"))
  }
}

fn host_matches(pattern: &str, host: &str) -> bool {
  let pattern = pattern.trim().to_ascii_lowercase();
  let host = host.to_ascii_lowercase();
  if let Some(suffix) = pattern.strip_prefix("*.") {
    host != suffix && host.ends_with(&format!(".{suffix}"))
  } else {
    host == pattern
  }
}

fn require_installed<'a>(registry: &'a AddonRegistry, addon_id: &str) -> R<&'a InstalledAddon> {
  let record = registry.addons.get(addon_id).ok_or_else(|| format!("Unknown addon: {addon_id}"))?;
  if !record.enabled {
    return Err(format!("Addon is disabled: {addon_id}"));
  }
  Ok(record)
}

fn parse_manifest_from_archive(archive: &mut ZipArchive<fs::File>) -> R<AddonManifest> {
  let mut file = archive.by_name("manifest.json").map_err(|_| "Addon package must contain manifest.json at its root".to_string())?;
  if file.size() > 256 * 1024 {
    return Err("Addon manifest is too large".to_string());
  }
  let mut raw = String::new();
  file.read_to_string(&mut raw).map_err(|error| format!("Failed to read addon manifest: {error}"))?;
  let manifest: AddonManifest = serde_json::from_str(&raw).map_err(|error| format!("Invalid addon manifest: {error}"))?;
  validate_manifest(&manifest)?;
  Ok(manifest)
}

fn validate_archive(archive: &mut ZipArchive<fs::File>) -> R<()> {
  if archive.len() > MAX_ARCHIVE_FILES {
    return Err(format!("Addon package contains too many files (maximum {MAX_ARCHIVE_FILES})"));
  }
  let mut extracted_bytes = 0_u64;
  for index in 0..archive.len() {
    let file = archive.by_index(index).map_err(|error| error.to_string())?;
    file.enclosed_name().ok_or_else(|| format!("Unsafe path in addon package: {}", file.name()))?;
    if file.unix_mode().map(|mode| mode & 0o170000 == 0o120000).unwrap_or(false) {
      return Err(format!("Symbolic links are not allowed in addon packages: {}", file.name()));
    }
    extracted_bytes = extracted_bytes.saturating_add(file.size());
    if extracted_bytes > MAX_EXTRACTED_BYTES {
      return Err("Addon package expands beyond the allowed size".to_string());
    }
  }
  Ok(())
}

fn extract_archive(archive: &mut ZipArchive<fs::File>, staging: &Path) -> R<()> {
  for index in 0..archive.len() {
    let mut file = archive.by_index(index).map_err(|error| error.to_string())?;
    let enclosed = file.enclosed_name().ok_or_else(|| format!("Unsafe path in addon package: {}", file.name()))?.to_path_buf();
    let output_path = staging.join(enclosed);
    if file.is_dir() {
      fs::create_dir_all(&output_path).map_err(|error| error.to_string())?;
      continue;
    }
    if let Some(parent) = output_path.parent() {
      fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut output = fs::File::create(&output_path).map_err(|error| error.to_string())?;
    io::copy(&mut file, &mut output).map_err(|error| error.to_string())?;
    output.flush().map_err(|error| error.to_string())?;
  }
  Ok(())
}

fn replace_package_directory(staging: &Path, target: &Path) -> R<Option<PathBuf>> {
  let backup = target.with_extension(format!("backup-{}", chrono::Utc::now().timestamp_millis()));
  let had_existing = target.exists();
  if had_existing {
    if backup.exists() {
      fs::remove_dir_all(&backup).map_err(|error| error.to_string())?;
    }
    fs::rename(target, &backup).map_err(|error| format!("Failed to stage previous addon version: {error}"))?;
  }
  if let Err(error) = fs::rename(staging, target) {
    if had_existing {
      let _ = fs::rename(&backup, target);
    }
    return Err(format!("Failed to install addon package: {error}"));
  }
  Ok(had_existing.then_some(backup))
}

fn read_storage(app: &AppHandle, addon_id: &str) -> R<Map<String, Value>> {
  let path = data_dir(app, addon_id)?.join("storage.json");
  if !path.exists() {
    return Ok(Map::new());
  }
  let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
  serde_json::from_str::<Value>(&raw)
    .map_err(|error| error.to_string())?
    .as_object()
    .cloned()
    .ok_or_else(|| "Addon storage file must contain an object".to_string())
}

fn write_storage(app: &AppHandle, addon_id: &str, storage: &Map<String, Value>) -> R<()> {
  let path = data_dir(app, addon_id)?.join("storage.json");
  let raw = serde_json::to_vec_pretty(storage).map_err(|error| error.to_string())?;
  if raw.len() > 1024 * 1024 {
    return Err("Addon storage exceeds the 1 MiB limit".to_string());
  }
  let temp = path.with_extension("json.tmp");
  fs::write(&temp, raw).map_err(|error| error.to_string())?;
  fs::rename(temp, path).map_err(|error| error.to_string())
}

fn storage_key(params: &Value) -> R<String> {
  let key = params.get("key").and_then(Value::as_str).unwrap_or("").trim();
  if key.is_empty() || key.len() > 128 || !key.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')) {
    return Err("Invalid addon storage key".to_string());
  }
  Ok(key.to_string())
}

fn active_vault_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  let root = PathBuf::from(vault.path);
  fs::create_dir_all(&root).map_err(|error| error.to_string())?;
  fs::canonicalize(root).map_err(|error| error.to_string())
}

fn note_path(app: &AppHandle, relative_path: &str, create_parent: bool) -> R<PathBuf> {
  let normalized = normalize_note_path(relative_path)?;
  let root = active_vault_root(app)?;
  let candidate = root.join(&normalized);
  if create_parent {
    let parent = candidate.parent().ok_or_else(|| "Note path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    if !canonical_parent.starts_with(&root) {
      return Err("Refusing to write outside the active vault".to_string());
    }
    let file_name = candidate.file_name().ok_or_else(|| "Note path has no file name".to_string())?;
    return Ok(canonical_parent.join(file_name));
  }
  let canonical = fs::canonicalize(candidate).map_err(|error| error.to_string())?;
  if !canonical.starts_with(&root) {
    return Err("Refusing to read outside the active vault".to_string());
  }
  Ok(canonical)
}

fn broker_storage(app: &AppHandle, record: &InstalledAddon, method: &str, params: &Value) -> R<Value> {
  if !record.manifest.permissions.storage {
    return Err("Addon storage permission was not granted".to_string());
  }
  let mut storage = read_storage(app, &record.manifest.id)?;
  match method {
    "storage.get" => {
      let key = storage_key(params)?;
      Ok(storage.get(&key).cloned().unwrap_or(Value::Null))
    }
    "storage.set" => {
      let key = storage_key(params)?;
      storage.insert(key, params.get("value").cloned().unwrap_or(Value::Null));
      write_storage(app, &record.manifest.id, &storage)?;
      Ok(json!({ "ok": true }))
    }
    "storage.remove" => {
      let key = storage_key(params)?;
      storage.remove(&key);
      write_storage(app, &record.manifest.id, &storage)?;
      Ok(json!({ "ok": true }))
    }
    "storage.entries" => Ok(Value::Object(storage)),
    _ => Err(format!("Unsupported storage operation: {method}")),
  }
}

fn broker_notes(app: &AppHandle, record: &InstalledAddon, method: &str, params: &Value) -> R<Value> {
  let relative_path = normalize_note_path(params.get("path").and_then(Value::as_str).unwrap_or(""))?;
  match method {
    "notes.read" => {
      require_note_scope(&record.manifest.permissions.notes.read, &relative_path, "read")?;
      let path = note_path(app, &relative_path, false)?;
      let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
      if !metadata.is_file() || metadata.len() > MAX_ENTRY_BYTES {
        return Err("Note is not a readable file or exceeds the size limit".to_string());
      }
      let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
      Ok(json!({ "path": relative_path, "content": content }))
    }
    "notes.write" => {
      require_note_scope(&record.manifest.permissions.notes.write, &relative_path, "write")?;
      let content = params.get("content").and_then(Value::as_str).unwrap_or("");
      if content.len() as u64 > MAX_ENTRY_BYTES {
        return Err("Note content exceeds the size limit".to_string());
      }
      let path = note_path(app, &relative_path, true)?;
      fs::write(path, content).map_err(|error| error.to_string())?;
      Ok(json!({ "ok": true, "path": relative_path }))
    }
    _ => Err(format!("Unsupported notes operation: {method}")),
  }
}

fn broker_http(record: &InstalledAddon, method: &str, params: &Value) -> R<Value> {
  if method != "http.request" {
    return Err(format!("Unsupported HTTP operation: {method}"));
  }
  let raw_url = params.get("url").and_then(Value::as_str).unwrap_or("");
  let url = Url::parse(raw_url).map_err(|error| format!("Invalid URL: {error}"))?;
  if url.scheme() != "https" {
    return Err("External addons may only request HTTPS URLs".to_string());
  }
  let host = url.host_str().ok_or_else(|| "URL has no host".to_string())?;
  if !record.manifest.permissions.network.hosts.iter().any(|pattern| host_matches(pattern, host)) {
    return Err(format!("Network access to {host} was not granted"));
  }
  let method_name = params.get("method").and_then(Value::as_str).unwrap_or("GET").to_ascii_uppercase();
  let request_method = reqwest::Method::from_bytes(method_name.as_bytes()).map_err(|error| error.to_string())?;
  if !matches!(request_method, reqwest::Method::GET | reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::PATCH | reqwest::Method::DELETE) {
    return Err(format!("Unsupported HTTP method: {method_name}"));
  }
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(30))
    .redirect(reqwest::redirect::Policy::limited(5))
    .build()
    .map_err(|error| error.to_string())?;
  let mut request = client.request(request_method, url);
  if let Some(headers) = params.get("headers").and_then(Value::as_object) {
    for (name, value) in headers {
      if matches!(name.to_ascii_lowercase().as_str(), "host" | "content-length" | "cookie") {
        continue;
      }
      if let Some(value) = value.as_str() {
        request = request.header(name, value);
      }
    }
  }
  if let Some(body) = params.get("body").and_then(Value::as_str) {
    request = request.body(body.to_string());
  }
  let mut response = request.send().map_err(|error| error.to_string())?;
  let status = response.status().as_u16();
  let response_headers = response
    .headers()
    .iter()
    .filter_map(|(name, value)| value.to_str().ok().map(|value| (name.to_string(), Value::String(value.to_string()))))
    .collect::<Map<String, Value>>();
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(MAX_HTTP_RESPONSE_BYTES + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > MAX_HTTP_RESPONSE_BYTES {
    return Err("HTTP response exceeds the 5 MiB limit".to_string());
  }
  Ok(json!({
    "status": status,
    "ok": (200..300).contains(&status),
    "headers": response_headers,
    "body": String::from_utf8_lossy(&bytes).to_string()
  }))
}

#[tauri::command]
pub fn tauri_addons_list(app: AppHandle, state: State<'_, AddonState>) -> R<Vec<InstalledAddon>> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  Ok(read_registry(&app)?.addons.into_values().collect())
}

#[tauri::command]
pub fn tauri_addons_install(app: AppHandle, state: State<'_, AddonState>, package_path: String) -> R<InstalledAddon> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let package_path = PathBuf::from(package_path);
  let metadata = fs::metadata(&package_path).map_err(|error| format!("Cannot read addon package: {error}"))?;
  if !metadata.is_file() || metadata.len() > MAX_PACKAGE_BYTES {
    return Err("Addon package must be a file smaller than 25 MiB".to_string());
  }
  let extension = package_path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
  if extension != "enaddon" && extension != "zip" {
    return Err("Addon package must use the .enaddon or .zip extension".to_string());
  }
  let package_bytes = fs::read(&package_path).map_err(|error| error.to_string())?;
  let package_hash = blake3::hash(&package_bytes).to_hex().to_string();
  let file = fs::File::open(&package_path).map_err(|error| error.to_string())?;
  let mut archive = ZipArchive::new(file).map_err(|error| format!("Invalid addon archive: {error}"))?;
  validate_archive(&mut archive)?;
  let manifest = parse_manifest_from_archive(&mut archive)?;

  let root = addons_root(&app)?;
  let staging = root.join(format!(".staging-{}-{}", manifest.id, chrono::Utc::now().timestamp_millis()));
  if staging.exists() {
    fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
  }
  fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
  if let Err(error) = extract_archive(&mut archive, &staging) {
    let _ = fs::remove_dir_all(&staging);
    return Err(error);
  }
  let entry_path = staging.join(safe_relative_path(&manifest.runtime.entry)?);
  let entry_metadata = fs::metadata(&entry_path).map_err(|_| format!("Addon entry does not exist: {}", manifest.runtime.entry))?;
  if !entry_metadata.is_file() || entry_metadata.len() > MAX_ENTRY_BYTES {
    let _ = fs::remove_dir_all(&staging);
    return Err("Addon entry is not a readable JavaScript file or exceeds 5 MiB".to_string());
  }

  let target = package_dir(&app, &manifest.id)?;
  let backup = replace_package_directory(&staging, &target)?;
  let mut registry = read_registry(&app)?;
  let enabled = registry.addons.get(&manifest.id).map(|record| record.enabled).unwrap_or(false);
  let record = InstalledAddon {
    manifest: manifest.clone(),
    enabled,
    package_hash,
    installed_at: now(),
    source: "external".to_string(),
  };
  registry.addons.insert(manifest.id.clone(), record.clone());
  if let Err(error) = write_registry(&app, &registry) {
    let _ = fs::remove_dir_all(&target);
    if let Some(backup) = &backup {
      let _ = fs::rename(backup, &target);
    }
    return Err(error);
  }
  if let Some(backup) = backup {
    let _ = fs::remove_dir_all(backup);
  }
  Ok(record)
}

#[tauri::command]
pub fn tauri_addons_uninstall(app: AppHandle, state: State<'_, AddonState>, addon_id: String) -> R<Value> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let mut registry = read_registry(&app)?;
  if registry.addons.remove(&addon_id).is_none() {
    return Err(format!("Unknown addon: {addon_id}"));
  }
  write_registry(&app, &registry)?;
  let package = package_dir(&app, &addon_id)?;
  if package.exists() {
    fs::remove_dir_all(package).map_err(|error| error.to_string())?;
  }
  Ok(json!({ "ok": true, "id": addon_id }))
}

#[tauri::command]
pub fn tauri_addons_set_enabled(app: AppHandle, state: State<'_, AddonState>, addon_id: String, enabled: bool) -> R<InstalledAddon> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let mut registry = read_registry(&app)?;
  let record = registry.addons.get_mut(&addon_id).ok_or_else(|| format!("Unknown addon: {addon_id}"))?;
  record.enabled = enabled;
  let result = record.clone();
  write_registry(&app, &registry)?;
  Ok(result)
}

#[tauri::command]
pub fn tauri_addons_read_entry(app: AppHandle, state: State<'_, AddonState>, addon_id: String) -> R<Value> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let registry = read_registry(&app)?;
  let record = require_installed(&registry, &addon_id)?;
  let path = package_dir(&app, &addon_id)?.join(safe_relative_path(&record.manifest.runtime.entry)?);
  let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
  if !metadata.is_file() || metadata.len() > MAX_ENTRY_BYTES {
    return Err("Addon entry is not readable or exceeds 5 MiB".to_string());
  }
  let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
  Ok(json!({ "source": source, "packageHash": record.package_hash }))
}

#[tauri::command]
pub fn tauri_addons_read_module(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
  path: String,
) -> R<Value> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let registry = read_registry(&app)?;
  let record = require_installed(&registry, &addon_id)?;
  let relative = safe_relative_path(&path)?;
  if relative.extension().and_then(|value| value.to_str()) != Some("js") {
    return Err("Addon modules must use the .js extension".to_string());
  }
  let package = package_dir(&app, &addon_id)?;
  let canonical_package = fs::canonicalize(&package)
    .map_err(|error| format!("Addon package directory is unavailable: {error}"))?;
  let module = fs::canonicalize(package.join(&relative))
    .map_err(|error| format!("Addon module is unavailable: {path}: {error}"))?;
  if !module.starts_with(&canonical_package) {
    return Err(format!("Addon module escapes its package directory: {path}"));
  }
  let metadata = fs::metadata(&module).map_err(|error| error.to_string())?;
  if !metadata.is_file() || metadata.len() > MAX_ENTRY_BYTES {
    return Err("Addon module is not readable or exceeds 5 MiB".to_string());
  }
  let source = fs::read_to_string(module).map_err(|error| error.to_string())?;
  Ok(json!({
    "path": relative.to_string_lossy().replace('\\', "/"),
    "source": source,
    "packageHash": record.package_hash
  }))
}

#[tauri::command]
pub fn tauri_addons_call(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
  method: String,
  params: Option<Value>,
) -> R<Value> {
  let _guard = state.lock.lock().map_err(|_| "Addon registry lock is poisoned".to_string())?;
  let registry = read_registry(&app)?;
  let record = require_installed(&registry, &addon_id)?;
  let params = params.unwrap_or_else(empty_object);
  if method.starts_with("storage.") {
    broker_storage(&app, record, &method, &params)
  } else if method.starts_with("notes.") {
    broker_notes(&app, record, &method, &params)
  } else if method.starts_with("http.") {
    broker_http(record, &method, &params)
  } else if method == "app.info" {
    Ok(json!({
      "name": "ElephantNote",
      "version": env!("CARGO_PKG_VERSION"),
      "addonApiVersion": ADDON_API_VERSION
    }))
  } else {
    Err(format!("Unsupported addon broker method: {method}"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_path_traversal() {
    assert!(safe_relative_path("../secret.md").is_err());
    assert!(safe_relative_path("Finance/../../secret.md").is_err());
    assert!(safe_relative_path("Finance/report.md").is_ok());
  }

  #[test]
  fn note_scopes_are_prefix_bounded() {
    assert!(scope_matches("Finance/**", "Finance/AAPL.md"));
    assert!(scope_matches("Finance/**", "Finance"));
    assert!(!scope_matches("Finance/**", "Finances/AAPL.md"));
    assert!(scope_matches("*", "Any/Note.md"));
  }

  #[test]
  fn wildcard_hosts_do_not_match_the_root_domain() {
    assert!(host_matches("*.example.com", "api.example.com"));
    assert!(!host_matches("*.example.com", "example.com"));
    assert!(!host_matches("*.example.com", "example.com.evil.test"));
  }

  #[test]
  fn validates_worker_manifest() {
    let manifest = AddonManifest {
      id: "com.example.finance".to_string(),
      name: "Finance".to_string(),
      version: "1.0.0".to_string(),
      description: String::new(),
      author: String::new(),
      api_version: ADDON_API_VERSION,
      min_app_version: String::new(),
      runtime: AddonRuntime { kind: "javascript-worker".to_string(), entry: "main.js".to_string() },
      permissions: AddonPermissions::default(),
      contributes: empty_object(),
      activation_events: Vec::new(),
    };
    assert!(validate_manifest(&manifest).is_ok());
  }
}
