//! Native Freya Excalidraw scene canvas.
//!
//! The scene is the real persisted Excalidraw JSON. Freya owns the visible
//! primitives and pointer state; no WebView, React renderer, or fake success
//! state is involved.

#[path = "drawing_image.rs"]
mod drawing_image;
mod drawing_render;
mod drawing_scene;
#[path = "drawing_text_edit.rs"]
mod drawing_text_edit;

use freya::prelude::*;
use serde_json::{json, Map, Value};

pub use drawing_scene::{DrawingCanvasState, DrawingElement, DrawingScene};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingTool {
    Selection,
    Hand,
    Freehand,
    Rectangle,
    Diamond,
    Ellipse,
    Arrow,
    Line,
    Text,
    Image,
    Eraser,
    Frame,
}

impl DrawingTool {
    fn from_id(id: &str) -> Self {
        match id {
            "freehand" | "freedraw" | "pencil" => Self::Freehand,
            "rectangle" => Self::Rectangle,
            "diamond" => Self::Diamond,
            "ellipse" => Self::Ellipse,
            "arrow" => Self::Arrow,
            "line" => Self::Line,
            "text" => Self::Text,
            "image" => Self::Image,
            "eraser" => Self::Eraser,
            "hand" => Self::Hand,
            "frame" => Self::Frame,
            _ => Self::Selection,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::Hand => "hand",
            Self::Freehand => "freehand",
            Self::Rectangle => "rectangle",
            Self::Diamond => "diamond",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Line => "line",
            Self::Text => "text",
            Self::Image => "image",
            Self::Eraser => "eraser",
            Self::Frame => "frame",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Selection => "Selection",
            Self::Hand => "Hand",
            Self::Freehand => "Freehand",
            Self::Rectangle => "Rectangle",
            Self::Diamond => "Diamond",
            Self::Ellipse => "Ellipse",
            Self::Arrow => "Arrow",
            Self::Line => "Line",
            Self::Text => "Text",
            Self::Image => "Image",
            Self::Eraser => "Eraser",
            Self::Frame => "Frame",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Gesture {
    tool: DrawingTool,
    index: usize,
    start: [f32; 2],
}

pub fn drawing_canvas() -> Element {
    let state = use_consume::<State<DrawingCanvasState>>();
    drawing_canvas_with_state_and_palette(state, true)
}

pub fn drawing_canvas_with_state(state: State<DrawingCanvasState>) -> Element {
    drawing_canvas_with_state_and_palette(state, false)
}

pub fn drawing_canvas_with_state_and_palette(
    state: State<DrawingCanvasState>,
    show_palette: bool,
) -> Element {
    let snapshot = state.read().clone();
    let gesture_state = use_state(|| Option::<Gesture>::None);
    let pointer_down_state = use_state(|| false);
    let editing_text_state = use_state(|| Option::<usize>::None);
    let last_active_tool = use_state(|| snapshot.active_tool.clone());
    let rendered_active_tool = snapshot.active_tool.clone();
    let mut reset_gesture = gesture_state;
    let mut last_tool_state = last_active_tool;
    use_side_effect(move || {
        if last_tool_state.read().as_str() != rendered_active_tool.as_str() {
            last_tool_state.set(rendered_active_tool.clone());
            reset_gesture.set(None);
        }
    });
    let mut pointer_state = state;
    let mut move_state = state;
    let mut wheel_state = state;
    let mut pointer_gesture = gesture_state;
    let move_gesture = gesture_state;
    let mut local_end_state = state;
    let mut local_end_gesture = gesture_state;
    let mut pointer_down = pointer_down_state;
    let move_pointer_down = pointer_down_state;
    let mut release_pointer_down = pointer_down_state;
    let mut pointer_editing = editing_text_state;
    let primitives = drawing_render::render(&snapshot);
    let editing_index = *editing_text_state.read();

    rect()
        .key(("native-excalidraw-canvas", snapshot.revision))
        .position(
            Position::new_absolute()
                .left(0.)
                .right(0.)
                .top(0.)
                .bottom(0.),
        )
        .width(Size::fill())
        .height(Size::fill())
        .background(Color::from_rgb(238, 238, 238))
        .overflow(Overflow::Clip)
        .a11y_alt("DrawingCanvas")
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.button() != Some(MouseButton::Left) || !event.is_primary() {
                return;
            }
            pointer_down.set(true);
            let screen = point(event.global_location());
            let tool = DrawingTool::from_id(pointer_state.read().active_tool.as_str());
            if tool != DrawingTool::Text && pointer_editing.read().is_some() {
                pointer_editing.set(None);
            }
            match tool {
                DrawingTool::Selection | DrawingTool::Hand | DrawingTool::Eraser => {
                    pointer_state.write().begin_pointer(screen);
                    pointer_gesture.set(None);
                }
                DrawingTool::Image => {
                    pointer_gesture.set(None);
                    pointer_down.set(false);
                    match drawing_image::pick_image() {
                        Ok(Some(asset)) => {
                            let mut canvas = pointer_state.write();
                            let world = canvas.to_world(screen);
                            insert_image(&mut canvas, world, asset);
                            canvas.set_active_tool_label("Selection");
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("[freya][drawing] action:failure action=insert-image error={error}");
                        }
                    }
                }
                DrawingTool::Text => {
                    pointer_gesture.set(None);
                    pointer_down.set(false);
                    let mut canvas = pointer_state.write();
                    let world = canvas.to_world(screen);
                    let existing = canvas
                        .document
                        .elements
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(index, element)| {
                            (!element.is_deleted
                                && element.kind == "text"
                                && !element.is_locked()
                                && element.hit_test(world))
                            .then_some(index)
                        });
                    let index = if let Some(index) = existing {
                        canvas.checkpoint();
                        index
                    } else {
                        let index = canvas.document.elements.len();
                        canvas.checkpoint();
                        let mut element = new_element(DrawingTool::Text, world, index);
                        element.text.clear();
                        element.extra.insert("originalText".to_owned(), Value::String(String::new()));
                        element.width = element.font_size.max(20.0);
                        element.height = element.font_size.max(20.0) * 1.25;
                        canvas.document.elements.push(element);
                        canvas.touch_element(index);
                        index
                    };
                    drop(canvas);
                    pointer_editing.set(Some(index));
                }
                _ => {
                    let mut canvas = pointer_state.write();
                    let world = canvas.to_world(screen);
                    let index = canvas.document.elements.len();
                    canvas.checkpoint();
                    canvas
                        .document
                        .elements
                        .push(new_element(tool, world, index));
                    canvas.revision = canvas.revision.wrapping_add(1);
                    drop(canvas);
                    pointer_gesture.set(Some(Gesture {
                        tool,
                        index,
                        start: world,
                    }));
                }
            }
            event.stop_propagation();
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if !*move_pointer_down.read() {
                return;
            }
            let location = point(event.global_location());
            let active_tool = DrawingTool::from_id(move_state.read().active_tool.as_str());
            if let Some(gesture) = *move_gesture.read() {
                draw_gesture(&mut move_state.write(), gesture, location);
            } else if matches!(
                active_tool,
                DrawingTool::Selection | DrawingTool::Hand | DrawingTool::Eraser
            ) {
                move_state.write().move_pointer(location);
            }
            event.stop_propagation();
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
            if !event.is_primary() {
                return;
            }
            release_pointer_down.set(false);
            local_end_state.write().end_pointer();
            local_end_gesture.set(None);
            event.stop_propagation();
        })
        .on_wheel(move |event: Event<WheelEventData>| {
            wheel_state
                .write()
                .zoom_at(point(event.global_location), event.delta_y);
            event.stop_propagation();
        })
        .children(primitives)
        .maybe_child(editing_index.map(|index| {
            drawing_text_edit::TextEditOverlay {
                canvas: state,
                editing: editing_text_state,
                index,
            }
            .into()
        }))
        .maybe_child(show_palette.then(|| tool_palette(state)))
        .into_element()
}

fn tool_palette(state: State<DrawingCanvasState>) -> Element {
    rect()
        .position(Position::new_absolute().left(8.).top(8.))
        .horizontal()
        .spacing(4.)
        .padding(Gaps::new(4., 6., 4., 6.))
        .background(Color::from_argb(220, 248, 250, 252))
        .with_corner_radius(8.)
        .a11y_alt(format!(
            "Active drawing tool: {}",
            DrawingTool::from_id(state.read().active_tool.as_str()).label()
        ))
        .child(tool_button(state, DrawingTool::Hand))
        .child(tool_button(state, DrawingTool::Selection))
        .child(tool_button(state, DrawingTool::Freehand))
        .child(tool_button(state, DrawingTool::Rectangle))
        .child(tool_button(state, DrawingTool::Diamond))
        .child(tool_button(state, DrawingTool::Ellipse))
        .child(tool_button(state, DrawingTool::Arrow))
        .child(tool_button(state, DrawingTool::Line))
        .child(tool_button(state, DrawingTool::Text))
        .child(tool_button(state, DrawingTool::Image))
        .child(tool_button(state, DrawingTool::Frame))
        .child(tool_button(state, DrawingTool::Eraser))
        .into_element()
}

fn tool_button(mut state: State<DrawingCanvasState>, tool: DrawingTool) -> Element {
    rect()
        .height(Size::px(26.))
        .padding(Gaps::new(0., 8., 0., 8.))
        .center()
        .with_corner_radius(6.)
        .a11y_alt(format!("{} tool", tool.label()))
        .on_mouse_up(move |event: Event<MouseEventData>| {
            let mut canvas = state.write();
            if canvas.active_tool != tool.id() {
                canvas.active_tool = tool.id().to_owned();
                canvas.revision = canvas.revision.wrapping_add(1);
            }
            event.stop_propagation();
        })
        .child(label().font_size(11.).text(tool.label()))
        .into_element()
}

fn new_element(tool: DrawingTool, start: [f32; 2], index: usize) -> DrawingElement {
    let core_tool = match tool {
        DrawingTool::Selection => elephant_draw::DrawingTool::Selection,
        DrawingTool::Hand => elephant_draw::DrawingTool::Hand,
        DrawingTool::Freehand => elephant_draw::DrawingTool::Freehand,
        DrawingTool::Rectangle => elephant_draw::DrawingTool::Rectangle,
        DrawingTool::Diamond => elephant_draw::DrawingTool::Diamond,
        DrawingTool::Ellipse => elephant_draw::DrawingTool::Ellipse,
        DrawingTool::Arrow => elephant_draw::DrawingTool::Arrow,
        DrawingTool::Line => elephant_draw::DrawingTool::Line,
        DrawingTool::Text => elephant_draw::DrawingTool::Text,
        DrawingTool::Image => elephant_draw::DrawingTool::Image,
        DrawingTool::Eraser => elephant_draw::DrawingTool::Eraser,
        DrawingTool::Frame => elephant_draw::DrawingTool::Frame,
    };
    elephant_draw::create_element(core_tool, start, format!("freya-element-{index}"))
}

fn insert_image(
    canvas: &mut DrawingCanvasState,
    world: [f32; 2],
    asset: drawing_image::DrawingImageAsset,
) {
    canvas.checkpoint();
    let index = canvas.document.elements.len();
    let mut element = elephant_draw::create_element(
        elephant_draw::DrawingTool::Image,
        world,
        format!("freya-element-{index}"),
    );
    element.width = asset.width;
    element.height = asset.height;
    element
        .extra
        .insert("fileId".to_owned(), Value::String(asset.file_id.clone()));
    element
        .extra
        .insert("status".to_owned(), Value::String("saved".to_owned()));
    element.extra.insert("scale".to_owned(), json!([1, 1]));
    element.extra.insert("crop".to_owned(), Value::Null);

    if !canvas.document.files.is_object() {
        canvas.document.files = Value::Object(Map::new());
    }
    if let Some(files) = canvas.document.files.as_object_mut() {
        files.insert(
            asset.file_id.clone(),
            json!({
                "id": asset.file_id,
                "mimeType": asset.mime_type,
                "dataURL": asset.data_url,
                "created": asset.created
            }),
        );
    }
    canvas.document.elements.push(element);
    canvas.touch_element(index);
}

fn draw_gesture(canvas: &mut DrawingCanvasState, gesture: Gesture, point: [f32; 2]) {
    let world = canvas.to_world(point);
    {
        let Some(element) = canvas.document.elements.get_mut(gesture.index) else {
            return;
        };
        match gesture.tool {
            DrawingTool::Freehand => {
                element
                    .points
                    .push([world[0] - gesture.start[0], world[1] - gesture.start[1]]);
                update_linear_dimensions(element);
            }
            DrawingTool::Rectangle
            | DrawingTool::Diamond
            | DrawingTool::Ellipse
            | DrawingTool::Frame => {
                element.x = gesture.start[0].min(world[0]);
                element.y = gesture.start[1].min(world[1]);
                element.width = (world[0] - gesture.start[0]).abs();
                element.height = (world[1] - gesture.start[1]).abs();
            }
            DrawingTool::Arrow | DrawingTool::Line => {
                element.x = gesture.start[0];
                element.y = gesture.start[1];
                element.points = vec![
                    [0.0, 0.0],
                    [world[0] - gesture.start[0], world[1] - gesture.start[1]],
                ];
                update_linear_dimensions(element);
            }
            _ => {}
        }
    }
    canvas.touch_element(gesture.index);
}

fn update_linear_dimensions(element: &mut DrawingElement) {
    if element.points.is_empty() {
        return;
    }
    let mut min_x = 0.0_f32;
    let mut min_y = 0.0_f32;
    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    for [x, y] in &element.points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    element.width = max_x - min_x;
    element.height = max_y - min_y;
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}
