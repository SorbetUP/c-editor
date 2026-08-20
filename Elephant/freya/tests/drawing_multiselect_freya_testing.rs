#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

fn fixture_state() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "version": 2,
            "source": "drawing-multiselect-freya-testing",
            "elements": [
                {"id": "left", "type": "rectangle", "x": 80, "y": 70, "width": 100, "height": 70, "strokeColor": "#111111", "backgroundColor": "#ffeeee", "strokeWidth": 3, "opacity": 100},
                {"id": "right", "type": "ellipse", "x": 230, "y": 70, "width": 100, "height": 70, "strokeColor": "#111111", "backgroundColor": "#eeffee", "strokeWidth": 3, "opacity": 100},
                {"id": "outside", "type": "diamond", "x": 450, "y": 300, "width": 80, "height": 80, "strokeColor": "#111111", "backgroundColor": "#eeeeff", "strokeWidth": 3, "opacity": 100}
            ],
            "appState": {"viewBackgroundColor": "#ffffff"},
            "files": {}
        }))
        .expect("fixture JSON"),
    )
    .expect("real Excalidraw fixture must parse")
}

fn test_runner() -> (TestingRunner, State<DrawingCanvasState>) {
    let state = fixture_state();
    TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

fn has_label(runner: &TestingRunner, label: &str) -> bool {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .is_some()
}

#[test]
fn box_select_renders_marquee_and_group_drag_moves_every_selected_element() {
    let (mut runner, state) = test_runner();

    runner.press_cursor((60., 50.));
    runner.move_cursor((360., 180.));

    let selected = state.peek().selected_element_ids();
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&"left"));
    assert!(selected.contains(&"right"));
    assert!(!selected.contains(&"outside"));
    assert!(has_label(&runner, "Drawing selection 0"));
    assert!(has_label(&runner, "Drawing selection 1"));
    assert!(has_label(&runner, "Drawing group selection"));
    assert!(has_label(&runner, "Drawing selection marquee"));

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-multiselect.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(&evidence).expect("multi-select screenshot").len() > 0);

    runner.release_cursor((360., 180.));
    assert!(!has_label(&runner, "Drawing selection marquee"));
    assert!(has_label(&runner, "Drawing group selection"));

    let before = state.peek().document.clone();
    runner.press_cursor((100., 100.));
    runner.move_cursor((130., 125.));
    runner.release_cursor((130., 125.));

    let snapshot = state.peek();
    assert_eq!((snapshot.document.elements[0].x, snapshot.document.elements[0].y), (110., 95.));
    assert_eq!((snapshot.document.elements[1].x, snapshot.document.elements[1].y), (260., 95.));
    assert_eq!((snapshot.document.elements[2].x, snapshot.document.elements[2].y), (450., 300.));
    assert_eq!(snapshot.selected_element_ids().len(), 2);
    let serialized = snapshot.serialize_json().expect("serialize group move");
    let value: serde_json::Value = serde_json::from_str(&serialized).expect("valid JSON");
    assert_eq!(value["elements"][0]["x"], 110.0);
    assert_eq!(value["elements"][1]["x"], 260.0);
    drop(snapshot);

    assert!(state.write().undo());
    let undone = state.peek();
    assert_eq!(undone.document.elements[0].x, before.elements[0].x);
    assert_eq!(undone.document.elements[0].y, before.elements[0].y);
    assert_eq!(undone.document.elements[1].x, before.elements[1].x);
    assert_eq!(undone.document.elements[1].y, before.elements[1].y);
}
