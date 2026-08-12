//! Native Freya Excalidraw scene canvas.
//!
//! The scene is the real persisted Excalidraw JSON. Freya owns the visible
//! primitives and pointer state; no WebView, React renderer, or fake success
//! state is involved.

#[path = "drawing_render.rs"]
mod drawing_render;
#[path = "drawing_scene.rs"]
mod drawing_scene;

use freya::prelude::*;

pub use drawing_scene::{DrawingCanvasState, DrawingTool};

pub fn drawing_canvas_with_state(state: State<DrawingCanvasState>) -> Element {
    let snapshot = state.read().clone();
    let mut pointer_state = state;
    let mut move_state = state;
    let mut end_state = state;
    let mut wheel_state = state;
    let primitives = drawing_render::render(&snapshot);
    let background = color(snapshot.canvas_background());

    rect()
        .key(("native-excalidraw-canvas", snapshot.revision))
        .expanded()
        .background(background)
        .overflow(Overflow::Clip)
        .a11y_alt(format!("Drawing canvas · {}", snapshot.active_tool().label()))
        .on_mouse_down(move |event: Event<MouseEventData>| {
            if event.button == Some(MouseButton::Left) {
                pointer_state
                    .write()
                    .begin_pointer(point(event.element_location));
                event.stop_propagation();
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                move_state
                    .write()
                    .move_pointer(point(event.element_location()));
                event.stop_propagation();
            }
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                end_state.write().end_pointer();
            }
        })
        .on_wheel(move |event: Event<WheelEventData>| {
            wheel_state
                .write()
                .zoom_at(point(event.element_location), event.delta_y);
            event.stop_propagation();
        })
        .children(primitives)
        .into_element()
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}

fn color(value: &str) -> Color {
    let value = value.trim().trim_start_matches('#');
    if value.len() == 6 {
        let r = u8::from_str_radix(&value[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&value[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&value[4..6], 16).unwrap_or(255);
        Color::from_rgb(r, g, b)
    } else {
        Color::WHITE
    }
}
