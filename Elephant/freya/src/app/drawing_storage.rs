//! Filesystem boundary for Elephant's Excalidraw scene/preview pair.
//!
//! The `.excalidraw` JSON is the canonical editable document. PNG is a derived
//! preview/export sidecar, matching the Vue/Tauri path. Native editing remains
//! tolerant of a missing preview, while every successful native create/save
//! regenerates it from the canonical scene.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[path = "drawing_export.rs"]
mod export;

const ASSETS_DIR: &str = ".assets";
const LIGHT_BACKGROUND: &str = "#ffffff";

#[derive(Clone, Debug)]
pub(super) struct SceneRead {
    pub(super) raw: String,
    pub(super) relative_path: String,
    pub(super) title: String,
    pub(super) element_count: usize,
    pub(super) preview_size: Option<u64>,
}

#[derive(Clone, Debug)]
pub(super) struct CreatedScene {
    pub(super) path: PathBuf,
    pub(super) relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RenamedScene {
    pub(super) relative_path: String,
    pub(super) title: String,
}

/// Create a hidden sidecar scene for image-backed/Tauri-compatible assets.
///
/// Native library creation should use [`create_standalone_scene`] instead.
pub(super) fn create_scene(root: &Path, title: &str) -> Result<CreatedScene, String> {
    let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let assets = root.join(ASSETS_DIR);
    reject_symlink(&assets, "Excalidraw asset directory")?;
    fs::create_dir_all(&assets).map_err(|error| {
        format!(
            "Unable to create Excalidraw asset directory {}: {error}",
            assets.display()
        )
    })?;
    create_scene_at(&root, &assets, title)
}

/// Create a visible, standalone `.excalidraw` document in a library directory.
pub(super) fn create_standalone_scene(
    root: &Path,
    relative_directory: &str,
    title: &str,
) -> Result<CreatedScene, String> {
    let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let directory = if relative_directory.trim().is_empty() {
        root.clone()
    } else {
        let relative = safe_relative_path(relative_directory)?;
        let candidate = root.join(relative);
        reject_symlink(&candidate, "drawing directory")?;
        let canonical = fs::canonicalize(&candidate).map_err(|error| {
            format!(
                "Drawing directory unavailable at {}: {error}",
                candidate.display()
            )
        })?;
        if !canonical.starts_with(&root) {
            return Err(format!(
                "Refusing drawing directory outside vault: {relative_directory}"
            ));
        }
        if !canonical.is_dir() {
            return Err(format!(
                "Drawing directory unavailable at {}: expected a directory",
                canonical.display()
            ));
        }
        canonical
    };
    create_scene_at(&root, &directory, title)
}

fn create_scene_at(root: &Path, directory: &Path, title: &str) -> Result<CreatedScene, String> {
    let safe_title = sanitize_scene_title(title, "Untitled Drawing");
    let (path, resolved_title) = unique_scene_path(directory, &safe_title);
    let scene = json!({
        "kind": "excalidraw",
        "type": "excalidraw",
        "version": 2,
        "source": "https://excalidraw.com",
        "title": resolved_title,
        "elements": [],
        "appState": {
            "viewBackgroundColor": LIGHT_BACKGROUND,
            "exportBackground": true,
            "exportEmbedScene": true
        },
        "files": {}
    });
    let raw = serde_json::to_string_pretty(&scene).map_err(|error| error.to_string())?;
    fs::write(&path, raw.as_bytes()).map_err(|error| {
        format!(
            "Unable to persist Excalidraw scene {}: {error}",
            path.display()
        )
    })?;
    if let Err(error) = write_preview(root, &path, &raw) {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(CreatedScene {
        relative_path: relative_from_root(root, &path)?,
        path,
    })
}

pub(super) fn read_scene(root: &Path, relative_path: &str) -> Result<SceneRead, String> {
    let canonical_root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let scene_path = scene_path(&canonical_root, relative_path)?;
    let raw = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "Drawing scene unavailable at {}: {error}",
            scene_path.display()
        )
    })?;
    let scene = validate_scene(&raw, &scene_path)?;
    let elements = scene["elements"]
        .as_array()
        .expect("validate_scene requires an elements array");
    let title = scene
        .get("title")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            scene_path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "Drawing".to_owned());
    let preview_size = optional_preview_size(&canonical_root, &scene_path)?;
    Ok(SceneRead {
        raw,
        relative_path: relative_from_root(&canonical_root, &scene_path)?,
        title,
        element_count: elements.len(),
        preview_size,
    })
}

pub(super) fn write_scene(root: &Path, relative_path: &str, raw: &str) -> Result<(), String> {
    let canonical_root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let scene_path = scene_path(&canonical_root, relative_path)?;
    validate_scene(raw, &scene_path)?;
    let png = export::render_png(raw)?;

    let temp_scene = temp_path(&scene_path, "scene");
    let preview = preview_path(&scene_path);
    let temp_preview = temp_path(&preview, "preview");
    clear_stale_temp(&temp_scene)?;
    clear_stale_temp(&temp_preview)?;

    fs::write(&temp_scene, raw.as_bytes()).map_err(|error| {
        format!(
            "Unable to stage drawing save {}: {error}",
            temp_scene.display()
        )
    })?;
    if let Err(error) = fs::write(&temp_preview, &png) {
        let _ = fs::remove_file(&temp_scene);
        return Err(format!(
            "Unable to stage drawing preview {}: {error}",
            temp_preview.display()
        ));
    }

    if let Err(error) = replace_file(&temp_scene, &scene_path) {
        let _ = fs::remove_file(&temp_preview);
        return Err(error);
    }
    // The canonical JSON has already been replaced at this point. Never leave
    // the old preview silently stale: direct-write fallback is used on
    // platforms where rename-over-existing is unavailable.
    replace_file(&temp_preview, &preview).map_err(|error| {
        format!(
            "Drawing JSON saved but preview update failed at {}: {error}",
            preview.display()
        )
    })?;
    Ok(())
}

fn write_preview(root: &Path, scene_path: &Path, raw: &str) -> Result<(), String> {
    let preview = preview_path(scene_path);
    if let Some(parent) = preview.parent() {
        let canonical_parent = fs::canonicalize(parent).map_err(|error| {
            format!(
                "Drawing preview directory unavailable at {}: {error}",
                parent.display()
            )
        })?;
        if !canonical_parent.starts_with(root) {
            return Err(format!(
                "Refusing drawing preview outside vault: {}",
                preview.display()
            ));
        }
    }
    reject_symlink(&preview, "drawing preview")?;
    let png = export::render_png(raw)?;
    let temp = temp_path(&preview, "preview");
    clear_stale_temp(&temp)?;
    fs::write(&temp, &png).map_err(|error| {
        format!(
            "Unable to stage drawing preview {}: {error}",
            temp.display()
        )
    })?;
    replace_file(&temp, &preview)
}

fn temp_path(path: &Path, kind: &str) -> PathBuf {
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("tmp");
    path.with_extension(format!("{extension}.freya-{kind}-tmp-{}", std::process::id()))
}

fn clear_stale_temp(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|error| {
            format!("Unable to clear stale drawing save {}: {error}", path.display())
        })?;
    }
    Ok(())
}

fn replace_file(temp: &Path, target: &Path) -> Result<(), String> {
    if let Err(rename_error) = fs::rename(temp, target) {
        let bytes = fs::read(temp).map_err(|read_error| {
            format!(
                "Unable to replace drawing file {} (rename: {rename_error}; staged read: {read_error})",
                target.display()
            )
        })?;
        fs::write(target, bytes).map_err(|write_error| {
            let _ = fs::remove_file(temp);
            format!(
                "Unable to replace drawing file {} (rename: {rename_error}; write: {write_error})",
                target.display()
            )
        })?;
        let _ = fs::remove_file(temp);
    }
    Ok(())
}

pub(super) fn can_rename_standalone_scene(relative_path: &str, _has_preview: bool) -> bool {
    let normalized = relative_path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    lower.ends_with(".excalidraw")
        && lower != ASSETS_DIR
        && !lower.starts_with(&format!("{ASSETS_DIR}/"))
        && !lower.contains(&format!("/{ASSETS_DIR}/"))
}

pub(super) fn rename_standalone_scene(
    root: &Path,
    relative_path: &str,
    title: &str,
) -> Result<RenamedScene, String> {
    let canonical_root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let current_path = scene_path(&canonical_root, relative_path)?;
    let current_preview = preview_path(&current_path);
    let has_preview = current_preview.exists();
    if !can_rename_standalone_scene(relative_path, has_preview) {
        return Err(format!(
            "Refusing to rename image-backed or hidden drawing: {relative_path}"
        ));
    }
    if has_preview {
        optional_preview_size(&canonical_root, &current_path)?;
    }
    let directory = current_path
        .parent()
        .ok_or_else(|| format!("Drawing has no parent directory: {relative_path}"))?;
    let safe_title = sanitize_scene_title(title, "Untitled Drawing");
    let current_title = current_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if current_title == safe_title {
        return Ok(RenamedScene {
            relative_path: relative_from_root(&canonical_root, &current_path)?,
            title: safe_title,
        });
    }

    let (target, resolved_title) = unique_scene_path(directory, &safe_title);
    let target_preview = preview_path(&target);
    reject_symlink(&target, "drawing rename target")?;
    reject_symlink(&target_preview, "drawing preview rename target")?;
    fs::rename(&current_path, &target).map_err(|error| {
        format!(
            "Unable to rename drawing {} to {}: {error}",
            current_path.display(),
            target.display()
        )
    })?;
    if has_preview {
        if let Err(error) = fs::rename(&current_preview, &target_preview) {
            let rollback = fs::rename(&target, &current_path);
            return Err(format!(
                "Unable to rename drawing preview {} to {}: {error}; scene rollback={}",
                current_preview.display(),
                target_preview.display(),
                if rollback.is_ok() { "ok" } else { "failed" }
            ));
        }
    }
    Ok(RenamedScene {
        relative_path: relative_from_root(&canonical_root, &target)?,
        title: resolved_title,
    })
}

fn validate_scene(raw: &str, scene_path: &Path) -> Result<Value, String> {
    let scene: Value = serde_json::from_str(raw)
        .map_err(|error| format!("Drawing scene invalid at {}: {error}", scene_path.display()))?;
    if scene.get("type").and_then(Value::as_str) != Some("excalidraw") {
        return Err(format!(
            "Drawing scene invalid at {}: expected type=excalidraw",
            scene_path.display()
        ));
    }
    if !scene.get("elements").is_some_and(Value::is_array) {
        return Err(format!(
            "Drawing scene invalid at {}: elements must be an array",
            scene_path.display()
        ));
    }
    Ok(scene)
}

fn optional_preview_size(root: &Path, scene_path: &Path) -> Result<Option<u64>, String> {
    let preview_path = preview_path(scene_path);
    let preview_metadata = match fs::symlink_metadata(&preview_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Drawing preview unavailable at {}: {error}",
                preview_path.display()
            ))
        }
    };
    if preview_metadata.file_type().is_symlink() {
        return Err(format!(
            "Refusing symlinked drawing preview: {}",
            preview_path.display()
        ));
    }
    if !preview_metadata.is_file() {
        return Err(format!(
            "Drawing preview unavailable at {}: expected a file",
            preview_path.display()
        ));
    }
    let canonical_preview = fs::canonicalize(&preview_path).map_err(|error| {
        format!(
            "Drawing preview unavailable at {}: {error}",
            preview_path.display()
        )
    })?;
    if !canonical_preview.starts_with(root) {
        return Err(format!(
            "Refusing drawing preview outside vault: {}",
            preview_path.display()
        ));
    }
    Ok(Some(preview_metadata.len()))
}

fn preview_path(scene_path: &Path) -> PathBuf {
    let mut preview = scene_path.to_path_buf();
    preview.set_extension("png");
    preview
}

fn scene_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let visible_path = safe_relative_path(relative_path)?;
    let lower_path = visible_path.to_string_lossy().to_ascii_lowercase();
    let candidate = if lower_path.ends_with(".md") {
        let note_path = root.join(&visible_path);
        reject_symlink(&note_path, "drawing note")?;
        let canonical_note = fs::canonicalize(&note_path).map_err(|error| {
            format!("Drawing note unavailable at {}: {error}", note_path.display())
        })?;
        if !canonical_note.starts_with(root) {
            return Err(format!("Refusing drawing note outside vault: {relative_path}"));
        }
        let markdown = fs::read_to_string(&canonical_note).map_err(|error| {
            format!(
                "Drawing note unavailable at {}: {error}",
                canonical_note.display()
            )
        })?;
        let asset = markdown_asset_path(&markdown).ok_or_else(|| {
            format!(
                "Drawing scene unavailable: no .assets PNG link in {}",
                canonical_note.display()
            )
        })?;
        canonical_note
            .parent()
            .unwrap_or(root)
            .join(asset)
            .with_extension("excalidraw")
    } else if lower_path.ends_with(".excalidraw.png") {
        let mut scene = visible_path.clone();
        scene.set_extension("");
        root.join(scene)
    } else if lower_path.ends_with(".png") {
        let mut scene = visible_path.clone();
        scene.set_extension("excalidraw");
        root.join(scene)
    } else {
        root.join(&visible_path)
    };
    reject_symlink(&candidate, "drawing scene")?;
    let canonical_scene = fs::canonicalize(&candidate).map_err(|error| {
        format!(
            "Drawing scene unavailable at {}: {error}",
            candidate.display()
        )
    })?;
    if !canonical_scene.starts_with(root) {
        return Err(format!(
            "Refusing drawing scene outside vault: {relative_path}"
        ));
    }
    Ok(canonical_scene)
}

fn reject_symlink(path: &Path, label: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "Refusing symlinked {label}: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Unable to inspect {label} {}: {error}",
            path.display()
        )),
    }
}

fn unique_scene_path(directory: &Path, base_title: &str) -> (PathBuf, String) {
    for suffix in 1usize.. {
        let title = if suffix == 1 {
            base_title.to_owned()
        } else {
            format!("{base_title} {suffix}")
        };
        let path = directory.join(format!("{title}.excalidraw"));
        if !path.exists() {
            return (path, title);
        }
    }
    unreachable!("unbounded suffix iterator always returns a candidate")
}

fn sanitize_scene_title(value: &str, fallback: &str) -> String {
    let cleaned = sanitize_asset_name(value, fallback);
    let lower = cleaned.to_ascii_lowercase();
    let stripped = [".excalidraw.png", ".excalidraw", ".png"]
        .into_iter()
        .find(|suffix| lower.ends_with(suffix))
        .map(|suffix| cleaned[..cleaned.len() - suffix.len()].trim())
        .unwrap_or(cleaned.as_str());
    if stripped.is_empty() {
        fallback.to_owned()
    } else {
        stripped.to_owned()
    }
}

fn sanitize_asset_name(value: &str, fallback: &str) -> String {
    let normalized = value.replace('\\', "/");
    let filename = normalized.rsplit('/').next().unwrap_or(fallback);
    let cleaned = filename
        .chars()
        .map(|character| {
            if character.is_control() || "<>:\"|?*".contains(character) {
                '-'
            } else {
                character
            }
        })
        .collect::<String>()
        .trim_start_matches('.')
        .trim()
        .to_owned();
    if cleaned.is_empty() {
        fallback.to_owned()
    } else {
        cleaned
    }
}

fn relative_from_root(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map_err(|_| format!("Drawing path escaped vault: {}", path.display()))
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("Refusing unsafe drawing path: {value}"));
    }
    Ok(path.to_path_buf())
}

fn markdown_asset_path(markdown: &str) -> Option<PathBuf> {
    let marker = ".assets/";
    let start = markdown.find(marker)?;
    let mut begin = start;
    while begin > 0 && matches!(markdown.as_bytes()[begin - 1] as char, '.' | '/') {
        begin -= 1;
    }
    let tail = &markdown[begin..];
    let end = tail
        .find(|character: char| character.is_whitespace() || matches!(character, ')' | ']' | '"'))
        .unwrap_or(tail.len());
    let candidate = tail[..end].trim().trim_start_matches("./");
    candidate
        .to_ascii_lowercase()
        .ends_with(".png")
        .then(|| PathBuf::from(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::canvas::{DrawingCanvasState, DrawingTool};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestVault(PathBuf);

    impl TestVault {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "elephant-freya-drawing-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for TestVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn storage_roundtrip_tolerates_missing_preview_and_regenerates_it_on_save() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Roundtrip").unwrap();
        let preview = preview_path(&created.path);
        assert!(created.path.is_file());
        assert!(preview.is_file());
        fs::remove_file(&preview).unwrap();

        let first = read_scene(&vault.0, &created.relative_path).unwrap();
        assert_eq!(first.element_count, 0);
        assert_eq!(first.preview_size, None);
        assert_eq!(first.title, "Roundtrip");

        let mut scene: Value = serde_json::from_str(&first.raw).unwrap();
        scene["elements"] = json!([{
            "id":"rect-1",
            "type":"rectangle",
            "x":-20,
            "y":10,
            "width":80,
            "height":50
        }]);
        scene["futureTopLevelField"] = json!({"preserved": true});
        let raw = serde_json::to_string_pretty(&scene).unwrap();
        write_scene(&vault.0, &created.relative_path, &raw).unwrap();
        assert!(preview.is_file());
        assert!(fs::read(&preview).unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));

        let reopened = read_scene(&vault.0, &created.relative_path).unwrap();
        assert_eq!(reopened.element_count, 1);
        assert!(reopened.preview_size.unwrap_or(0) > 8);
        let reopened_json: Value = serde_json::from_str(&reopened.raw).unwrap();
        assert_eq!(reopened_json["elements"][0]["id"], "rect-1");
        assert_eq!(reopened_json["futureTopLevelField"]["preserved"], true);
    }

    #[test]
    fn native_engine_draw_save_close_reopen_roundtrip_regenerates_png() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Native Roundtrip").unwrap();
        let loaded = read_scene(&vault.0, &created.relative_path).unwrap();
        let mut canvas = DrawingCanvasState::from_json(&loaded.raw).unwrap();

        canvas.set_tool(DrawingTool::Rectangle);
        canvas.begin_pointer([-20., 10.]);
        canvas.move_pointer([80., 70.]);
        canvas.end_pointer();
        canvas.set_tool(DrawingTool::Arrow);
        canvas.begin_pointer([5., 5.]);
        canvas.move_pointer([45., 35.]);
        canvas.end_pointer();

        let raw = canvas.serialize_json().unwrap();
        write_scene(&vault.0, &created.relative_path, &raw).unwrap();
        drop(canvas);

        let reopened = read_scene(&vault.0, &created.relative_path).unwrap();
        assert_eq!(reopened.element_count, 2);
        assert!(reopened.preview_size.unwrap_or(0) > 8);
        let reopened_canvas = DrawingCanvasState::from_json(&reopened.raw).unwrap();
        let reopened_json: Value =
            serde_json::from_str(&reopened_canvas.serialize_json().unwrap()).unwrap();
        assert_eq!(reopened_json["elements"][0]["type"], "rectangle");
        assert_eq!(reopened_json["elements"][0]["x"], -20.0);
        assert_eq!(reopened_json["elements"][0]["width"], 100.0);
        assert_eq!(reopened_json["elements"][1]["type"], "arrow");
        assert_eq!(reopened_json["elements"][1]["points"][1], json!([40.0, 30.0]));
    }

    #[test]
    fn standalone_scene_is_created_with_preview_in_visible_current_directory() {
        let vault = TestVault::new();
        fs::create_dir_all(vault.0.join("Projects")).unwrap();

        let created = create_standalone_scene(&vault.0, "Projects", "Sketch").unwrap();

        assert_eq!(created.relative_path, "Projects/Sketch.excalidraw");
        assert!(vault.0.join(&created.relative_path).is_file());
        assert!(vault.0.join("Projects/Sketch.png").is_file());
        assert!(!created.relative_path.contains("/.assets/"));
        assert_eq!(
            read_scene(&vault.0, &created.relative_path).unwrap().title,
            "Sketch"
        );
    }

    #[test]
    fn standalone_scene_rejects_unsafe_or_missing_directories() {
        let vault = TestVault::new();
        assert!(create_standalone_scene(&vault.0, "../outside", "Sketch").is_err());
        assert!(create_standalone_scene(&vault.0, "missing", "Sketch").is_err());
    }

    #[test]
    fn standalone_rename_moves_scene_and_preview_and_avoids_collisions() {
        let vault = TestVault::new();
        let first = create_standalone_scene(&vault.0, "", "Sketch").unwrap();
        let second = create_standalone_scene(&vault.0, "", "Other").unwrap();
        let old_preview = vault.0.join("Other.png");
        assert!(old_preview.is_file());

        let renamed =
            rename_standalone_scene(&vault.0, &second.relative_path, "Sketch.excalidraw.png")
                .unwrap();

        assert_eq!(renamed.title, "Sketch 2");
        assert_eq!(renamed.relative_path, "Sketch 2.excalidraw");
        assert!(vault.0.join(&first.relative_path).is_file());
        assert!(vault.0.join(&renamed.relative_path).is_file());
        assert!(vault.0.join("Sketch 2.png").is_file());
        assert!(!vault.0.join(&second.relative_path).exists());
        assert!(!old_preview.exists());
    }

    #[test]
    fn hidden_image_backed_sidecar_is_not_renamed_without_rewriting_owner() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Linked").unwrap();
        assert!(!can_rename_standalone_scene(&created.relative_path, true));
        assert!(rename_standalone_scene(&vault.0, &created.relative_path, "Other").is_err());
    }

    #[test]
    fn create_scene_never_overwrites_existing_drawing_or_preview() {
        let vault = TestVault::new();
        let first = create_scene(&vault.0, "Untitled Drawing").unwrap();
        let second = create_scene(&vault.0, "Untitled Drawing").unwrap();
        assert_ne!(first.path, second.path);
        assert!(second
            .relative_path
            .ends_with("Untitled Drawing 2.excalidraw"));
        assert!(preview_path(&first.path).is_file());
        assert!(preview_path(&second.path).is_file());
    }

    #[test]
    fn png_and_markdown_paths_resolve_back_to_scene() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Linked").unwrap();
        let png_relative = created.relative_path.replace(".excalidraw", ".png");
        assert!(vault.0.join(&png_relative).is_file());
        assert_eq!(
            read_scene(&vault.0, &png_relative).unwrap().title,
            "Linked"
        );

        let note = vault.0.join("drawing-note.md");
        fs::write(&note, format!("![Linked]({png_relative})")).unwrap();
        assert_eq!(
            read_scene(&vault.0, "drawing-note.md").unwrap().title,
            "Linked"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_markdown_cannot_escape_vault_before_asset_resolution() {
        use std::os::unix::fs::symlink;

        let vault = TestVault::new();
        let outside = TestVault::new();
        let outside_note = outside.0.join("outside.md");
        fs::write(&outside_note, "![Outside](.assets/outside.png)").unwrap();
        symlink(&outside_note, vault.0.join("drawing-note.md")).unwrap();
        let error = read_scene(&vault.0, "drawing-note.md").unwrap_err();
        assert!(error.contains("symlinked drawing note"));
    }

    #[test]
    fn rejects_path_traversal_and_non_excalidraw_json() {
        let vault = TestVault::new();
        assert!(read_scene(&vault.0, "../outside.excalidraw").is_err());
        let created = create_scene(&vault.0, "Invalid").unwrap();
        let error = write_scene(
            &vault.0,
            &created.relative_path,
            r#"{"type":"other","elements":[]}"#,
        )
        .unwrap_err();
        assert!(error.contains("expected type=excalidraw"));
    }
}
