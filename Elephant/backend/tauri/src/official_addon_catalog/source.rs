fn assert_mobile_compatibility(manifest_bytes: &[u8]) -> R<()> {
  let manifest: Value = serde_json::from_slice(manifest_bytes)
    .map_err(|error| format!("Invalid official addon manifest: {error}"))?;
  let native = manifest.pointer("/permissions/native").and_then(Value::as_bool).unwrap_or(false);
  if !native {
    return Ok(());
  }
  let platform = if cfg!(target_os = "android") {
    "android"
  } else if cfg!(target_os = "ios") {
    "ios"
  } else {
    return Ok(());
  };
  if manifest
    .pointer(&format!("/native/mobile/{platform}/supported"))
    .and_then(Value::as_bool)
    != Some(true)
  {
    let reason = manifest
      .pointer(&format!("/native/mobile/{platform}/reason"))
      .and_then(Value::as_str)
      .unwrap_or("This addon has no native implementation for this mobile platform.");
    return Err(reason.to_string());
  }
  Ok(())
}

fn package_prefix(item: &CatalogAddon) -> R<String> {
  let manifest = safe_official_path(&item.manifest_path)?;
  Path::new(&manifest)
    .parent()
    .map(|path| path.to_string_lossy().replace('\\', "/"))
    .ok_or_else(|| format!("Official addon has no package directory: {}", item.id))
}

fn local_package_directory(item: &CatalogAddon) -> R<PathBuf> {
  let prefix = package_prefix(item)?;
  Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../../addons")
    .join(prefix))
}

fn bundled_package_directory(item: &CatalogAddon, addons_root: &Path) -> R<PathBuf> {
  Ok(addons_root.join(package_prefix(item)?))
}

fn collect_local_files(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> R<()> {
  for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
    let entry = entry.map_err(|error| error.to_string())?;
    let path = entry.path();
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
      continue;
    }
    if metadata.is_dir() {
      if entry.file_name() == "target" || entry.file_name() == "node_modules" || entry.file_name() == "releases" {
        continue;
      }
      collect_local_files(root, &path, files)?;
      continue;
    }
    if !metadata.is_file() {
      continue;
    }
    if metadata.len() > MAX_PACKAGE_FILE_BYTES {
      return Err(format!("Official addon package file is too large: {}", path.display()));
    }
    let relative = path
      .strip_prefix(root)
      .map_err(|error| error.to_string())?
      .to_string_lossy()
      .replace('\\', "/");
    files.insert(relative, fs::read(&path).map_err(|error| error.to_string())?);
  }
  Ok(())
}

fn static_relative_imports(source: &str) -> Vec<String> {
  let mut imports = Vec::new();
  for line in source.lines() {
    let trimmed = line.trim();
    if !trimmed.starts_with("import ") {
      continue;
    }
    for quote in ['\'', '"'] {
      let Some(start) = trimmed.find(quote) else { continue };
      let rest = &trimmed[start + 1..];
      let Some(end) = rest.find(quote) else { continue };
      let candidate = &rest[..end];
      if candidate.starts_with('.') {
        imports.push(candidate.to_string());
      }
    }
  }
  imports
}

fn resolve_remote_module(package_prefix: &str, current: &str, specifier: &str) -> R<String> {
  let current_parent = Path::new(current).parent().unwrap_or_else(|| Path::new(""));
  let mut relative = current_parent.join(specifier).to_string_lossy().replace('\\', "/");
  if Path::new(&relative).extension().is_none() {
    relative.push_str(".js");
  }
  let full = format!("{package_prefix}/{relative}");
  let normalized = safe_official_path(&full)?;
  if !normalized.starts_with(&format!("{package_prefix}/")) {
    return Err(format!("Official addon module escaped its package: {specifier}"));
  }
  Ok(normalized)
}

fn current_sidecar_path(manifest: &Value) -> Option<String> {
  manifest
    .pointer(&format!("/native/sidecars/{}", platform_key()))
    .and_then(Value::as_str)
    .map(str::to_string)
}

fn requires_desktop_service(manifest: &Value) -> bool {
  !matches!(std::env::consts::OS, "android" | "ios")
    && manifest.pointer("/permissions/native").and_then(Value::as_bool) == Some(true)
    && manifest.pointer("/native/runner").and_then(Value::as_str) == Some("service")
}

fn required_sidecar_path(item: &CatalogAddon, manifest: &Value) -> R<Option<String>> {
  if !requires_desktop_service(manifest) {
    return Ok(None);
  }
  current_sidecar_path(manifest).map(Some).ok_or_else(|| {
    format!(
      "Official addon {} has no native service executable for {}",
      item.id,
      platform_key()
    )
  })
}

fn require_declared_sidecar(
  item: &CatalogAddon,
  manifest: &Value,
  files: &BTreeMap<String, Vec<u8>>,
) -> R<()> {
  let Some(sidecar) = required_sidecar_path(item, manifest)? else {
    return Ok(());
  };
  if !files.contains_key(&sidecar) {
    return Err(format!(
      "Official addon package is incomplete for {} on {}: missing native executable {sidecar}",
      item.id,
      platform_key()
    ));
  }
  Ok(())
}
