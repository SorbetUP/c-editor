//! Real Excalidraw file probing for the native Freya library.
//!
//! The Vue product embeds the installed React Excalidraw renderer. Freya
//! cannot embed that Web/React canvas, so this module owns the real file
//! boundary and reports that capability gap explicitly. It never fabricates a
//! scene, PNG preview, success result, or editable canvas.
//!
//! Provenance integrated from the working Vue/Tauri path:
//! - `a66ce839a`: Tauri scene/PNG write contract and `type=excalidraw` JSON.
//! - `76f092759`: JSON validation plus persisted PNG companion requirement.
//! - `cde9f2ebb`: direct `.excalidraw` library-entry opening contract.
//! - `feature-incoming/drawing-entry-toolbar-clickability/BUG.md`: the
//!   recorded Tauri/Playwright proof and its remaining packaged gap.

use super::ShellState;
use freya::prelude::State;

#[path = "drawing_storage.rs"]
mod storage;

pub(crate) const RENDERER_ERROR_LABEL: &str = "Native drawing renderer unavailable";
const RENDERER_ERROR_PREFIX: &str = "Native drawing renderer unavailable";

pub(crate) fn error_accessibility_label(error: &str) -> &'static str {
    if error.starts_with(RENDERER_ERROR_LABEL) {
        RENDERER_ERROR_LABEL
    } else if error.starts_with("Drawing preview unavailable") {
        "Drawing preview unavailable"
    } else if error.starts_with("Drawing scene invalid") {
        "Drawing scene invalid"
    } else {
        "Library error"
    }
}

pub(super) fn request_create(mut state: State<ShellState>) {
    eprintln!("[freya][drawing] action:start action=create renderer=native-excalidraw");
    let root = state
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let result = root
        .ok_or_else(|| "No vault selected.".to_owned())
        .and_then(|root| storage::create_scene(&root, "Untitled Drawing"));
    match result {
        Ok(created) => {
            eprintln!(
                "[freya][drawing] action:blocked action=create path={} scene_type=excalidraw elements=0 reason=renderer-unavailable",
                created.path.display()
            );
            state.write().error = Some(format!(
                "{RENDERER_ERROR_PREFIX}: the real Excalidraw scene was created at {}, but Freya cannot embed the installed React/canvas renderer or generate its PNG preview.",
                created.relative_path
            ));
        }
        Err(error) => {
            eprintln!("[freya][drawing] action:failure action=create error={error}");
            state.write().error = Some(format!("Drawing create failed: {error}"));
        }
    }
}

pub(super) fn open_existing(mut state: State<ShellState>, relative_path: &str) {
    let path = relative_path.to_owned();
    let root = state
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let result = root
        .ok_or_else(|| "No vault selected.".to_owned())
        .and_then(|root| storage::read_scene(&root, &path));

    match result {
        Ok(scene) => {
            eprintln!(
                "[freya][drawing] action:blocked action=open path={} elements={} preview_bytes={} reason=renderer-unavailable",
                path, scene.element_count, scene.preview_size
            );
            state.write().error = Some(format!(
                "{RENDERER_ERROR_PREFIX}: the real Excalidraw scene was read from {path}, but Freya has no compatible React/canvas renderer."
            ));
        }
        Err(error) => {
            eprintln!(
                "[freya][drawing] action:failure action=open path={} error={error}",
                path
            );
            state.write().error = Some(error);
        }
    }
}
