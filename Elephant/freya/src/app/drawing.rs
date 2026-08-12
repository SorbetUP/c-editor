//! Native Freya drawing editor for Elephant's existing Excalidraw files.
//!
//! Tauri remains the product reference: the editable `.excalidraw` scene is
//! canonical, PNG is a derived preview/export, and the shell provides a slim
//! name row plus explicit close/save controls around the canvas.

use super::ShellState;
use freya::{components::SvgViewer, prelude::*};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

#[path = "drawing_storage.rs"]
mod storage;

#[path = "drawing_canvas.rs"]
pub(super) mod canvas;

use canvas::{DrawingCanvasState, DrawingTool};

fn excalidraw_purple() -> Color {
    Color::from_rgb(105, 101, 219)
}

fn light_shell() -> Color {
    Color::from_rgb(246, 246, 248)
}

fn light_surface() -> Color {
    Color::WHITE
}

fn light_text() -> Color {
    Color::from_rgb(30, 30, 34)
}

fn dark_shell() -> Color {
    Color::from_rgb(18, 18, 18)
}

fn dark_surface() -> Color {
    Color::from_rgb(36, 36, 38)
}

fn dark_text() -> Color {
    Color::from_rgb(238, 242, 255)
}

/// Drawing sessions are scoped to a vault instead of being a process-wide singleton.
///
/// Freya tests intentionally create several app instances in parallel, and desktop
/// users may also have multiple vault-backed windows during a process lifetime. A
/// single global `Option<DrawingSession>` lets one window steal another one's editor.
static ACTIVE_DRAWINGS: Mutex<Vec<(PathBuf, DrawingSession)>> = Mutex::new(Vec::new());

#[derive(Clone, Debug, PartialEq)]
struct DrawingSession {
    relative_path: String,
    title: String,
    canvas: DrawingCanvasState,
    rename_allowed: bool,
}

pub(crate) fn error_accessibility_label(error: &str) -> &'static str {
    if error.starts_with("Drawing save failed") {
        "Drawing save failed"
    } else if error.starts_with("Drawing create failed") {
        "Drawing create failed"
    } else if error.starts_with("Drawing preview unavailable") {
        "Drawing preview unavailable"
    } else if error.starts_with("Drawing scene invalid") {
        "Drawing scene invalid"
    } else if error.starts_with("Drawing") {
        "Drawing error"
    } else {
        "Library error"
    }
}

pub(super) fn request_create(mut state: State<ShellState>) {
    eprintln!("[freya][drawing] action:start action=create renderer=native");
    let snapshot = state.read().clone();
    let root = snapshot
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let current_directory = snapshot.library.current_path.as_str().to_owned();
    let create_directory = current_directory.clone();
    let result = root
        .ok_or_else(|| "No vault selected.".to_owned())
        .and_then(|root| {
            let created = storage::create_standalone_scene(
                &root,
                &create_directory,
                "Untitled Drawing",
            )?;
            let scene = storage::read_scene(&root, &created.relative_path)?;
            let session = session_from_read(scene)?;
            Ok((root, created.path, session))
        });

    match result {
        Ok((root, path, session)) => {
            eprintln!(
                "[freya][drawing] action:complete action=create path={} elements=0",
                path.display()
            );
            set_active(root, session);
            let mut shell = state.write();
            shell.error = None;
            shell.reload_directory(&current_directory);
        }
        Err(error) => {
            eprintln!("[freya][drawing] action:failure action=create error={error}");
            state.write().error = Some(format!("Drawing create failed: {error}"));
        }
    }
}

pub(super) fn open_existing(mut state: State<ShellState>, relative_path: &str) {
    let path = relative_path.to_owned();
    eprintln!("[freya][drawing] action:start action=open path={path}");
    let root = state
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let result = root
        .ok_or_else(|| "No vault selected.".to_owned())
        .and_then(|root| {
            let scene = storage::read_scene(&root, &path)?;
            let session = session_from_read(scene)?;
            Ok((root, session))
        });

    match result {
        Ok((root, session)) => {
            eprintln!(
                "[freya][drawing] action:complete action=open path={} elements={}",
                session.relative_path,
                session.canvas.renderable_elements().len()
            );
            set_active(root, session);
            state.write().error = None;
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

fn session_from_read(scene: storage::SceneRead) -> Result<DrawingSession, String> {
    let has_preview = scene.preview_size.is_some();
    let rename_allowed =
        storage::can_rename_standalone_scene(&scene.relative_path, has_preview);
    eprintln!(
        "[freya][drawing] scene:read path={} elements={} preview_bytes={} rename_allowed={}",
        scene.relative_path,
        scene.element_count,
        scene
            .preview_size
            .map(|size| size.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        rename_allowed
    );
    Ok(DrawingSession {
        relative_path: scene.relative_path,
        title: scene.title,
        canvas: DrawingCanvasState::from_json(&scene.raw)?,
        rename_allowed,
    })
}

fn active_drawings() -> MutexGuard<'static, Vec<(PathBuf, DrawingSession)>> {
    ACTIVE_DRAWINGS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn set_active(root: PathBuf, session: DrawingSession) {
    let mut active = active_drawings();
    if let Some((_, current)) = active.iter_mut().find(|(key, _)| key == &root) {
        *current = session;
    } else {
        active.push((root, session));
    }
}

fn clear_active(root: &Path) {
    active_drawings().retain(|(key, _)| key != root);
}

fn sync_active_identity(root: &Path, old_path: &str, new_path: &str, title: &str) {
    if let Some((_, session)) = active_drawings()
        .iter_mut()
        .find(|(key, session)| key == root && session.relative_path == old_path)
    {
        session.relative_path = new_path.to_owned();
        session.title = title.to_owned();
    }
}

fn sync_active_canvas(
    root: &Path,
    relative_path: &str,
    title: &str,
    canvas: &DrawingCanvasState,
) {
    if let Some((_, session)) = active_drawings()
        .iter_mut()
        .find(|(key, session)| key == root && session.relative_path == relative_path)
    {
        session.title = title.to_owned();
        session.canvas = canvas.clone();
    }
}

fn active_session(root: &Path) -> Option<DrawingSession> {
    active_drawings()
        .iter()
        .find(|(key, _)| key == root)
        .map(|(_, session)| session.clone())
}

fn vault_root(state: State<ShellState>) -> Option<PathBuf> {
    state
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf())
}

/// Minimal integration seam used by `library::main_content`.
pub(super) fn active_panel(state: State<ShellState>) -> Option<Element> {
    let root = vault_root(state)?;
    active_session(&root).map(|session| DrawingPanel { state, session }.into_element())
}

#[derive(PartialEq)]
struct DrawingPanel {
    state: State<ShellState>,
    session: DrawingSession,
}

impl Component for DrawingPanel {
    fn render(&self) -> impl IntoElement {
        let initial_canvas = self.session.canvas.clone();
        let canvas_state = use_state(move || initial_canvas);
        let status = use_state(String::new);
        let initial_path = self.session.relative_path.clone();
        let relative_path = use_state(move || initial_path);
        let initial_title = self.session.title.clone();
        let title = use_state(move || initial_title);
        drawing_shell(
            self.state,
            canvas_state,
            status,
            relative_path,
            title,
            self.session.rename_allowed,
        )
    }
}

fn drawing_shell(
    shell_state: State<ShellState>,
    canvas_state: State<DrawingCanvasState>,
    status: State<String>,
    relative_path: State<String>,
    title: State<String>,
    rename_allowed: bool,
) -> Element {
    let snapshot = canvas_state.read().clone();
    let path_snapshot = relative_path.read().clone();
    let title_snapshot = title.read().clone();
    let dark = snapshot.is_dark_canvas();
    let shell = if dark { dark_shell() } else { light_shell() };
    let surface = if dark { dark_surface() } else { light_surface() };
    let text = if dark { dark_text() } else { light_text() };
    let muted = if dark {
        Color::from_rgb(170, 170, 180)
    } else {
        Color::from_rgb(96, 96, 108)
    };
    let active_root = vault_root(shell_state);

    let mut close_state = shell_state;
    let mut key_shell_state = shell_state;
    let save_shell_state = shell_state;
    let submit_shell_state = shell_state;
    let save_canvas_state = canvas_state;
    let submit_canvas_state = canvas_state;
    let save_status = status;
    let submit_status = status;
    let mut key_canvas_state = canvas_state;
    let key_status = status;
    let key_path = relative_path;
    let key_title = title;
    let save_path = relative_path;
    let save_title = title;
    let submit_path = relative_path;
    let submit_title = title;
    let key_root = active_root.clone();
    let close_root = active_root;

    let name_control = if rename_allowed {
        Some(
            Input::new(title)
                .placeholder("Drawing name")
                .flat()
                .compact()
                .width(Size::px(220.))
                .on_submit(move |_| {
                    save_scene(
                        submit_shell_state,
                        submit_canvas_state,
                        submit_status,
                        submit_path,
                        submit_title,
                        true,
                    );
                })
                .on_pre_key_down(|event: Event<KeyboardEventData>| {
                    let command = event.modifiers.ctrl() || event.modifiers.meta();
                    if command {
                        if let Key::Character(value) = &event.key {
                            if value.to_ascii_lowercase() == "s" {
                                // Let the shell own the save shortcut even while the name input is focused.
                                return false;
                            }
                        }
                    }
                    match &event.key {
                        Key::Named(NamedKey::Enter)
                        | Key::Named(NamedKey::Escape)
                        | Key::Named(NamedKey::Shift) => true,
                        Key::Named(NamedKey::Tab) => false,
                        _ => {
                            event.stop_propagation();
                            event.prevent_default();
                            true
                        }
                    }
                })
                .into_element(),
        )
    } else {
        Some(
            rect()
                .height(Size::px(20.))
                .padding(Gaps::new(0., 8., 0., 8.))
                .center()
                .background(if dark {
                    Color::from_argb(30, 148, 163, 184)
                } else {
                    Color::from_argb(18, 40, 40, 52)
                })
                .with_corner_radius(4.)
                .child(label().font_size(12.).color(text).text(title_snapshot.clone()))
                .into_element(),
        )
    };

    rect()
        .key(("native-drawing-shell", path_snapshot.clone(), snapshot.revision))
        .width(Size::fill())
        .height(Size::fill())
        .background(shell)
        .color(text)
        .overflow(Overflow::Clip)
        .a11y_alt(format!("Drawing editor {title_snapshot}"))
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Escape) {
                if let Some(root) = key_root.as_deref() {
                    clear_active(root);
                }
                key_shell_state.write().error = None;
                event.stop_propagation();
                return;
            }
            if matches!(
                event.key,
                Key::Named(NamedKey::Delete | NamedKey::Backspace)
            ) {
                key_canvas_state.write().delete_selected();
                event.stop_propagation();
                return;
            }
            let command = event.modifiers.ctrl() || event.modifiers.meta();
            if command {
                if let Key::Character(value) = &event.key {
                    match value.to_ascii_lowercase().as_str() {
                        "s" => {
                            save_scene(
                                key_shell_state,
                                key_canvas_state,
                                key_status,
                                key_path,
                                key_title,
                                rename_allowed,
                            );
                            event.stop_propagation();
                        }
                        "z" if event.modifiers.shift() => {
                            key_canvas_state.write().redo();
                            event.stop_propagation();
                        }
                        "z" => {
                            key_canvas_state.write().undo();
                            event.stop_propagation();
                        }
                        "y" => {
                            key_canvas_state.write().redo();
                            event.stop_propagation();
                        }
                        _ => {}
                    }
                }
            }
        })
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(28.))
                .padding(Gaps::new(0., 8., 0., 86.))
                .horizontal()
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .background(shell)
                .border(
                    Border::new()
                        .fill(if dark {
                            Color::from_argb(28, 148, 163, 184)
                        } else {
                            Color::from_argb(20, 20, 20, 28)
                        })
                        .width(1.),
                )
                .maybe_child(name_control)
                .child(
                    rect()
                        .horizontal()
                        .spacing(6.)
                        .child(circle_action(
                            DrawingIcon::Close,
                            false,
                            text,
                            surface,
                            "Close drawing · Esc",
                            move |event| {
                                if let Some(root) = close_root.as_deref() {
                                    clear_active(root);
                                }
                                close_state.write().error = None;
                                event.stop_propagation();
                            },
                        ))
                        .child(circle_action(
                            DrawingIcon::Save,
                            true,
                            Color::WHITE,
                            excalidraw_purple(),
                            "Save drawing · Ctrl/Cmd S",
                            move |event| {
                                save_scene(
                                    save_shell_state,
                                    save_canvas_state,
                                    save_status,
                                    save_path,
                                    save_title,
                                    rename_allowed,
                                );
                                event.stop_propagation();
                            },
                        )),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .child(canvas::drawing_canvas_with_state(canvas_state))
                .child(tool_palette(canvas_state, dark, text, surface))
                .child(bottom_controls(
                    canvas_state,
                    status,
                    dark,
                    text,
                    muted,
                    surface,
                )),
        )
        .into_element()
}

fn save_scene(
    mut shell_state: State<ShellState>,
    canvas_state: State<DrawingCanvasState>,
    mut status: State<String>,
    mut relative_path: State<String>,
    mut title: State<String>,
    rename_allowed: bool,
) {
    let shell_snapshot = shell_state.read().clone();
    let root = shell_snapshot
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let library_path = shell_snapshot.library.current_path.as_str().to_owned();
    let old_path = relative_path.read().clone();
    let current_title = title.read().clone();
    let requested_title = current_title.trim().to_owned();
    let requested_title = if requested_title.is_empty() {
        "Untitled Drawing".to_owned()
    } else {
        requested_title
    };
    let Some(root) = root else {
        let error = "No vault selected.".to_owned();
        *status.write() = "Save failed".to_owned();
        shell_state.write().error = Some(format!("Drawing save failed: {error}"));
        return;
    };

    let (target_path, target_title) = if rename_allowed {
        match storage::rename_standalone_scene(&root, &old_path, &requested_title) {
            Ok(renamed) => {
                if renamed.relative_path != old_path || renamed.title != current_title {
                    sync_active_identity(&root, &old_path, &renamed.relative_path, &renamed.title);
                    *relative_path.write() = renamed.relative_path.clone();
                    *title.write() = renamed.title.clone();
                    shell_state.write().reload_directory(&library_path);
                }
                (renamed.relative_path, renamed.title)
            }
            Err(error) => {
                eprintln!(
                    "[freya][drawing] action:failure action=rename path={} error={error}",
                    old_path
                );
                *status.write() = "Rename failed".to_owned();
                shell_state.write().error = Some(format!("Drawing save failed: {error}"));
                return;
            }
        }
    } else {
        (old_path, current_title)
    };

    let mut canvas_snapshot = canvas_state.read().clone();
    if rename_allowed {
        canvas_snapshot
            .document
            .extra
            .insert("title".to_owned(), Value::String(target_title.clone()));
    }
    let result = canvas_snapshot
        .serialize_json()
        .and_then(|raw| storage::write_scene(&root, &target_path, &raw));
    match result {
        Ok(()) => {
            eprintln!("[freya][drawing] action:complete action=save path={target_path}");
            sync_active_canvas(&root, &target_path, &target_title, &canvas_snapshot);
            *status.write() = "Saved".to_owned();
            shell_state.write().error = None;
        }
        Err(error) => {
            eprintln!(
                "[freya][drawing] action:failure action=save path={} error={error}",
                target_path
            );
            *status.write() = "Save failed".to_owned();
            shell_state.write().error = Some(format!("Drawing save failed: {error}"));
        }
    }
}

fn tool_palette(
    state: State<DrawingCanvasState>,
    dark: bool,
    text: Color,
    surface: Color,
) -> Element {
    let active = state.read().active_tool();
    let tools = DrawingTool::ALL
        .into_iter()
        .map(|tool| tool_button(state, tool, tool == active, dark, text))
        .collect::<Vec<_>>();
    rect()
        .position(Position::new_absolute().top(16.).left(16.))
        .height(Size::px(46.))
        .padding(Gaps::new_all(5.))
        .horizontal()
        .spacing(4.)
        .background(surface)
        .border(
            Border::new()
                .fill(if dark {
                    Color::from_rgb(58, 58, 64)
                } else {
                    Color::from_rgb(226, 226, 232)
                })
                .width(1.),
        )
        .with_corner_radius(10.)
        .layer(Layer::OverlayLevel(20))
        .a11y_alt("Drawing tools")
        .children(tools)
        .into_element()
}

fn tool_button(
    mut state: State<DrawingCanvasState>,
    tool: DrawingTool,
    selected: bool,
    dark: bool,
    text: Color,
) -> Element {
    let background = if selected {
        if dark {
            Color::from_rgb(63, 60, 105)
        } else {
            Color::from_rgb(227, 226, 254)
        }
    } else {
        Color::from_argb(0, 0, 0, 0)
    };
    let icon_color = if selected { excalidraw_purple() } else { text };
    rect()
        .width(Size::px(34.))
        .height(Size::px(34.))
        .center()
        .background(background)
        .with_corner_radius(7.)
        .a11y_alt(tool.label())
        .on_mouse_up(move |event: Event<MouseEventData>| {
            state.write().set_tool(tool);
            event.stop_propagation();
        })
        .child(svg_icon(icon_for_tool(tool), icon_color, 18.))
        .into_element()
}

fn bottom_controls(
    state: State<DrawingCanvasState>,
    status: State<String>,
    dark: bool,
    text: Color,
    muted: Color,
    surface: Color,
) -> Element {
    let snapshot = state.read().clone();
    let zoom = (snapshot.viewport.zoom * 100.).round() as i32;
    let elements = snapshot.renderable_elements().len();
    let message = status.read().clone();
    let mut undo_state = state;
    let mut redo_state = state;

    rect()
        .position(Position::new_absolute().left(16.).bottom(16.))
        .height(Size::px(36.))
        .padding(Gaps::new(0., 6., 0., 6.))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(4.)
        .background(surface)
        .border(
            Border::new()
                .fill(if dark {
                    Color::from_rgb(58, 58, 64)
                } else {
                    Color::from_rgb(226, 226, 232)
                })
                .width(1.),
        )
        .with_corner_radius(9.)
        .layer(Layer::OverlayLevel(20))
        .child(compact_action(
            DrawingIcon::Undo,
            text,
            snapshot.can_undo(),
            "Undo",
            move |event| {
                undo_state.write().undo();
                event.stop_propagation();
            },
        ))
        .child(compact_action(
            DrawingIcon::Redo,
            text,
            snapshot.can_redo(),
            "Redo",
            move |event| {
                redo_state.write().redo();
                event.stop_propagation();
            },
        ))
        .child(
            label()
                .font_size(11.)
                .color(muted)
                .text(format!("{zoom}% · {elements} elements")),
        )
        .maybe_child((!message.is_empty()).then(|| {
            label()
                .font_size(11.)
                .color(muted)
                .text(message)
                .into_element()
        }))
        .into_element()
}

fn circle_action<F>(
    icon: DrawingIcon,
    primary: bool,
    color: Color,
    background: Color,
    alt: &'static str,
    on_press: F,
) -> Element
where
    F: FnMut(Event<MouseEventData>) + 'static,
{
    rect()
        .width(Size::px(22.))
        .height(Size::px(22.))
        .center()
        .background(background)
        .border(
            Border::new()
                .fill(if primary {
                    background
                } else {
                    Color::from_argb(52, 148, 163, 184)
                })
                .width(1.),
        )
        .with_corner_radius(999.)
        .a11y_alt(alt)
        .on_mouse_up(on_press)
        .child(svg_icon(icon, color, 14.))
        .into_element()
}

fn compact_action<F>(
    icon: DrawingIcon,
    color: Color,
    enabled: bool,
    alt: &'static str,
    on_press: F,
) -> Element
where
    F: FnMut(Event<MouseEventData>) + 'static,
{
    rect()
        .width(Size::px(26.))
        .height(Size::px(26.))
        .center()
        .with_corner_radius(6.)
        .a11y_alt(alt)
        .on_mouse_up(on_press)
        .child(svg_icon(
            icon,
            if enabled {
                color
            } else {
                Color::from_argb(80, 128, 128, 138)
            },
            15.,
        ))
        .into_element()
}

#[derive(Clone, Copy)]
enum DrawingIcon {
    Select,
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Pencil,
    Eraser,
    Undo,
    Redo,
    Close,
    Save,
}

const SELECT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 3 7.1 17 2.6-7.3L20 10.1 3 3Z"/></svg>"#;
const RECTANGLE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="18" height="14" rx="2"/></svg>"#;
const ELLIPSE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="12" rx="9" ry="7"/></svg>"#;
const LINE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 19 19 5"/></svg>"#;
const ARROW: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 19 19 5"/><path d="M11 5h8v8"/></svg>"#;
const PENCIL: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>"#;
const ERASER: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 21-4-4 10.5-10.5a2.8 2.8 0 0 1 4 0l.9.9a2.8 2.8 0 0 1 0 4L9 21Z"/><path d="m10 10 7 7"/><path d="M7 21h14"/></svg>"#;
const UNDO: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 7 4 12l5 5"/><path d="M20 17a8 8 0 0 0-16-5"/></svg>"#;
const REDO: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 7 5 5-5 5"/><path d="M4 17a8 8 0 0 1 16-5"/></svg>"#;
const CLOSE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6 6 18 18M18 6 6 18"/></svg>"#;
const SAVE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 4 4L19 6"/></svg>"#;

fn icon_for_tool(tool: DrawingTool) -> DrawingIcon {
    match tool {
        DrawingTool::Selection => DrawingIcon::Select,
        DrawingTool::Rectangle => DrawingIcon::Rectangle,
        DrawingTool::Ellipse => DrawingIcon::Ellipse,
        DrawingTool::Line => DrawingIcon::Line,
        DrawingTool::Arrow => DrawingIcon::Arrow,
        DrawingTool::Freedraw => DrawingIcon::Pencil,
        DrawingTool::Eraser => DrawingIcon::Eraser,
    }
}

fn svg_icon(icon: DrawingIcon, color: Color, size: f32) -> Element {
    SvgViewer::new(match icon {
        DrawingIcon::Select => SELECT,
        DrawingIcon::Rectangle => RECTANGLE,
        DrawingIcon::Ellipse => ELLIPSE,
        DrawingIcon::Line => LINE,
        DrawingIcon::Arrow => ARROW,
        DrawingIcon::Pencil => PENCIL,
        DrawingIcon::Eraser => ERASER,
        DrawingIcon::Undo => UNDO,
        DrawingIcon::Redo => REDO,
        DrawingIcon::Close => CLOSE,
        DrawingIcon::Save => SAVE,
    })
    .width(Size::px(size))
    .height(Size::px(size))
    .show_loader(false)
    .color(color)
    .stroke(color)
    .stroke_width(2.)
    .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas_with_elements(count: usize) -> DrawingCanvasState {
        let elements = (0..count)
            .map(|index| {
                format!(
                    r#"{{"id":"rect-{index}","type":"rectangle","x":0,"y":0,"width":10,"height":10}}"#
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        DrawingCanvasState::from_json(&format!(
            r#"{{"type":"excalidraw","elements":[{elements}],"appState":{{}},"files":{{}}}}"#
        ))
        .unwrap()
    }

    fn session(path: &str, title: &str, elements: usize) -> DrawingSession {
        DrawingSession {
            relative_path: path.to_owned(),
            title: title.to_owned(),
            canvas: canvas_with_elements(elements),
            rename_allowed: true,
        }
    }

    #[test]
    fn save_snapshot_refreshes_the_active_session_only_for_the_same_path() {
        let root = PathBuf::from("/tmp/elephant-drawing-test-save");
        clear_active(&root);
        set_active(root.clone(), session("Sketch.excalidraw", "Sketch", 0));

        sync_active_canvas(
            &root,
            "Other.excalidraw",
            "Other",
            &canvas_with_elements(2),
        );
        assert_eq!(
            active_session(&root)
                .unwrap()
                .canvas
                .renderable_elements()
                .len(),
            0
        );

        sync_active_canvas(
            &root,
            "Sketch.excalidraw",
            "Sketch",
            &canvas_with_elements(2),
        );
        let active = active_session(&root).unwrap();
        assert_eq!(active.canvas.renderable_elements().len(), 2);
        assert_eq!(active.title, "Sketch");
        clear_active(&root);
    }

    #[test]
    fn identity_sync_moves_the_live_session_to_the_real_renamed_path() {
        let root = PathBuf::from("/tmp/elephant-drawing-test-rename");
        clear_active(&root);
        set_active(root.clone(), session("Old.excalidraw", "Old", 1));
        sync_active_identity(&root, "Old.excalidraw", "New.excalidraw", "New");
        let active = active_session(&root).unwrap();
        assert_eq!(active.relative_path, "New.excalidraw");
        assert_eq!(active.title, "New");
        assert_eq!(active.canvas.renderable_elements().len(), 1);
        clear_active(&root);
    }

    #[test]
    fn active_sessions_are_isolated_by_vault_root() {
        let root_a = PathBuf::from("/tmp/elephant-drawing-vault-a");
        let root_b = PathBuf::from("/tmp/elephant-drawing-vault-b");
        clear_active(&root_a);
        clear_active(&root_b);
        set_active(root_a.clone(), session("A.excalidraw", "A", 1));
        set_active(root_b.clone(), session("B.excalidraw", "B", 2));

        sync_active_canvas(&root_a, "A.excalidraw", "A", &canvas_with_elements(3));
        assert_eq!(
            active_session(&root_a)
                .unwrap()
                .canvas
                .renderable_elements()
                .len(),
            3
        );
        assert_eq!(
            active_session(&root_b)
                .unwrap()
                .canvas
                .renderable_elements()
                .len(),
            2
        );

        clear_active(&root_a);
        assert!(active_session(&root_a).is_none());
        assert_eq!(active_session(&root_b).unwrap().title, "B");
        clear_active(&root_b);
    }
}
