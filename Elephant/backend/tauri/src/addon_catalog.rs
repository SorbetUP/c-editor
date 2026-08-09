use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  io::{Read, Write},
  path::{Component, Path, PathBuf},
  time::Duration,
};
use tauri::{AppHandle, State};
use url::Url;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::addons::{self, AddonManifest, AddonState, InstalledAddon};
use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;

const CATALOG_VERSION: u32 = 1;
const CATALOG_ROOT: &str = "https://raw.githubusercontent.com/SorbetUP/ElephantNote/addon-catalog/";
const MAX_CATALOG_BYTES: u64 = 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_ENTRY_BYTES: u64 = 5 * 1024 * 1024;
const MAX_PACKAGE_BYTES: u64 = 250 * 1024 * 1024;
const BUNDLED_TRUSTED_LAB_ID: &str = "com.elephantnote.examples.trusted-workspace-lab";
const BUNDLED_TRUSTED_LAB_MANIFEST: &str = include_str!("../../../../examples/addons/trusted-workspace-lab/manifest.json");
const BUNDLED_TRUSTED_LAB_ENTRY: &str = include_str!("../../../../examples/addons/trusted-workspace-lab/main.js");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPackage {
  pub path: String,
  pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAddon {
  pub id: String,
  pub slug: String,
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub author: String,
  #[serde(default)]
  pub official: bool,
  #[serde(default)]
  pub requires_platform_package: bool,
  #[serde(default)]
  pub manifest_path: String,
  #[serde(default)]
  pub entry_path: String,
  #[serde(default)]
  pub package_path: String,
  #[serde(default)]
  pub package_hash: String,
  #[serde(default)]
  pub packages: BTreeMap<String, CatalogPackage>,
  #[serde(default)]
  pub readme_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddonCatalog {
  version: u32,
  branch: String,
  #[serde(default)]
  updated_at: String,
  addons: Vec<CatalogAddon>,
}

fn bundled_trusted_lab_item() -> CatalogAddon {
  CatalogAddon {
    id: BUNDLED_TRUSTED_LAB_ID.to_string(),
    slug: "trusted-workspace-lab".to_string(),
    name: "Trusted Workspace Lab".to_string(),
    version: "1.0.0".to_string(),
    description: "Full app access reference addon that visibly modifies Settings, workspace UI and application behavior.".to_string(),
    author: "ElephantNote".to_string(),
    official: false,
    requires_platform_package: false,
    manifest_path: "bundled/trusted-workspace-lab/manifest.json".to_string(),
    entry_path: "bundled/trusted-workspace-lab/main.js".to_string(),
    package_path: String::new(),
    package_hash: String::new(),
    packages: BTreeMap::new(),
    readme_path: String::new(),
  }
}

fn safe_catalog_path(value: &str) -> R<String> {
  let path = Path::new(value);
  if value.trim().is_empty() || path.is_absolute() {
    return Err("Catalogue paths must be safe relative paths".to_string());
  }
  let mut parts = Vec::new();
  for component in path.components() {
    match component {
      Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
      Component::CurDir => {}
      Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
        return Err(format!("Unsafe catalogue path: {value}"));
      }
    }
  }
  if parts.is_empty() {
    return Err("Catalogue paths must not be empty".to_string());
  }
  Ok(parts.join("/"))
}

fn valid_platform_key(value: &str) -> bool {
  let mut parts = value.split('-');
  let os = parts.next().unwrap_or_default();
  let arch = parts.next().unwrap_or_default();
  parts.next().is_none()
    && matches!(os, "macos" | "linux" | "windows" | "android" | "ios")
    && matches!(arch, "aarch64" | "x86_64" | "armv7" | "i686")
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

fn available_for_platform(item: &CatalogAddon, platform: &str) -> bool {
  !item.requires_platform_package || item.packages.contains_key(platform)
}

fn validate_catalog_item(item: &CatalogAddon) -> R<()> {
  if item.id.trim().is_empty() || item.slug.trim().is_empty() || item.name.trim().is_empty() || item.version.trim().is_empty() {
    return Err("Catalogue addons require id, slug, name and version".to_string());
  }
  if !item.slug.chars().all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-') {
    return Err(format!("Invalid addon catalogue slug: {}", item.slug));
  }
  if item.official && !item.id.starts_with("elephant.") {
    return Err(format!("Only first-party elephant.* packages may be marked official: {}", item.id));
  }
  if item.id == BUNDLED_TRUSTED_LAB_ID {
    return Ok(());
  }

  let expected_prefix = format!("addons/{}/", item.slug);
  if item.requires_platform_package && item.packages.is_empty() {
    return Err(format!("Catalogue addon {} requires at least one platform package", item.id));
  }
  if !item.packages.is_empty() {
    if !item.package_path.trim().is_empty() || !item.package_hash.trim().is_empty() {
      return Err(format!("Catalogue addon {} cannot mix packages with legacy packagePath/packageHash", item.id));
    }
    for (platform, package) in &item.packages {
      if !valid_platform_key(platform) {
        return Err(format!("Invalid addon package platform key for {}: {}", item.id, platform));
      }
      let package_path = safe_catalog_path(&package.path)?;
      if !package_path.starts_with(&expected_prefix) || !package_path.ends_with(".enaddon") {
        return Err(format!(
          "Catalogue package for {} on {} must be an .enaddon file under {}",
          item.id, platform, expected_prefix
        ));
      }
      if package.hash.len() != 64 || !package.hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
          "Catalogue package for {} on {} requires a 64-character blake3 hash",
          item.id, platform
        ));
      }
    }
  } else if !item.package_path.trim().is_empty() {
    let package_path = safe_catalog_path(&item.package_path)?;
    if !package_path.starts_with(&expected_prefix) || !package_path.ends_with(".enaddon") {
      return Err(format!("Catalogue package for {} must be an .enaddon file under {}", item.id, expected_prefix));
    }
    if item.package_hash.trim().is_empty() {
      return Err(format!("Catalogue package for {} requires a blake3 packageHash", item.id));
    }
  } else {
    let manifest_path = safe_catalog_path(&item.manifest_path)?;
    let entry_path = safe_catalog_path(&item.entry_path)?;
    if !manifest_path.starts_with(&expected_prefix) || !entry_path.starts_with(&expected_prefix) {
      return Err(format!("Catalogue files for {} must stay under {}", item.id, expected_prefix));
    }
  }

  if !item.readme_path.trim().is_empty() {
    let readme_path = safe_catalog_path(&item.readme_path)?;
    if !readme_path.starts_with(&expected_prefix) {
      return Err(format!("Catalogue README for {} must stay under {}", item.id, expected_prefix));
    }
  }
  Ok(())
}

fn catalog_url(relative_path: &str) -> R<Url> {
  let relative_path = safe_catalog_path(relative_path)?;
  let root = Url::parse(CATALOG_ROOT).map_err(|error| error.to_string())?;
  let url = root.join(&relative_path).map_err(|error| error.to_string())?;
  if url.scheme() != "https" || url.host_str() != Some("raw.githubusercontent.com") {
    return Err("Addon catalogue URLs must use the trusted GitHub raw host".to_string());
  }
  if !url.path().starts_with("/SorbetUP/ElephantNote/addon-catalog/") {
    return Err("Addon catalogue URL escaped the trusted branch".to_string());
  }
  Ok(url)
}

fn fetch_bytes(relative_path: &str, max_bytes: u64) -> R<Vec<u8>> {
  let url = catalog_url(relative_path)?;
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(60))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|error| error.to_string())?;
  let mut response = client.get(url).send().map_err(|error| format!("Failed to reach addon catalogue: {error}"))?;
  if !response.status().is_success() {
    return Err(format!("Addon catalogue returned HTTP {}", response.status()));
  }
  if response.content_length().is_some_and(|length| length > max_bytes) {
    return Err("Addon catalogue response exceeds the allowed size".to_string());
  }
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(max_bytes + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > max_bytes {
    return Err("Addon catalogue response exceeds the allowed size".to_string());
  }
  Ok(bytes)
}

fn fetch_catalog() -> R<AddonCatalog> {
  let raw = fetch_bytes("catalog.json", MAX_CATALOG_BYTES)?;
  let catalog: AddonCatalog = serde_json::from_slice(&raw).map_err(|error| format!("Invalid addon catalogue: {error}"))?;
  if catalog.version != CATALOG_VERSION {
    return Err(format!("Unsupported addon catalogue version {}", catalog.version));
  }
  if catalog.branch != "addon-catalog" {
    return Err("Addon catalogue branch marker is invalid".to_string());
  }
  let mut ids = BTreeSet::new();
  for item in &catalog.addons {
    validate_catalog_item(item)?;
    if !ids.insert(item.id.clone()) {
      return Err(format!("Duplicate addon id in catalogue: {}", item.id));
    }
  }
  Ok(catalog)
}

fn temporary_package_path(item: &CatalogAddon) -> PathBuf {
  std::env::temp_dir().join(format!(
    "elephantnote-catalog-{}-{}.enaddon",
    item.slug,
    chrono::Utc::now().timestamp_millis()
  ))
}

fn parse_source_manifest(item: &CatalogAddon, manifest_bytes: &[u8]) -> R<(AddonManifest, bool)> {
  let manifest_value: Value = serde_json::from_slice(manifest_bytes)
    .map_err(|error| format!("Invalid manifest for {}: {error}", item.id))?;
  let requests_native = manifest_value
    .pointer("/permissions/native")
    .and_then(Value::as_bool)
    .unwrap_or(false);
  let manifest: AddonManifest = serde_json::from_value(manifest_value)
    .map_err(|error| format!("Invalid manifest for {}: {error}", item.id))?;
  Ok((manifest, requests_native))
}

fn write_temporary_package(item: &CatalogAddon, manifest_bytes: &[u8], entry_bytes: &[u8]) -> R<PathBuf> {
  let (manifest, requests_native) = parse_source_manifest(item, manifest_bytes)?;
  if requests_native {
    return Err(format!(
      "Native addon {} requires a complete hashed .enaddon package for the current platform; source-only catalogue entries cannot contain sidecars",
      item.id
    ));
  }
  if manifest.id != item.id || manifest.version != item.version || manifest.name != item.name {
    return Err(format!("Catalogue metadata does not match manifest for {}", item.id));
  }
  if entry_bytes.len() as u64 > MAX_ENTRY_BYTES {
    return Err(format!("Addon entry exceeds the allowed size for {}", item.id));
  }

  let temp_path = temporary_package_path(item);
  let file = fs::File::create(&temp_path).map_err(|error| error.to_string())?;
  let mut archive = ZipWriter::new(file);
  let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
  archive.start_file("manifest.json", options).map_err(|error| error.to_string())?;
  archive.write_all(manifest_bytes).map_err(|error| error.to_string())?;
  archive
    .start_file(manifest.runtime.entry.clone(), options)
    .map_err(|error| error.to_string())?;
  archive.write_all(entry_bytes).map_err(|error| error.to_string())?;
  archive.finish().map_err(|error| error.to_string())?;
  Ok(temp_path)
}

fn download_prebuilt_package(item: &CatalogAddon, package_path: &str, package_hash: &str) -> R<PathBuf> {
  let bytes = fetch_bytes(package_path, MAX_PACKAGE_BYTES)?;
  let actual_hash = blake3::hash(&bytes).to_hex().to_string();
  if actual_hash != package_hash.trim().to_ascii_lowercase() {
    return Err(format!("Addon package hash mismatch for {}", item.id));
  }
  let temp_path = temporary_package_path(item);
  fs::write(&temp_path, bytes).map_err(|error| error.to_string())?;
  Ok(temp_path)
}

fn create_temporary_package(item: &CatalogAddon) -> R<PathBuf> {
  if item.id == BUNDLED_TRUSTED_LAB_ID {
    return write_temporary_package(
      item,
      BUNDLED_TRUSTED_LAB_MANIFEST.as_bytes(),
      BUNDLED_TRUSTED_LAB_ENTRY.as_bytes(),
    );
  }

  let platform = platform_key();
  if let Some(package) = item.packages.get(&platform) {
    return download_prebuilt_package(item, &package.path, &package.hash);
  }
  if !item.packages.is_empty() {
    return Err(format!("Addon {} is not available for platform {}", item.id, platform));
  }
  if !item.package_path.trim().is_empty() {
    return download_prebuilt_package(item, &item.package_path, &item.package_hash);
  }

  let manifest_bytes = fetch_bytes(&item.manifest_path, MAX_MANIFEST_BYTES)?;
  let entry_bytes = fetch_bytes(&item.entry_path, MAX_ENTRY_BYTES)?;
  write_temporary_package(item, &manifest_bytes, &entry_bytes)
}

fn registry_path(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  Ok(vault_layout::addons_dir(&vault.path).join("registry.json"))
}

fn persist_official_source(app: &AppHandle, addon_id: &str) -> R<()> {
  let path = registry_path(app)?;
  let raw = fs::read_to_string(&path).map_err(|error| format!("Failed to read addon registry: {error}"))?;
  let mut registry: Value = serde_json::from_str(&raw).map_err(|error| format!("Invalid addon registry: {error}"))?;
  let record = registry
    .get_mut("addons")
    .and_then(Value::as_object_mut)
    .and_then(|addons| addons.get_mut(addon_id))
    .and_then(Value::as_object_mut)
    .ok_or_else(|| format!("Installed addon is absent from the registry: {addon_id}"))?;
  record.insert("source".to_string(), Value::String("official".to_string()));
  let encoded = serde_json::to_vec_pretty(&registry).map_err(|error| error.to_string())?;
  let temp = path.with_extension(format!("json.{}.tmp", chrono::Utc::now().timestamp_millis()));
  fs::write(&temp, encoded).map_err(|error| format!("Failed to stage addon registry provenance: {error}"))?;
  fs::rename(&temp, &path).map_err(|error| format!("Failed to persist addon registry provenance: {error}"))
}

#[tauri::command]
pub fn tauri_addons_catalog_list() -> R<Vec<CatalogAddon>> {
  let platform = platform_key();
  let mut addons = fetch_catalog()?.addons;
  addons.retain(|item| available_for_platform(item, &platform));
  if !addons.iter().any(|item| item.id == BUNDLED_TRUSTED_LAB_ID) {
    addons.push(bundled_trusted_lab_item());
  }
  Ok(addons)
}

#[tauri::command]
pub fn tauri_addons_catalog_install(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
) -> R<InstalledAddon> {
  let item = if addon_id == BUNDLED_TRUSTED_LAB_ID {
    bundled_trusted_lab_item()
  } else {
    fetch_catalog()?
      .addons
      .into_iter()
      .find(|item| item.id == addon_id && available_for_platform(item, &platform_key()))
      .ok_or_else(|| format!("Addon is not available for this platform: {addon_id}"))?
  };
  let package_path = create_temporary_package(&item)?;
  let package_string = package_path.to_string_lossy().to_string();
  let result = addons::tauri_addons_install(app.clone(), state, package_string);
  let _ = fs::remove_file(package_path);
  let mut record = result?;
  if item.official {
    persist_official_source(&app, &record.manifest.id)?;
    record.source = "official".to_string();
  }
  Ok(record)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn source_item() -> CatalogAddon {
    CatalogAddon {
      id: "com.example.test".to_string(),
      slug: "test".to_string(),
      name: "Test".to_string(),
      version: "1.0.0".to_string(),
      description: String::new(),
      author: String::new(),
      official: false,
      requires_platform_package: false,
      manifest_path: "addons/test/manifest.json".to_string(),
      entry_path: "addons/test/main.js".to_string(),
      package_path: String::new(),
      package_hash: String::new(),
      packages: BTreeMap::new(),
      readme_path: "addons/test/README.md".to_string(),
    }
  }

  #[test]
  fn rejects_catalog_path_traversal() {
    assert!(safe_catalog_path("../manifest.json").is_err());
    assert!(safe_catalog_path("addons/example/main.js").is_ok());
  }

  #[test]
  fn requires_source_files_to_stay_inside_the_addon_slug() {
    assert!(validate_catalog_item(&source_item()).is_ok());
  }

  #[test]
  fn rejects_source_only_native_packages() {
    let item = source_item();
    let manifest = br#"{
      "id": "com.example.test",
      "name": "Test",
      "version": "1.0.0",
      "apiVersion": 1,
      "runtime": { "type": "javascript-worker", "entry": "main.js" },
      "permissions": { "native": true }
    }"#;
    let error = write_temporary_package(&item, manifest, b"export default class TestAddon {}")
      .expect_err("native source packages must be rejected");
    assert!(error.contains("complete hashed .enaddon package"));
  }

  #[test]
  fn accepts_hashed_prebuilt_packages() {
    let item = CatalogAddon {
      id: "elephant.native".to_string(),
      slug: "native".to_string(),
      name: "Native".to_string(),
      version: "1.0.0".to_string(),
      description: String::new(),
      author: String::new(),
      official: true,
      requires_platform_package: false,
      manifest_path: String::new(),
      entry_path: String::new(),
      package_path: "addons/native/native-1.0.0-linux-x86_64.enaddon".to_string(),
      package_hash: "a".repeat(64),
      packages: BTreeMap::new(),
      readme_path: "addons/native/README.md".to_string(),
    };
    assert!(validate_catalog_item(&item).is_ok());
  }

  #[test]
  fn selects_only_published_platform_variants() {
    let mut item = source_item();
    item.requires_platform_package = true;
    item.packages.insert(
      "linux-x86_64".to_string(),
      CatalogPackage {
        path: "addons/test/releases/test-linux-x86_64.enaddon".to_string(),
        hash: "a".repeat(64),
      },
    );
    item.manifest_path.clear();
    item.entry_path.clear();
    assert!(validate_catalog_item(&item).is_ok());
    assert!(available_for_platform(&item, "linux-x86_64"));
    assert!(!available_for_platform(&item, "android-aarch64"));
  }

  #[test]
  fn rejects_non_elephant_official_marker() {
    let mut item = source_item();
    item.official = true;
    assert!(validate_catalog_item(&item).is_err());
  }

  #[test]
  fn bundles_the_full_access_lab_without_network_paths() {
    let item = bundled_trusted_lab_item();
    assert!(!item.official);
    assert_eq!(item.id, BUNDLED_TRUSTED_LAB_ID);
    assert!(validate_catalog_item(&item).is_ok());
    let manifest: AddonManifest = serde_json::from_str(BUNDLED_TRUSTED_LAB_MANIFEST).unwrap();
    assert_eq!(manifest.id, BUNDLED_TRUSTED_LAB_ID);
    assert_eq!(manifest.runtime.entry, "main.js");
    assert!(BUNDLED_TRUSTED_LAB_ENTRY.contains("export default class TrustedWorkspaceLab"));
  }
}
