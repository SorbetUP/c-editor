#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::{json, Value};

fn state() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "elements": [
                {"id":"a","type":"rectangle","x":10,"y":10,"width":40,"height":40,"strokeColor":"#1e1e1e","backgroundColor":"transparent","strokeWidth":2,"strokeStyle":"solid","fillStyle":"solid","opacity":100},
                {"id":"b","type":"rectangle","x":80,"y":10,"width":40,"height":40,"strokeColor":"#1e1e1e","backgroundColor":"transparent","strokeWidth":2,"strokeStyle":"solid","fillStyle":"solid","opacity":100},
                {"id":"c","type":"rectangle","x":180,"y":10,"width":40,"height":40,"strokeColor":"#1e1e1e","backgroundColor":"transparent","strokeWidth":2,"strokeStyle":"solid","fillStyle":"solid","opacity":100},
                {"id":"d","type":"rectangle","x":250,"y":10,"width":40,"height":40,"strokeColor":"#1e1e1e","backgroundColor":"transparent","strokeWidth":2,"strokeStyle":"solid","fillStyle":"solid","opacity":100}
            ],
            "appState": {},
            "files": {}
        }))
        .unwrap(),
    )
    .unwrap()
}

fn select_first_two(canvas: &mut DrawingCanvasState) {
    canvas.set_active_tool_label("Selection");
    canvas.begin_pointer([0.0, 0.0]);
    canvas.move_pointer([130.0, 60.0]);
    canvas.end_pointer();
    let mut ids = canvas.selected_element_ids();
    ids.sort_unstable();
    assert_eq!(ids, vec!["a", "b"]);
}

#[test]
fn multi_selection_style_is_applied_atomically_and_undo_restores_every_element() {
    let mut canvas = state();
    select_first_two(&mut canvas);

    assert!(canvas.set_selected_stroke("#ff0000"));
    assert_eq!(canvas.document.elements[0].stroke_color, "#ff0000");
    assert_eq!(canvas.document.elements[1].stroke_color, "#ff0000");
    assert_eq!(canvas.document.elements[2].stroke_color, "#1e1e1e");
    assert!(canvas.can_undo());

    assert!(canvas.undo());
    assert_eq!(canvas.document.elements[0].stroke_color, "#1e1e1e");
    assert_eq!(canvas.document.elements[1].stroke_color, "#1e1e1e");
    assert_eq!(canvas.document.elements[2].stroke_color, "#1e1e1e");
}

#[test]
fn multi_selection_front_reorder_preserves_internal_order_and_fractional_indices() {
    let mut canvas = state();
    select_first_two(&mut canvas);

    assert!(canvas.bring_selection_to_front());
    assert_eq!(
        canvas
            .document
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>(),
        vec!["c", "d", "a", "b"]
    );

    let indices = canvas
        .document
        .elements
        .iter()
        .map(|element| {
            element
                .extra
                .get("index")
                .and_then(Value::as_str)
                .expect("reorder must persist a fractional index")
        })
        .collect::<Vec<_>>();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));

    assert!(canvas.undo());
    assert_eq!(
        canvas
            .document
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b", "c", "d"]
    );
}

#[test]
fn multi_selection_forward_moves_one_layer_without_reversing_selected_elements() {
    let mut canvas = state();
    select_first_two(&mut canvas);

    assert!(canvas.bring_selection_forward());
    assert_eq!(
        canvas
            .document
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>(),
        vec!["c", "a", "b", "d"]
    );
}
