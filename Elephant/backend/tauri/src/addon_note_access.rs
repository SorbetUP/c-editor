use serde::Serialize;
use std::{
  collections::BTreeSet,
  fs::{self, OpenOptions},
  io::{ErrorKind, Write},
  path::{Component, Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

use crate::addon_runtime_access::{canonical_vault_root, normalize_relative_path, read_enabled_addon, scope_matches};

type R<T> = Result<T, String>;

const MAX_DIRECTORY_DEPTH: usize = 64;
const MAX_LISTED_NOTES: usize = 1_000;
const MAX_NOTE_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonNoteEntry {
  path: String,
  size: u64,
  modified_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonNoteDocument {
  path: String,
  size: u64,
  modified_at: Option<u64>,
  markdown: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonNoteWriteResult {
  path: String,
  size: u64,
  modified_at: Option<u64>,
  created: bool,
}

fn normalize_listing_prefix(value: &str) -> R<String> {
  if value.trim() == "." {
    return Ok(String::new());
  }
  normalize_relative_path(value, "A note directory or '.' for the vault root is required")
}

fn is_hidden_component(path: &Path) -> bool {
  path.components().any(|component| match component {
    Component::Normal(part) => part.to_string_lossy().starts_with('.'),
    _ => false,
  })
}

fn permitted(scopes: &[String], relative_path: &str) -> bool {
  !scopes.is_empty() && scopes.iter().any(|scope| scope_matches(scope, relative_path))
}

fn modified_at(metadata: &fs::Metadata) -> Option<u64> {
  metadata
    .modified()
    .ok()
    .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
    .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
}

fn validate_markdown_path(value: &str) -> R<String> {
  let relative_path = normalize_relative_path(value, "A note path is required")?;
  let relative = Path::new(&relative_path);
  if is_hidden_component(relative) {
    return Err("Addons cannot access notes in hidden directories".to_string());
  }
  if relative
    .extension()
    .and_then(|extension| extension.to_str())
    .map(|extension| extension.eq_ignore_ascii_case("md"))
    != Some(true)
  {
    return Err("Addon note access is limited to Markdown files".to_string());
  }
  Ok(relative_path)
}

fn collect_markdown_notes(root: &Path, prefix: &str, scopes: &[String]) -> R<Vec<AddonNoteEntry>> {
  let start = if prefix.is_empty() { root.to_path_buf() } else { root.join(prefix) };
  if !start.exists() {
    return Ok(Vec::new());
  }
  let canonical_start = fs::canonicalize(&start).map_err(|error| error.to_string())?;
  if !canonical_start.starts_with(root) {
    return Err("Refusing to list notes outside the active vault".to_string());
  }

  let mut stack = vec![(canonical_start, 0_usize)];
  let mut seen = BTreeSet::new();
  let mut notes = Vec::new();

  while let Some((directory, depth)) = stack.pop() {
    if depth > MAX_DIRECTORY_DEPTH {
      return Err(format!("Addon note listing exceeded the maximum directory depth of {MAX_DIRECTORY_DEPTH}"));
    }
    for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
      let entry = entry.map_err(|error| error.to_string())?;
      let file_type = entry.file_type().map_err(|error| error.to_string())?;
      if file_type.is_symlink() {
        continue;
      }
      let path = entry.path();
      let relative = path
        .strip_prefix(root)
        .map_err(|_| "Listed note escaped the active vault".to_string())?;
      if is_hidden_component(relative) {
        continue;
      }
      let relative_path = relative.to_string_lossy().replace('\\', "/");

      if file_type.is_dir() {
        stack.push((path, depth + 1));
        continue;
      }
      if !file_type.is_file()
        || path
          .extension()
          .and_then(|value| value.to_str())
          .map(|value| value.eq_ignore_ascii_case("md"))
          != Some(true)
      {
        continue;
      }
      if !permitted(scopes, &relative_path) || !seen.insert(relative_path.clone()) {
        continue;
      }

      if notes.len() >= MAX_LISTED_NOTES {
        return Err(format!("Addon note listing exceeded the maximum of {MAX_LISTED_NOTES} notes"));
      }

      let metadata = entry.metadata().map_err(|error| error.to_string())?;
      notes.push(AddonNoteEntry {
        path: relative_path,
        size: metadata.len(),
        modified_at: modified_at(&metadata),
      });
    }
  }

  notes.sort_by(|left, right| left.path.cmp(&right.path));
  Ok(notes)
}

fn prepare_write_target(root: &Path, relative_path: &str) -> R<PathBuf> {
  let relative = Path::new(relative_path);
  let file_name = relative
    .file_name()
    .ok_or_else(|| "A note file name is required".to_string())?
    .to_os_string();
  let parent = relative.parent().unwrap_or_else(|| Path::new(""));
  let mut current = root.to_path_buf();

  for component in parent.components() {
    let Component::Normal(part) = component else {
      return Err("Unsafe addon note path".to_string());
    };
    current.push(part);
    match fs::symlink_metadata(&current) {
      Ok(metadata) if metadata.file_type().is_symlink() => {
        return Err(format!("Addon note parent is a symbolic link: {}", current.display()));
      }
      Ok(metadata) if !metadata.is_dir() => {
        return Err(format!("Addon note parent is not a directory: {}", current.display()));
      }
      Ok(_) => {}
      Err(error) if error.kind() == ErrorKind::NotFound => {
        fs::create_dir(&current).map_err(|create_error| create_error.to_string())?;
      }
      Err(error) => return Err(error.to_string()),
    }
    let canonical = fs::canonicalize(&current).map_err(|error| error.to_string())?;
    if !canonical.starts_with(root) {
      return Err("Refusing to write a note outside the active vault".to_string());
    }
  }

  Ok(current.join(file_name))
}

fn write_markdown_atomic(target: &Path, markdown: &str, overwrite: bool) -> R<bool> {
  let created = !target.exists();
  if !created {
    let metadata = fs::symlink_metadata(target).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
      return Err("Addon note target is not a regular file".to_string());
    }
    if !overwrite {
      return Err("Addon note already exists and overwrite was not requested".to_string());
    }
  }

  let file_name = target.file_name().and_then(|value| value.to_str()).unwrap_or("note.md");
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_nanos())
    .unwrap_or_default();
  let temporary = target.with_file_name(format!(".{file_name}.{}-{nonce}.addonpart", std::process::id()));
  let mut file = OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&temporary)
    .map_err(|error| error.to_string())?;
  file.write_all(markdown.as_bytes()).map_err(|error| error.to_string())?;
  file.sync_all().map_err(|error| error.to_string())?;
  drop(file);

  if !overwrite {
    match fs::hard_link(&temporary, target) {
      Ok(()) => {
        fs::remove_file(&temporary).map_err(|error| error.to_string())?;
        return Ok(true);
      }
      Err(error) => {
        let _ = fs::remove_file(&temporary);
        if matches!(error.kind(), ErrorKind::AlreadyExists | ErrorKind::PermissionDenied) {
          return Err("Addon note already exists and overwrite was not requested".to_string());
        }
        return Err(error.to_string());
      }
    }
  }

  match fs::rename(&temporary, target) {
    Ok(()) => Ok(created),
    Err(error) if matches!(error.kind(), ErrorKind::AlreadyExists | ErrorKind::PermissionDenied) => {
      fs::remove_file(target).map_err(|remove_error| remove_error.to_string())?;
      fs::rename(&temporary, target).map_err(|rename_error| rename_error.to_string())?;
      Ok(false)
    }
    Err(error) => {
      let _ = fs::remove_file(&temporary);
      Err(error.to_string())
    }
  }
}

#[tauri::command]
pub fn tauri_addons_notes_list(app: AppHandle, addon_id: String, prefix: String) -> R<Vec<AddonNoteEntry>> {
  let prefix = normalize_listing_prefix(&prefix)?;
  let record = read_enabled_addon(&app, &addon_id)?;
  let scopes = &record.manifest.permissions.notes.read;
  let can_list = if prefix.is_empty() {
    scopes.iter().any(|scope| scope.trim() == "*")
  } else {
    permitted(scopes, &prefix)
  };
  if !can_list {
    let display = if prefix.is_empty() { "the vault root" } else { &prefix };
    return Err(format!("Addon is not permitted to list notes under {display}"));
  }
  collect_markdown_notes(&canonical_vault_root(&app)?, &prefix, scopes)
}

#[tauri::command]
pub fn tauri_addons_notes_read(app: AppHandle, addon_id: String, path: String) -> R<AddonNoteDocument> {
  let relative_path = validate_markdown_path(&path)?;
  let record = read_enabled_addon(&app, &addon_id)?;
  let scopes = &record.manifest.permissions.notes.read;
  if !permitted(scopes, &relative_path) {
    return Err(format!("Addon is not permitted to read {relative_path}"));
  }

  let root = canonical_vault_root(&app)?;
  let target = fs::canonicalize(root.join(&relative_path))
    .map_err(|error| format!("Failed to resolve note {relative_path}: {error}"))?;
  if !target.starts_with(&root) {
    return Err("Refusing to read a note outside the active vault".to_string());
  }
  let metadata = fs::metadata(&target).map_err(|error| error.to_string())?;
  if !metadata.is_file() {
    return Err(format!("Note is not a regular file: {relative_path}"));
  }
  if metadata.len() > MAX_NOTE_BYTES {
    return Err(format!("Note exceeds the {MAX_NOTE_BYTES} byte addon read limit"));
  }
  let markdown = fs::read_to_string(&target)
    .map_err(|error| format!("Failed to read note {relative_path} as UTF-8: {error}"))?;
  Ok(AddonNoteDocument {
    path: relative_path,
    size: metadata.len(),
    modified_at: modified_at(&metadata),
    markdown,
  })
}

#[tauri::command]
pub fn tauri_addons_notes_write(
  app: AppHandle,
  addon_id: String,
  path: String,
  markdown: String,
  overwrite: Option<bool>,
) -> R<AddonNoteWriteResult> {
  if markdown.len() as u64 > MAX_NOTE_BYTES {
    return Err(format!("Note exceeds the {MAX_NOTE_BYTES} byte addon write limit"));
  }
  let relative_path = validate_markdown_path(&path)?;
  let record = read_enabled_addon(&app, &addon_id)?;
  if !permitted(&record.manifest.permissions.notes.write, &relative_path) {
    return Err(format!("Addon is not permitted to write {relative_path}"));
  }

  let root = canonical_vault_root(&app)?;
  let target = prepare_write_target(&root, &relative_path)?;
  let created = write_markdown_atomic(&target, &markdown, overwrite.unwrap_or(false))?;
  let metadata = fs::metadata(&target).map_err(|error| error.to_string())?;
  Ok(AddonNoteWriteResult {
    path: relative_path,
    size: metadata.len(),
    modified_at: modified_at(&metadata),
    created,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_root_sentinel_and_rejects_traversal() {
    assert_eq!(normalize_listing_prefix(".").unwrap(), "");
    assert!(normalize_listing_prefix("../Inbox").is_err());
    assert_eq!(normalize_listing_prefix("Inbox/2026").unwrap(), "Inbox/2026");
  }

  #[test]
  fn hidden_paths_are_never_listed_read_or_written() {
    assert!(is_hidden_component(Path::new(".elephantnote/addons/registry.json")));
    assert!(is_hidden_component(Path::new("Inbox/.private/note.md")));
    assert!(!is_hidden_component(Path::new("Inbox/note.md")));
    assert!(validate_markdown_path(".private/note.md").is_err());
  }

  #[test]
  fn permission_scopes_are_boundary_aware() {
    let scopes = vec!["Inbox/**".to_string()];
    assert!(permitted(&scopes, "Inbox/note.md"));
    assert!(!permitted(&scopes, "Inbox-old/note.md"));
  }

  #[test]
  fn atomic_writer_rejects_overwrite_without_explicit_permission() {
    let root = std::env::temp_dir().join(format!(
      "elephant-addon-note-write-{}-{}",
      std::process::id(),
      SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let target = root.join("note.md");
    assert!(write_markdown_atomic(&target, "first", false).unwrap());
    assert!(write_markdown_atomic(&target, "second", false).is_err());
    assert!(!write_markdown_atomic(&target, "second", true).unwrap());
    assert_eq!(fs::read_to_string(&target).unwrap(), "second");
    let _ = fs::remove_dir_all(root);
  }

  #[cfg(unix)]
  #[test]
  fn write_target_rejects_symlinked_parent() {
    use std::os::unix::fs::symlink;
    let root = std::env::temp_dir().join(format!(
      "elephant-addon-note-root-{}-{}",
      std::process::id(),
      SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    let outside = std::env::temp_dir().join(format!(
      "elephant-addon-note-outside-{}-{}",
      std::process::id(),
      SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("Imported")).unwrap();
    assert!(prepare_write_target(&root, "Imported/note.md").is_err());
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
  }
}
