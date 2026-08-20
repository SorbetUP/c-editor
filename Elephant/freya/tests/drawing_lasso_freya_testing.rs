#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

fn fixture() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "version": 2,
            "elements": [
                {
                    "id": "inside",
                    "type": "rectangle",
                    "x": 50,
                    "y": 50,
                    "width": 40,
                    "height": 40,
                    "strokeColor": "#1e1e1e",
                    "backgroundColor": "#a5d8ff",
                    "strokeWidth": 2
                },
                {
                    "id": "outside",
                    "type": "rectangle",
                    "x": 220,
                    "y": 180,
                    "width": 60,
                    "height": 50,
                    "strokeColor": "#e03131",
                    "backgroundColor": "#ffc9c9",
                    "strokeWidth": 2
                }
            ],
            "appState": {"viewBackgroundColor": "#ffffff"},
            "files": {}
        }))
        .expect("serialize fixture"),
    )
    .expect("valid lasso fixture")
}

fn trace_lasso(state: &mut DrawingCanvasState, finish: bool) {
    state.set_active_tool_label("Lasso");
    state.begin_pointer([25.0, 25.0]);
    for point in [
        [120.0, 25.0],
        [120.0, 120.0],
        [25.0, 120.0],
        [25.0, 25.0],
    ] {
        state.move_pointer(point);
    }
    if finish {
        state.end_pointer();
    }
}

#[test]
fn lasso_selects_contained_elements_without_persisting_a_fake_element() {
    let mut state = fixture();
    trace_lasso(&mut state, true);

    assert_eq!(state.selected_element_ids(), vec!["inside"]);
    assert!(!state.is_element_selected("outside"));

    let raw = state.serialize_json().expect("serialize lasso result");
    assert!(!raw.contains("\"type\": \"lasso\""));
    assert_eq!(state.document.elements.len(), 2);
}

#[test]
fn native_lasso_path_is_visible_while_dragging() {
    let mut state = fixture();
    trace_lasso(&mut state, false);
    assert!(state.selection_lasso_world().is_some());

    let output = std::env::temp_dir().join("elephant-freya-drawing-lasso.png");
    let mut runner = TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    );
    runner.render_to_file(&output);
    let bytes = fs::read(&output).expect("lasso screenshot");
    assert!(bytes.len() > 1_000, "native lasso screenshot must contain rendered pixels");
}
