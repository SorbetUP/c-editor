#[cfg(test)]
mod tests {
  use super::*;

  fn sync_item(version: &str) -> CatalogAddon {
    serde_json::from_value(serde_json::json!({
      "id": "elephant.sync",
      "slug": "sync",
      "name": "Sync",
      "version": version,
      "official": true,
      "manifestPath": "official/sync/manifest.json",
      "entryPath": "official/sync/main.service.js"
    }))
    .unwrap()
  }

  #[test]
  fn integrated_catalog_contains_dashboard_and_mobile_code_execution() {
    let addons = catalog().unwrap();
    assert!(addons.iter().any(|item| item.id == "elephant.dashboard"));
    assert!(addons.iter().any(|item| item.id == "elephant.code-execution"));
    assert!(addons.iter().all(|item| item.official));
  }

  #[test]
  fn local_official_packages_include_manifest_and_entry() {
    for item in catalog().unwrap() {
      let local = local_package_directory(&item).unwrap();
      assert!(local.join("manifest.json").is_file(), "missing manifest for {}", item.id);
      let manifest: Value = serde_json::from_slice(&fs::read(local.join("manifest.json")).unwrap()).unwrap();
      let entry = manifest.pointer("/runtime/entry").and_then(Value::as_str).unwrap();
      assert!(local.join(entry).is_file(), "missing entry for {}", item.id);
    }
  }

  #[test]
  fn native_catalog_entries_require_platform_packages() {
    let raw = br#"{
      "addons": [{
        "id": "elephant.sync",
        "slug": "sync",
        "name": "Sync",
        "version": "1.2.0",
        "official": true,
        "manifestPath": "official/sync/manifest.json",
        "entryPath": "official/sync/main.service.js",
        "requiresPlatformPackage": true,
        "packages": {}
      }]
    }"#;
    let error = parse_catalog(raw).expect_err("native source-only catalogue entries must be rejected");
    assert!(error.contains("requires a published package"));
  }

  #[test]
  fn package_hashes_are_validated_before_download() {
    let platform = platform_key();
    let raw = format!(
      r#"{{
        "addons": [{{
          "id": "elephant.sync",
          "slug": "sync",
          "name": "Sync",
          "version": "1.2.0",
          "official": true,
          "manifestPath": "official/sync/manifest.json",
          "entryPath": "official/sync/main.service.js",
          "requiresPlatformPackage": true,
          "packages": {{
            "{platform}": {{
              "path": "official/sync/releases/elephant.sync-1.2.0-{platform}.enaddon",
              "hash": "invalid"
            }}
          }}
        }}]
      }}"#
    );
    let error = parse_catalog(raw.as_bytes()).expect_err("invalid hashes must be rejected");
    assert!(error.contains("BLAKE3 hash"));
  }

  #[test]
  fn source_only_sync_uses_the_addon_repository_package() {
    let item = sync_item("1.2.0");
    assert!(available_for_platform(&item, "macos-aarch64"));
    assert!(item.packages.is_empty());
    assert!(!item.requires_platform_package);
  }

  #[test]
  fn incomplete_prebuilt_sync_package_is_rejected_before_installation() {
    let platform = platform_key();
    let sidecar = format!("native/{platform}/elephant-sync-service");
    let manifest = serde_json::json!({
      "id": "elephant.sync",
      "version": "1.2.0",
      "runtime": { "entry": "main.service.js" },
      "permissions": { "native": true },
      "native": {
        "runner": "service",
        "sidecars": { (platform): sidecar.clone() }
      }
    });
    let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer.start_file("manifest.json", options).unwrap();
    writer.write_all(serde_json::to_string(&manifest).unwrap().as_bytes()).unwrap();
    writer.start_file("main.service.js", options).unwrap();
    writer.write_all(b"export default {};").unwrap();
    let bytes = writer.finish().unwrap().into_inner();

    let error = validate_prebuilt_package(&sync_item("1.2.0"), &bytes)
      .expect_err("a package without its declared service must fail before installation");
    assert!(error.contains(&sidecar));
  }

  #[test]
  fn source_packages_cannot_omit_the_declared_service_executable() {
    let platform = platform_key();
    let sidecar = format!("native/{platform}/elephant-sync-service");
    let manifest = serde_json::json!({
      "permissions": { "native": true },
      "native": {
        "runner": "service",
        "sidecars": { (platform): sidecar.clone() }
      }
    });
    let files = BTreeMap::from([("manifest.json".to_string(), Vec::new())]);
    let error = require_declared_sidecar(&sync_item("1.2.0"), &manifest, &files)
      .expect_err("missing native executables must fail during package construction");
    assert!(error.contains(&sidecar));
  }
}
