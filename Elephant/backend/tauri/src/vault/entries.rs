use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::vault_layout;

use super::config::now_string;
use super::metadata::{read_json_or, write_json};
use super::types::VaultDescriptor;

type R<T> = Result<T, String>;
const MARKDOWN_PREVIEW_LIMIT: usize = 16 * 1024;
const EXCERPT_LINE_LIMIT: usize = 3;

pub fn normalize_relative_path(path: &str) -> String {
  path
    .replace('\\', "/")
    .split('/')
    .filter(|part| !part.is_empty() && *part != "." && *part != "..")
    .collect::<Vec<_>>()
    .join("/")
}

pub fn inside(vault_root: &str, relative_path: &str) -> PathBuf {
  let relative_path = normalize_relative_path(relative_path);
  if relative_path.is_empty() { PathBuf::from(vault_root) } else { PathBuf::from(vault_root).join(relative_path) }
}

fn canonical_root(root: &str) -> R<PathBuf> {
  let root = PathBuf::from(root);
  fs::create_dir_all(&root).map_err(|error| error.to_string())?;
  fs::canonicalize(root).map_err(|error| error.to_string())
}

fn existing_path_inside_vault(vault_root: &str, relative_path: &str) -> R<PathBuf> {
  let root = canonical_root(vault_root)?;
  let path = fs::canonicalize(inside(vault_root, relative_path)).map_err(|error| error.to_string())?;
  if !path.starts_with(&root) {
    return Err(format!("Refusing to access a path outside the active vault: {}", path.to_string_lossy()));
  }
  Ok(path)
}

fn writable_dir_inside_vault(vault_root: &str, relative_path: &str) -> R<PathBuf> {
  let root = canonical_root(vault_root)?;
  let directory = inside(vault_root, relative_path);
  fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
  let directory = fs::canonicalize(directory).map_err(|error| error.to_string())?;
  if !directory.starts_with(&root) {
    return Err(format!("Refusing to write outside the active vault: {}", directory.to_string_lossy()));
  }
  Ok(directory)
}

fn writable_path_inside_vault(vault_root: &str, relative_path: &str) -> R<PathBuf> {
  let root = canonical_root(vault_root)?;
  let normalized = normalize_relative_path(relative_path);
  if normalized.is_empty() {
    return Err("A file path is required.".to_string());
  }
  let path = root.join(&normalized);
  let parent = path.parent().ok_or_else(|| "Cannot write a path without a parent directory.".to_string())?;
  fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  let parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
  if !parent.starts_with(&root) {
    return Err(format!("Refusing to write outside the active vault: {}", path.to_string_lossy()));
  }
  let file_name = path.file_name().ok_or_else(|| "Cannot write a path without a file name.".to_string())?;
  Ok(parent.join(file_name))
}

fn sanitize_leaf_name(value: Option<String>, fallback: &str, force_markdown: bool) -> R<String> {
  let mut name = value.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| fallback.to_string());
  name = name.trim().replace('\\', "/");
  if name.contains('/') || name == "." || name == ".." || name.contains('\0') {
    return Err(format!("Invalid entry file name: {name}"));
  }
  if name.starts_with('.') || name.ends_with('~') || name.ends_with(".tmp") {
    return Err(format!("Refusing unsafe entry file name: {name}"));
  }
  if force_markdown && !is_markdown_name(&name) {
    name.push_str(".md");
  }
  Ok(name)
}

fn metadata_updated_at(metadata: &fs::Metadata) -> String {
  metadata
    .modified()
    .ok()
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .map(|duration| duration.as_secs().to_string())
    .unwrap_or_else(now_string)
}

pub fn is_ignored_entry(name: &str) -> bool {
  name == vault_layout::HIDDEN_ROOT
    || name == ".git"
    || name == "node_modules"
    || name.starts_with('.')
    || name.ends_with('~')
    || name.ends_with(".tmp")
}

fn is_markdown_name(name: &str) -> bool {
  name.to_ascii_lowercase().ends_with(".md")
}

fn title_from_name(name: &str) -> String {
  name.strip_suffix(".md").unwrap_or(name).to_string()
}

fn is_generated_untitled_title(value: &str) -> bool {
  let normalized = value.trim().to_ascii_lowercase();
  if normalized == "untitled" {
    return true;
  }
  normalized
    .strip_prefix("untitled ")
    .map(|suffix| !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()))
    .unwrap_or(false)
}

fn markdown_preview(path: &Path, fallback_title: &str) -> Value {
  let file = fs::File::open(path);
  let Ok(file) = file else {
    return json!({ "title": fallback_title, "excerpt": "" });
  };

  let mut limited = file.take(MARKDOWN_PREVIEW_LIMIT as u64);
  let mut buffer = Vec::with_capacity(4096);
  if limited.read_to_end(&mut buffer).is_err() {
    return json!({ "title": fallback_title, "excerpt": "" });
  }

  let raw = String::from_utf8_lossy(&buffer);
  let mut title = String::new();
  let mut excerpt_lines = Vec::new();
  let mut in_frontmatter = raw.trim_start().starts_with("---");
  let mut seen_frontmatter_start = false;

  for line in raw.lines() {
    let trimmed = line.trim();
    if trimmed == "---" {
      if in_frontmatter && !seen_frontmatter_start {
        seen_frontmatter_start = true;
        continue;
      }
      if in_frontmatter {
        in_frontmatter = false;
        continue;
      }
    }
    if in_frontmatter {
      if let Some(value) = trimmed.strip_prefix("title:") {
        title = value.trim().trim_matches('"').to_string();
      }
      continue;
    }
    if trimmed.is_empty() {
      continue;
    }
    if title.is_empty() {
      if let Some(value) = trimmed.strip_prefix("# ") {
        title = value.trim().to_string();
        continue;
      }
    }
    let cleaned = trimmed.trim_start_matches(|ch| matches!(ch, '#' | '-' | '*' | ' ')).trim();
    if !cleaned.is_empty() && cleaned != title {
      excerpt_lines.push(cleaned.to_string());
      if excerpt_lines.len() >= EXCERPT_LINE_LIMIT {
        break;
      }
    }
  }

  if title.is_empty() && !is_generated_untitled_title(fallback_title) {
    title = fallback_title.to_string();
  }
  json!({ "title": title, "excerpt": excerpt_lines.join(" ") })
}

fn direct_markdown_note_count(path: &Path) -> usize {
  fs::read_dir(path)
    .ok()
    .map(|children| {
      children
        .filter_map(Result::ok)
        .filter(|child| {
          let file_name = child.file_name();
          let name = file_name.to_string_lossy();
          !is_ignored_entry(&name) && is_markdown_name(&name)
        })
        .count()
    })
    .unwrap_or(0)
}

fn entry_summary(root: &Path, path: &Path, metadata: &fs::Metadata, include_preview: bool) -> Value {
  let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();
  let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
  let is_dir = metadata.is_dir();
  let preview = if include_preview && metadata.is_file() && is_markdown_name(&name) {
    markdown_preview(path, &title_from_name(&name))
  } else {
    json!({ "title": title_from_name(&name), "excerpt": "" })
  };
  json!({
    "name": name,
    "title": preview.get("title").and_then(Value::as_str).unwrap_or("").to_string(),
    "path": relative,
    "fullPath": path.to_string_lossy(),
    "type": if is_dir { "folder" } else if is_markdown_name(&name) { "note" } else { "file" },
    "isDirectory": is_dir,
    "noteCount": if is_dir { direct_markdown_note_count(path) } else { 0 },
    "preview": preview.get("excerpt").and_then(Value::as_str).unwrap_or("").to_string(),
    "excerpt": preview.get("excerpt").and_then(Value::as_str).unwrap_or("").to_string(),
    "updatedAt": metadata_updated_at(metadata)
  })
}

pub fn list_directory_page(vault: &VaultDescriptor, relative_path: &str, offset: usize, limit: Option<usize>, include_preview: bool) -> R<Vec<Value>> {
  let root = canonical_root(&vault.path)?;
  let directory = existing_path_inside_vault(&vault.path, relative_path)?;
  if !directory.is_dir() {
    return Err(format!("Not a directory: {}", directory.to_string_lossy()));
  }

  let mut entries = Vec::new();
  for item in fs::read_dir(&directory).map_err(|error| error.to_string())? {
    let item = item.map_err(|error| error.to_string())?;
    let path = item.path();
    let name = item.file_name().to_string_lossy().to_string();
    if is_ignored_entry(&name) {
      continue;
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
      continue;
    }
    if metadata.is_dir() || metadata.is_file() {
      entries.push((metadata.is_dir(), name.to_ascii_lowercase(), entry_summary(&root, &path, &metadata, include_preview)));
    }
  }

  entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
  let limit = limit.unwrap_or(entries.len());
  Ok(entries.into_iter().skip(offset).take(limit).map(|(_, _, value)| value).collect())
}

fn unique_path(mut path: PathBuf) -> PathBuf {
  if !path.exists() {
    return path;
  }
  let parent = path.parent().map(Path::to_path_buf).unwrap_or_else(PathBuf::new);
  let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("Untitled").to_string();
  let extension = path.extension().and_then(|value| value.to_str()).map(str::to_string);
  for index in 2..1000 {
    let file_name = match &extension {
      Some(extension) => format!("{} {}.{}", stem, index, extension),
      None => format!("{} {}", stem, index),
    };
    path = parent.join(file_name);
    if !path.exists() {
      return path;
    }
  }
  path
}

pub fn create_note(vault: &VaultDescriptor, relative_path: Option<String>, filename: Option<String>, title: Option<String>) -> R<Value> {
  let directory = writable_dir_inside_vault(&vault.path, relative_path.as_deref().unwrap_or(""))?;
  let requested_title = title.filter(|value| !value.trim().is_empty());
  let fallback_title = requested_title.clone().unwrap_or_else(|| "Untitled".to_string());
  let file_name = sanitize_leaf_name(filename.or_else(|| Some(format!("{}.md", fallback_title))), "Untitled.md", true)?;
  let path = unique_path(directory.join(file_name));
  let markdown = requested_title.map(|value| format!("# {}\n", value)).unwrap_or_default();
  fs::write(&path, markdown).map_err(|error| error.to_string())?;
  let root = canonical_root(&vault.path)?;
  let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
  Ok(entry_summary(&root, &path, &metadata, true))
}

pub fn create_folder(vault: &VaultDescriptor, relative_path: Option<String>) -> R<Value> {
  let relative_path = relative_path.unwrap_or_else(|| "New Folder".to_string());
  let path = writable_path_inside_vault(&vault.path, &relative_path)?;
  let path = unique_path(path);
  fs::create_dir_all(&path).map_err(|error| error.to_string())?;
  let root = canonical_root(&vault.path)?;
  let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
  Ok(entry_summary(&root, &path, &metadata, false))
}

fn sidebar_path(vault: &VaultDescriptor) -> PathBuf {
  vault_layout::config_file(&vault.path, vault_layout::WORKSPACE_FILE)
}

pub fn attach_sidebar_entry(vault: &VaultDescriptor, relative_path: String, title: Option<String>, entry_type: Option<String>) -> R<Value> {
  let normalized = normalize_relative_path(&relative_path);
  let path = sidebar_path(vault);
  let mut workspace = read_json_or(&path, json!({ "sidebar": [] }));
  let sidebar = workspace.as_object_mut().and_then(|object| object.get_mut("sidebar")).and_then(Value::as_array_mut);
  if sidebar.is_none() {
    workspace["sidebar"] = json!([]);
  }
  let sidebar = workspace.get_mut("sidebar").and_then(Value::as_array_mut).unwrap();
  if !sidebar.iter().any(|item| item.get("path").and_then(Value::as_str) == Some(normalized.as_str())) {
    sidebar.push(json!({
      "path": normalized,
      "title": title.unwrap_or_else(|| title_from_name(&relative_path)),
      "type": entry_type.unwrap_or_else(|| "note".to_string())
    }));
  }
  write_json(path, &workspace)?;
  Ok(workspace)
}

pub fn detach_sidebar_entry(vault: &VaultDescriptor, relative_path: String) -> R<Value> {
  let normalized = normalize_relative_path(&relative_path);
  let path = sidebar_path(vault);
  let mut workspace = read_json_or(&path, json!({ "sidebar": [] }));
  if let Some(sidebar) = workspace.get_mut("sidebar").and_then(Value::as_array_mut) {
    sidebar.retain(|item| item.get("path").and_then(Value::as_str) != Some(normalized.as_str()));
  }
  write_json(path, &workspace)?;
  Ok(workspace)
}

pub fn rename_entry(vault: &VaultDescriptor, relative_path: String, title: String) -> R<()> {
  let path = existing_path_inside_vault(&vault.path, &relative_path)?;
  let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
  let mut name = sanitize_leaf_name(Some(title), "Untitled", metadata.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md"))?;
  if metadata.is_file() && path.extension().is_some() && Path::new(&name).extension().is_none() {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
      name.push('.');
      name.push_str(extension);
    }
  }
  let target = unique_path(path.parent().unwrap_or_else(|| Path::new(&vault.path)).join(name));
  fs::rename(path, target).map_err(|error| error.to_string())
}

pub fn move_entry(vault: &VaultDescriptor, relative_path: String, target_directory_path: Option<String>) -> R<()> {
  let path = existing_path_inside_vault(&vault.path, &relative_path)?;
  let target_dir = writable_dir_inside_vault(&vault.path, target_directory_path.as_deref().unwrap_or(""))?;
  let file_name = path.file_name().ok_or_else(|| "Cannot move an entry without a file name.".to_string())?;
  let target = unique_path(target_dir.join(file_name));
  fs::rename(path, target).map_err(|error| error.to_string())
}

pub fn delete_entry(vault: &VaultDescriptor, relative_path: String) -> R<Value> {
  let path = existing_path_inside_vault(&vault.path, &relative_path)?;
  let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
  if metadata.is_dir() {
    fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
  } else {
    fs::remove_file(&path).map_err(|error| error.to_string())?;
  }
  Ok(json!({ "deleted": true, "path": normalize_relative_path(&relative_path) }))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn temp_dir(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    std::env::temp_dir().join(format!("elephant-entries-{name}-{stamp}"))
  }

  fn test_vault(path: &Path) -> VaultDescriptor {
    VaultDescriptor {
      id: "test".to_string(),
      name: "Test".to_string(),
      path: path.to_string_lossy().to_string(),
      icon: String::new(),
      last_opened_at: "0".to_string(),
    }
  }

  #[test]
  fn normalizes_paths_without_traversal_components() {
    assert_eq!(normalize_relative_path("./a//b/../c.md"), "a/b/c.md");
    assert_eq!(normalize_relative_path("..\\outside.md"), "outside.md");
  }

  #[test]
  fn ignores_hidden_entries() {
    assert!(is_ignored_entry(".git"));
    assert!(is_ignored_entry(".elephantnote"));
    assert!(!is_ignored_entry("note.md"));
  }

  #[test]
  fn creates_new_notes_without_a_default_title_or_body() {
    let root = temp_dir("empty-note");
    fs::create_dir_all(&root).unwrap();
    let vault = test_vault(&root);

    let entry = create_note(&vault, None, None, None).unwrap();
    let relative_path = entry.get("path").and_then(Value::as_str).unwrap();
    let markdown = fs::read_to_string(root.join(relative_path)).unwrap();

    assert!(markdown.is_empty());
    assert_eq!(entry.get("title").and_then(Value::as_str), Some(""));

    let _ = fs::remove_dir_all(&root);
  }

  #[test]
  fn preserves_an_explicit_title_during_note_creation() {
    let root = temp_dir("titled-note");
    fs::create_dir_all(&root).unwrap();
    let vault = test_vault(&root);

    let entry = create_note(&vault, None, None, Some("Dashboard".to_string())).unwrap();
    let relative_path = entry.get("path").and_then(Value::as_str).unwrap();
    let markdown = fs::read_to_string(root.join(relative_path)).unwrap();

    assert_eq!(markdown, "# Dashboard\n");
    assert_eq!(entry.get("title").and_then(Value::as_str), Some("Dashboard"));

    let _ = fs::remove_dir_all(&root);
  }
}
