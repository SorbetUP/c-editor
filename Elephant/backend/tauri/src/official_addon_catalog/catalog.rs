use serde::Deserialize;
use serde_json::Value;
use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  io::{Read, Write},
  path::{Component, Path, PathBuf},
  time::Duration,
};
use tauri::{AppHandle, Manager, State};
use url::Url;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::{
  addon_catalog::CatalogAddon,
  addons::{self, AddonState, InstalledAddon},
};

type R<T> = Result<T, String>;
const OFFICIAL_ROOT: &str = "https://raw.githubusercontent.com/SorbetUP/Elephant-Addons/main/";
const OFFICIAL_CATALOG_PATH: &str = "catalog.json";
const MAX_CATALOG_BYTES: u64 = 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_ENTRY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PACKAGE_BYTES: u64 = 25 * 1024 * 1024;
const MAX_PACKAGE_FILE_BYTES: u64 = 128 * 1024 * 1024;
const LEGACY_SYNC_ROOT: &str = "https://raw.githubusercontent.com/SorbetUP/ElephantNote/2a4547c17e3ce1e581e9956dc970c37039d49329/";
const LEGACY_SYNC_VERSION: &str = "1.2.0";

#[derive(Deserialize)]
struct IntegratedCatalog {
  addons: Vec<CatalogAddon>,
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

fn valid_platform_key(value: &str) -> bool {
  let mut parts = value.split('-');
  let os = parts.next().unwrap_or_default();
  let arch = parts.next().unwrap_or_default();
  parts.next().is_none()
    && matches!(os, "macos" | "linux" | "windows" | "android" | "ios")
    && matches!(arch, "aarch64" | "x86_64" | "armv7" | "i686")
}

fn legacy_sync_package(platform: &str) -> Option<(&'static str, &'static str)> {
  match platform {
    "linux-x86_64" => Some((
      "addons/sync/releases/elephant.sync-1.2.0-linux-x86_64.enaddon",
      "6a9996f9542616d18835d807a03577a976ffbb2c9b321ad24fe8ce37cbe9eaad",
    )),
    "macos-aarch64" => Some((
      "addons/sync/releases/elephant.sync-1.2.0-macos-aarch64.enaddon",
      "ada18591bb94f4a7218c098dee79673d652c77031562bd9f2c856b051c3ccda3",
    )),
    "macos-x86_64" => Some((
      "addons/sync/releases/elephant.sync-1.2.0-macos-x86_64.enaddon",
      "bd3d5d28f561d3fa88042025729067d6f5ba044663dc490b7359b1d469118bc3",
    )),
    "windows-x86_64" => Some((
      "addons/sync/releases/elephant.sync-1.2.0-windows-x86_64.enaddon",
      "318d9ba10d85d9496fadcf539bed4fb6391da9cc1d53f20cc12c5dc2560aa8f6",
    )),
    _ => None,
  }
}

fn uses_legacy_sync_package(item: &CatalogAddon) -> bool {
  item.id == "elephant.sync" && item.version == LEGACY_SYNC_VERSION && item.packages.is_empty()
}

fn available_for_platform(item: &CatalogAddon, platform: &str) -> bool {
  if uses_legacy_sync_package(item) {
    return legacy_sync_package(platform).is_some();
  }
  if item.id == "elephant.sync" && item.packages.is_empty() {
    return false;
  }
  if item.packages.is_empty() {
    !item.requires_platform_package
  } else {
    item.packages.contains_key(platform)
  }
}

fn parse_catalog(bytes: &[u8]) -> R<Vec<CatalogAddon>> {
  let parsed: IntegratedCatalog = serde_json::from_slice(bytes)
    .map_err(|error| format!("Invalid official addon catalogue: {error}"))?;
  let mut ids = BTreeSet::new();
  for item in &parsed.addons {
    if !item.official || !item.id.starts_with("elephant.") {
      return Err(format!("Official catalogue contains a non-official addon: {}", item.id));
    }
    if !ids.insert(item.id.clone()) {
      return Err(format!("Official catalogue contains duplicate addon id: {}", item.id));
    }
    safe_official_path(&item.manifest_path)?;
    safe_official_path(&item.entry_path)?;
    if item.requires_platform_package && item.packages.is_empty() {
      return Err(format!(
        "Official addon {} requires a published package for each supported platform",
        item.id
      ));
    }
    let expected_prefix = format!("official/{}/releases/", item.slug);
    for (platform, package) in &item.packages {
      if !valid_platform_key(platform) {
        return Err(format!("Invalid official addon package platform for {}: {platform}", item.id));
      }
      let path = safe_official_path(&package.path)?;
      if !path.starts_with(&expected_prefix) || !path.ends_with(".enaddon") {
        return Err(format!(
          "Official addon package for {} on {platform} must be an .enaddon file under {expected_prefix}",
          item.id
        ));
      }
      if package.hash.len() != 64 || !package.hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
          "Official addon package for {} on {platform} requires a 64-character BLAKE3 hash",
          item.id
        ));
      }
    }
  }
  Ok(parsed.addons)
}

fn fetch_catalog_bytes() -> R<Vec<u8>> {
  let root = Url::parse(OFFICIAL_ROOT).map_err(|error| error.to_string())?;
  let url = root.join(OFFICIAL_CATALOG_PATH).map_err(|error| error.to_string())?;
  if url.scheme() != "https"
    || url.host_str() != Some("raw.githubusercontent.com")
    || url.path() != "/SorbetUP/Elephant-Addons/main/catalog.json"
  {
    return Err("Official addon catalogue URL escaped the dedicated repository.".to_string());
  }
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(90))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|error| error.to_string())?;
  let mut response = client
    .get(url)
    .send()
    .map_err(|error| format!("Failed to reach the official addon catalogue: {error}"))?;
  if !response.status().is_success() {
    return Err(format!("Official addon catalogue returned HTTP {}", response.status()));
  }
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(MAX_CATALOG_BYTES + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > MAX_CATALOG_BYTES {
    return Err("Official addon catalogue exceeds the allowed size.".to_string());
  }
  Ok(bytes)
}

fn catalog() -> R<Vec<CatalogAddon>> {
  let local = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../addons/catalog.json");
  let bytes = if local.is_file() {
    fs::read(local).map_err(|error| error.to_string())?
  } else {
    fetch_catalog_bytes()?
  };
  parse_catalog(&bytes)
}

fn bundled_addons_root(app: &AppHandle) -> Option<PathBuf> {
  let resource_dir = app.path().resource_dir().ok()?;
  [resource_dir.join("official-addons"), resource_dir.join("resources/official-addons")]
    .into_iter()
    .find(|root| root.join("catalog.json").is_file())
}

fn catalog_for_app(app: &AppHandle) -> R<(Vec<CatalogAddon>, Option<PathBuf>)> {
  if let Some(root) = bundled_addons_root(app) {
    println!("[official-addon-catalog] source=bundled root={}", root.display());
    let bytes = fs::read(root.join("catalog.json")).map_err(|error| error.to_string())?;
    return Ok((parse_catalog(&bytes)?, Some(root)));
  }
  println!("[official-addon-catalog] source=local-or-remote");
  Ok((catalog()?, None))
}

fn safe_official_path(value: &str) -> R<String> {
  let path = Path::new(value);
  if value.trim().is_empty() || path.is_absolute() {
    return Err("Official addon paths must be safe relative paths.".to_string());
  }
  let mut parts = Vec::new();
  for component in path.components() {
    match component {
      Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
      Component::CurDir => {}
      Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
        return Err(format!("Unsafe official addon path: {value}"));
      }
    }
  }
  let normalized = parts.join("/");
  if !normalized.starts_with("official/") {
    return Err(format!("Official addon path escaped the official package root: {value}"));
  }
  Ok(normalized)
}

fn fetch_official_bytes(relative_path: &str, max_bytes: u64) -> R<Vec<u8>> {
  let relative_path = safe_official_path(relative_path)?;
  let root = Url::parse(OFFICIAL_ROOT).map_err(|error| error.to_string())?;
  let url = root.join(&relative_path).map_err(|error| error.to_string())?;
  if url.scheme() != "https"
    || url.host_str() != Some("raw.githubusercontent.com")
    || !url.path().starts_with("/SorbetUP/Elephant-Addons/main/official/")
  {
    return Err("Official addon URL escaped the dedicated addon repository.".to_string());
  }
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(90))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|error| error.to_string())?;
  let mut response = client
    .get(url)
    .send()
    .map_err(|error| format!("Failed to reach the official addon catalogue: {error}"))?;
  if !response.status().is_success() {
    return Err(format!("Official addon source returned HTTP {} for {relative_path}", response.status()));
  }
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(max_bytes + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > max_bytes {
    return Err(format!("Official addon source exceeds the allowed size: {relative_path}"));
  }
  Ok(bytes)
}
