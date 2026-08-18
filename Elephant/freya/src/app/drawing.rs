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

use super::{
    navigation_icons::{svg_icon, Icon},
    ShellState,
};
use crate::theme;
use freya::prelude::*;

#[path = "drawing_storage.rs"]
mod storage;
#[path = "drawing_icons.rs"]
mod drawing_icons;
use drawing_icons::{CHECK_ICON, HELP_ICON, LOCK_ICON, MENU_ICON, SHAPES_ICON, X_ICON};

// The native renderer is compiled into the production Freya crate here. The
// shell owns the route switch while this module owns drawing state and actions.
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
            Self::Ellipse => "4",
            Self::Diamond => "3",
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
            let mut shell = state.write();
            shell.drawing = Some(canvas);
            shell.error = None;
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

pub(super) fn close(
    mut state: State<ShellState>,
    canvas_state: State<DrawingCanvasState>,
) -> Result<(), String> {
    if let Err(error) = save(state, canvas_state) {
        eprintln!("[freya][drawing] action:close blocked reason=save-failure");
        return Err(error);
    }
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
        .child(
            label()
                .color(theme::color(theme::DANGER))
                .text(error.to_owned()),
        )
        .into_element()
}

fn drawing_tool_button(
    tool: DrawingTool,
    active: DrawingTool,
    mut selected: State<DrawingTool>,
    mut canvas_state: State<DrawingCanvasState>,
) -> Element {
    let button_label = format!("{} tool ({})", tool.label(), tool.shortcut());
    let is_active = tool == active;
    let icon_color = if is_active {
        Color::WHITE
    } else {
        Color::from_rgb(215, 215, 220)
    };
    rect()
        .width(Size::px(38.))
        .height(Size::px(38.))
        .center()
        .background(if is_active {
            theme::color(theme::PRIMARY)
        } else {
            Color::TRANSPARENT
        })
        .with_corner_radius(7.)
        .on_mouse_up(move |_| {
            canvas_state.write().set_active_tool_label(tool.label());
            selected.set(tool);
        })
        .a11y_alt(button_label)
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(2.)
                .child(
                    SvgViewer::new(tool.icon_source())
                        .width(Size::px(18.))
                        .height(Size::px(18.))
                        .show_loader(false)
                        .color(icon_color)
                        .stroke(icon_color)
                        .stroke_width(2.),
                )
                .child(
                    label()
                        .font_size(9.)
                        .color(if is_active { Color::WHITE } else { Color::from_rgb(150, 150, 155) })
                        .text(tool.shortcut()),
                ),
        )
        .into_element()
}

fn drawing_icon_button(
    source: &'static [u8],
    label_text: &'static str,
    size: f32,
    color: Color,
    background: Color,
    mut on_press: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::px(size))
        .height(Size::px(size))
        .center()
        .background(background)
        .with_corner_radius(if matches!(label_text, "Close drawing" | "Cancel drawing" | "Save drawing") {
            20.
        } else {
            9.
        })
        .a11y_alt(label_text)
        .on_mouse_up(move |event| on_press(event))
        .child(
            SvgViewer::new(source)
                .width(Size::px(20.))
                .height(Size::px(20.))
                .show_loader(false)
                .color(color)
                .stroke(color)
                .stroke_width(2.),
        )
        .into_element()
}

fn drawing_toolbar_button(
    tool: DrawingTool,
    active: DrawingTool,
    selected: State<DrawingTool>,
    canvas_state: State<DrawingCanvasState>,
) -> Element {
    drawing_tool_button(tool, active, selected, canvas_state)
}

fn drawing_properties_panel(palette: Color) -> Element {
    let panel = Color::from_rgb(35, 35, 42);
    let control = Color::from_rgb(48, 48, 58);
    let text_color = Color::from_rgb(235, 235, 240);
    let swatch = |color: Color, name: &'static str| {
        rect()
            .width(Size::px(28.))
            .height(Size::px(28.))
            .background(color)
            .border(Border::new().fill(Color::from_rgb(80, 80, 90)).width(1.))
            .with_corner_radius(5.)
            .a11y_alt(name)
            .into_element()
    };
    let choice = |caption: &'static str, name: &'static str| {
        rect()
            .width(Size::px(42.))
            .height(Size::px(36.))
            .center()
            .background(control)
            .with_corner_radius(9.)
            .a11y_alt(name)
            .child(label().font_size(13.).color(text_color).text(caption))
            .into_element()
    };
    rect()
        .position(Position::new_absolute().left(16.).top(72.))
        .width(Size::px(290.))
        .height(Size::px(468.))
        .padding(Gaps::new(18., 16., 18., 16.))
        .background(panel)
        .with_corner_radius(10.)
        .a11y_alt("Excalidraw properties")
        .child(label().font_size(16.).color(text_color).text("Stroke"))
        .child(
            rect()
                .height(Size::px(38.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .child(swatch(Color::from_rgb(224, 224, 224), "Stroke white"))
                .child(swatch(Color::from_rgb(255, 111, 117), "Stroke red"))
                .child(swatch(Color::from_rgb(48, 155, 73), "Stroke green"))
                .child(swatch(Color::from_rgb(78, 145, 216), "Stroke blue"))
                .child(swatch(Color::from_rgb(184, 101, 0), "Stroke orange")),
        )
        .child(label().font_size(16.).color(text_color).text("Background"))
        .child(
            rect()
                .height(Size::px(38.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .child(swatch(Color::TRANSPARENT, "Transparent background"))
                .child(swatch(Color::from_rgb(105, 50, 50), "Red background"))
                .child(swatch(Color::from_rgb(0, 88, 24), "Green background"))
                .child(swatch(Color::from_rgb(18, 75, 105), "Blue background"))
                .child(swatch(Color::from_rgb(68, 50, 0), "Orange background")),
        )
        .child(label().font_size(16.).color(text_color).text("Stroke width"))
        .child(
            rect()
                .height(Size::px(42.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(10.)
                .child(choice("—", "Thin stroke"))
                .child(choice("—", "Medium stroke"))
                .child(choice("━", "Wide stroke")),
        )
        .child(label().font_size(16.).color(text_color).text("Stroke style"))
        .child(
            rect()
                .height(Size::px(42.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(10.)
                .child(choice("—", "Solid stroke"))
                .child(choice("╌", "Dashed stroke"))
                .child(choice("┈", "Dotted stroke")),
        )
        .child(label().font_size(16.).color(text_color).text("Opacity"))
        .child(
            rect()
                .height(Size::px(18.))
                .width(Size::fill())
                .background(Color::from_rgb(30, 125, 245))
                .with_corner_radius(9.)
                .a11y_alt("Opacity 100 percent"),
        )
        .child(label().font_size(16.).color(text_color).text("Layers"))
        .child(
            rect()
                .height(Size::px(42.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(10.)
                .child(choice("↗", "Bring to front"))
                .child(choice("□", "Bring forward"))
                .child(choice("↙", "Send backward"))
                .child(choice("■", "Send to back")),
        )
        .child(rect().width(Size::px(1.)).height(Size::px(1.)).background(palette))
        .into_element()
}

fn drawing_library_panel(mut open: State<bool>) -> Element {
    rect()
        .position(Position::new_absolute().right(0.).top(0.).bottom(0.))
        .width(Size::px(340.))
        .height(Size::fill())
        .padding(Gaps::new(22., 18., 22., 18.))
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .a11y_alt("Excalidraw Library")
        .child(
            rect()
                .height(Size::px(42.))
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(label().font_size(22.).color(Color::from_rgb(170, 160, 255)).text("Library"))
                .child(drawing_icon_button(
                    X_ICON,
                    "Close library",
                    36.,
                    Color::from_rgb(235, 235, 240),
                    Color::TRANSPARENT,
                    move |_| open.set(false),
                )),
        )
        .child(
            rect()
                .height(Size::px(1.))
                .width(Size::fill())
                .background(Color::from_rgb(70, 70, 78)),
        )
        .child(
            rect()
                .height(Size::fill())
                .center()
                .a11y_alt("Empty drawing library")
                .child(label().font_size(18.).color(Color::from_rgb(180, 170, 255)).text("No items added yet..."))
                .child(label().font_size(14.).color(Color::from_rgb(190, 190, 198)).text("Select an item on the canvas to add it here."))
                .child(
                    rect()
                        .height(Size::px(42.))
                        .width(Size::fill())
                        .center()
                        .background(Color::from_rgb(170, 160, 255))
                        .with_corner_radius(8.)
                        .a11y_alt("Browse drawing libraries")
                        .child(label().color(Color::from_rgb(25, 25, 30)).text("Browse libraries")),
                ),
        )
        .into_element()
}

fn drawing_menu_panel(mut open: State<bool>) -> Element {
    let item = |text: &'static str, alt: &'static str| {
        rect()
            .height(Size::px(40.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .padding(Gaps::new(0., 10., 0., 10.))
            .a11y_alt(alt)
            .child(label().color(Color::from_rgb(235, 235, 240)).text(text))
            .into_element()
    };
    rect()
        .position(Position::new_absolute().left(16.).top(64.))
        .width(Size::px(300.))
        .height(Size::px(300.))
        .padding(Gaps::new(10., 8., 10., 8.))
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .with_corner_radius(9.)
        .a11y_alt("Excalidraw menu")
        .child(item("Export image…", "Export drawing image"))
        .child(item("Help", "Open drawing help"))
        .child(item("Reset the canvas", "Reset drawing canvas"))
        .child(
            rect()
                .height(Size::px(1.))
                .width(Size::fill())
                .background(Color::from_rgb(70, 70, 78)),
        )
        .child(item("GitHub", "Open Excalidraw GitHub"))
        .child(item("Discord", "Open Excalidraw Discord"))
        .child(drawing_icon_button(
            X_ICON,
            "Close Excalidraw menu",
            30.,
            Color::from_rgb(235, 235, 240),
            Color::TRANSPARENT,
            move |_| open.set(false),
        ))
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
                    app_state: serde_json::Value::Object(Default::default()),
                    files: serde_json::Value::Object(Default::default()),
                    extra: Default::default(),
                })
            })
        });
        let toolbar_error = use_state(|| Option::<String>::None);
        let save_state = state;
        let close_state = state;
        let save_canvas = canvas_state;
        let close_canvas = canvas_state;
        let active_tool = use_state(|| DrawingTool::Select);
        let library_open = use_state(|| false);
        let menu_open = use_state(|| false);
        let mut keyboard_tool = active_tool;
        let mut save_error = toolbar_error;
        let mut close_error = toolbar_error;
        let mut keyboard_error = toolbar_error;
        let keyboard_state = state;
        let mut keyboard_canvas = canvas_state;

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(Color::from_rgb(238, 238, 238))
            .a11y_alt("Drawing editor")
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if event.modifiers.contains(Modifiers::ctrl_or_meta())
                    && matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("s"))
                {
                    keyboard_error.set(save(keyboard_state, keyboard_canvas).err());
                    event.stop_propagation();
                    return;
                }
                match &event.key {
                    Key::Named(NamedKey::Escape) => {
                        keyboard_error.set(close(keyboard_state, keyboard_canvas).err());
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
                    .position(Position::new_absolute().left(0.).right(0.).top(16.))
                    .width(Size::fill())
                    .height(Size::px(54.))
                    .horizontal()
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .width(Size::auto())
                            .height(Size::px(54.))
                            .padding(Gaps::new(8., 12., 8., 12.))
                            .horizontal()
                            .spacing(5.)
                            .background(Color::from_rgb(35, 35, 40))
                            .with_corner_radius(10.)
                            .a11y_alt("Drawing tools")
                            .child(
                                rect()
                                    .width(Size::px(1.))
                                    .height(Size::px(1.))
                                    .a11y_alt(format!(
                                        "Drawing toolbar active tool: {}",
                                        active_tool.read().label()
                                    )),
                            )
                            .child(drawing_icon_button(
                                LOCK_ICON,
                                "Lock toolbar",
                                38.,
                                Color::from_rgb(235, 235, 240),
                                Color::TRANSPARENT,
                                |_| {},
                            ))
                            .child(
                                rect()
                                    .width(Size::px(1.))
                                    .height(Size::px(30.))
                                    .background(Color::from_rgb(75, 75, 82)),
                            )
                            .children(
                                DrawingTool::ALL
                                    .into_iter()
                                    .map(|tool| {
                                        drawing_toolbar_button(
                                            tool,
                                            *active_tool.read(),
                                            active_tool,
                                            canvas_state,
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .child(
                                rect()
                                    .width(Size::px(1.))
                                    .height(Size::px(30.))
                                    .background(Color::from_rgb(75, 75, 82)),
                            )
                            .child(drawing_icon_button(
                                SHAPES_ICON,
                                "More drawing tools",
                                38.,
                                Color::from_rgb(235, 235, 240),
                                Color::TRANSPARENT,
                                |_| {},
                            )),
                    ),
            )
            .maybe_child((!matches!(
                *active_tool.read(),
                DrawingTool::Select | DrawingTool::Hand
            ))
                .then(|| drawing_properties_panel(theme::color(theme::PRIMARY))))
            .child(
                rect()
                    .position(Position::new_absolute().left(16.).top(16.))
                    .child(drawing_icon_button(
                        MENU_ICON,
                        "Open Excalidraw menu",
                        40.,
                        Color::from_rgb(235, 235, 240),
                        Color::from_rgb(35, 35, 40),
                        {
                            let mut menu = menu_open;
                            move |_| {
                                let next = !*menu.read();
                                menu.set(next);
                            }
                        },
                    )),
            )
            .child(
                rect()
                    .position(Position::new_absolute().right(16.).top(16.))
                    .height(Size::px(40.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child({
                        let close_state = close_state;
                        let close_canvas = close_canvas;
                        drawing_icon_button(
                            X_ICON,
                            "Close drawing",
                            40.,
                            Color::from_rgb(235, 235, 240),
                            Color::from_rgb(65, 65, 70),
                            move |_| {
                                close_error.set(close(close_state, close_canvas).err());
                            },
                        )
                    })
                    .child({
                        let save_state = save_state;
                        let save_canvas = save_canvas;
                        drawing_icon_button(
                            CHECK_ICON,
                            "Save drawing",
                            40.,
                            Color::from_rgb(235, 235, 240),
                            Color::from_rgb(65, 65, 70),
                            move |_| {
                                save_error.set(save(save_state, save_canvas).err());
                            },
                        )
                    })
                    .child(
                        rect()
                            .height(Size::px(40.))
                            .padding(Gaps::new(0., 16., 0., 16.))
                            .center()
                            .background(Color::from_rgb(35, 35, 40))
                            .with_corner_radius(9.)
                            .a11y_alt("Open drawing library")
                            .on_mouse_up({
                                let mut library = library_open;
                                move |_| {
                                    let next = !*library.read();
                                    library.set(next);
                                }
                            })
                            .child(svg_icon(Icon::BookOpen, Color::from_rgb(235, 235, 240), 19.))
                            .child(label().color(Color::from_rgb(235, 235, 240)).text("Library")),
                    ),
            )
            .child(
                rect()
                    .position(Position::new_absolute().left(0.).top(100.))
                    .width(Size::px(10.))
                    .height(Size::px(10.))
                    .background(Color::TRANSPARENT)
                    .a11y_alt("Back to library")
                    .on_press(move |_| {
                        close_error.set(close(close_state, close_canvas).err());
                    }),
            )
            .child(
                rect()
                    .position(Position::new_absolute().left(16.).bottom(16.))
                    .height(Size::px(42.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(18.)
                    .padding(Gaps::new(0., 12., 0., 12.))
                    .background(Color::from_rgb(35, 35, 40))
                    .with_corner_radius(9.)
                    .a11y_alt("Drawing zoom")
                    .child(label().color(Color::from_rgb(235, 235, 240)).text("−"))
                    .child(label().color(Color::from_rgb(235, 235, 240)).text("100%"))
                    .child(label().color(Color::from_rgb(235, 235, 240)).text("+")),
            )
            .child(
                rect()
                    .position(Position::new_absolute().left(176.).bottom(16.))
                    .height(Size::px(42.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(18.)
                    .padding(Gaps::new(0., 12., 0., 12.))
                    .background(Color::from_rgb(35, 35, 40))
                    .with_corner_radius(9.)
                    .a11y_alt("Drawing history")
                    .child(label().color(Color::from_rgb(235, 235, 240)).text("↶"))
                    .child(label().color(Color::from_rgb(235, 235, 240)).text("↷")),
            )
            .child(
                rect()
                    .position(Position::new_absolute().right(16.).bottom(16.))
                    .child(drawing_icon_button(
                        HELP_ICON,
                        "Open drawing help",
                        40.,
                        Color::from_rgb(235, 235, 240),
                        Color::from_rgb(35, 35, 40),
                        |_| {},
                    )),
            )
            .maybe_child((*menu_open.read()).then(|| drawing_menu_panel(menu_open)))
            .maybe_child((*library_open.read()).then(|| drawing_library_panel(library_open)))
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
