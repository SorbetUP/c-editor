use elephant_draw_contract::{
    create_element, ArrowEndpoint, DrawingScene, DrawingTool, SelectionSet,
};
use serde_json::{json, Value};

fn fixture() -> DrawingScene {
    let mut scene = DrawingScene::empty();
    let mut frame = create_element(DrawingTool::Frame, [0.0, 0.0], "frame");
    frame.width = 300.0;
    frame.height = 200.0;
    let mut shape = create_element(DrawingTool::Rectangle, [40.0, 40.0], "shape");
    shape.width = 80.0;
    shape.height = 50.0;
    shape.extra.insert("groupIds".to_owned(), json!(["g1"]));
    let mut text = create_element(DrawingTool::Text, [50.0, 50.0], "text");
    text.text = "Bound".to_owned();
    let mut arrow = create_element(DrawingTool::Arrow, [350.0, 80.0], "arrow");
    arrow.points = vec![[0.0, 0.0], [-240.0, 0.0]];
    scene.elements = vec![shape, text, arrow, frame];
    assert!(scene.bind_text_to_container("text", "shape"));
    assert!(scene.bind_arrow_endpoint(
        "arrow",
        ArrowEndpoint::End,
        "shape",
        0.0,
        1.0,
        None,
    ));
    assert_eq!(scene.add_elements_to_frame("frame", &["shape"]), 2);
    scene
}

fn selected_by_kind<'a>(
    scene: &'a DrawingScene,
    selection: &SelectionSet,
    kind: &str,
) -> &'a elephant_draw_contract::DrawingElement {
    scene
        .elements
        .iter()
        .find(|element| selection.contains(&element.id) && element.kind == kind)
        .unwrap_or_else(|| panic!("missing duplicated {kind}"))
}

#[test]
fn duplicating_frame_also_duplicates_children_and_rewires_frame_and_text_links() {
    let mut scene = fixture();
    let selection = SelectionSet::from_ids(["frame".to_owned()]);
    let outcome = scene.duplicate_selection_excalidraw(&selection, [10.0, 10.0], 42);

    assert_eq!(outcome.selection.len(), 3);
    let duplicate_frame = selected_by_kind(&scene, &outcome.selection, "frame");
    let duplicate_shape = selected_by_kind(&scene, &outcome.selection, "rectangle");
    let duplicate_text = selected_by_kind(&scene, &outcome.selection, "text");
    assert_ne!(duplicate_frame.id, "frame");
    assert_eq!(
        duplicate_shape.extra["frameId"].as_str(),
        Some(duplicate_frame.id.as_str())
    );
    assert_eq!(
        duplicate_text.extra["frameId"].as_str(),
        Some(duplicate_frame.id.as_str())
    );
    assert_eq!(
        duplicate_text.extra["containerId"].as_str(),
        Some(duplicate_shape.id.as_str())
    );
    assert_eq!((duplicate_frame.x, duplicate_frame.y), (10.0, 10.0));
    assert_eq!((duplicate_shape.x, duplicate_shape.y), (50.0, 50.0));
    assert_ne!(duplicate_shape.extra["groupIds"][0], "g1");
}

#[test]
fn duplicating_bound_shape_pulls_in_text_but_not_unselected_arrow() {
    let mut scene = fixture();
    let selection = SelectionSet::from_ids(["shape".to_owned()]);
    let outcome = scene.duplicate_selection_excalidraw(&selection, [5.0, 5.0], 7);

    assert_eq!(outcome.selection.len(), 2);
    let duplicate_shape = selected_by_kind(&scene, &outcome.selection, "rectangle");
    let duplicate_text = selected_by_kind(&scene, &outcome.selection, "text");
    assert_eq!(
        duplicate_text.extra["containerId"].as_str(),
        Some(duplicate_shape.id.as_str())
    );
    let bound = duplicate_shape.extra["boundElements"]
        .as_array()
        .expect("boundElements");
    assert_eq!(bound.len(), 1);
    assert_eq!(bound[0]["type"], "text");
    assert_eq!(bound[0]["id"].as_str(), Some(duplicate_text.id.as_str()));
    assert_eq!(duplicate_shape.extra["frameId"], "frame");
}

#[test]
fn duplicating_arrow_keeps_binding_to_original_target_and_registers_new_arrow() {
    let mut scene = fixture();
    let selection = SelectionSet::from_ids(["arrow".to_owned()]);
    let outcome = scene.duplicate_selection_excalidraw(&selection, [10.0, 0.0], 99);

    assert_eq!(outcome.selection.len(), 1);
    let duplicate_arrow = selected_by_kind(&scene, &outcome.selection, "arrow");
    assert_eq!(
        duplicate_arrow.extra["endBinding"]["elementId"],
        Value::String("shape".to_owned())
    );
    let shape = scene.element_by_id("shape").unwrap();
    assert!(shape
        .extra["boundElements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"].as_str() == Some(duplicate_arrow.id.as_str())));
}
