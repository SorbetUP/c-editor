use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::{fs, path::{Path, PathBuf}};
use tauri::AppHandle;

use crate::vault::config as vault_config;

type R<T> = Result<T, String>;

fn canonical_root(app: &AppHandle) -> R<PathBuf> {
  let vault = vault_config::get_active_vault(app)?;
  let root = PathBuf::from(vault.path);
  fs::create_dir_all(&root).map_err(|error| error.to_string())?;
  fs::canonicalize(root).map_err(|error| error.to_string())
}

fn candidate_path(root: &Path, pathname: &str) -> R<PathBuf> {
  if pathname.trim().is_empty() {
    return Err("A vault path is required.".to_string());
  }
  let candidate = PathBuf::from(pathname);
  Ok(if candidate.is_absolute() { candidate } else { root.join(candidate) })
}

fn existing_path_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  let resolved = fs::canonicalize(&candidate).map_err(|error| error.to_string())?;
  if !resolved.starts_with(&root) {
    return Err(format!("Refusing to access a path outside the active vault: {}", resolved.to_string_lossy()));
  }
  Ok(resolved)
}

fn writable_path_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  let parent = candidate.parent().ok_or_else(|| "The destination has no parent directory.".to_string())?;
  fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  let parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
  if !parent.starts_with(&root) {
    return Err(format!("Refusing to write outside the active vault: {}", candidate.to_string_lossy()));
  }
  let file_name = candidate.file_name().ok_or_else(|| "The destination has no file name.".to_string())?;
  Ok(parent.join(file_name))
}

fn writable_directory_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  if candidate == root {
    return Ok(root);
  }
  let parent = candidate.parent().ok_or_else(|| "The directory has no parent.".to_string())?;
  fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  let parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
  if !parent.starts_with(&root) {
    return Err(format!("Refusing to create a directory outside the active vault: {}", candidate.to_string_lossy()));
  }
  let name = candidate.file_name().ok_or_else(|| "The directory has no name.".to_string())?;
  Ok(parent.join(name))
}

#[tauri::command]
pub fn tauri_vault_read_binary(app: AppHandle, pathname: String) -> R<Value> {
  let path = existing_path_inside_root(&app, &pathname)?;
  if !path.is_file() {
    return Err(format!("Cannot read a non-file vault path: {}", path.to_string_lossy()));
  }
  let bytes = fs::read(&path).map_err(|error| error.to_string())?;
  Ok(json!({
    "ok": true,
    "pathname": path.to_string_lossy(),
    "dataBase64": STANDARD.encode(bytes)
  }))
}

#[tauri::command]
pub fn tauri_vault_write_binary(app: AppHandle, pathname: String, data_base64: String) -> R<Value> {
  let path = writable_path_inside_root(&app, &pathname)?;
  let bytes = STANDARD.decode(data_base64.as_bytes()).map_err(|error| format!("Invalid base64 vault payload: {error}"))?;
  fs::write(&path, &bytes).map_err(|error| error.to_string())?;
  Ok(json!({ "ok": true, "pathname": path.to_string_lossy(), "bytesWritten": bytes.len() }))
}

#[tauri::command]
pub fn tauri_vault_ensure_dir(app: AppHandle, pathname: String) -> R<Value> {
  let path = writable_directory_inside_root(&app, &pathname)?;
  fs::create_dir_all(&path).map_err(|error| error.to_string())?;
  Ok(json!({ "ok": true, "pathname": path.to_string_lossy() }))
}

#[tauri::command]
pub fn tauri_vault_remove_path(app: AppHandle, pathname: String) -> R<Value> {
  let path = existing_path_inside_root(&app, &pathname)?;
  if path.is_dir() {
    fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
  } else {
    fs::remove_file(&path).map_err(|error| error.to_string())?;
  }
  Ok(json!({ "ok": true, "pathname": path.to_string_lossy() }))
}

#[tauri::command]
pub fn tauri_vault_rename_path(app: AppHandle, source: String, destination: String) -> R<Value> {
  let source_path = existing_path_inside_root(&app, &source)?;
  let destination_path = writable_path_inside_root(&app, &destination)?;
  fs::rename(&source_path, &destination_path).map_err(|error| error.to_string())?;
  Ok(json!({
    "ok": true,
    "source": source_path.to_string_lossy(),
    "destination": destination_path.to_string_lossy()
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_candidates_stay_under_root() {
    let root = PathBuf::from("/tmp/elephant-vault");
    assert_eq!(candidate_path(&root, "assets/image.png").unwrap(), root.join("assets/image.png"));
  }

  #[test]
  fn empty_paths_are_rejected() {
    let root = PathBuf::from("/tmp/elephant-vault");
    assert!(candidate_path(&root, "").is_err());
  }
}
