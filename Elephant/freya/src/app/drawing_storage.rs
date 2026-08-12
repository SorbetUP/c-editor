//! Filesystem boundary for Elephant's Excalidraw scene/preview pair.
//!
//! The `.excalidraw` JSON is the canonical editable document. A PNG remains a
//! compatible optional preview/export sidecar, matching the Vue/Tauri path,
//! but native editing must not fail merely because that derived preview is
//! absent or stale. New library drawings are standalone `.excalidraw` files in
//! the current visible directory so closing the editor never strands an orphan
//! inside the hidden `.assets` directory.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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

    let temp_path =
        scene_path.with_extension(format!("excalidraw.freya-tmp-{}", std::process::id()));
    if temp_path.exists() {
        fs::remove_file(&temp_path).map_err(|error| {
            format!(
                "Unable to clear stale drawing save {}: {error}",
                temp_path.display()
            )
        })?;
    }
    fs::write(&temp_path, raw.as_bytes()).map_err(|error| {
        format!(
            "Unable to stage drawing save {}: {error}",
            temp_path.display()
        )
    })?;

    if let Err(rename_error) = fs::rename(&temp_path, &scene_path) {
        // Windows does not replace an existing destination with rename. Keep a
        // portable fallback while still validating/staging the complete JSON
        // before the canonical file is touched.
        fs::write(&scene_path, raw.as_bytes()).map_err(|write_error| {
            let _ = fs::remove_file(&temp_path);
            format!(
                "Unable to save drawing {} (rename: {rename_error}; write: {write_error})",
                scene_path.display()
            )
        })?;
        let _ = fs::remove_file(&temp_path);
    }
    Ok(())
}

pub(super) fn can_rename_standalone_scene(relative_path: &str, has_preview: bool) -> bool {
    if has_preview {
        return false;
    }
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
    if !can_rename_standalone_scene(relative_path, preview_path(&current_path).exists()) {
        return Err(format!(
            "Refusing to rename image-backed or hidden drawing: {relative_path}"
        ));
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
    reject_symlink(&target, "drawing rename target")?;
    fs::rename(&current_path, &target).map_err(|error| {
        format!(
            "Unable to rename drawing {} to {}: {error}",
            current_path.display(),
            target.display()
        )
    })?;
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
        let markdown = fs::read_to_string(&note_path).map_err(|error| {
            format!(
                "Drawing note unavailable at {}: {error}",
                note_path.display()
            )
        })?;
        let asset = markdown_asset_path(&markdown).ok_or_else(|| {
            format!(
                "Drawing scene unavailable: no .assets PNG link in {}",
                note_path.display()
            )
        })?;
        note_path
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
    fn storage_roundtrip_and_reopen_do_not_require_png_preview() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Roundtrip").unwrap();
        assert!(created.path.is_file());
        assert!(!preview_path(&created.path).exists());

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

        let reopened = read_scene(&vault.0, &created.relative_path).unwrap();
        assert_eq!(reopened.element_count, 1);
        let reopened_json: Value = serde_json::from_str(&reopened.raw).unwrap();
        assert_eq!(reopened_json["elements"][0]["id"], "rect-1");
        assert_eq!(reopened_json["futureTopLevelField"]["preserved"], true);
    }

    #[test]
    fn native_engine_draw_save_close_reopen_roundtrip() {
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
    fn standalone_scene_is_created_in_the_visible_current_directory() {
        let vault = TestVault::new();
        fs::create_dir_all(vault.0.join("Projects")).unwrap();

        let created = create_standalone_scene(&vault.0, "Projects", "Sketch").unwrap();

        assert_eq!(created.relative_path, "Projects/Sketch.excalidraw");
        assert!(vault.0.join(&created.relative_path).is_file());
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
    fn standalone_rename_normalizes_extensions_and_avoids_collisions() {
        let vault = TestVault::new();
        let first = create_standalone_scene(&vault.0, "", "Sketch").unwrap();
        let second = create_standalone_scene(&vault.0, "", "Other").unwrap();

        let renamed =
            rename_standalone_scene(&vault.0, &second.relative_path, "Sketch.excalidraw.png")
                .unwrap();

        assert_eq!(renamed.title, "Sketch 2");
        assert_eq!(renamed.relative_path, "Sketch 2.excalidraw");
        assert!(vault.0.join(&first.relative_path).is_file());
        assert!(vault.0.join(&renamed.relative_path).is_file());
        assert!(!vault.0.join(&second.relative_path).exists());
    }

    #[test]
    fn image_backed_sidecar_is_not_renamed_without_rewriting_its_owner() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Linked").unwrap();
        assert!(!can_rename_standalone_scene(&created.relative_path, false));
        assert!(rename_standalone_scene(&vault.0, &created.relative_path, "Other").is_err());
    }

    #[test]
    fn create_scene_never_overwrites_existing_drawing() {
        let vault = TestVault::new();
        let first = create_scene(&vault.0, "Untitled Drawing").unwrap();
        let second = create_scene(&vault.0, "Untitled Drawing").unwrap();
        assert_ne!(first.path, second.path);
        assert!(second
            .relative_path
            .ends_with("Untitled Drawing 2.excalidraw"));
    }

    #[test]
    fn png_and_markdown_paths_resolve_back_to_scene() {
        let vault = TestVault::new();
        let created = create_scene(&vault.0, "Linked").unwrap();
        let png_relative = created.relative_path.replace(".excalidraw", ".png");
        fs::write(vault.0.join(&png_relative), b"png").unwrap();
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
