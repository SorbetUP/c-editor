#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::json;

fn state() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "elements": [
                {"id":"a","type":"rectangle","x":0,"y":0,"width":50,"height":50,"strokeColor":"#000000","backgroundColor":"transparent","strokeWidth":2,"opacity":100},
                {"id":"b","type":"rectangle","x":100,"y":0,"width":50,"height":50,"strokeColor":"#000000","backgroundColor":"transparent","strokeWidth":2,"opacity":100}
            ],
            "appState": {},
            "files": {}
        }))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selection_move_is_one_undo_redo_transaction() {
    let mut canvas = state();
    canvas.set_active_tool_label("Selection");
    canvas.begin_pointer([20.0, 20.0]);
    canvas.move_pointer([70.0, 80.0]);
    canvas.move_pointer([90.0, 95.0]);
    canvas.end_pointer();

    assert_eq!((canvas.document.elements[0].x, canvas.document.elements[0].y), (70.0, 75.0));
    assert!(canvas.can_undo());
    assert!(canvas.undo());
    assert_eq!((canvas.document.elements[0].x, canvas.document.elements[0].y), (0.0, 0.0));
    assert!(canvas.can_redo());
    assert!(canvas.redo());
    assert_eq!((canvas.document.elements[0].x, canvas.document.elements[0].y), (70.0, 75.0));
}

#[test]
fn eraser_drag_deletes_multiple_elements_as_one_transaction() {
    let mut canvas = state();
    canvas.set_active_tool_label("Eraser");
    canvas.begin_pointer([25.0, 25.0]);
    canvas.move_pointer([125.0, 25.0]);
    canvas.end_pointer();

    assert!(canvas.document.elements.iter().all(|element| element.is_deleted));
    assert!(canvas.undo());
    assert!(canvas.document.elements.iter().all(|element| !element.is_deleted));
}

#[test]
fn delete_selection_is_undoable_and_does_not_remove_json_identity() {
    let mut canvas = state();
    canvas.set_active_tool_label("Selection");
    canvas.begin_pointer([20.0, 20.0]);
    canvas.end_pointer();
    assert_eq!(canvas.selected_element_id(), Some("a"));

    assert!(canvas.delete_selection());
    assert!(canvas.document.elements[0].is_deleted);
    assert_eq!(canvas.document.elements[0].id, "a");
    assert!(canvas.undo());
    assert!(!canvas.document.elements[0].is_deleted);
    assert_eq!(canvas.document.elements[0].id, "a");
}

#[test]
fn escape_style_cancel_deselects_and_returns_to_selection_without_mutating_scene() {
    let mut canvas = state();
    let before = canvas.document.clone();
    canvas.set_active_tool_label("Rectangle");
    canvas.cancel_interaction();
    assert_eq!(canvas.active_tool, "selection");
    assert_eq!(canvas.selected_element_id(), None);
    assert_eq!(canvas.document, before);
}

#[test]
fn explicit_zoom_is_clamped_to_draw_contract() {
    let mut canvas = state();
    canvas.set_zoom(1000.0);
    assert_eq!(canvas.viewport.zoom, elephant_draw::MAX_ZOOM);
    canvas.set_zoom(0.001);
    assert_eq!(canvas.viewport.zoom, elephant_draw::MIN_ZOOM);
}
