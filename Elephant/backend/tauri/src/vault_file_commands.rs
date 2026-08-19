use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::{
  fs,
  path::{Component, Path, PathBuf},
};
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
  if pathname.contains('\0') || pathname.replace('\\', "/").split('/').any(|part| part == "..") {
    return Err(format!("Parent path components are not allowed: {pathname}"));
  }
  let candidate = PathBuf::from(pathname);
  let candidate = if candidate.is_absolute() {
    candidate
  } else {
    root.join(candidate)
  };
  // Reject lexically-outside absolute destinations before any directory is
  // created. Canonical checks below still protect existing symlink components.
  if !candidate.starts_with(root) {
    return Err(format!(
      "Refusing a vault path outside the active root: {}",
      candidate.to_string_lossy()
    ));
  }
  Ok(candidate)
}

fn existing_path_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  let metadata = fs::symlink_metadata(&candidate).map_err(|error| error.to_string())?;
  if metadata.file_type().is_symlink() {
    return Err(format!(
      "Refusing to access a symlinked vault path: {}",
      candidate.to_string_lossy()
    ));
  }
  let resolved = fs::canonicalize(&candidate).map_err(|error| error.to_string())?;
  if !resolved.starts_with(&root) {
    return Err(format!("Refusing to access a path outside the active vault: {}", resolved.to_string_lossy()));
  }
  Ok(resolved)
}

/// Walk/create a directory beneath a canonical vault root without ever
/// traversing a symlink component. This check happens before each creation, so
/// a malicious `vault/link -> /outside` cannot make `create_dir_all` mutate the
/// external target before we notice the escape.
fn ensure_directory_inside_root(root: &Path, directory: &Path) -> R<PathBuf> {
  let relative = directory
    .strip_prefix(root)
    .map_err(|_| format!("Directory is outside the active vault: {}", directory.to_string_lossy()))?;
  let mut current = root.to_path_buf();
  for component in relative.components() {
    let Component::Normal(part) = component else {
      return Err(format!("Unsafe vault directory component: {}", directory.to_string_lossy()));
    };
    current.push(part);
    match fs::symlink_metadata(&current) {
      Ok(metadata) if metadata.file_type().is_symlink() => {
        return Err(format!(
          "Refusing to follow a symlinked vault directory: {}",
          current.to_string_lossy()
        ));
      }
      Ok(metadata) if metadata.is_dir() => {}
      Ok(_) => {
        return Err(format!(
          "Vault directory component is not a directory: {}",
          current.to_string_lossy()
        ));
      }
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        fs::create_dir(&current).map_err(|error| error.to_string())?;
      }
      Err(error) => return Err(error.to_string()),
    }
    let canonical = fs::canonicalize(&current).map_err(|error| error.to_string())?;
    if !canonical.starts_with(root) {
      return Err(format!(
        "Vault directory escaped the active root: {}",
        canonical.to_string_lossy()
      ));
    }
    current = canonical;
  }
  Ok(current)
}

fn writable_path_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  let parent = candidate.parent().ok_or_else(|| "The destination has no parent directory.".to_string())?;
  let parent = ensure_directory_inside_root(&root, parent)?;
  let file_name = candidate.file_name().ok_or_else(|| "The destination has no file name.".to_string())?;
  let target = parent.join(file_name);
  if let Ok(metadata) = fs::symlink_metadata(&target) {
    if metadata.file_type().is_symlink() {
      return Err(format!("Refusing to write through a symlink: {}", target.to_string_lossy()));
    }
    let canonical = fs::canonicalize(&target).map_err(|error| error.to_string())?;
    if !canonical.starts_with(&root) {
      return Err(format!("Refusing to write outside the active vault: {}", canonical.to_string_lossy()));
    }
  }
  Ok(target)
}

fn writable_directory_inside_root(app: &AppHandle, pathname: &str) -> R<PathBuf> {
  let root = canonical_root(app)?;
  let candidate = candidate_path(&root, pathname)?;
  if candidate == root {
    return Ok(root);
  }
  ensure_directory_inside_root(&root, &candidate)
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
  Ok(json!({ "ok": true, "pathname": path.to_string_lossy() }))
}

#[tauri::command]
pub fn tauri_vault_remove_path(app: AppHandle, pathname: String) -> R<Value> {
  let path = existing_path_inside_root(&app, &pathname)?;
  let root = canonical_root(&app)?;
  if path == root {
    return Err("The vault root cannot be removed.".to_string());
  }
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

  #[test]
  fn lexical_absolute_escape_is_rejected_before_creation() {
    let root = PathBuf::from("/tmp/elephant-vault");
    assert!(candidate_path(&root, "/tmp/outside/new/file.bin").is_err());
  }

  #[test]
  fn nested_directory_creation_stays_beneath_root() {
    let root = std::env::temp_dir().join(format!("elephant-vault-path-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let root = fs::canonicalize(&root).unwrap();
    let nested = root.join("A/B/C");
    let created = ensure_directory_inside_root(&root, &nested).unwrap();
    assert!(created.starts_with(&root));
    assert!(created.is_dir());
    fs::remove_dir_all(root).ok();
  }

  #[cfg(unix)]
  #[test]
  fn symlink_parent_is_rejected_before_external_subdirectory_is_created() {
    use std::os::unix::fs::symlink;
    let base = std::env::temp_dir().join(format!("elephant-vault-symlink-{}", std::process::id()));
    let root = base.join("vault");
    let outside = base.join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let root = fs::canonicalize(&root).unwrap();
    symlink(&outside, root.join("link")).unwrap();
    let target = root.join("link/should-not-exist");
    assert!(ensure_directory_inside_root(&root, &target).is_err());
    assert!(!outside.join("should-not-exist").exists());
    fs::remove_dir_all(base).ok();
  }
}
