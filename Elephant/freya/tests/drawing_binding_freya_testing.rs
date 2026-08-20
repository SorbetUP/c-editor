#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;

fn runner() -> (TestingRunner, State<DrawingCanvasState>) {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "version":2,
            "source":"drawing-binding-freya-testing",
            "elements":[
                {"id":"left","type":"rectangle","x":80,"y":100,"width":110,"height":90,"strokeColor":"#1e1e1e","backgroundColor":"#dbe4ff","strokeWidth":2,"opacity":100,"isDeleted":false,"locked":false,"boundElements":[]},
                {"id":"right","type":"ellipse","x":300,"y":100,"width":110,"height":90,"strokeColor":"#1e1e1e","backgroundColor":"#d3f9d8","strokeWidth":2,"opacity":100,"isDeleted":false,"locked":false,"boundElements":[]}
            ],
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
fn drawing_arrow_between_shapes_creates_excalidraw_endpoint_bindings() {
    let (mut runner, state) = runner();
    let arrow_tool = node(&runner, "Arrow tool");
    runner.click_cursor(arrow_tool.layout().area.center().to_f64());

    runner.press_cursor((135., 145.));
    runner.move_cursor((355., 145.));
    runner.release_cursor((355., 145.));

    let snapshot = state.peek();
    let arrow = snapshot.document.elements.last().expect("arrow created");
    assert_eq!(arrow.kind, "arrow");
    assert_eq!(arrow.extra["startBinding"]["elementId"], "left");
    assert_eq!(arrow.extra["endBinding"]["elementId"], "right");
    assert_eq!(arrow.extra["startBinding"]["mode"], "orbit");
    assert_eq!(arrow.extra["endBinding"]["mode"], "orbit");
    assert!(snapshot.document.element_by_id("left").unwrap().extra["boundElements"]
        .as_array().unwrap().iter().any(|entry| entry["id"] == arrow.id));
    assert!(snapshot.document.element_by_id("right").unwrap().extra["boundElements"]
        .as_array().unwrap().iter().any(|entry| entry["id"] == arrow.id));
}

#[test]
fn text_created_inside_shape_becomes_bound_text() {
    let (mut runner, state) = runner();
    let text_tool = node(&runner, "Text tool");
    runner.click_cursor(text_tool.layout().area.center().to_f64());
    runner.click_cursor((120., 125.));

    let snapshot = state.peek();
    let text = snapshot.document.elements.last().expect("text created");
    assert_eq!(text.kind, "text");
    assert_eq!(text.extra["containerId"], "left");
    let left = snapshot.document.element_by_id("left").unwrap();
    assert!(left.extra["boundElements"]
        .as_array().unwrap().iter().any(|entry| entry["type"] == "text" && entry["id"] == text.id));
}
