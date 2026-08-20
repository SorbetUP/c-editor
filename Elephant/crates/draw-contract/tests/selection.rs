use elephant_draw::{create_element, DrawingScene, DrawingTool, SelectionMode};
use serde_json::Value;

fn scene() -> DrawingScene {
    let mut scene = DrawingScene::empty();
    let mut left = create_element(DrawingTool::Rectangle, [10.0, 10.0], "left");
    left.width = 40.0;
    left.height = 40.0;
    let mut center = create_element(DrawingTool::Ellipse, [80.0, 20.0], "center");
    center.width = 60.0;
    center.height = 40.0;
    let mut right = create_element(DrawingTool::Diamond, [180.0, 10.0], "right");
    right.width = 40.0;
    right.height = 40.0;
    scene.elements.extend([left, center, right]);
    scene
}

#[test]
fn rectangle_selection_distinguishes_touching_from_contained() {
    let scene = scene();
    let touching = scene.select_in_rect([0.0, 0.0], [100.0, 60.0], SelectionMode::Touching);
    assert!(touching.contains("left"));
    assert!(touching.contains("center"));
    assert!(!touching.contains("right"));

    let contained = scene.select_in_rect([0.0, 0.0], [100.0, 60.0], SelectionMode::Contained);
    assert!(contained.contains("left"));
    assert!(!contained.contains("center"));
}

#[test]
fn lasso_selection_uses_polygon_geometry_not_only_bounding_boxes() {
    let scene = scene();
    let lasso = [[0.0, 0.0], [70.0, 0.0], [70.0, 70.0], [0.0, 70.0]];
    let selection = scene.select_in_lasso(&lasso, SelectionMode::Touching);
    assert_eq!(selection.ids().collect::<Vec<_>>(), vec!["left"]);
}

#[test]
fn locked_and_deleted_elements_are_not_box_or_lasso_selected() {
    let mut scene = scene();
    scene.elements[0]
        .extra
        .insert("locked".to_owned(), Value::Bool(true));
    scene.elements[1].is_deleted = true;
    let selection = scene.select_in_rect([0.0, 0.0], [300.0, 100.0], SelectionMode::Touching);
    assert_eq!(selection.ids().collect::<Vec<_>>(), vec!["right"]);
}

#[test]
fn batch_translation_mutates_only_the_selected_elements_and_versions() {
    let mut scene = scene();
    let selection = scene.select_in_rect([0.0, 0.0], [150.0, 100.0], SelectionMode::Contained);
    let left_version = scene.elements[0].extra["version"].as_u64().unwrap();
    let right_before = (scene.elements[2].x, scene.elements[2].y);

    assert_eq!(scene.translate_selection(&selection, [25.0, -5.0]), 2);
    assert_eq!((scene.elements[0].x, scene.elements[0].y), (35.0, 5.0));
    assert_eq!((scene.elements[1].x, scene.elements[1].y), (105.0, 15.0));
    assert_eq!((scene.elements[2].x, scene.elements[2].y), right_before);
    assert!(scene.elements[0].extra["version"].as_u64().unwrap() > left_version);
}

#[test]
fn selection_bounds_include_rotation() {
    let mut scene = DrawingScene::empty();
    let mut rectangle = create_element(DrawingTool::Rectangle, [0.0, 0.0], "rotated");
    rectangle.width = 100.0;
    rectangle.height = 20.0;
    rectangle.angle = std::f32::consts::FRAC_PI_2;
    scene.elements.push(rectangle);
    let selection =
        scene.select_in_rect([-100.0, -100.0], [200.0, 200.0], SelectionMode::Contained);
    let bounds = selection.bounds(&scene).expect("selection bounds");
    assert!((bounds.2 - 20.0).abs() < 0.001);
    assert!((bounds.3 - 100.0).abs() < 0.001);
}
