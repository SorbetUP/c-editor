use serde_json::{json, Value};
use std::io::Read;
use tauri::AppHandle;

use super::config::{get_active_vault, read_config, remove_vault, set_active_vault, set_vault_icon, set_vault_name, upsert_vault, write_config};
use super::entries;
use super::metadata::{initialize_vault, read_json_or};
use super::types::active_vault;
use crate::vault_layout;

type R<T> = Result<T, String>;
const SEARCH_RESULT_LIMIT: usize = 50;
const SEARCH_RESULT_LIMIT_MAX: usize = 200;
const SEARCH_FILE_READ_LIMIT: usize = 256 * 1024;
const SEARCH_FILE_READ_LIMIT_MIN: usize = 4 * 1024;
const SEARCH_FILE_READ_LIMIT_MAX: usize = 1024 * 1024;
const SEARCH_EXCERPT_LIMIT: usize = 600;
const DIRECTORY_LIST_LIMIT_MAX: usize = 500;

fn payload(app: &AppHandle, vault: Option<super::types::VaultDescriptor>) -> R<Value> {
  let config = read_config(app)?;
  if let Some(vault) = vault {
    let workspace = initialize_vault(&vault.path)?;
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
  let vault = upsert_vault(&mut config, vault_path);
  write_config(&app, &config)?;
  payload(&app, Some(vault))
}

#[tauri::command]
pub fn tauri_vaults_set_active(app: AppHandle, vault_id: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_active_vault(&mut config, vault_id);
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_set_icon(app: AppHandle, vault_id: String, icon: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_vault_icon(&mut config, &vault_id, icon);
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_set_name(app: AppHandle, vault_id: String, name: String) -> R<Value> {
  let mut config = read_config(&app)?;
  set_vault_name(&mut config, &vault_id, name);
  let vault = active_vault(&config);
  write_config(&app, &config)?;
  payload(&app, vault)
}

#[tauri::command]
pub fn tauri_vaults_remove(app: AppHandle, vault_id: String) -> R<Value> {
  let mut config = read_config(&app)?;
  remove_vault(&mut config, &vault_id);
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
pub fn tauri_sources_list(app: AppHandle) -> R<Vec<Value>> {
  let vault = get_active_vault(&app)?;
  Ok(read_json_or(vault_layout::config_file(&vault.path, vault_layout::SOURCES_FILE), json!({ "sources": [] })).get("sources").and_then(Value::as_array).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn tauri_search_query(app: AppHandle, params: Option<Value>) -> R<Vec<Value>> {
  let vault = get_active_vault(&app)?;
  let params_ref = params.as_ref();
  let query = params_ref
    .and_then(|p| p.get("query").or_else(|| p.get("q")).and_then(Value::as_str).map(str::to_string))
    .unwrap_or_default()
    .to_lowercase();
  if query.trim().is_empty() {
    return Ok(Vec::new());
  }

  let limit = params_ref
    .and_then(|p| p.get("limit").or_else(|| p.get("maxResults")).and_then(Value::as_u64))
    .map(|value| value.clamp(1, SEARCH_RESULT_LIMIT_MAX as u64) as usize)
    .unwrap_or(SEARCH_RESULT_LIMIT);
  let max_file_bytes = params_ref
    .and_then(|p| p.get("maxBytesPerFile").and_then(Value::as_u64))
    .map(|value| value.clamp(SEARCH_FILE_READ_LIMIT_MIN as u64, SEARCH_FILE_READ_LIMIT_MAX as u64) as usize)
    .unwrap_or(SEARCH_FILE_READ_LIMIT);

  let root = std::path::PathBuf::from(&vault.path);
  if let Ok(index) = crate::fts::FtsIndex::open(&root) {
    let hits = index.search(&query, limit).unwrap_or_default();
    if !hits.is_empty() || index.count(&vault.id).unwrap_or(0) > 0 {
      return Ok(hits
        .into_iter()
        .map(|hit| json!({
          "path": hit.path,
          "fullPath": hit.full_path,
          "title": hit.title,
          "excerpt": hit.excerpt,
          "tags": hit.tags,
          "score": hit.score,
        }))
        .collect());
    }
  }

  let mut results = Vec::with_capacity(limit.min(SEARCH_RESULT_LIMIT));
  scan_notes(&root, &root, &mut results, &query, limit, max_file_bytes)?;
  Ok(results)
}

fn read_text_prefix(path: &std::path::Path, max_bytes: usize) -> R<String> {
  let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut limited = file.take(max_bytes as u64);
  let mut buffer = Vec::with_capacity(max_bytes.min(8 * 1024));
  limited.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
  Ok(String::from_utf8_lossy(&buffer).to_string())
}

fn search_excerpt(markdown: &str) -> String {
  let excerpt = markdown
    .lines()
    .filter(|line| !line.trim().is_empty())
    .take(3)
    .collect::<Vec<_>>()
    .join(" ");
  if excerpt.chars().count() <= SEARCH_EXCERPT_LIMIT {
    excerpt
  } else {
    format!("{}…", excerpt.chars().take(SEARCH_EXCERPT_LIMIT).collect::<String>())
  }
}

fn scan_notes(root: &std::path::Path, current: &std::path::Path, out: &mut Vec<Value>, query: &str, limit: usize, max_file_bytes: usize) -> R<()> {
  if out.len() >= limit {
    return Ok(());
  }

  for item in std::fs::read_dir(current).map_err(|e| e.to_string())? {
    if out.len() >= limit {
      break;
    }

    let item = item.map_err(|e| e.to_string())?;
    let name = item.file_name().to_string_lossy().to_string();
    if entries::is_ignored_entry(&name) {
      continue;
    }

    let path = item.path();
    let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() {
      continue;
    }

    if metadata.is_dir() {
      scan_notes(root, &path, out, query, limit, max_file_bytes)?;
    } else if metadata.is_file() && name.to_ascii_lowercase().ends_with(".md") {
      let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
      let path_matches = relative.to_lowercase().contains(query) || name.to_lowercase().contains(query);
      let preview = read_text_prefix(&path, max_file_bytes).unwrap_or_default();
      let content_matches = path_matches || preview.to_lowercase().contains(query);
      if content_matches {
        out.push(json!({
          "path": relative,
          "fullPath": path.to_string_lossy(),
          "title": name.trim_end_matches(".md"),
          "excerpt": search_excerpt(&preview),
          "tags": [],
          "score": if path_matches { 2 } else { 1 }
        }));
      }
    }
  }
  Ok(())
}

#[tauri::command]
pub fn tauri_search_status(app: AppHandle) -> R<Value> {
  Ok(json!({ "enabled": true, "runtime": "tauri-rust", "activeVault": active_vault(&read_config(&app)?) }))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn temp_dir(name: &str) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    std::env::temp_dir().join(format!("elephant-{name}-{stamp}"))
  }

  #[test]
  fn search_scan_respects_result_limit() {
    let root = temp_dir("search-limit");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.md"), "needle one").unwrap();
    std::fs::write(root.join("b.md"), "needle two").unwrap();

    let mut out = Vec::new();
    scan_notes(&root, &root, &mut out, "needle", 1, 1024).unwrap();
    assert_eq!(out.len(), 1);

    let _ = std::fs::remove_dir_all(&root);
  }

  #[test]
  fn search_scan_ignores_hidden_directories() {
    let root = temp_dir("search-hidden");
    let hidden = root.join(".git");
    std::fs::create_dir_all(&hidden).unwrap();
    std::fs::write(hidden.join("secret.md"), "needle").unwrap();

    let mut out = Vec::new();
    scan_notes(&root, &root, &mut out, "needle", 10, 1024).unwrap();
    assert!(out.is_empty());

    let _ = std::fs::remove_dir_all(&root);
  }
}
