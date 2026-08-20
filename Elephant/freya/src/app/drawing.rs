//! Native Freya rendering and persistence for Excalidraw scenes.
//! The persisted `.excalidraw` JSON remains the source of truth.

use super::{
    navigation_icons::{svg_icon, Icon},
    ShellState,
};
use crate::theme;
use freya::prelude::*;
use serde_json::{Map, Value};

#[path = "drawing_storage.rs"]
mod storage;
#[path = "drawing_icons.rs"]
mod drawing_icons;
use drawing_icons::{CHECK_ICON, HELP_ICON, LOCK_ICON, MENU_ICON, SHAPES_ICON, X_ICON};

#[path = "drawing_canvas.rs"]
pub(super) mod canvas;
pub(super) use canvas::DrawingCanvasState;

pub(crate) const RENDERER_ERROR_LABEL: &str = "Native drawing renderer unavailable";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawingTool {
    Select,
    Hand,
    Rectangle,
    Ellipse,
    Diamond,
    Arrow,
    Line,
    Pencil,
    Text,
    Image,
    Eraser,
}

impl DrawingTool {
    const ALL: [Self; 11] = [
        Self::Hand,
        Self::Select,
        Self::Rectangle,
        Self::Diamond,
        Self::Ellipse,
        Self::Arrow,
        Self::Line,
        Self::Pencil,
        Self::Text,
        Self::Image,
        Self::Eraser,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Select => "Selection",
            Self::Hand => "Hand",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Diamond => "Diamond",
            Self::Arrow => "Arrow",
            Self::Line => "Line",
            Self::Pencil => "Freedraw",
            Self::Text => "Text",
            Self::Image => "Image",
            Self::Eraser => "Eraser",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::Select => "1",
            Self::Hand => "H",
            Self::Rectangle => "2",
            Self::Diamond => "3",
            Self::Ellipse => "4",
            Self::Arrow => "5",
            Self::Line => "6",
            Self::Pencil => "7",
            Self::Text => "8",
            Self::Image => "9",
            Self::Eraser => "0",
        }
    }

    fn icon_source(self) -> &'static [u8] {
        match self {
            Self::Select => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m5 3 14 9-6 2-2 6Z"/></svg>"#,
            Self::Hand => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 11V6a2 2 0 0 0-4 0v5"/><path d="M14 10V4a2 2 0 0 0-4 0v6"/><path d="M10 10V6a2 2 0 0 0-4 0v8"/><path d="M6 14v-1a2 2 0 0 0-4 0c0 5 3 9 8 9h1a7 7 0 0 0 7-7v-4a2 2 0 0 0-4 0"/></svg>"#,
            Self::Rectangle => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="14" x="3" y="5" rx="2"/></svg>"#,
            Self::Ellipse => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="8"/></svg>"#,
            Self::Diamond => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 3 9 9-9 9-9-9Z"/></svg>"#,
            Self::Arrow => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m13 6 6 6-6 6"/></svg>"#,
            Self::Line => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 19 19 5"/></svg>"#,
            Self::Pencil => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m4 20 4.5-1L19 8.5a2.12 2.12 0 0 0-3-3L5.5 16Z"/><path d="m14.5 6.5 3 3"/></svg>"#,
            Self::Text => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V5h16v2"/><path d="M12 5v14"/><path d="M8 19h8"/></svg>"#,
            Self::Image => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>"#,
            Self::Eraser => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 21 10-10"/><path d="m5 17 8-8 5 5-8 8H5a2 2 0 0 1 0-4Z"/><path d="m16 8 2-2a2 2 0 0 1 3 3l-2 2"/></svg>"#,
        }
    }
}

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
        Ok(created) => eprintln!(
            "[freya][drawing] action:complete action=create path={} renderer=native-freya",
            created.path.display()
        ),
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
        Ok(scene) => match DrawingCanvasState::from_json(&scene.raw) {
            Ok(canvas) => {
                let mut shell = state.write();
                shell.editor = None;
                shell.drawing = Some(canvas);
                shell.drawing_path = Some(scene.path);
                shell.error = None;
                shell.view = crate::navigation_contract::WorkspaceView::Notes;
                eprintln!("[freya][drawing] action:complete action=open path={path} renderer=native-freya");
            }
            Err(error) => state.write().error = Some(error),
        },
        Err(error) => {
            eprintln!("[freya][drawing] action:failure action=open path={path} error={error}");
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
        let raw = canvas.serialize_json()?;
        (path.clone(), raw, canvas)
    };
    match storage::write_scene(&path, &raw) {
        Ok(()) => {
            let mut shell = state.write();
            shell.drawing = Some(canvas);
            shell.error = None;
            eprintln!("[freya][drawing] action:complete action=save path={}", path.display());
            Ok(())
        }
        Err(error) => {
            eprintln!("[freya][drawing] action:failure action=save path={} error={error}", path.display());
            state.write().error = Some(error.clone());
            Err(error)
        }
    }
}

pub(super) fn close(
    mut state: State<ShellState>,
    canvas_state: State<DrawingCanvasState>,
) -> Result<(), String> {
    save(state, canvas_state)?;
    let mut shell = state.write();
    shell.drawing = None;
    shell.drawing_path = None;
    shell.error = None;
    Ok(())
}

fn drawing_error_notice(error: &str) -> Element {
    rect()
        .position(Position::new_absolute().left(12.).right(12.).top(58.))
        .padding(Gaps::new_all(10.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::DANGER)).width(1.))
        .with_corner_radius(8.)
        .layer(Layer::OverlayLevel(30))
        .a11y_alt(error_accessibility_label(error))
        .child(label().color(theme::color(theme::DANGER)).text(error.to_owned()))
        .into_element()
}

fn icon_button(
    source: &'static [u8],
    alt: &'static str,
    size: f32,
    color: Color,
    background: Color,
    mut action: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::px(size))
        .height(Size::px(size))
        .center()
        .background(background)
        .with_corner_radius(if matches!(alt, "Close drawing" | "Save drawing") { 20.0 } else { 9.0 })
        .a11y_alt(alt)
        .on_mouse_up(move |event| action(event))
        .child(
            SvgViewer::new(source)
                .width(Size::px(20.))
                .height(Size::px(20.))
                .show_loader(false)
                .color(color)
                .stroke(color)
                .stroke_width(2.0),
        )
        .into_element()
}

fn action_button(
    caption: impl Into<String>,
    alt: impl Into<String>,
    width: f32,
    mut action: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::px(width))
        .height(Size::px(34.0))
        .center()
        .background(Color::from_rgb(48, 48, 58))
        .with_corner_radius(7.0)
        .a11y_alt(alt.into())
        .on_mouse_up(move |event| action(event))
        .child(label().font_size(12.0).color(Color::from_rgb(235, 235, 240)).text(caption.into()))
        .into_element()
}

fn tool_button(
    tool: DrawingTool,
    active: DrawingTool,
    mut selected: State<DrawingTool>,
    mut canvas: State<DrawingCanvasState>,
) -> Element {
    let active = tool == active;
    let color = if active { Color::WHITE } else { Color::from_rgb(215, 215, 220) };
    rect()
        .width(Size::px(38.0))
        .height(Size::px(38.0))
        .center()
        .background(if active { theme::color(theme::PRIMARY) } else { Color::TRANSPARENT })
        .with_corner_radius(7.0)
        .a11y_alt(format!("{} tool ({})", tool.label(), tool.shortcut()))
        .on_mouse_up(move |_| {
            canvas.write().set_active_tool_label(tool.label());
            selected.set(tool);
        })
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(2.0)
                .child(
                    SvgViewer::new(tool.icon_source())
                        .width(Size::px(18.0))
                        .height(Size::px(18.0))
                        .show_loader(false)
                        .color(color)
                        .stroke(color)
                        .stroke_width(2.0),
                )
                .child(label().font_size(9.0).color(color).text(tool.shortcut())),
        )
        .into_element()
}

fn swatch(
    color: Color,
    alt: &'static str,
    mut action: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::px(28.0))
        .height(Size::px(28.0))
        .background(color)
        .border(Border::new().fill(Color::from_rgb(80, 80, 90)).width(1.0))
        .with_corner_radius(5.0)
        .a11y_alt(alt)
        .on_mouse_up(move |event| action(event))
        .into_element()
}

fn properties_panel(canvas: State<DrawingCanvasState>) -> Element {
    let panel = Color::from_rgb(35, 35, 42);
    let text = Color::from_rgb(235, 235, 240);
    let mut stroke_white = canvas;
    let mut stroke_red = canvas;
    let mut stroke_green = canvas;
    let mut stroke_blue = canvas;
    let mut stroke_orange = canvas;
    let mut bg_none = canvas;
    let mut bg_red = canvas;
    let mut bg_green = canvas;
    let mut bg_blue = canvas;
    let mut bg_orange = canvas;
    let mut width_1 = canvas;
    let mut width_2 = canvas;
    let mut width_4 = canvas;
    let mut style_solid = canvas;
    let mut style_dash = canvas;
    let mut style_dot = canvas;
    let mut opacity_25 = canvas;
    let mut opacity_50 = canvas;
    let mut opacity_75 = canvas;
    let mut opacity_100 = canvas;
    let mut front = canvas;
    let mut forward = canvas;
    let mut backward = canvas;
    let mut back = canvas;

    rect()
        .position(Position::new_absolute().left(16.0).top(72.0))
        .width(Size::px(290.0))
        .padding(Gaps::new(16.0, 14.0, 16.0, 14.0))
        .background(panel)
        .with_corner_radius(10.0)
        .a11y_alt("Excalidraw properties")
        .child(label().font_size(15.0).color(text).text("Stroke"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(swatch(Color::from_rgb(224, 224, 224), "Stroke white", move |_| { stroke_white.write().set_selected_stroke("#e0e0e0"); }))
                .child(swatch(Color::from_rgb(255, 111, 117), "Stroke red", move |_| { stroke_red.write().set_selected_stroke("#ff6f75"); }))
                .child(swatch(Color::from_rgb(48, 155, 73), "Stroke green", move |_| { stroke_green.write().set_selected_stroke("#309b49"); }))
                .child(swatch(Color::from_rgb(78, 145, 216), "Stroke blue", move |_| { stroke_blue.write().set_selected_stroke("#4e91d8"); }))
                .child(swatch(Color::from_rgb(184, 101, 0), "Stroke orange", move |_| { stroke_orange.write().set_selected_stroke("#b86500"); })),
        )
        .child(label().font_size(15.0).color(text).text("Background"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(swatch(Color::TRANSPARENT, "Transparent background", move |_| { bg_none.write().set_selected_background("transparent"); }))
                .child(swatch(Color::from_rgb(105, 50, 50), "Red background", move |_| { bg_red.write().set_selected_background("#693232"); }))
                .child(swatch(Color::from_rgb(0, 88, 24), "Green background", move |_| { bg_green.write().set_selected_background("#005818"); }))
                .child(swatch(Color::from_rgb(18, 75, 105), "Blue background", move |_| { bg_blue.write().set_selected_background("#124b69"); }))
                .child(swatch(Color::from_rgb(68, 50, 0), "Orange background", move |_| { bg_orange.write().set_selected_background("#443200"); })),
        )
        .child(label().font_size(15.0).color(text).text("Stroke width"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(action_button("1", "Thin stroke", 42.0, move |_| { width_1.write().set_selected_stroke_width(1.0); }))
                .child(action_button("2", "Medium stroke", 42.0, move |_| { width_2.write().set_selected_stroke_width(2.0); }))
                .child(action_button("4", "Wide stroke", 42.0, move |_| { width_4.write().set_selected_stroke_width(4.0); })),
        )
        .child(label().font_size(15.0).color(text).text("Stroke style"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(action_button("—", "Solid stroke", 42.0, move |_| { style_solid.write().set_selected_stroke_style("solid"); }))
                .child(action_button("╌", "Dashed stroke", 42.0, move |_| { style_dash.write().set_selected_stroke_style("dashed"); }))
                .child(action_button("┈", "Dotted stroke", 42.0, move |_| { style_dot.write().set_selected_stroke_style("dotted"); })),
        )
        .child(label().font_size(15.0).color(text).text("Opacity"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(action_button("25", "Opacity 25 percent", 48.0, move |_| { opacity_25.write().set_selected_opacity(25.0); }))
                .child(action_button("50", "Opacity 50 percent", 48.0, move |_| { opacity_50.write().set_selected_opacity(50.0); }))
                .child(action_button("75", "Opacity 75 percent", 48.0, move |_| { opacity_75.write().set_selected_opacity(75.0); }))
                .child(action_button("100", "Opacity 100 percent", 48.0, move |_| { opacity_100.write().set_selected_opacity(100.0); })),
        )
        .child(label().font_size(15.0).color(text).text("Layers"))
        .child(
            rect().horizontal().spacing(8.0)
                .child(action_button("↗", "Bring to front", 48.0, move |_| { front.write().bring_selection_to_front(); }))
                .child(action_button("↑", "Bring forward", 48.0, move |_| { forward.write().bring_selection_forward(); }))
                .child(action_button("↓", "Send backward", 48.0, move |_| { backward.write().send_selection_backward(); }))
                .child(action_button("↙", "Send to back", 48.0, move |_| { back.write().send_selection_to_back(); })),
        )
        .into_element()
}

fn unlock_all(canvas: State<DrawingCanvasState>) {
    let mut canvas = canvas;
    let mut state = canvas.write();
    let locked = state
        .document
        .elements
        .iter()
        .enumerate()
        .filter_map(|(index, element)| {
            element
                .extra
                .get("locked")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    if locked.is_empty() {
        return;
    }
    state.checkpoint();
    for index in &locked {
        state.document.elements[*index]
            .extra
            .insert("locked".to_owned(), Value::Bool(false));
    }
    for index in locked {
        state.touch_element(index);
    }
}

fn reset_canvas(canvas: State<DrawingCanvasState>) {
    let mut canvas = canvas;
    let mut state = canvas.write();
    if state.document.elements.is_empty() && state.document.files.as_object().is_none_or(Map::is_empty) {
        return;
    }
    state.checkpoint();
    state.document.elements.clear();
    state.document.files = Value::Object(Map::new());
    state.revision = state.revision.wrapping_add(1);
}

fn menu_panel(mut open: State<bool>, canvas: State<DrawingCanvasState>) -> Element {
    let mut reset = canvas;
    let unlock = canvas;
    rect()
        .position(Position::new_absolute().left(16.0).top(64.0))
        .width(Size::px(260.0))
        .padding(Gaps::new_all(10.0))
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .with_corner_radius(9.0)
        .a11y_alt("Excalidraw menu")
        .child(action_button("Reset canvas", "Reset drawing canvas", 220.0, move |_| reset_canvas(reset)))
        .child(action_button("Unlock all", "Unlock all drawing elements", 220.0, move |_| unlock_all(unlock)))
        .child(icon_button(X_ICON, "Close Excalidraw menu", 30.0, Color::from_rgb(235, 235, 240), Color::TRANSPARENT, move |_| open.set(false)))
        .into_element()
}

fn library_panel(mut open: State<bool>) -> Element {
    rect()
        .position(Position::new_absolute().right(0.0).top(0.0).bottom(0.0))
        .width(Size::px(320.0))
        .padding(Gaps::new_all(18.0))
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .a11y_alt("Excalidraw Library")
        .child(label().font_size(22.0).color(Color::from_rgb(170, 160, 255)).text("Library"))
        .child(label().font_size(14.0).color(Color::from_rgb(190, 190, 198)).text("No library items in this vault."))
        .child(icon_button(X_ICON, "Close library", 34.0, Color::from_rgb(235, 235, 240), Color::TRANSPARENT, move |_| open.set(false)))
        .into_element()
}

fn zoom_controls(canvas: State<DrawingCanvasState>) -> Element {
    let zoom = (canvas.read().viewport.zoom * 100.0).round() as i32;
    let mut out = canvas;
    let mut reset = canvas;
    let mut input = canvas;
    rect()
        .position(Position::new_absolute().left(16.0).bottom(16.0))
        .height(Size::px(42.0))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .padding(Gaps::new(4.0, 6.0, 4.0, 6.0))
        .background(Color::from_rgb(35, 35, 40))
        .with_corner_radius(9.0)
        .a11y_alt("Drawing zoom")
        .child(action_button("−", "Zoom out", 34.0, move |_| out.write().zoom_out()))
        .child(action_button(format!("{zoom}%"), "Reset zoom", 58.0, move |_| reset.write().reset_zoom()))
        .child(action_button("+", "Zoom in", 34.0, move |_| input.write().zoom_in()))
        .into_element()
}

fn history_controls(canvas: State<DrawingCanvasState>) -> Element {
    let mut undo = canvas;
    let mut redo = canvas;
    rect()
        .position(Position::new_absolute().left(176.0).bottom(16.0))
        .height(Size::px(42.0))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .padding(Gaps::new(4.0, 6.0, 4.0, 6.0))
        .background(Color::from_rgb(35, 35, 40))
        .with_corner_radius(9.0)
        .a11y_alt("Drawing history")
        .child(action_button("↶", "Undo drawing", 38.0, move |_| { undo.write().undo(); }))
        .child(action_button("↷", "Redo drawing", 38.0, move |_| { redo.write().redo(); }))
        .into_element()
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
                    app_state: Value::Object(Map::new()),
                    files: Value::Object(Map::new()),
                    extra: Map::new(),
                })
            })
        });
        let toolbar_error = use_state(|| Option::<String>::None);
        let active_tool = use_state(|| DrawingTool::Select);
        let library_open = use_state(|| false);
        let menu_open = use_state(|| false);
        let mut keyboard_tool = active_tool;
        let mut keyboard_error = toolbar_error;
        let keyboard_state = state;
        let mut keyboard_canvas = canvas_state;

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(Color::from_rgb(238, 238, 238))
            .a11y_alt("Drawing editor")
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                let command = event.modifiers.contains(Modifiers::ctrl_or_meta());
                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("s")) {
                        keyboard_error.set(save(keyboard_state, keyboard_canvas).err());
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("z")) {
                        keyboard_canvas.write().undo();
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("y")) {
                        keyboard_canvas.write().redo();
                        event.stop_propagation();
                        return;
                    }
                }
                match &event.key {
                    Key::Named(NamedKey::Escape) => {
                        keyboard_error.set(close(keyboard_state, keyboard_canvas).err());
                        event.stop_propagation();
                    }
                    Key::Named(NamedKey::Backspace) | Key::Named(NamedKey::Delete) => {
                        keyboard_canvas.write().delete_selection();
                        event.stop_propagation();
                    }
                    Key::Character(value) => {
                        if let Some(tool) = DrawingTool::ALL
                            .into_iter()
                            .find(|tool| tool.shortcut().eq_ignore_ascii_case(value))
                        {
                            keyboard_tool.set(tool);
                            keyboard_canvas.write().set_active_tool_label(tool.label());
                            event.stop_propagation();
                        }
                    }
                    _ => {}
                }
            })
            .child(canvas::drawing_canvas_with_state_and_palette(canvas_state, false))
            .child(
                rect()
                    .position(Position::new_absolute().left(0.0).right(0.0).top(16.0))
                    .width(Size::fill())
                    .height(Size::px(54.0))
                    .horizontal()
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .height(Size::px(54.0))
                            .padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
                            .horizontal()
                            .spacing(5.0)
                            .background(Color::from_rgb(35, 35, 40))
                            .with_corner_radius(10.0)
                            .a11y_alt("Drawing tools")
                            .child(rect().width(Size::px(1.0)).height(Size::px(1.0)).a11y_alt(format!("Drawing toolbar active tool: {}", active_tool.read().label())))
                            .child({
                                let mut lock_canvas = canvas_state;
                                icon_button(LOCK_ICON, "Lock selected element or unlock all", 38.0, Color::from_rgb(235, 235, 240), Color::TRANSPARENT, move |_| {
                                    if lock_canvas.read().selected_element_id().is_some() {
                                        lock_canvas.write().set_selection_locked(true);
                                    } else {
                                        unlock_all(lock_canvas);
                                    }
                                })
                            })
                            .child(rect().width(Size::px(1.0)).height(Size::px(30.0)).background(Color::from_rgb(75, 75, 82)))
                            .children(DrawingTool::ALL.into_iter().map(|tool| tool_button(tool, *active_tool.read(), active_tool, canvas_state)).collect::<Vec<_>>())
                            .child(rect().width(Size::px(1.0)).height(Size::px(30.0)).background(Color::from_rgb(75, 75, 82)))
                            .child(icon_button(SHAPES_ICON, "More drawing tools", 38.0, Color::from_rgb(235, 235, 240), Color::TRANSPARENT, |_| {})),
                    ),
            )
            .maybe_child(canvas_state.read().selected_element_id().is_some().then(|| properties_panel(canvas_state)))
            .child(
                rect().position(Position::new_absolute().left(16.0).top(16.0)).child(
                    icon_button(MENU_ICON, "Open Excalidraw menu", 40.0, Color::from_rgb(235, 235, 240), Color::from_rgb(35, 35, 40), {
                        let mut menu = menu_open;
                        move |_| menu.set(!*menu.read())
                    }),
                ),
            )
            .child(
                rect()
                    .position(Position::new_absolute().right(16.0).top(16.0))
                    .height(Size::px(40.0))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(8.0)
                    .child({
                        let mut error = toolbar_error;
                        icon_button(X_ICON, "Close drawing", 40.0, Color::from_rgb(235, 235, 240), Color::from_rgb(65, 65, 70), move |_| error.set(close(state, canvas_state).err()))
                    })
                    .child({
                        let mut error = toolbar_error;
                        icon_button(CHECK_ICON, "Save drawing", 40.0, Color::from_rgb(235, 235, 240), Color::from_rgb(65, 65, 70), move |_| error.set(save(state, canvas_state).err()))
                    })
                    .child(
                        rect()
                            .height(Size::px(40.0))
                            .padding(Gaps::new(0.0, 16.0, 0.0, 16.0))
                            .center()
                            .background(Color::from_rgb(35, 35, 40))
                            .with_corner_radius(9.0)
                            .a11y_alt("Open drawing library")
                            .on_mouse_up({
                                let mut library = library_open;
                                move |_| library.set(!*library.read())
                            })
                            .child(svg_icon(Icon::BookOpen, Color::from_rgb(235, 235, 240), 19.0))
                            .child(label().color(Color::from_rgb(235, 235, 240)).text("Library")),
                    ),
            )
            .child(
                rect()
                    .position(Position::new_absolute().left(0.0).top(100.0))
                    .width(Size::px(10.0))
                    .height(Size::px(10.0))
                    .background(Color::TRANSPARENT)
                    .a11y_alt("Back to library")
                    .on_press({
                        let mut error = toolbar_error;
                        move |_| error.set(close(state, canvas_state).err())
                    }),
            )
            .child(zoom_controls(canvas_state))
            .child(history_controls(canvas_state))
            .child(
                rect().position(Position::new_absolute().right(16.0).bottom(16.0)).child(
                    icon_button(HELP_ICON, "Open drawing help", 40.0, Color::from_rgb(235, 235, 240), Color::from_rgb(35, 35, 40), |_| {}),
                ),
            )
            .maybe_child((*menu_open.read()).then(|| menu_panel(menu_open, canvas_state)))
            .maybe_child((*library_open.read()).then(|| library_panel(library_open)))
            .maybe_child(
                toolbar_error
                    .read()
                    .as_deref()
                    .or(state.read().error.as_deref())
                    .map(drawing_error_notice),
            )
            .into_element()
    }
}

pub(super) fn drawing_view(state: State<ShellState>) -> Element {
    DrawingView { state }.into_element()
}
