use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

use crate::vault::config as vault_config;
use crate::vault_layout;

type R<T> = Result<T, String>;
const META_DIR: &str = vault_layout::HIDDEN_ROOT;
const FEATURES_FILE: &str = "tauri-features.json";
const FEATURES_CONFIG_CATEGORY: &str = "features";
const FEATURES_CONFIG_FILE: &str = "flags.json";

fn now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn normalize_relative_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<Vec<_>>()
        .join("/")
}

fn inside(root: &str, relative_path: &str) -> PathBuf {
    let relative_path = normalize_relative_path(relative_path);
    if relative_path.is_empty() {
        PathBuf::from(root)
    } else {
        PathBuf::from(root).join(relative_path)
    }
}

fn active_vault_root(app: &AppHandle) -> R<String> {
    vault_config::get_active_vault(app).map(|vault| vault.path)
}

fn read_json(path: PathBuf, fallback: Value) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(fallback)
}

fn write_text_if_changed(path: &Path, content: &str) -> R<bool> {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_file() && metadata.len() == content.len() as u64 {
            if let Ok(existing) = fs::read(path) {
                if existing == content.as_bytes() {
                    return Ok(false);
                }
            }
        }
    }
    fs::write(path, content).map_err(|error| error.to_string())?;
    Ok(true)
}

fn write_json(path: PathBuf, value: &Value) -> R<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let raw = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    write_text_if_changed(&path, &raw).map(|_| ())
}

fn app_json_path(app: &AppHandle, file_name: &str) -> R<PathBuf> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(file_name))
}

fn features_config_path(app: &AppHandle) -> R<PathBuf> {
    match active_vault_root(app) {
        Ok(root) if !root.trim().is_empty() => Ok(vault_layout::portable_config_file(
            root,
            FEATURES_CONFIG_CATEGORY,
            FEATURES_CONFIG_FILE,
        )),
        _ => app_json_path(app, FEATURES_FILE),
    }
}

fn canonical_root(root: &Path) -> R<PathBuf> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    fs::canonicalize(root).map_err(|error| error.to_string())
}

fn assert_existing_path_inside_root(root: &Path, target: &Path) -> R<PathBuf> {
    let root = canonical_root(root)?;
    let target = fs::canonicalize(target).map_err(|error| error.to_string())?;
    if !target.starts_with(&root) {
        return Err(format!(
            "Refusing to access a path outside the active vault: {}",
            target.to_string_lossy()
        ));
    }
    Ok(target)
}

fn writable_path_inside_root(root: &Path, candidate: &Path) -> R<PathBuf> {
    let root = canonical_root(root)?;
    let target = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    let parent = target
        .parent()
        .ok_or_else(|| "Cannot write a path without a parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    if !parent.starts_with(&root) {
        return Err(format!(
            "Refusing to write outside the active vault: {}",
            target.to_string_lossy()
        ));
    }
    let file_name = target
        .file_name()
        .ok_or_else(|| "Cannot write a path without a file name.".to_string())?;
    Ok(parent.join(file_name))
}

fn writable_relative_path(root: &str, relative_path: &str) -> R<PathBuf> {
    let normalized = normalize_relative_path(relative_path);
    if normalized.is_empty() {
        return Err("A file path is required.".to_string());
    }
    writable_path_inside_root(Path::new(root), Path::new(&normalized))
}

fn ignored_name(name: &str) -> bool {
    name == META_DIR
        || name == ".git"
        || name == "node_modules"
        || name.starts_with('.')
        || name.ends_with('~')
        || name.ends_with(".tmp")
}

fn file_summary(root: &Path, path: &Path) -> Value {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let updated_at = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(now);
    json!({ "name": name, "path": relative, "updatedAt": updated_at })
}

fn scan_files(root: &Path, current: &Path, out: &mut Vec<Value>, extension: Option<&str>) -> R<()> {
    for item in fs::read_dir(current).map_err(|error| error.to_string())? {
        let item = item.map_err(|error| error.to_string())?;
        let path = item.path();
        let name = item.file_name().to_string_lossy().to_string();
        if ignored_name(&name) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            scan_files(root, &path, out, extension)?;
        } else if metadata.is_file() {
            let matches = extension
                .map(|value| name.to_ascii_lowercase().ends_with(value))
                .unwrap_or(true);
            if matches {
                out.push(file_summary(root, &path));
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn shell_exec(
    app: AppHandle,
    command: String,
    args: Option<Vec<String>>,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
) -> R<Value> {
    let command_name = Path::new(&command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command.as_str());
    if command_name != "pandoc" {
        return Err(format!(
            "Refusing to execute unsupported command: {command_name}"
        ));
    }

    let mut process = Command::new(command_name);
    process.args(args.unwrap_or_default());
    if let Some(cwd) = cwd.filter(|value| !value.trim().is_empty()) {
        let root = active_vault_root(&app)?;
        let cwd = assert_existing_path_inside_root(Path::new(&root), Path::new(&cwd))?;
        process.current_dir(cwd);
    }
    if let Some(env) = env {
        for (key, value) in env {
            if key.starts_with("PANDOC_") {
                process.env(key, value);
            }
        }
    }

    let output = process.output().map_err(|error| error.to_string())?;
    Ok(json!({
      "success": output.status.success(),
      "code": output.status.code(),
      "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
      "stderr": String::from_utf8_lossy(&output.stderr).to_string()
    }))
}

#[tauri::command]
pub fn tauri_notes_read(app: AppHandle, relative_path: String) -> R<Value> {
    let root = active_vault_root(&app)?;
    let normalized = normalize_relative_path(&relative_path);
    let path = assert_existing_path_inside_root(Path::new(&root), &inside(&root, &normalized))?;
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    if metadata.is_dir() {
        return Err(format!("Cannot open a folder as a note: {normalized}"));
    }
    if !metadata.is_file() {
        return Err(format!(
            "Cannot open a non-file path as a note: {normalized}"
        ));
    }
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    Ok(json!({ "path": normalized, "fullPath": path.to_string_lossy(), "content": content }))
}

#[tauri::command]
pub fn tauri_notes_write(
    app: AppHandle,
    relative_path: String,
    content: Option<String>,
    markdown: Option<String>,
) -> R<Value> {
    let root = active_vault_root(&app)?;
    let path = writable_relative_path(&root, &relative_path)?;
    let content = content.or(markdown).unwrap_or_default();
    let changed = write_text_if_changed(&path, &content)?;
    Ok(json!({
      "ok": true,
      "changed": changed,
      "path": normalize_relative_path(&relative_path),
      "fullPath": path.to_string_lossy(),
      "updatedAt": now()
    }))
}

#[tauri::command]
pub fn tauri_marktext_write_file(app: AppHandle, pathname: String, content: String) -> R<Value> {
    if pathname.trim().is_empty() {
        return Err("Cannot save MarkText file without a pathname.".to_string());
    }
    let root = active_vault_root(&app)?;
    let path = writable_path_inside_root(Path::new(&root), Path::new(&pathname))?;
    let changed = write_text_if_changed(&path, &content)?;
    Ok(json!({
      "ok": true,
      "changed": changed,
      "fullPath": path.to_string_lossy(),
      "updatedAt": now()
    }))
}

#[tauri::command]
pub fn tauri_attachments_list(app: AppHandle) -> R<Vec<Value>> {
    let root = active_vault_root(&app)?;
    let root_path = PathBuf::from(&root);
    let assets = vault_layout::assets_dir(&root_path);
    if !assets.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    scan_files(&root_path, &assets, &mut out, None)?;
    Ok(out)
}

#[tauri::command]
pub fn tauri_attachments_write_text(
    app: AppHandle,
    relative_path: String,
    content: String,
) -> R<Value> {
    let root = active_vault_root(&app)?;
    let assets = vault_layout::assets_dir(&root);
    let relative_path = normalize_relative_path(&relative_path);
    let path = writable_path_inside_root(&assets, Path::new(&relative_path))?;
    let changed = write_text_if_changed(&path, &content)?;
    let public_path = path
        .strip_prefix(&root)
        .unwrap_or(&path)
        .to_string_lossy()
        .replace('\\', "/");
    Ok(
        json!({ "ok": true, "changed": changed, "path": public_path, "fullPath": path.to_string_lossy() }),
    )
}

#[tauri::command]
pub fn tauri_drawings_list(app: AppHandle) -> R<Vec<Value>> {
    let root = active_vault_root(&app)?;
    let root_path = PathBuf::from(&root);
    let assets = vault_layout::assets_dir(&root_path);
    if !assets.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    scan_files(&root_path, &assets, &mut out, Some(".excalidraw"))?;
    Ok(out)
}

#[tauri::command]
pub fn tauri_drawings_create(app: AppHandle, title: Option<String>) -> R<Value> {
    let root = active_vault_root(&app)?;
    let title = title
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Untitled Drawing".to_string());
    let safe_title = title.replace('/', "-").replace('\\', "-");
    let assets = vault_layout::assets_dir(&root);
    let relative_path = normalize_relative_path(&format!("{safe_title}.excalidraw"));
    let path = writable_path_inside_root(&assets, Path::new(&relative_path))?;
    let scene = json!({
      "kind": "excalidraw",
      "type": "excalidraw",
      "version": 1,
      "title": title,
      "elements": [],
      "files": {}
    });
    write_json(path.clone(), &scene)?;
    let public_path = fs::canonicalize(&path)
        .ok()
        .and_then(|canonical_path| {
            fs::canonicalize(&root).ok().and_then(|canonical_root| {
                canonical_path
                    .strip_prefix(canonical_root)
                    .ok()
                    .map(Path::to_path_buf)
            })
        })
        .unwrap_or_else(|| path.strip_prefix(&root).unwrap_or(&path).to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    Ok(json!({ "path": public_path, "fullPath": path.to_string_lossy(), "scene": scene }))
}

#[tauri::command]
pub fn tauri_drawings_read(app: AppHandle, relative_path: String) -> R<Value> {
    let root = active_vault_root(&app)?;
    let normalized = normalize_relative_path(&relative_path);
    let path = assert_existing_path_inside_root(Path::new(&root), &inside(&root, &normalized))?;
    Ok(read_json(
        path,
        json!({ "kind": "excalidraw", "elements": [], "files": {} }),
    ))
}

#[tauri::command]
pub fn tauri_drawings_write(app: AppHandle, relative_path: String, scene: Value) -> R<Value> {
    let root = active_vault_root(&app)?;
    let path = writable_relative_path(&root, &relative_path)?;
    write_json(path.clone(), &scene)?;
    Ok(json!({
      "ok": true,
      "path": normalize_relative_path(&relative_path),
      "fullPath": path.to_string_lossy()
    }))
}

#[tauri::command]
pub fn tauri_features_get(app: AppHandle) -> R<Value> {
    let path = features_config_path(&app)?;
    Ok(read_json(path, json!({})))
}

#[tauri::command]
pub fn tauri_features_set(app: AppHandle, key: String, enabled: bool) -> R<Value> {
    let path = features_config_path(&app)?;
    let mut config = read_json(path.clone(), json!({}));
    if let Some(object) = config.as_object_mut() {
        object.insert(key, Value::Bool(enabled));
    }
    write_json(path, &config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("elephant-{name}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn write_text_if_changed_skips_identical_content() {
        let directory = temp_test_root("write-skip");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("note.md");
        assert!(write_text_if_changed(&path, "same").unwrap());
        assert!(!write_text_if_changed(&path, "same").unwrap());
        assert!(write_text_if_changed(&path, "changed").unwrap());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn writable_paths_cannot_escape_the_vault() {
        let directory = temp_test_root("write-containment");
        fs::create_dir_all(&directory).unwrap();
        let outside = directory.parent().unwrap().join("outside.md");
        assert!(writable_path_inside_root(&directory, &outside).is_err());
        let _ = fs::remove_dir_all(directory);
    }
}
