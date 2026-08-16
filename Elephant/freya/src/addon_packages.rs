//! Verified package boundary for external add-ons.
//!
//! This module only handles archive validation and filesystem replacement.
//! The registry and eventual worker lifecycle remain owned by
//! `addon_adapter`, so an archive cannot silently become an active runtime.

use crate::{
    addon_adapter::{AddonManifest, ADDON_API_VERSION},
    vault_layout,
};
use std::{
    fs,
    io::{self, Cursor, Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

const MAX_PACKAGE_BYTES: u64 = 25 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 100 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 5 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 512;

pub(crate) struct InstalledPackage {
    pub(crate) manifest: AddonManifest,
    pub(crate) package_hash: String,
    pub(crate) target: PathBuf,
    pub(crate) backup: Option<PathBuf>,
}

pub(crate) fn install(root: &Path, package_path: &Path) -> Result<InstalledPackage, String> {
    let metadata = fs::metadata(package_path)
        .map_err(|error| format!("Cannot read addon package: {error}"))?;
    if !metadata.is_file() || metadata.len() > MAX_PACKAGE_BYTES {
        return Err("Addon package must be a file smaller than 25 MiB".to_owned());
    }
    let extension = package_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "enaddon" && extension != "zip" {
        return Err("Addon package must use the .enaddon or .zip extension".to_owned());
    }
    let bytes = fs::read(package_path).map_err(|error| error.to_string())?;
    let package_hash = blake3::hash(&bytes).to_hex().to_string();
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Invalid addon archive: {error}"))?;
    validate_archive(&mut archive)?;
    let manifest = parse_manifest(&mut archive)?;

    let packages = vault_layout::addons_dir(root).join("packages");
    fs::create_dir_all(&packages).map_err(|error| {
        format!(
            "Create addon package directory {}: {error}",
            packages.display()
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let staging = packages.join(format!(".staging-{}-{nonce}", manifest.id));
    let target = packages.join(&manifest.id);
    let backup = target.with_extension(format!("backup-{nonce}"));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    if let Err(error) = extract_archive(&mut archive, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let entry = staging.join(safe_relative_path(&manifest.runtime.entry)?);
    let entry_metadata = match fs::metadata(&entry) {
        Ok(metadata) => metadata,
        Err(_) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!(
                "Addon entry does not exist: {}",
                manifest.runtime.entry
            ));
        }
    };
    if !entry_metadata.is_file() || entry_metadata.len() > MAX_ENTRY_BYTES {
        let _ = fs::remove_dir_all(&staging);
        return Err("Addon entry is not a readable JavaScript file or exceeds 5 MiB".to_owned());
    }

    let had_existing = target.exists();
    if had_existing {
        if backup.exists() {
            fs::remove_dir_all(&backup).map_err(|error| error.to_string())?;
        }
        fs::rename(&target, &backup)
            .map_err(|error| format!("Failed to stage previous addon version: {error}"))?;
    }
    if let Err(error) = fs::rename(&staging, &target) {
        if had_existing {
            let _ = fs::rename(&backup, &target);
        }
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("Failed to install addon package: {error}"));
    }
    Ok(InstalledPackage {
        manifest,
        package_hash,
        target,
        backup: had_existing.then_some(backup),
    })
}

fn parse_manifest<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<AddonManifest, String> {
    let mut file = archive
        .by_name("manifest.json")
        .map_err(|_| "Addon package must contain manifest.json at its root".to_owned())?;
    if file.size() > 256 * 1024 {
        return Err("Addon manifest is too large".to_owned());
    }
    let mut raw = String::new();
    file.read_to_string(&mut raw)
        .map_err(|error| format!("Failed to read addon manifest: {error}"))?;
    let manifest: AddonManifest =
        serde_json::from_str(&raw).map_err(|error| format!("Invalid addon manifest: {error}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &AddonManifest) -> Result<(), String> {
    if !valid_addon_id(&manifest.id) {
        return Err(
            "Addon id must contain lowercase ASCII letters, numbers, dots, dashes or underscores"
                .to_owned(),
        );
    }
    if manifest.name.trim().is_empty() || manifest.version.trim().is_empty() {
        return Err("Addon name and version are required".to_owned());
    }
    if manifest.api_version != ADDON_API_VERSION {
        return Err(format!(
            "Unsupported addon apiVersion {}",
            manifest.api_version
        ));
    }
    if manifest.runtime.kind != "javascript-worker" {
        return Err(format!(
            "Unsupported addon runtime {}",
            manifest.runtime.kind
        ));
    }
    let entry = safe_relative_path(&manifest.runtime.entry)?;
    if entry.extension().and_then(|value| value.to_str()) != Some("js") {
        return Err("javascript-worker entry must be a .js file".to_owned());
    }
    Ok(())
}

fn valid_addon_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.as_bytes()[0].is_ascii_lowercase()
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.trim().is_empty() || path.is_absolute() {
        return Err("A safe relative path is required".to_owned());
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
        return Err("A safe relative path is required".to_owned());
    }
    Ok(normalized)
}

fn validate_archive<R: Read + io::Seek>(archive: &mut ZipArchive<R>) -> Result<(), String> {
    if archive.len() > MAX_ARCHIVE_FILES {
        return Err(format!(
            "Addon package contains too many files (maximum {MAX_ARCHIVE_FILES})"
        ));
    }
    let mut extracted_bytes = 0_u64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|error| error.to_string())?;
        file.enclosed_name()
            .ok_or_else(|| format!("Unsafe path in addon package: {}", file.name()))?;
        if file
            .unix_mode()
            .map(|mode| mode & 0o170000 == 0o120000)
            .unwrap_or(false)
        {
            return Err(format!(
                "Symbolic links are not allowed in addon packages: {}",
                file.name()
            ));
        }
        extracted_bytes = extracted_bytes.saturating_add(file.size());
        if extracted_bytes > MAX_EXTRACTED_BYTES {
            return Err("Addon package expands beyond the allowed size".to_owned());
        }
    }
    Ok(())
}

fn extract_archive<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    staging: &Path,
) -> Result<(), String> {
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|error| error.to_string())?;
        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe path in addon package: {}", file.name()))?
            .to_path_buf();
        let output = staging.join(enclosed);
        if file.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut destination = fs::File::create(&output).map_err(|error| error.to_string())?;
        io::copy(&mut file, &mut destination).map_err(|error| error.to_string())?;
        destination.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_adapter;
    use serde_json::json;

    #[test]
    fn installs_and_uninstalls_a_verified_package_without_data_residue() {
        let root = temp_root("install");
        let package = root.join("example.enaddon");
        write_package(&package, "example.addon", "main.js", "export default {};");

        let record = addon_adapter::install(&root, &package).expect("install package");
        assert_eq!(record.manifest.id, "example.addon");
        assert!(root
            .join(".elephantnote/addons/packages/example.addon/main.js")
            .is_file());
        let data = root.join(".elephantnote/addons/data/example.addon");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("state.json"), "{}").unwrap();

        addon_adapter::uninstall(&root, "example.addon").expect("uninstall package");
        assert!(!root
            .join(".elephantnote/addons/packages/example.addon")
            .exists());
        assert!(!data.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_archive_path_traversal_before_extraction() {
        let root = temp_root("traversal");
        let package = root.join("unsafe.enaddon");
        let file = fs::File::create(&package).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("../escape.js", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"bad").unwrap();
        writer.finish().unwrap();
        let error = addon_adapter::install(&root, &package).unwrap_err();
        assert!(error.contains("Unsafe path"));
        assert!(!root.join("escape.js").exists());
        let _ = fs::remove_dir_all(root);
    }

    fn write_package(path: &Path, id: &str, entry: &str, source: &str) {
        let file = fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("manifest.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer
            .write_all(
                serde_json::to_string(&json!({
                    "id": id,
                    "name": "Example",
                    "version": "1.0.0",
                    "apiVersion": 1,
                    "runtime": {"type": "javascript-worker", "entry": entry}
                }))
                .unwrap()
                .as_bytes(),
            )
            .unwrap();
        writer
            .start_file(entry, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(source.as_bytes()).unwrap();
        writer.finish().unwrap();
    }

    fn temp_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-addon-{label}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
