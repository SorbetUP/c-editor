use elephant_draw_contract::{
    create_element, ArrowEndpoint, DrawingScene, DrawingTool, SelectionSet,
};
use serde_json::Value;

fn scene_with_frame_child_and_binding() -> DrawingScene {
    let mut scene = DrawingScene::empty();
    let mut frame = create_element(DrawingTool::Frame, [0.0, 0.0], "frame");
    frame.width = 300.0;
    frame.height = 200.0;
    let mut shape = create_element(DrawingTool::Rectangle, [40.0, 40.0], "shape");
    shape.width = 80.0;
    shape.height = 50.0;
    let mut text = create_element(DrawingTool::Text, [55.0, 55.0], "text");
    text.width = 40.0;
    text.height = 20.0;
    let mut arrow = create_element(DrawingTool::Arrow, [350.0, 80.0], "arrow");
    arrow.points = vec![[0.0, 0.0], [-250.0, 0.0]];
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

#[test]
fn deleting_frame_keeps_children_detaches_them_and_selects_container() {
    let mut scene = scene_with_frame_child_and_binding();
    let selection = SelectionSet::from_ids(["frame".to_owned(), "shape".to_owned()]);

    let outcome = scene.delete_selection_excalidraw(&selection);
    assert!(outcome.changed >= 3);
    assert!(scene.element_by_id("frame").unwrap().is_deleted);
    assert!(!scene.element_by_id("shape").unwrap().is_deleted);
    assert!(!scene.element_by_id("text").unwrap().is_deleted);
    assert!(scene.element_by_id("shape").unwrap().extra["frameId"].is_null());
    assert!(scene.element_by_id("text").unwrap().extra["frameId"].is_null());
    assert!(outcome.selection.contains("shape"));
    assert!(!outcome.selection.contains("text"));
}

#[test]
fn deleting_bound_container_deletes_its_text_and_cleans_surviving_arrow_binding() {
    let mut scene = scene_with_frame_child_and_binding();
    scene.remove_elements_from_frame(&["shape"]);
    let selection = SelectionSet::from_ids(["shape".to_owned()]);

    let outcome = scene.delete_selection_excalidraw(&selection);
    assert!(outcome.changed >= 3);
    assert!(scene.element_by_id("shape").unwrap().is_deleted);
    assert!(scene.element_by_id("text").unwrap().is_deleted);
    let arrow = scene.element_by_id("arrow").unwrap();
    assert!(arrow.extra["endBinding"].is_null());
}

#[test]
fn deleting_arrow_removes_stale_bound_element_reference_from_target() {
    let mut scene = scene_with_frame_child_and_binding();
    let selection = SelectionSet::from_ids(["arrow".to_owned()]);

    scene.delete_selection_excalidraw(&selection);
    assert!(scene.element_by_id("arrow").unwrap().is_deleted);
    let shape = scene.element_by_id("shape").unwrap();
    assert!(shape
        .extra
        .get("boundElements")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .all(|entry| entry["id"] != "arrow"));
}
