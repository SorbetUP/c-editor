use serde_json::{json, Value};
use tauri::AppHandle;

use super::config::{get_active_vault, read_config, remove_vault, set_active_vault, set_vault_enabled, set_vault_icon, set_vault_name, upsert_vault, write_config};
use super::entries;
use super::metadata::{initialize_vault, open_vault, read_json_or};
use super::types::active_vault;
use crate::vault_layout;

type R<T> = Result<T, String>;
const SEARCH_RESULT_LIMIT: usize = 50;
const SEARCH_RESULT_LIMIT_MAX: usize = 200;
const DIRECTORY_LIST_LIMIT_MAX: usize = 500;

fn payload(app: &AppHandle, vault: Option<super::types::VaultDescriptor>) -> R<Value> {
  let config = read_config(app)?;
  if let Some(vault) = vault {
    let workspace = open_vault(&vault.path)?;
    let listed_entries = entries::list_directory_page(&vault, "", 0, Some(120), true)?;
    Ok(json!({
      "vaults": config.vaults,
      "activeVaultId": config.active_vault_id,
      "activeVault": vault,
      "workspace": workspace,
      "entries": listed_entries
    }))
  } else {
    Ok(json!({
      "vaults": config.vaults,
      "activeVaultId": config.active_vault_id,
      "activeVault": null,
      "workspace": null,
      "entries": []
    }))
  }
}

#[tauri::command]
pub fn tauri_vaults_get(app: AppHandle) -> R<Value> {
  let config = read_config(&app)?;
  payload(&app, active_vault(&config))
}

#[tauri::command]
pub fn tauri_vaults_select_path(app: AppHandle, vault_path: String) -> R<Value> {
  let mut config = read_config(&app)?;
  let vault = upsert_vault(&mut config, vault_path)?;
  write_config(&app, &config)?;
  initialize_vault(&vault.path)?;
  payload(&app, Some(vault))
}

#[tauri::command]
pub fn tauri_vaults_set_active(app: AppHandle, vault_id: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_active_vault(&mut config, vault_id)?;
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_set_icon(app: AppHandle, vault_id: String, icon: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_vault_icon(&mut config, &vault_id, icon)?;
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_set_name(app: AppHandle, vault_id: String, name: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_vault_name(&mut config, &vault_id, name)?;
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_remove(app: AppHandle, vault_id: String) -> R<Value> {
  let mut config = read_config(&app)?;
  remove_vault(&mut config, &vault_id)?;
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_set_enabled(app: AppHandle, vault_id: String, enabled: bool) -> R<Value> {
  let mut config = read_config(&app)?;
  set_vault_enabled(&mut config, &vault_id, enabled)?;
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_directory_list(app: AppHandle, relative_path: Option<String>, offset: Option<u64>, limit: Option<u64>, include_preview: Option<bool>) -> R<Vec<Value>> {
  let offset = offset.unwrap_or(0) as usize;
  let limit = limit.map(|value| value.clamp(1, DIRECTORY_LIST_LIMIT_MAX as u64) as usize);
  entries::list_directory_page(
    &get_active_vault(&app)?,
    relative_path.as_deref().unwrap_or(""),
    offset,
    limit,
    include_preview.unwrap_or(true),
  )
}

#[tauri::command]
pub fn tauri_notes_create(app: AppHandle, relative_path: Option<String>, filename: Option<String>, title: Option<String>) -> R<Value> {
  entries::create_note(&get_active_vault(&app)?, relative_path, filename, title)
}

#[tauri::command]
pub fn tauri_folders_create(app: AppHandle, relative_path: Option<String>) -> R<Value> {
  entries::create_folder(&get_active_vault(&app)?, relative_path)
}

#[tauri::command]
pub fn tauri_sidebar_attach(app: AppHandle, relative_path: String, title: Option<String>, entry_type: Option<String>) -> R<Value> {
  entries::attach_sidebar_entry(&get_active_vault(&app)?, relative_path, title, entry_type)
}

#[tauri::command]
pub fn tauri_sidebar_detach(app: AppHandle, relative_path: String) -> R<Value> {
  entries::detach_sidebar_entry(&get_active_vault(&app)?, relative_path)
}

#[tauri::command]
pub fn tauri_entries_rename(app: AppHandle, relative_path: String, title: String) -> R<Value> {
  let vault = get_active_vault(&app)?;
  entries::rename_entry(&vault, relative_path, title)?;
  payload(&app, Some(vault))
}

#[tauri::command]
pub fn tauri_entries_move(app: AppHandle, relative_path: String, target_directory_path: Option<String>) -> R<Value> {
  let vault = get_active_vault(&app)?;
  entries::move_entry(&vault, relative_path, target_directory_path)?;
  payload(&app, Some(vault))
}

#[tauri::command]
pub fn tauri_entries_delete(app: AppHandle, relative_path: String) -> R<Value> {
  entries::delete_entry(&get_active_vault(&app)?, relative_path)
}

#[tauri::command]
pub fn tauri_vault_trash_list(app: AppHandle) -> R<Vec<Value>> {
  entries::list_trash(&get_active_vault(&app)?)
}

#[tauri::command]
pub fn tauri_vault_trash_restore(app: AppHandle, trash_path: String) -> R<Value> {
  entries::restore_trash(&get_active_vault(&app)?, trash_path)
}

#[tauri::command]
pub fn tauri_vault_trash_empty(app: AppHandle) -> R<Value> {
  entries::empty_trash(&get_active_vault(&app)?)
}

#[tauri::command]
pub fn tauri_sources_list(app: AppHandle) -> R<Vec<Value>> {
  let vault = get_active_vault(&app)?;
  Ok(read_json_or(vault_layout::config_file(&vault.path, vault_layout::SOURCES_FILE), json!({ "sources": [] })).get("sources").and_then(Value::as_array).cloned().unwrap_or_default())
}

fn ensure_search_index(vault: &super::types::VaultDescriptor) -> R<crate::fts::FtsIndex> {
  let root = std::path::PathBuf::from(&vault.path);
  let index = crate::fts::FtsIndex::open(&root)
    .map_err(|error| format!("Unable to open search index: {error}"))?;
  let count = index
    .count(&vault.id)
    .map_err(|error| format!("Unable to inspect search index: {error}"))?;
  if count == 0 {
    let refresh = index
      .rebuild_from_files(&vault.id, &root)
      .map_err(|error| format!("Unable to rebuild search index: {error}"))?;
    if !refresh.failed.is_empty() && refresh.indexed == 0 && refresh.unchanged == 0 {
      return Err(format!(
        "Search index rebuild failed for {} path(s)",
        refresh.failed.len()
      ));
    }
  }
  Ok(index)
}

#[tauri::command]
pub fn tauri_search_query(app: AppHandle, params: Option<Value>) -> R<Vec<Value>> {
  let vault = get_active_vault(&app)?;
  let params_ref = params.as_ref();
  let query = params_ref
    .and_then(|p| p.get("query").or_else(|| p.get("q")).and_then(Value::as_str).map(str::to_string))
    .unwrap_or_default();
  if query.trim().is_empty() {
    return Ok(Vec::new());
  }

  let limit = params_ref
    .and_then(|p| p.get("limit").or_else(|| p.get("maxResults")).and_then(Value::as_u64))
    .map(|value| value.clamp(1, SEARCH_RESULT_LIMIT_MAX as u64) as usize)
    .unwrap_or(SEARCH_RESULT_LIMIT);

  let index = ensure_search_index(&vault)?;
  let hits = index
    .search(&query, limit)
    .map_err(|error| format!("Unable to query search index: {error}"))?;
  Ok(hits
    .into_iter()
    .map(|hit| json!({
      "path": hit.path,
      "fullPath": hit.full_path,
      "title": hit.title,
      "excerpt": hit.excerpt,
      "tags": hit.tags,
      "score": hit.score,
    }))
    .collect())
}

#[tauri::command]
pub fn tauri_search_status(app: AppHandle) -> R<Value> {
  let config = read_config(&app)?;
  let active = active_vault(&config);
  let Some(vault) = active.as_ref() else {
    return Ok(json!({
      "enabled": true,
      "runtime": "tauri-rust-fts",
      "activeVault": null,
      "indexedDocuments": 0
    }));
  };
  let root = std::path::PathBuf::from(&vault.path);
  let index = crate::fts::FtsIndex::open(&root)
    .map_err(|error| format!("Unable to open search index: {error}"))?;
  let count = index
    .count(&vault.id)
    .map_err(|error| format!("Unable to inspect search index: {error}"))?;
  Ok(json!({
    "enabled": true,
    "runtime": "tauri-rust-fts",
    "activeVault": vault,
    "indexedDocuments": count.max(0),
    "indexPath": crate::vault_layout::hidden_dir(&root, crate::vault_layout::INDEX_DIR).join("notes.sqlite").to_string_lossy()
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::vault::types::VaultDescriptor;

  fn temp_dir(name: &str) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    std::env::temp_dir().join(format!("elephant-{name}-{stamp}"))
  }

  fn fixture_vault(root: &std::path::Path) -> VaultDescriptor {
    VaultDescriptor {
      id: "search-test".into(),
      name: "Search Test".into(),
      path: root.to_string_lossy().replace('\\', "/"),
      icon: String::new(),
      last_opened_at: "0".into(),
      enabled: true,
    }
  }

  #[test]
  fn search_index_returns_empty_without_falling_back_to_filesystem_scan() {
    let root = temp_dir("search-empty");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.md"), "needle one").unwrap();
    let vault = fixture_vault(&root);
    let index = ensure_search_index(&vault).unwrap();
    assert_eq!(index.search("absent-term", 10).unwrap().len(), 0);
    assert_eq!(index.count(&vault.id).unwrap(), 1);
    let _ = std::fs::remove_dir_all(&root);
  }

  #[test]
  fn initial_search_index_build_ignores_hidden_directories() {
    let root = temp_dir("search-hidden");
    let hidden = root.join(".git");
    std::fs::create_dir_all(&hidden).unwrap();
    std::fs::write(hidden.join("secret.md"), "needle").unwrap();
    std::fs::write(root.join("visible.md"), "visible").unwrap();
    let vault = fixture_vault(&root);
    let index = ensure_search_index(&vault).unwrap();
    assert_eq!(index.count(&vault.id).unwrap(), 1);
    assert!(index.search("needle", 10).unwrap().is_empty());
    let _ = std::fs::remove_dir_all(&root);
  }
}
