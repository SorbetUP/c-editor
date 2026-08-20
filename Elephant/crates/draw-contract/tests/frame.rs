use elephant_draw_contract::{create_element, validate_order_key, DrawingScene, DrawingTool};
use serde_json::Value;

fn frame_id(scene: &DrawingScene, id: &str) -> Option<String> {
    scene
        .element_by_id(id)
        .and_then(|element| element.extra.get("frameId"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn scene_with_frame() -> DrawingScene {
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
    scene.elements = vec![frame, shape, text];
    scene
}

#[test]
fn bound_text_follows_container_into_frame_and_indices_follow_render_order() {
    let mut scene = scene_with_frame();
    assert!(scene.bind_text_to_container("text", "shape"));
    assert_eq!(scene.add_elements_to_frame("frame", &["shape"]), 2);
    assert_eq!(frame_id(&scene, "shape").as_deref(), Some("frame"));
    assert_eq!(frame_id(&scene, "text").as_deref(), Some("frame"));
    assert_eq!(
        scene.elements.iter().map(|element| element.id.as_str()).collect::<Vec<_>>(),
        vec!["shape", "text", "frame"]
    );
    let indices = scene
        .elements
        .iter()
        .map(|element| element.extra.get("index").and_then(Value::as_str).expect("index"))
        .collect::<Vec<_>>();
    assert!(indices.iter().all(|index| validate_order_key(index)));
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn frame_translation_moves_direct_children_and_geometry_can_detach_them() {
    let mut scene = scene_with_frame();
    assert_eq!(scene.add_elements_to_frame("frame", &["shape"]), 1);
    let shape_before = scene.element_by_id("shape").unwrap().bounds();
    assert_eq!(scene.translate_frame_with_children("frame", [25.0, 10.0]), 2);
    let shape_after = scene.element_by_id("shape").unwrap().bounds();
    assert_eq!(shape_after.0, shape_before.0 + 25.0);
    assert_eq!(shape_after.1, shape_before.1 + 10.0);

    assert!(scene.translate_element("shape", [400.0, 0.0]));
    assert!(scene.sync_element_frame_membership("shape"));
    assert_eq!(frame_id(&scene, "shape"), None);
}

#[test]
fn frame_border_is_selectable_without_swallowing_child_interior() {
    let mut scene = scene_with_frame();
    assert_eq!(scene.add_elements_to_frame("frame", &["shape"]), 1);
    let frame = scene.element_by_id("frame").unwrap();
    let shape = scene.element_by_id("shape").unwrap();

    assert!(frame.hit_test([2.0, 100.0]));
    assert!(shape.hit_test([60.0, 60.0]));
    assert!(!frame.hit_test([60.0, 60.0]));
}
