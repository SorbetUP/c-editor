//! Native Freya Excalidraw scene canvas.
//!
//! The scene is the real persisted Excalidraw JSON. Freya owns the visible
//! primitives and pointer state; no WebView, React renderer, or fake success
//! state is involved.

mod drawing_render;
mod drawing_scene;

use freya::prelude::*;

pub use drawing_scene::{DrawingCanvasState, DrawingScene};

pub fn drawing_canvas() -> Element {
    let state = use_consume::<State<DrawingCanvasState>>();
    drawing_canvas_with_state(state)
}

pub fn drawing_canvas_with_state(state: State<DrawingCanvasState>) -> Element {
    let snapshot = state.read().clone();
    let mut pointer_state = state;
    let mut move_state = state;
    let mut end_state = state;
    let mut wheel_state = state;
    let primitives = drawing_render::render(&snapshot);

    rect()
        .key(("native-excalidraw-canvas", snapshot.revision))
        .expanded()
        .background(Color::WHITE)
        .overflow(Overflow::Clip)
        .a11y_alt("DrawingCanvas")
        .on_mouse_down(move |event: Event<MouseEventData>| {
            if event.button == Some(MouseButton::Left) {
                pointer_state
                    .write()
                    .begin_pointer(point(event.global_location));
                event.stop_propagation();
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                move_state
                    .write()
                    .move_pointer(point(event.global_location()));
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
                .zoom_at(point(event.global_location), event.delta_y);
            event.stop_propagation();
        })
        .children(primitives)
        .into_element()
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}
