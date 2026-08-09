fn collect_remote_files(item: &CatalogAddon) -> R<BTreeMap<String, Vec<u8>>> {
  let prefix = package_prefix(item)?;
  let manifest_bytes = fetch_official_bytes(&item.manifest_path, MAX_MANIFEST_BYTES)?;
  assert_mobile_compatibility(&manifest_bytes)?;
  let manifest: Value = serde_json::from_slice(&manifest_bytes).map_err(|error| error.to_string())?;
  let entry_name = manifest
    .pointer("/runtime/entry")
    .and_then(Value::as_str)
    .ok_or_else(|| "Official addon manifest is missing runtime.entry.".to_string())?;

  let mut files = BTreeMap::new();
  files.insert("manifest.json".to_string(), manifest_bytes);
  let mut pending = vec![format!("{prefix}/{entry_name}")];
  let mut visited = BTreeSet::new();
  while let Some(remote_path) = pending.pop() {
    if !visited.insert(remote_path.clone()) {
      continue;
    }
    let bytes = fetch_official_bytes(&remote_path, MAX_ENTRY_BYTES)?;
    let archive_path = remote_path
      .strip_prefix(&format!("{prefix}/"))
      .ok_or_else(|| format!("Official addon path escaped package: {remote_path}"))?
      .to_string();
    if archive_path.ends_with(".js") {
      let source = String::from_utf8_lossy(&bytes);
      for specifier in static_relative_imports(&source) {
        pending.push(resolve_remote_module(&prefix, &archive_path, &specifier)?);
      }
    }
    files.insert(archive_path, bytes);
  }

  if let Some(sidecar) = current_sidecar_path(&manifest) {
    let remote_path = safe_official_path(&format!("{prefix}/{sidecar}"))?;
    files.insert(sidecar, fetch_official_bytes(&remote_path, MAX_PACKAGE_FILE_BYTES)?);
  }
  require_declared_sidecar(item, &manifest, &files)?;
  Ok(files)
}

fn package_files(item: &CatalogAddon, bundled_root: Option<&Path>) -> R<BTreeMap<String, Vec<u8>>> {
  let local = match bundled_root {
    Some(root) => bundled_package_directory(item, root)?,
    None => local_package_directory(item)?,
  };
  if local.is_dir() {
    let manifest_bytes = fs::read(local.join("manifest.json")).map_err(|error| error.to_string())?;
    assert_mobile_compatibility(&manifest_bytes)?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes).map_err(|error| error.to_string())?;
    let mut files = BTreeMap::new();
    collect_local_files(&local, &local, &mut files)?;
    if files.contains_key("manifest.json") {
      require_declared_sidecar(item, &manifest, &files)?;
      return Ok(files);
    }
  }
  collect_remote_files(item)
}

fn temporary_package_path(item: &CatalogAddon) -> PathBuf {
  std::env::temp_dir().join(format!(
    "elephant-official-{}-{}.enaddon",
    item.slug,
    chrono::Utc::now().timestamp_millis()
  ))
}

fn validate_prebuilt_package(item: &CatalogAddon, bytes: &[u8]) -> R<()> {
  let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
    .map_err(|error| format!("Invalid official addon archive for {}: {error}", item.id))?;
  let manifest: Value = {
    let mut entry = archive
      .by_name("manifest.json")
      .map_err(|_| format!("Official addon package is missing manifest.json: {}", item.id))?;
    serde_json::from_reader(&mut entry)
      .map_err(|error| format!("Invalid official addon manifest for {}: {error}", item.id))?
  };
  if manifest.get("id").and_then(Value::as_str) != Some(item.id.as_str()) {
    return Err(format!("Official addon package id mismatch for {}", item.id));
  }
  if manifest.get("version").and_then(Value::as_str) != Some(item.version.as_str()) {
    return Err(format!("Official addon package version mismatch for {}", item.id));
  }
  if let Some(sidecar) = required_sidecar_path(item, &manifest)? {
    let entry = archive.by_name(&sidecar).map_err(|_| {
      format!(
        "Official addon package is incomplete for {} on {}: missing native executable {sidecar}",
        item.id,
        platform_key()
      )
    })?;
    if entry.is_dir() || entry.size() == 0 {
      return Err(format!(
        "Official addon package contains an invalid native executable for {}: {sidecar}",
        item.id
      ));
    }
  }
  Ok(())
}

fn write_verified_package(item: &CatalogAddon, bytes: Vec<u8>, package_hash: &str) -> R<PathBuf> {
  let actual_hash = blake3::hash(&bytes).to_hex().to_string();
  if actual_hash != package_hash.trim().to_ascii_lowercase() {
    return Err(format!("Official addon package hash mismatch for {}", item.id));
  }
  validate_prebuilt_package(item, &bytes)?;
  let path = temporary_package_path(item);
  fs::write(&path, bytes).map_err(|error| error.to_string())?;
  Ok(path)
}

fn download_prebuilt_package(item: &CatalogAddon, package_path: &str, package_hash: &str) -> R<PathBuf> {
  let bytes = fetch_official_bytes(package_path, MAX_PACKAGE_BYTES)?;
  write_verified_package(item, bytes, package_hash)
}

fn fetch_legacy_sync_bytes(relative_path: &str) -> R<Vec<u8>> {
  let normalized = safe_official_path(&format!("official/{relative_path}"))?
    .strip_prefix("official/")
    .ok_or_else(|| "Invalid immutable Sync package path".to_string())?
    .to_string();
  if !normalized.starts_with("addons/sync/releases/") || !normalized.ends_with(".enaddon") {
    return Err("Immutable Sync packages must stay under addons/sync/releases".to_string());
  }
  let root = Url::parse(LEGACY_SYNC_ROOT).map_err(|error| error.to_string())?;
  let url = root.join(&normalized).map_err(|error| error.to_string())?;
  if url.scheme() != "https"
    || url.host_str() != Some("raw.githubusercontent.com")
    || !url.path().starts_with(
      "/SorbetUP/ElephantNote/2a4547c17e3ce1e581e9956dc970c37039d49329/addons/sync/releases/",
    )
  {
    return Err("Immutable Sync package URL escaped its pinned repository revision".to_string());
  }
  let client = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(90))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|error| error.to_string())?;
  let mut response = client
    .get(url)
    .send()
    .map_err(|error| format!("Failed to reach immutable Sync package: {error}"))?;
  if !response.status().is_success() {
    return Err(format!("Immutable Sync package returned HTTP {}", response.status()));
  }
  if response.content_length().is_some_and(|length| length > MAX_PACKAGE_BYTES) {
    return Err("Immutable Sync package exceeds the allowed size".to_string());
  }
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(MAX_PACKAGE_BYTES + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > MAX_PACKAGE_BYTES {
    return Err("Immutable Sync package exceeds the allowed size".to_string());
  }
  Ok(bytes)
}

fn prebuilt_package(item: &CatalogAddon) -> R<Option<PathBuf>> {
  let platform = platform_key();
  if uses_legacy_sync_package(item) {
    let (package_path, package_hash) = legacy_sync_package(&platform)
      .ok_or_else(|| format!("Official addon {} is not available for platform {platform}", item.id))?;
    let bytes = fetch_legacy_sync_bytes(package_path)?;
    return write_verified_package(item, bytes, package_hash).map(Some);
  }
  if let Some(package) = item.packages.get(&platform) {
    return download_prebuilt_package(item, &package.path, &package.hash).map(Some);
  }
  if item.requires_platform_package || !item.packages.is_empty() || item.id == "elephant.sync" {
    return Err(format!("Official addon {} is not available for platform {platform}", item.id));
  }
  Ok(None)
}

fn temporary_package(item: &CatalogAddon, bundled_root: Option<&Path>) -> R<PathBuf> {
  if let Some(package) = prebuilt_package(item)? {
    return Ok(package);
  }
  let files = package_files(item, bundled_root)?;
  let path = temporary_package_path(item);
  let file = fs::File::create(&path).map_err(|error| error.to_string())?;
  let mut archive = ZipWriter::new(file);
  let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
  for (archive_path, bytes) in files {
    archive.start_file(archive_path, options).map_err(|error| error.to_string())?;
    archive.write_all(&bytes).map_err(|error| error.to_string())?;
  }
  archive.finish().map_err(|error| error.to_string())?;
  Ok(path)
}

#[tauri::command]
pub fn tauri_official_addons_catalog_list(app: AppHandle) -> R<Vec<CatalogAddon>> {
  let platform = platform_key();
  let (mut addons, _) = catalog_for_app(&app)?;
  addons.retain(|item| available_for_platform(item, &platform));
  Ok(addons)
}

#[tauri::command]
pub fn tauri_official_addons_catalog_install(
  app: AppHandle,
  state: State<'_, AddonState>,
  addon_id: String,
) -> R<InstalledAddon> {
  let platform = platform_key();
  let (catalog, bundled_root) = catalog_for_app(&app)?;
  let item = catalog
    .into_iter()
    .find(|item| item.id == addon_id && available_for_platform(item, &platform))
    .ok_or_else(|| format!("Official addon is not available for {platform}: {addon_id}"))?;
  let package_path = temporary_package(&item, bundled_root.as_deref())?;
  let result = addons::tauri_addons_install(app, state, package_path.to_string_lossy().to_string());
  let _ = fs::remove_file(package_path);
  let mut record = result?;
  record.source = "official".to_string();
  Ok(record)
}
