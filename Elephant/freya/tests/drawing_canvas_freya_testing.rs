#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::{Color, Fill, Label, Rect, State};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::fs;

fn fixture_state() -> DrawingCanvasState {
    DrawingCanvasState::from_json(&serde_json::to_string(&json!({
        "type": "excalidraw",
        "version": 2,
        "source": "drawing-canvas-freya-testing",
        "elements": [
            {"id": "rectangle", "type": "rectangle", "x": 80, "y": 70, "width": 120, "height": 80, "strokeColor": "#ff0000", "backgroundColor": "#ffeeee", "strokeWidth": 4, "opacity": 100},
            {"id": "ellipse", "type": "ellipse", "x": 260, "y": 70, "width": 120, "height": 80, "strokeColor": "#00aa00", "backgroundColor": "#eeffee", "strokeWidth": 4, "opacity": 100},
            {"id": "arrow", "type": "arrow", "x": 80, "y": 230, "width": 160, "height": 0, "points": [[0, 0], [160, 0]], "strokeColor": "#0000ff", "strokeWidth": 4, "endArrowhead": "arrow", "opacity": 100},
            {"id": "text", "type": "text", "x": 280, "y": 210, "width": 120, "height": 30, "text": "Native", "fontSize": 24, "strokeColor": "#111111", "opacity": 100},
            {"id": "freedraw", "type": "freedraw", "x": 90, "y": 330, "width": 100, "height": 70, "points": [[0, 0], [20, 30], [45, 5], [70, 50], [100, 20]], "strokeColor": "#aa00aa", "strokeWidth": 4, "opacity": 100}
        ],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {}
    })).expect("fixture JSON"))
    .expect("real Excalidraw fixture must parse")
}

fn canvas_node(runner: &TestingRunner) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some("DrawingCanvas")).then_some(node)
        })
        .expect("native Freya canvas must be visible")
}

fn labeled_node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing visible drawing primitive {label:?}"))
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

#[test]
fn all_supported_excalidraw_primitives_render_from_real_geometry_and_style() {
    let (mut runner, state) = test_runner();
    let canvas = canvas_node(&runner);
    assert_eq!(canvas.layout().area.size.width, 640.);
    assert_eq!(canvas.layout().area.size.height, 480.);

    let rendered = state.peek().renderable_elements();
    assert_eq!(rendered.len(), 5);
    assert_eq!(rendered[0].kind, "rectangle");
    assert_eq!(rendered[0].bounds, (80., 70., 120., 80.));
    assert_eq!(rendered[0].stroke_rgba, [255, 0, 0, 255]);
    assert_eq!(rendered[1].kind, "ellipse");
    assert_eq!(rendered[2].kind, "arrow");
    assert_eq!(rendered[3].kind, "text");
    assert_eq!(rendered[4].kind, "freedraw");

    let rectangle = Rect::try_downcast(
        labeled_node(&runner, "Drawing element rectangle rectangle")
            .element()
            .as_ref(),
    )
    .expect("rectangle must be a native Freya rect");
    assert_eq!(
        rectangle.style.background.as_color(),
        Some(Color::from_rgb(255, 238, 238))
    );
    let rectangle_area = labeled_node(&runner, "Drawing element rectangle rectangle")
        .layout()
        .area;
    assert_eq!(rectangle_area.origin.x, 80.);
    assert_eq!(rectangle_area.origin.y, 70.);
    assert_eq!(rectangle_area.size.width, 120.);
    assert_eq!(rectangle_area.size.height, 80.);

    let ellipse = Rect::try_downcast(
        labeled_node(&runner, "Drawing element ellipse ellipse")
            .element()
            .as_ref(),
    )
    .expect("ellipse must be a native Freya rect with rounded geometry");
    assert_eq!(
        ellipse.style.background.as_color(),
        Some(Color::from_rgb(238, 255, 238))
    );
    assert_eq!(
        labeled_node(&runner, "Drawing element ellipse ellipse")
            .layout()
            .area
            .size
            .width,
        120.
    );

    let arrow_segment = Rect::try_downcast(
        labeled_node(&runner, "Drawing element arrow arrow segment 0")
            .element()
            .as_ref(),
    )
    .expect("arrow must render as a native Freya stroke segment");
    assert_eq!(arrow_segment.style.background.as_color(), Some(Color::BLUE));
    assert_eq!(
        labeled_node(&runner, "Drawing element arrow arrow segment 0")
            .layout()
            .area
            .size
            .width,
        160.
    );

    let text = Label::try_downcast(
        labeled_node(&runner, "Drawing element text text")
            .element()
            .as_ref(),
    )
    .expect("text must be a native Freya label");
    assert_eq!(text.text.as_ref(), "Native");
    assert_eq!(
        text.text_style_data.color.as_ref().and_then(Fill::as_color),
        Some(Color::from_rgb(17, 17, 17))
    );

    let freehand = Rect::try_downcast(
        labeled_node(&runner, "Drawing element freedraw freedraw segment 0")
            .element()
            .as_ref(),
    )
    .expect("freehand must render as native Freya stroke segments");
    assert_eq!(
        freehand.style.background.as_color(),
        Some(Color::from_rgb(170, 0, 170))
    );

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-canvas-primitives.png");
    runner.render_to_file(&evidence);
    assert!(
        fs::metadata(evidence)
            .expect("native canvas screenshot")
            .len()
            > 0
    );
}

#[test]
fn pointer_selection_and_drag_update_model_and_excalidraw_serialization() {
    let (mut runner, state) = test_runner();
    runner.click_cursor((140., 110.));
    assert_eq!(state.peek().selected_element_id(), Some("rectangle"));
    assert!(runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some("Drawing selection 0")).then_some(node)
        })
        .is_some());

    runner.press_cursor((140., 110.));
    runner.move_cursor((190., 145.));
    runner.release_cursor((190., 145.));

    assert_eq!(state.peek().selected_element_id(), Some("rectangle"));
    assert_eq!(state.peek().document.elements[0].x, 130.);
    assert_eq!(state.peek().document.elements[0].y, 105.);
    let serialized = state
        .peek()
        .serialize_json()
        .expect("serialize moved scene");
    let value: serde_json::Value = serde_json::from_str(&serialized).expect("valid scene JSON");
    assert_eq!(value["elements"][0]["x"], 130.);
    assert_eq!(value["elements"][0]["y"], 105.);
}

#[test]
fn wheel_zoom_and_blank_drag_pan_change_the_viewport() {
    let (mut runner, state) = test_runner();
    let before = state.peek().viewport;
    runner.scroll((320., 240.), (0., -120.));
    let zoomed = state.peek().viewport;
    assert!(zoomed.zoom > before.zoom);

    runner.press_cursor((520., 420.));
    runner.move_cursor((560., 450.));
    runner.release_cursor((560., 450.));
    let panned = state.peek().viewport;
    assert_ne!(panned.pan, zoomed.pan);
}

#[test]
fn invalid_scene_is_rejected_instead_of_rendering_a_fake_canvas() {
    let error = DrawingCanvasState::from_json("{\"type\":\"not-excalidraw\",\"elements\":[]}")
        .expect_err("non-Excalidraw data must be rejected");
    assert!(error.contains("type=excalidraw"));
}

#[test]
fn active_freehand_tool_draws_into_shared_state_and_serializes() {
    let (mut runner, state) = test_runner();
    let tool = runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some("Freehand tool")).then_some(node)
        })
        .expect("the native canvas must expose a Freehand tool");
    runner.click_cursor(tool.layout().area.center().to_f64());
    runner.press_cursor((430., 100.));
    runner.move_cursor((470., 130.));
    runner.release_cursor((470., 130.));

    let snapshot = state.peek();
    let element = snapshot
        .document
        .elements
        .last()
        .expect("drawing gesture must append a real element");
    assert_eq!(element.kind, "freedraw");
    assert!(element.points.len() >= 2);
    let serialized = snapshot.serialize_json().expect("serialize drawn scene");
    assert!(serialized.contains(&element.id));
}
