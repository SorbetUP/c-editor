//! Native Freya rendering and persistence for the existing Excalidraw scene.
//!
//! The file boundary preserves the Tauri scene contract while the visible
//! canvas and pointer interactions are owned by Freya.
//!
//! Provenance integrated from the working Vue/Tauri path:
//! - `a66ce839a`: Tauri scene/PNG write contract and `type=excalidraw` JSON.
//! - `76f092759`: JSON validation plus persisted PNG companion requirement.
//! - `cde9f2ebb`: direct `.excalidraw` library-entry opening contract.
//! - `feature-incoming/drawing-entry-toolbar-clickability/BUG.md`: the
//!   recorded Tauri/Playwright proof and its remaining packaged gap.

use super::ShellState;
use crate::theme;
use freya::prelude::*;

#[path = "drawing_storage.rs"]
mod storage;

// The native renderer is compiled into the production Freya crate here. The
// shell owns the route switch while this module owns drawing state and actions.
#[path = "drawing_canvas.rs"]
pub(super) mod canvas;
pub(super) use canvas::DrawingCanvasState;

pub(crate) const RENDERER_ERROR_LABEL: &str = "Native drawing renderer unavailable";

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
    match result.and_then(|created| {
        let scene = storage::read_native_scene(
            state
                .read()
                .vault
                .as_ref()
                .ok_or_else(|| "No vault selected.".to_owned())?
                .root(),
            &created.relative_path,
        )?;
        let canvas = DrawingCanvasState::from_json(&scene.raw)?;
        let mut shell = state.write();
        shell.editor = None;
        shell.drawing = Some(canvas);
        shell.drawing_path = Some(scene.path);
        shell.error = None;
        shell.view = crate::navigation_contract::WorkspaceView::Notes;
        Ok(created)
    }) {
        Ok(created) => {
            eprintln!(
                "[freya][drawing] action:complete action=create path={} renderer=native-freya",
                created.path.display()
            );
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
        .and_then(|root| storage::read_native_scene(&root, &path));

    match result {
        Ok(scene) => {
            eprintln!(
                "[freya][drawing] action:complete action=open path={} renderer=native-freya",
                path
            );
            match DrawingCanvasState::from_json(&scene.raw) {
                Ok(canvas) => {
                    let mut shell = state.write();
                    shell.editor = None;
                    shell.drawing = Some(canvas);
                    shell.drawing_path = Some(scene.path);
                    shell.error = None;
                    shell.view = crate::navigation_contract::WorkspaceView::Notes;
                }
                Err(error) => state.write().error = Some(error),
            }
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

pub(super) fn save(
    mut state: State<ShellState>,
    canvas_state: State<DrawingCanvasState>,
) -> Result<(), String> {
    let (path, raw, canvas) = {
        let shell = state.read();
        let Some(path) = shell.drawing_path.as_ref() else {
            state.write().error = Some("Drawing has no persisted path.".to_owned());
            return Err("Drawing has no persisted path.".to_owned());
        };
        let canvas = canvas_state.read().clone();
        match canvas.serialize_json() {
            Ok(raw) => (path.clone(), raw, canvas),
            Err(error) => {
                state.write().error = Some(error.clone());
                return Err(error);
            }
        }
    };
    match storage::write_scene(&path, &raw) {
        Ok(()) => {
            state.write().drawing = Some(canvas);
            eprintln!(
                "[freya][drawing] action:complete action=save path={}",
                path.display()
            );
            Ok(())
        }
        Err(error) => {
            eprintln!(
                "[freya][drawing] action:failure action=save path={} error={error}",
                path.display()
            );
            state.write().error = Some(error.clone());
            Err(error)
        }
    }
}

pub(super) fn close(mut state: State<ShellState>, canvas_state: State<DrawingCanvasState>) {
    if save(state, canvas_state).is_err() {
        eprintln!("[freya][drawing] action:close blocked reason=save-failure");
        return;
    }
    let mut shell = state.write();
    shell.drawing = None;
    shell.drawing_path = None;
}

#[derive(PartialEq)]
struct DrawingView {
    state: State<ShellState>,
}

impl Component for DrawingView {
    fn render(&self) -> impl IntoElement {
        let state = self.state;
        let canvas_state = use_state(|| {
            state.read().drawing.clone().unwrap_or_else(|| {
                DrawingCanvasState::new(canvas::DrawingScene {
                    scene_type: "excalidraw".to_owned(),
                    elements: Vec::new(),
                    app_state: serde_json::Value::Object(Default::default()),
                    files: serde_json::Value::Object(Default::default()),
                    extra: Default::default(),
                })
            })
        });
        let save_state = state;
        let close_state = state;
        let save_canvas = canvas_state;
        let close_canvas = canvas_state;

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(theme::color(theme::BG))
            .a11y_alt("Drawing editor")
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(48.))
                    .padding(Gaps::new(0., 12., 0., 12.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child(label().font_size(16.).text("Drawing"))
                    .child(
                        rect()
                            .height(Size::px(30.))
                            .padding(Gaps::new(0., 12., 0., 12.))
                            .center()
                            .background(theme::color(theme::SURFACE))
                            .with_corner_radius(7.)
                            .on_press(move |_| {
                                let _ = save(save_state, save_canvas);
                            })
                            .a11y_alt("Save drawing")
                            .child(label().text("Save")),
                    )
                    .child(
                        rect()
                            .height(Size::px(30.))
                            .padding(Gaps::new(0., 12., 0., 12.))
                            .center()
                            .background(theme::color(theme::SURFACE))
                            .with_corner_radius(7.)
                            .on_press(move |_| close(close_state, close_canvas))
                            .a11y_alt("Close drawing")
                            .child(label().text("Close")),
                    ),
            )
            .child(canvas::drawing_canvas_with_state(canvas_state))
            .into_element()
    }
}

pub(super) fn drawing_view(state: State<ShellState>) -> Element {
    DrawingView { state }.into_element()
}
