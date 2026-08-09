use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::vault_layout;

use super::config::{basename, now_string};

type R<T> = Result<T, String>;

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
  let raw = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
  fs::write(path, raw).map_err(|e| e.to_string())
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
  let only_generated_wiki_file = entries.is_empty() || (entries.len() == 1 && entries[0] == wiki_file);
  if only_generated_wiki_file {
    let _ = fs::remove_file(&wiki_file);
    let _ = fs::remove_dir(&wiki_dir);
  }
}

pub fn workspace_json(vault_root: impl AsRef<Path>) -> Value {
  json!({
    "version": 1,
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
  write_json_if_missing(vault_layout::config_file(vault_root, vault_layout::WORKSPACE_FILE), workspace.clone())?;
  write_json_if_missing(vault_layout::config_file(vault_root, vault_layout::VAULT_FILE), json!({ "version": 1, "createdAt": now_string() }))?;
  write_json_if_missing(vault_layout::index_file(vault_root, vault_layout::INDEX_FILE), json!({ "version": 1, "updatedAt": now_string(), "entries": [] }))?;
  write_json_if_missing(vault_layout::config_file(vault_root, vault_layout::CALENDAR_FILE), json!({ "version": 1, "updatedAt": now_string(), "events": [] }))?;
  write_json_if_missing(vault_layout::config_file(vault_root, vault_layout::SOURCES_FILE), json!({ "version": 1, "updatedAt": now_string(), "sources": [] }))?;
  write_json_if_missing(vault_layout::models_file(vault_root, vault_layout::MODELS_FILE), json!({ "provider": "none", "modelId": "", "local": false }))?;
  write_json_if_missing(vault_layout::sync_file(vault_root, vault_layout::SYNC_FILE), json!({ "version": 1, "queue": [], "lastRunAt": null }))?;

  let welcome = root_path.join("Getting Started").join("Welcome.md");
  if !welcome.exists() {
    let stamp = now_string();
    fs::write(
      welcome,
      format!("---\ntitle: \"Welcome\"\ntype: \"note\"\ntags: []\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"\n---\n\n# Welcome to ElephantNote\n", stamp, stamp),
    ).map_err(|e| e.to_string())?;
  }

  let canonical = vault_layout::config_file(vault_root, vault_layout::WORKSPACE_FILE);
  let compatibility_path = vault_layout::hidden_root(vault_root).join(vault_layout::WORKSPACE_FILE);
  let fallback = read_json_or(compatibility_path, workspace.clone());
  Ok(read_json_or(canonical, fallback))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creates_workspace_shape() {
    let workspace = workspace_json("Personal");
    assert_eq!(workspace["vaultName"], "Personal");
    assert_eq!(workspace["sidebar"][0]["path"], "Getting Started");
  }
}
