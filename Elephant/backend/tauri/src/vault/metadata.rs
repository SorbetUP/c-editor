use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::vault_layout;

use super::config::{basename, now_string};

type R<T> = Result<T, String>;

fn replace_file(temp: &Path, destination: &Path) -> R<()> {
    let rename = fs::rename(temp, destination);
    if rename.is_ok() {
        return Ok(());
    }

    // Windows does not replace an existing destination with rename. Keep the
    // atomic path on platforms that support it, and use the narrowest fallback
    // possible for an already-existing metadata file on Windows.
    #[cfg(windows)]
    if destination.exists() {
        fs::remove_file(destination).map_err(|e| e.to_string())?;
        return fs::rename(temp, destination).map_err(|e| e.to_string());
    }

    rename.map_err(|e| e.to_string())
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> R<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "Metadata path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("metadata"),
        std::process::id(),
        stamp
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        replace_file(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub fn read_json_or(path: impl AsRef<Path>, fallback: Value) -> Value {
    fs::read_to_string(path.as_ref())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(fallback)
}

pub fn write_json(path: PathBuf, value: &Value) -> R<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut value = value.clone();
    if let Some(object) = value.as_object_mut() {
        object
            .entry("schemaVersion")
            .or_insert_with(|| json!(vault_layout::SCHEMA_VERSION));
    }
    let raw = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
    write_atomic(&path, &raw)
}

pub fn write_json_if_missing(path: PathBuf, value: Value) -> R<()> {
    if path.exists() {
        return Ok(());
    }
    write_json(path, &value)
}

fn remove_obsolete_wiki_metadata(vault_root: impl AsRef<Path>) {
    let wiki_dir = vault_layout::hidden_dir(vault_root, vault_layout::WIKI_DIR);
    if !wiki_dir.exists() {
        return;
    }

    let wiki_file = wiki_dir.join(vault_layout::WIKI_FILE);
    let entries = fs::read_dir(&wiki_dir)
        .map(|items| {
            items
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let only_generated_wiki_file =
        entries.is_empty() || (entries.len() == 1 && entries[0] == wiki_file);
    if only_generated_wiki_file {
        let _ = fs::remove_file(&wiki_file);
        let _ = fs::remove_dir(&wiki_dir);
    }
}

pub fn workspace_json(vault_root: impl AsRef<Path>) -> Value {
    json!({
      "version": 1,
      "schemaVersion": vault_layout::SCHEMA_VERSION,
      "vaultName": basename(vault_root.as_ref()),
      "sidebar": [{
        "id": "getting-started",
        "title": "Getting started",
        "type": "folder",
        "path": "Getting Started",
        "collapsed": false
      }]
    })
}

pub fn initialize_vault(vault_root: &str) -> R<Value> {
    let root_path = PathBuf::from(vault_root);
    fs::create_dir_all(vault_layout::hidden_root(&root_path)).map_err(|e| e.to_string())?;
    for dir in vault_layout::required_hidden_dirs() {
        fs::create_dir_all(vault_layout::hidden_dir(&root_path, dir)).map_err(|e| e.to_string())?;
    }
    remove_obsolete_wiki_metadata(&root_path);

    fs::create_dir_all(root_path.join("Getting Started")).map_err(|e| e.to_string())?;
    let workspace = workspace_json(&root_path);
    write_json_if_missing(
        vault_layout::config_file(vault_root, vault_layout::WORKSPACE_FILE),
        workspace.clone(),
    )?;
    write_json_if_missing(
        vault_layout::config_file(vault_root, vault_layout::VAULT_FILE),
        json!({ "version": 1, "schemaVersion": vault_layout::SCHEMA_VERSION, "createdAt": now_string() }),
    )?;
    write_json_if_missing(
        vault_layout::index_file(vault_root, vault_layout::INDEX_FILE),
        json!({ "version": 1, "schemaVersion": vault_layout::SCHEMA_VERSION, "updatedAt": now_string(), "entries": [] }),
    )?;
    write_json_if_missing(
        vault_layout::config_file(vault_root, vault_layout::CALENDAR_FILE),
        json!({ "version": 1, "schemaVersion": vault_layout::SCHEMA_VERSION, "updatedAt": now_string(), "events": [] }),
    )?;
    write_json_if_missing(
        vault_layout::config_file(vault_root, vault_layout::SOURCES_FILE),
        json!({ "version": 1, "schemaVersion": vault_layout::SCHEMA_VERSION, "updatedAt": now_string(), "sources": [] }),
    )?;
    write_json_if_missing(
        vault_layout::models_file(vault_root, vault_layout::MODELS_FILE),
        json!({ "provider": "none", "modelId": "", "local": false }),
    )?;
    write_json_if_missing(
        vault_layout::sync_file(vault_root, vault_layout::SYNC_FILE),
        json!({ "version": 1, "schemaVersion": vault_layout::SCHEMA_VERSION, "queue": [], "lastRunAt": null }),
    )?;

    let welcome = root_path.join("Getting Started").join("Welcome.md");
    if !welcome.exists() {
        let stamp = now_string();
        fs::write(
      welcome,
      format!("---\ntitle: \"Welcome\"\ntype: \"note\"\ntags: []\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"\n---\n\n# Welcome to ElephantNote\n", stamp, stamp),
    ).map_err(|e| e.to_string())?;
    }

    let canonical = vault_layout::config_file(vault_root, vault_layout::WORKSPACE_FILE);
    let compatibility_path =
        vault_layout::hidden_root(vault_root).join(vault_layout::WORKSPACE_FILE);
    let fallback = read_json_or(compatibility_path, workspace.clone());
    Ok(read_json_or(canonical, fallback))
}

pub fn open_vault(vault_root: &str) -> R<Value> {
    let root =
        fs::canonicalize(vault_root).map_err(|e| format!("Vault root is not accessible: {e}"))?;
    if !root.is_dir() {
        return Err(format!(
            "Vault root is not a directory: {}",
            root.to_string_lossy()
        ));
    }
    let workspace = vault_layout::config_file(&root, vault_layout::WORKSPACE_FILE);
    let fallback = vault_layout::hidden_root(&root).join(vault_layout::WORKSPACE_FILE);
    Ok(read_json_or(
        workspace,
        read_json_or(fallback, workspace_json(&root)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_workspace_shape() {
        let workspace = workspace_json("Personal");
        assert_eq!(workspace["vaultName"], "Personal");
        assert_eq!(workspace["sidebar"][0]["path"], "Getting Started");
        assert_eq!(workspace["schemaVersion"], 1);
    }

    #[test]
    fn writes_json_without_exposing_a_partial_file() {
        let root = std::env::temp_dir().join(format!("elephant-metadata-{}", now_string()));
        let path = root.join("metadata.json");
        write_json(path.clone(), &json!({ "version": 1 })).unwrap();
        assert_eq!(read_json_or(&path, json!({}))["schemaVersion"], 1);
        assert!(!root.join("metadata.json.tmp").exists());
        let _ = fs::remove_dir_all(root);
    }
}
