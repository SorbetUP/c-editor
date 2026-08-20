#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::fs;

fn labeled_node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing visible drawing primitive {label:?}"))
}

fn runner_for(value: serde_json::Value) -> (TestingRunner, State<DrawingCanvasState>) {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&value).expect("serialize Excalidraw fixture"),
    )
    .expect("fixture must be valid Excalidraw JSON");
    TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

#[test]
fn frame_tool_draws_a_real_persisted_excalidraw_frame() {
    let (mut runner, state) = runner_for(json!({
        "type": "excalidraw",
        "version": 2,
        "elements": [],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {}
    }));

    let frame_tool = labeled_node(&runner, "Frame tool");
    runner.click_cursor(frame_tool.layout().area.center().to_f64());
    assert_eq!(state.peek().active_tool, "frame");

    runner.press_cursor((120., 100.));
    runner.move_cursor((360., 260.));
    runner.release_cursor((360., 260.));

    let snapshot = state.peek();
    let frame = snapshot
        .document
        .elements
        .last()
        .expect("frame gesture must append an element");
    assert_eq!(frame.kind, "frame");
    assert_eq!((frame.x, frame.y), (120., 100.));
    assert_eq!((frame.width, frame.height), (240., 160.));
    assert!(frame.extra.contains_key("version"));
    let frame_id = frame.id.clone();

    let serialized = snapshot.serialize_json().expect("serialize frame scene");
    let value: serde_json::Value = serde_json::from_str(&serialized).expect("valid JSON");
    assert_eq!(value["elements"][0]["type"], "frame");
    drop(snapshot);

    labeled_node(
        &runner,
        &format!("Drawing element {frame_id} frame"),
    );
    let evidence = std::env::temp_dir().join("elephant-freya-drawing-frame.png");
    runner.render_to_file(&evidence);
    assert!(
        fs::metadata(evidence)
            .expect("native frame screenshot")
            .len()
            > 0
    );
}

#[test]
fn imported_frame_embed_and_iframe_elements_remain_visible() {
    let (mut runner, _state) = runner_for(json!({
        "type": "excalidraw",
        "version": 2,
        "elements": [
            {"id":"frame","type":"frame","x":20,"y":20,"width":120,"height":90},
            {"id":"magic","type":"magicframe","x":170,"y":20,"width":120,"height":90},
            {"id":"embed","type":"embeddable","x":20,"y":150,"width":120,"height":90},
            {"id":"iframe","type":"iframe","x":170,"y":150,"width":120,"height":90}
        ],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {}
    }));

    for label in [
        "Drawing element frame frame",
        "Drawing element magic magicframe",
        "Drawing element embed embeddable",
        "Drawing element iframe iframe",
    ] {
        labeled_node(&runner, label);
    }

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-frame-like-imports.png");
    runner.render_to_file(&evidence);
    assert!(
        fs::metadata(evidence)
            .expect("native frame-like screenshot")
            .len()
            > 0
    );
}

#[test]
fn start_arrowhead_changes_the_native_freya_pixels() {
    fn scene(start_arrowhead: serde_json::Value) -> serde_json::Value {
        json!({
            "type": "excalidraw",
            "version": 2,
            "elements": [{
                "id": "arrow",
                "type": "arrow",
                "x": 120,
                "y": 180,
                "width": 300,
                "height": 80,
                "points": [[0, 0], [300, 80]],
                "strokeColor": "#111111",
                "strokeWidth": 4,
                "startArrowhead": start_arrowhead,
                "endArrowhead": "arrow"
            }],
            "appState": {"viewBackgroundColor": "#ffffff"},
            "files": {}
        })
    }

    let (mut end_only_runner, _) = runner_for(scene(serde_json::Value::Null));
    let (mut two_sided_runner, _) = runner_for(scene(json!("arrow")));
    labeled_node(&end_only_runner, "Drawing element arrow arrow");
    labeled_node(&two_sided_runner, "Drawing element arrow arrow");

    let end_only = std::env::temp_dir().join("elephant-freya-arrow-end-only.png");
    let two_sided = std::env::temp_dir().join("elephant-freya-arrow-two-sided.png");
    end_only_runner.render_to_file(&end_only);
    two_sided_runner.render_to_file(&two_sided);

    assert_ne!(
        fs::read(end_only).expect("end-only arrow screenshot"),
        fs::read(two_sided).expect("two-sided arrow screenshot"),
        "startArrowhead must affect real native Freya pixels"
    );
}
