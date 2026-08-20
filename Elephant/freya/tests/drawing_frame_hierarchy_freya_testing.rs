#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::fs;

fn runner_for(state: DrawingCanvasState) -> (TestingRunner, State<DrawingCanvasState>) {
    TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

fn labeled_node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing visible drawing primitive {label:?}"))
}

fn fixture() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "version": 2,
            "elements": [
                {
                    "id": "child",
                    "type": "rectangle",
                    "x": 150,
                    "y": 130,
                    "width": 100,
                    "height": 60,
                    "frameId": "frame",
                    "strokeColor": "#1e1e1e",
                    "backgroundColor": "#a5d8ff",
                    "strokeWidth": 2
                },
                {
                    "id": "frame",
                    "type": "frame",
                    "x": 100,
                    "y": 80,
                    "width": 260,
                    "height": 180,
                    "strokeColor": "#bbb",
                    "backgroundColor": "transparent",
                    "strokeWidth": 2,
                    "name": "Frame"
                }
            ],
            "appState": {"viewBackgroundColor": "#ffffff"},
            "files": {}
        }))
        .expect("serialize fixture"),
    )
    .expect("valid Excalidraw frame fixture")
}

#[test]
fn moving_frame_with_core_moves_child_and_changes_native_freya_pixels() {
    let before_state = fixture();
    let before_frame = before_state
        .document
        .element_by_id("frame")
        .unwrap()
        .bounds();
    let before_child = before_state
        .document
        .element_by_id("child")
        .unwrap()
        .bounds();

    let mut after_state = before_state.clone();
    assert_eq!(
        after_state
            .document
            .translate_frame_with_children("frame", [80.0, 45.0]),
        2
    );
    after_state.revision = after_state.revision.wrapping_add(1);
    let after_frame = after_state.document.element_by_id("frame").unwrap().bounds();
    let after_child = after_state.document.element_by_id("child").unwrap().bounds();

    assert_eq!(
        (
            after_frame.0 - before_frame.0,
            after_frame.1 - before_frame.1
        ),
        (80.0, 45.0)
    );
    assert_eq!(
        (
            after_child.0 - before_child.0,
            after_child.1 - before_child.1
        ),
        (80.0, 45.0)
    );

    let (mut before_runner, _) = runner_for(before_state);
    let (mut after_runner, _) = runner_for(after_state);
    let before_png = std::env::temp_dir().join("elephant-freya-frame-hierarchy-before.png");
    let after_png = std::env::temp_dir().join("elephant-freya-frame-hierarchy-after.png");
    before_runner.render_to_file(&before_png);
    after_runner.render_to_file(&after_png);

    assert_ne!(
        fs::read(before_png).expect("before frame hierarchy screenshot"),
        fs::read(after_png).expect("after frame hierarchy screenshot"),
        "moving a frame hierarchy must change real native Freya pixels"
    );
}

#[test]
fn pointer_drag_on_frame_border_moves_frame_and_child_together() {
    let (mut runner, state) = runner_for(fixture());
    let frame_before = state
        .peek()
        .document
        .element_by_id("frame")
        .unwrap()
        .bounds();
    let child_before = state
        .peek()
        .document
        .element_by_id("child")
        .unwrap()
        .bounds();

    labeled_node(&runner, "Drawing element frame frame");
    runner.press_cursor((101.0, 170.0));
    runner.move_cursor((151.0, 200.0));
    runner.release_cursor((151.0, 200.0));

    let snapshot = state.peek();
    let frame_after = snapshot.document.element_by_id("frame").unwrap().bounds();
    let child_after = snapshot.document.element_by_id("child").unwrap().bounds();
    assert_eq!(
        (
            frame_after.0 - frame_before.0,
            frame_after.1 - frame_before.1
        ),
        (50.0, 30.0)
    );
    assert_eq!(
        (
            child_after.0 - child_before.0,
            child_after.1 - child_before.1
        ),
        (50.0, 30.0)
    );
    assert_eq!(
        snapshot
            .document
            .element_by_id("child")
            .unwrap()
            .extra["frameId"],
        "frame"
    );
}

#[test]
fn pointer_drag_child_outside_frame_detaches_membership_on_release() {
    let (mut runner, state) = runner_for(fixture());
    let frame_before = state
        .peek()
        .document
        .element_by_id("frame")
        .unwrap()
        .bounds();

    labeled_node(&runner, "Drawing element child rectangle");
    runner.press_cursor((180.0, 155.0));
    runner.move_cursor((500.0, 360.0));
    runner.release_cursor((500.0, 360.0));

    let snapshot = state.peek();
    let child = snapshot.document.element_by_id("child").unwrap();
    assert!(child.x > frame_before.0 + frame_before.2);
    assert!(child.extra["frameId"].is_null());
    assert_eq!(
        snapshot.document.element_by_id("frame").unwrap().bounds(),
        frame_before
    );
}

#[test]
fn deleting_selected_frame_keeps_child_visible_detached_and_selected() {
    let (mut runner, mut state) = runner_for(fixture());
    labeled_node(&runner, "Drawing element frame frame");
    runner.press_cursor((101.0, 170.0));
    runner.release_cursor((101.0, 170.0));

    assert_eq!(state.peek().selected_element_id(), Some("frame"));
    assert!(state.write().delete_selection());

    let snapshot = state.peek();
    assert!(snapshot.document.element_by_id("frame").unwrap().is_deleted);
    let child = snapshot.document.element_by_id("child").unwrap();
    assert!(!child.is_deleted);
    assert!(child.extra["frameId"].is_null());
    assert_eq!(snapshot.selected_element_id(), Some("child"));
}

#[test]
fn geometry_membership_persists_frame_id_through_freya_serialization() {
    let mut state = fixture();
    state.document.remove_elements_from_frame(&["child"]);
    assert!(state
        .document
        .element_by_id("child")
        .unwrap()
        .extra["frameId"]
        .is_null());
    assert!(state.document.sync_element_frame_membership("child"));

    let raw = state.serialize_json().expect("serialize frame membership");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");
    let child = value["elements"]
        .as_array()
        .expect("elements")
        .iter()
        .find(|element| element["id"] == "child")
        .expect("child");
    assert_eq!(child["frameId"], "frame");
}
