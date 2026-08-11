use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    config, production_entries, relative_path, types::VaultDescriptor, vault_layout, AdapterError,
    AdapterResult, DeleteResult, EmptyTrashResult, RestoreResult, TrashEntry,
};

const MANIFEST_FILE: &str = "manifest.json";
const CONTENT_DIR: &str = "content";
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashManifest {
    schema_version: u32,
    original_path: String,
    entry_name: String,
    deleted_at: String,
}

pub(super) fn delete(
    descriptor: &VaultDescriptor,
    root: &Path,
    original_path: &str,
) -> AdapterResult<DeleteResult> {
    let original_path = relative_path::validate(original_path)?;
    let entry_name = Path::new(&original_path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| AdapterError::new("Cannot trash an unnamed vault entry."))?
        .to_string();
    let (restore_token, item_path) = reserve_item(root)?;
    let content_path = item_path.join(CONTENT_DIR);
    if let Err(error) = fs::create_dir(&content_path) {
        let _ = fs::remove_dir(&item_path);
        return Err(error.into());
    }

    let manifest = TrashManifest {
        schema_version: MANIFEST_VERSION,
        original_path: original_path.clone(),
        entry_name,
        deleted_at: config::now_string(),
    };
    if let Err(error) = write_manifest(&item_path, &manifest) {
        let _ = fs::remove_dir_all(&item_path);
        return Err(error);
    }

    let trash_path = item_relative_path(&restore_token);
    let content_relative = format!("{trash_path}/{CONTENT_DIR}");
    if let Err(error) =
        production_entries::move_entry(descriptor, original_path.clone(), Some(content_relative))
    {
        let _ = fs::remove_dir_all(&item_path);
        return Err(AdapterError::new(format!(
            "Unable to move {original_path} to trash: {error}"
        )));
    }

    Ok(DeleteResult {
        deleted: true,
        path: original_path.clone(),
        original_path,
        trash_path,
        restore_token,
    })
}

pub(super) fn list(root: &Path) -> AdapterResult<Vec<TrashEntry>> {
    let trash_root = trash_root(root);
    let entries = match fs::read_dir(&trash_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut entries = entries.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut result = Vec::new();
    for entry in entries {
        let file_type = entry.file_type()?;
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        let token = entry.file_name().to_string_lossy().to_string();
        let manifest = read_manifest(&entry.path())?;
        result.push(TrashEntry {
            trash_path: item_relative_path(&token),
            restore_token: token,
            original_path: manifest.original_path,
            deleted_at: manifest.deleted_at,
            name: manifest.entry_name,
        });
    }
    Ok(result)
}

pub(super) fn restore(
    descriptor: &VaultDescriptor,
    root: &Path,
    trash_path: &str,
) -> AdapterResult<RestoreResult> {
    let trash_path = relative_path::validate(trash_path)?;
    let token = token_from_trash_path(&trash_path)?;
    let item_path = trash_root(root).join(token);
    let item_metadata = fs::symlink_metadata(&item_path)
        .map_err(|error| AdapterError::new(format!("Invalid trash item {trash_path}: {error}")))?;
    if !item_metadata.is_dir() || item_metadata.file_type().is_symlink() {
        return Err(AdapterError::new(format!(
            "Invalid trash item path: {trash_path}"
        )));
    }

    let manifest = read_manifest(&item_path)?;
    let original_path = relative_path::validate(&manifest.original_path)?;
    if original_path.is_empty()
        || !vault_layout::is_visible_vault_path(&original_path)
        || original_path
            .split('/')
            .any(production_entries::is_ignored_entry)
    {
        return Err(AdapterError::new(
            "Trash manifest contains an invalid restore path.",
        ));
    }
    let expected_name = Path::new(&original_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AdapterError::new("Trash manifest has no entry name."))?;
    if expected_name != manifest.entry_name {
        return Err(AdapterError::new(
            "Trash manifest entry name does not match its restore path.",
        ));
    }

    let target = root.join(&original_path);
    if target.exists() {
        return Err(AdapterError::new(format!(
            "Cannot restore over an existing path: {original_path}"
        )));
    }
    let source_relative = format!("{trash_path}/{CONTENT_DIR}/{}", manifest.entry_name);
    let target_parent = parent_relative_path(&original_path);
    production_entries::move_entry(
        descriptor,
        source_relative,
        (!target_parent.is_empty()).then_some(target_parent),
    )
    .map_err(|error| AdapterError::new(format!("Unable to restore {original_path}: {error}")))?;
    if !target.exists() {
        return Err(AdapterError::new(format!(
            "Restore did not produce the requested path: {original_path}"
        )));
    }

    let cleanup_error = cleanup_item(&item_path);
    Ok(RestoreResult {
        restored: true,
        path: original_path.clone(),
        original_path,
        trash_path,
        cleanup_error,
    })
}

pub(super) fn empty(root: &Path) -> AdapterResult<EmptyTrashResult> {
    let trash_root = trash_root(root);
    let entries = match fs::read_dir(&trash_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(EmptyTrashResult {
                emptied: true,
                count: 0,
                removed: Vec::new(),
            });
        }
        Err(error) => return Err(error.into()),
    };
    let mut entries = entries.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut removed = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let file_type = entry.file_type()?;
        if file_type.is_dir() && !file_type.is_symlink() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
        removed.push(name);
    }
    Ok(EmptyTrashResult {
        emptied: true,
        count: removed.len(),
        removed,
    })
}

fn reserve_item(root: &Path) -> AdapterResult<(String, PathBuf)> {
    let trash_root = trash_root(root);
    fs::create_dir_all(&trash_root)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for suffix in 0..1000_u16 {
        let token = format!("{stamp}-{}-{suffix}", std::process::id());
        let path = trash_root.join(&token);
        match fs::create_dir(&path) {
            Ok(()) => return Ok((token, path)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(AdapterError::new(
        "Unable to reserve a unique trash restore token.",
    ))
}

fn write_manifest(item_path: &Path, manifest: &TrashManifest) -> AdapterResult<()> {
    let destination = item_path.join(MANIFEST_FILE);
    let temporary = item_path.join("manifest.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(manifest)?)?;
    fs::rename(&temporary, &destination)?;
    Ok(())
}

fn read_manifest(item_path: &Path) -> AdapterResult<TrashManifest> {
    let path = item_path.join(MANIFEST_FILE);
    let raw = fs::read_to_string(&path).map_err(|error| {
        AdapterError::new(format!("Unable to read {}: {error}", path.display()))
    })?;
    let manifest: TrashManifest = serde_json::from_str(&raw).map_err(|error| {
        AdapterError::new(format!(
            "Invalid trash manifest {}: {error}",
            path.display()
        ))
    })?;
    if manifest.schema_version != MANIFEST_VERSION {
        return Err(AdapterError::new(format!(
            "Unsupported trash manifest version {} in {}",
            manifest.schema_version,
            path.display()
        )));
    }
    Ok(manifest)
}

fn cleanup_item(item_path: &Path) -> Option<String> {
    fs::remove_file(item_path.join(MANIFEST_FILE))
        .err()
        .or_else(|| fs::remove_dir(item_path.join(CONTENT_DIR)).err())
        .or_else(|| fs::remove_dir(item_path).err())
        .map(|error| error.to_string())
}

fn trash_root(root: &Path) -> PathBuf {
    vault_layout::hidden_dir(root, vault_layout::TRASH_DIR)
}

fn item_relative_path(token: &str) -> String {
    format!(
        "{}/{}/{}",
        vault_layout::HIDDEN_ROOT,
        vault_layout::TRASH_DIR,
        token
    )
}

fn token_from_trash_path(path: &str) -> AdapterResult<&str> {
    let prefix = format!("{}/{}/", vault_layout::HIDDEN_ROOT, vault_layout::TRASH_DIR);
    let token = path
        .strip_prefix(&prefix)
        .filter(|token| !token.is_empty() && !token.contains('/'))
        .ok_or_else(|| AdapterError::new(format!("Invalid trash item path: {path}")))?;
    Ok(token)
}

fn parent_relative_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}
