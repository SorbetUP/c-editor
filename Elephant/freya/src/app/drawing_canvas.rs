//! Native Freya Excalidraw scene canvas.
//!
//! The scene is the real persisted Excalidraw JSON. Freya owns the visible
//! primitives and pointer state; no WebView, React renderer, or fake success
//! state is involved.

mod drawing_render;
mod drawing_scene;

use freya::prelude::*;

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
    Eraser,
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
            "eraser" => Self::Eraser,
            "hand" => Self::Hand,
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
            Self::Eraser => "eraser",
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
            Self::Eraser => "Eraser",
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
    let primitives = drawing_render::render(&snapshot);

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
            if event.button() == Some(MouseButton::Left) && event.is_primary() {
                pointer_down.set(true);
                let point = point(event.global_location());
                let tool = DrawingTool::from_id(pointer_state.read().active_tool.as_str());
                if tool == DrawingTool::Selection {
                    pointer_state.write().begin_pointer(point);
                    pointer_gesture.set(None);
                } else if tool == DrawingTool::Hand {
                    pointer_state.write().begin_pointer(point);
                    pointer_gesture.set(None);
                } else if tool == DrawingTool::Eraser {
                    pointer_state.write().erase_at(point);
                    pointer_gesture.set(None);
                } else {
                    let mut canvas = pointer_state.write();
                    let world = canvas.to_world(point);
                    let index = canvas.document.elements.len();
                    canvas
                        .document
                        .elements
                        .push(new_element(tool, world, index));
                    canvas.begin_pointer(point);
                    canvas.revision = canvas.revision.wrapping_add(1);
                    drop(canvas);
                    pointer_gesture.set(Some(Gesture {
                        tool,
                        index,
                        start: world,
                    }));
                }
                event.stop_propagation();
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if !*move_pointer_down.read() {
                return;
            }
            let location = point(event.global_location());
            let active_tool = DrawingTool::from_id(move_state.read().active_tool.as_str());
            if active_tool == DrawingTool::Eraser {
                move_state.write().erase_at(location);
            } else if let Some(gesture) = *move_gesture.read() {
                draw_gesture(&mut move_state.write(), gesture, location);
            } else {
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
        .child(tool_button(state, DrawingTool::Selection))
        .child(tool_button(state, DrawingTool::Freehand))
        .child(tool_button(state, DrawingTool::Rectangle))
        .child(tool_button(state, DrawingTool::Diamond))
        .child(tool_button(state, DrawingTool::Ellipse))
        .child(tool_button(state, DrawingTool::Arrow))
        .child(tool_button(state, DrawingTool::Line))
        .child(tool_button(state, DrawingTool::Text))
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
    DrawingElement {
        id: format!("freya-element-{index}"),
        kind: match tool {
            DrawingTool::Freehand => "freedraw".to_owned(),
            DrawingTool::Rectangle => "rectangle".to_owned(),
            DrawingTool::Diamond => "diamond".to_owned(),
            DrawingTool::Ellipse => "ellipse".to_owned(),
            DrawingTool::Arrow => "arrow".to_owned(),
            DrawingTool::Line => "line".to_owned(),
            DrawingTool::Text => "text".to_owned(),
            DrawingTool::Selection => "rectangle".to_owned(),
            _ => "rectangle".to_owned(),
        },
        x: start[0],
        y: start[1],
        width: 1.,
        height: 1.,
        points: if tool == DrawingTool::Freehand {
            vec![[0., 0.]]
        } else if tool == DrawingTool::Arrow || tool == DrawingTool::Line {
            vec![[0., 0.], [0., 0.]]
        } else {
            Vec::new()
        },
        text: if tool == DrawingTool::Text {
            "Text".to_owned()
        } else {
            String::new()
        },
        stroke_color: "#000000".to_owned(),
        background_color: "transparent".to_owned(),
        stroke_width: 2.,
        stroke_style: "solid".to_owned(),
        fill_style: "solid".to_owned(),
        opacity: 100.,
        angle: 0.,
        font_size: 20.,
        end_arrowhead: if tool == DrawingTool::Arrow {
            Some("arrow".to_owned())
        } else {
            None
        },
        start_arrowhead: None,
        is_deleted: false,
        extra: Default::default(),
    }
}

fn draw_gesture(canvas: &mut DrawingCanvasState, gesture: Gesture, point: [f32; 2]) {
    let world = canvas.to_world(point);
    let Some(element) = canvas.document.elements.get_mut(gesture.index) else {
        return;
    };
    match gesture.tool {
        DrawingTool::Freehand => {
            element
                .points
                .push([world[0] - gesture.start[0], world[1] - gesture.start[1]]);
            element.width = (world[0] - gesture.start[0]).abs();
            element.height = (world[1] - gesture.start[1]).abs();
        }
        DrawingTool::Rectangle | DrawingTool::Diamond | DrawingTool::Ellipse => {
            element.x = gesture.start[0].min(world[0]);
            element.y = gesture.start[1].min(world[1]);
            element.width = (world[0] - gesture.start[0]).abs();
            element.height = (world[1] - gesture.start[1]).abs();
        }
        DrawingTool::Arrow | DrawingTool::Line => {
            element.x = gesture.start[0];
            element.y = gesture.start[1];
            element.points = vec![
                [0., 0.],
                [world[0] - gesture.start[0], world[1] - gesture.start[1]],
            ];
            element.width = (world[0] - gesture.start[0]).abs();
            element.height = (world[1] - gesture.start[1]).abs();
        }
        _ => {}
    }
    canvas.revision = canvas.revision.wrapping_add(1);
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}
