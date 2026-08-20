#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::fs;

fn runner_for(value: serde_json::Value) -> (TestingRunner, State<DrawingCanvasState>) {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&value).expect("serialize transform fixture"),
    )
    .expect("valid transform fixture");
    TestingRunner::new(
        drawing_canvas,
        (700., 500.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

fn labeled_node(runner: &TestingRunner, label: &str) -> Option<TestingNode> {
    runner.find(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn fixture() -> serde_json::Value {
    json!({
        "type":"excalidraw",
        "version":2,
        "elements":[
            {"id":"box","type":"rectangle","x":100,"y":100,"width":100,"height":80,"strokeColor":"#111","backgroundColor":"#a5d8ff","strokeWidth":2,"version":1},
            {"id":"child","type":"rectangle","x":340,"y":140,"width":60,"height":40,"frameId":"frame","strokeColor":"#111","backgroundColor":"#ffd8a8","strokeWidth":2,"version":1},
            {"id":"frame","type":"frame","x":300,"y":100,"width":180,"height":140,"strokeColor":"#868e96","backgroundColor":"transparent","strokeWidth":2,"version":1}
        ],
        "appState":{"viewBackgroundColor":"#ffffff"},
        "files":{}
    })
}

#[test]
fn pointer_drag_corner_handle_resizes_real_native_rectangle_pixels() {
    let (mut runner, state) = runner_for(fixture());
    runner.press_cursor((150.0, 140.0));
    runner.release_cursor((150.0, 140.0));

    let handle = labeled_node(&runner, "Drawing transform handle se")
        .expect("selected rectangle must expose south-east resize handle");
    let before = std::env::temp_dir().join("elephant-draw-transform-before.png");
    runner.render_to_file(&before);
    let center = handle.layout().area.center().to_f64();
    runner.press_cursor(center);
    runner.move_cursor((250.0, 220.0));
    runner.release_cursor((250.0, 220.0));

    let box_element = state.peek().document.element_by_id("box").unwrap().clone();
    assert!(box_element.width > 140.0, "resize drag must grow width");
    assert!(box_element.height > 110.0, "resize drag must grow height");

    let after = std::env::temp_dir().join("elephant-draw-transform-after.png");
    runner.render_to_file(&after);
    assert_ne!(fs::read(before).unwrap(), fs::read(after).unwrap());
}

#[test]
fn rotation_handle_rotates_regular_element_and_changes_pixels() {
    let (mut runner, state) = runner_for(fixture());
    runner.press_cursor((150.0, 140.0));
    runner.release_cursor((150.0, 140.0));
    let handle = labeled_node(&runner, "Drawing transform handle rotation")
        .expect("regular selection must expose rotation handle");
    let before = std::env::temp_dir().join("elephant-draw-rotation-before.png");
    runner.render_to_file(&before);
    let center = handle.layout().area.center().to_f64();
    runner.press_cursor(center);
    runner.move_cursor((230.0, 140.0));
    runner.release_cursor((230.0, 140.0));
    assert!(state.peek().document.element_by_id("box").unwrap().angle.abs() > 0.1);
    let after = std::env::temp_dir().join("elephant-draw-rotation-after.png");
    runner.render_to_file(&after);
    assert_ne!(fs::read(before).unwrap(), fs::read(after).unwrap());
}

#[test]
fn frame_resize_keeps_child_geometry_and_never_exposes_rotation_handle() {
    let (mut runner, state) = runner_for(fixture());
    let child_before = state.peek().document.element_by_id("child").unwrap().clone();
    runner.press_cursor((301.0, 170.0));
    runner.release_cursor((301.0, 170.0));

    assert!(labeled_node(&runner, "Drawing transform handle rotation").is_none());
    let handle = labeled_node(&runner, "Drawing transform handle se")
        .expect("frame selection must expose resize handles");
    let center = handle.layout().area.center().to_f64();
    runner.press_cursor(center);
    runner.move_cursor((540.0, 300.0));
    runner.release_cursor((540.0, 300.0));

    let snapshot = state.peek();
    let frame = snapshot.document.element_by_id("frame").unwrap();
    let child = snapshot.document.element_by_id("child").unwrap();
    assert!(frame.width > 220.0 && frame.height > 180.0);
    assert_eq!(child.x, child_before.x);
    assert_eq!(child.y, child_before.y);
    assert_eq!(child.width, child_before.width);
    assert_eq!(child.height, child_before.height);
    assert_eq!(child.extra["frameId"], "frame");
}
