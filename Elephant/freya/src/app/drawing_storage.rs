//! Filesystem boundary for the existing Excalidraw scene and preview format.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

const ASSETS_DIR: &str = ".assets";

pub(super) struct SceneRead {
    pub(super) path: PathBuf,
    pub(super) raw: String,
    pub(super) element_count: usize,
    pub(super) preview_size: u64,
}

pub(super) struct NativeSceneRead {
    pub(super) path: PathBuf,
    pub(super) raw: String,
}

pub(super) struct CreatedScene {
    pub(super) path: PathBuf,
    pub(super) relative_path: String,
}

pub(super) fn create_scene(root: &Path, title: &str) -> Result<CreatedScene, String> {
    let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let assets = root.join(ASSETS_DIR);
    if fs::symlink_metadata(&assets)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(format!(
            "Refusing symlinked Excalidraw asset directory: {}",
            assets.display()
        ));
    }
    fs::create_dir_all(&assets).map_err(|error| {
        format!(
            "Unable to create Excalidraw asset directory {}: {error}",
            assets.display()
        )
    })?;
    let safe_title = title.trim().replace(['/', '\\'], "-");
    let safe_title = if safe_title.is_empty() {
        "Untitled Drawing"
    } else {
        safe_title.as_str()
    };
    let path = assets.join(format!("{safe_title}.excalidraw"));
    let scene = json!({
        "kind": "excalidraw",
        "type": "excalidraw",
        "version": 1,
        "title": safe_title,
        "elements": [],
        "files": {}
    });
    let raw = serde_json::to_vec_pretty(&scene).map_err(|error| error.to_string())?;
    fs::write(&path, raw).map_err(|error| {
        format!(
            "Unable to persist Excalidraw scene {}: {error}",
            path.display()
        )
    })?;
    Ok(CreatedScene {
        relative_path: format!("{ASSETS_DIR}/{safe_title}.excalidraw"),
        path,
    })
}

pub(super) fn read_scene(root: &Path, relative_path: &str) -> Result<SceneRead, String> {
    let scene_path = scene_path(root, relative_path)?;
    let raw = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "Drawing scene unavailable at {}: {error}",
            scene_path.display()
        )
    })?;
    let scene: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Drawing scene invalid at {}: {error}", scene_path.display()))?;
    if scene.get("type").and_then(Value::as_str) != Some("excalidraw") {
        return Err(format!(
            "Drawing scene invalid at {}: expected type=excalidraw",
            scene_path.display()
        ));
    }
    let Some(elements) = scene.get("elements").and_then(Value::as_array) else {
        return Err(format!(
            "Drawing scene invalid at {}: elements must be an array",
            scene_path.display()
        ));
    };
    let preview_path = preview_path(&scene_path);
    let preview_metadata = fs::symlink_metadata(&preview_path).map_err(|error| {
        format!(
            "Drawing preview unavailable at {}: {error}",
            preview_path.display()
        )
    })?;
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
    let canonical_root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    if !canonical_preview.starts_with(canonical_root) {
        return Err(format!(
            "Refusing drawing preview outside vault: {}",
            preview_path.display()
        ));
    }
    Ok(SceneRead {
        path: scene_path,
        raw,
        element_count: elements.len(),
        preview_size: preview_metadata.len(),
    })
}

pub(super) fn read_native_scene(
    root: &Path,
    relative_path: &str,
) -> Result<NativeSceneRead, String> {
    let path = scene_path(root, relative_path)?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Drawing scene unavailable at {}: {error}", path.display()))?;
    let scene: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("Drawing scene invalid at {}: {error}", path.display()))?;
    if scene.get("type").and_then(Value::as_str) != Some("excalidraw") {
        return Err(format!(
            "Drawing scene invalid at {}: expected type=excalidraw",
            path.display()
        ));
    }
    if !scene.get("elements").is_some_and(Value::is_array) {
        return Err(format!(
            "Drawing scene invalid at {}: elements must be an array",
            path.display()
        ));
    }
    Ok(NativeSceneRead { path, raw })
}

pub(super) fn write_scene(path: &Path, raw: &str) -> Result<(), String> {
    fs::write(path, raw).map_err(|error| {
        format!(
            "Unable to persist Excalidraw scene {}: {error}",
            path.display()
        )
    })
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
        scene.set_extension("excalidraw");
        root.join(scene)
    } else {
        root.join(&visible_path)
    };
    let canonical_root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let canonical_scene = fs::canonicalize(&candidate).map_err(|error| {
        format!(
            "Drawing scene unavailable at {}: {error}",
            candidate.display()
        )
    })?;
    if !canonical_scene.starts_with(&canonical_root) {
        return Err(format!(
            "Refusing drawing scene outside vault: {}",
            relative_path
        ));
    }
    Ok(canonical_scene)
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
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
