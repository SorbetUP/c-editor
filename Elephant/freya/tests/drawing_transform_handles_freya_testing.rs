#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::fs;

fn runner() -> (TestingRunner, State<DrawingCanvasState>) {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "version":2,
            "source":"drawing-transform-handles-freya-testing",
            "elements":[{
                "id":"box",
                "type":"rectangle",
                "x":100,
                "y":100,
                "width":120,
                "height":80,
                "strokeColor":"#1e1e1e",
                "backgroundColor":"#a5d8ff",
                "strokeWidth":2,
                "strokeStyle":"solid",
                "fillStyle":"solid",
                "roughness":1,
                "opacity":100,
                "angle":0,
                "version":1,
                "versionNonce":1,
                "isDeleted":false,
                "locked":false
            }],
            "appState":{"viewBackgroundColor":"#ffffff"},
            "files":{}
        }))
        .unwrap(),
    )
    .unwrap();
    TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

fn node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing {label}"))
}

#[test]
fn selected_element_can_be_resized_and_rotated_with_native_handles() {
    let (mut runner, state) = runner();
    runner.click_cursor((160., 140.));
    assert_eq!(state.peek().selected_element_id(), Some("box"));

    let southeast = node(&runner, "Drawing resize handle southeast");
    let start = southeast.layout().area.center().to_f64();
    runner.press_cursor(start);
    runner.move_cursor((start.0 + 60.0, start.1 + 40.0));
    runner.release_cursor((start.0 + 60.0, start.1 + 40.0));

    let resized = state.peek().document.element_by_id("box").unwrap().clone();
    assert!(resized.width > 170.0, "resize handle must grow the selected width");
    assert!(resized.height > 110.0, "resize handle must grow the selected height");
    assert_eq!((resized.x, resized.y), (100.0, 100.0));

    let rotate = node(&runner, "Drawing rotate handle");
    let rotate_start = rotate.layout().area.center().to_f64();
    let bounds = state.peek().selection_bounds().unwrap();
    let center = (bounds.0 + bounds.2 / 2.0, bounds.1 + bounds.3 / 2.0);
    runner.press_cursor(rotate_start);
    runner.move_cursor((f64::from(center.0 + bounds.2 / 2.0 + 30.0), f64::from(center.1)));
    runner.release_cursor((f64::from(center.0 + bounds.2 / 2.0 + 30.0), f64::from(center.1)));

    let rotated = state.peek().document.element_by_id("box").unwrap().clone();
    assert!(rotated.angle > 0.5 && rotated.angle < 2.7, "rotation handle must update the Excalidraw angle");
    assert!(rotated.extra["version"].as_u64().unwrap_or(0) > 1);

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-transform-handles.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(evidence).unwrap().len() > 0);
}
