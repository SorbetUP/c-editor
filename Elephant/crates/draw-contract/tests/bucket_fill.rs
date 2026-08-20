use elephant_draw_contract::{
    compute_bucket_fill, BucketFillOptions, BucketFillPlacement, DrawingScene,
};
use serde_json::json;

fn scene(elements: serde_json::Value) -> DrawingScene {
    DrawingScene::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "elements": elements,
            "appState": {},
            "files": {}
        }))
        .unwrap(),
    )
    .unwrap()
}

fn area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        .abs()
        / 2.0
}

#[test]
fn fills_a_closed_shape_and_keeps_its_outline_above() {
    let scene = scene(json!([
        {"id":"box","type":"rectangle","x":10,"y":20,"width":100,"height":80,"backgroundColor":"transparent","opacity":100}
    ]));
    let result = compute_bucket_fill(&scene, [50.0, 50.0], BucketFillOptions::default()).unwrap();
    assert_eq!(result.owner_id.as_deref(), Some("box"));
    assert!(result.boundary_element_ids.is_empty());
    assert_eq!(result.insertion.placement, BucketFillPlacement::Below);
    assert_eq!(result.insertion.element_id, "box");
    assert!(area(&result.scene_points) > 7_900.0);
    assert_eq!(result.scene_points.first(), result.scene_points.last());
}

#[test]
fn fills_ownerless_regions_built_from_open_lines() {
    let scene = scene(json!([
        {"id":"top","type":"line","x":0,"y":0,"width":100,"height":0,"points":[[0,0],[100,0]]},
        {"id":"right","type":"line","x":100,"y":0,"width":0,"height":100,"points":[[0,0],[0,100]]},
        {"id":"bottom","type":"line","x":0,"y":100,"width":100,"height":0,"points":[[0,0],[100,0]]},
        {"id":"left","type":"line","x":0,"y":0,"width":0,"height":100,"points":[[0,0],[0,100]]}
    ]));
    let result = compute_bucket_fill(&scene, [50.0, 50.0], BucketFillOptions::default()).unwrap();
    assert_eq!(result.owner_id, None);
    assert_eq!(result.boundary_element_ids.len(), 4);
    for id in ["top", "right", "bottom", "left"] {
        assert!(result.boundary_element_ids.iter().any(|candidate| candidate == id));
    }
    assert!(area(&result.scene_points) > 9_900.0);
}

#[test]
fn chooses_the_smallest_face_after_intersections_split_the_region() {
    let scene = scene(json!([
        {"id":"box","type":"rectangle","x":0,"y":0,"width":100,"height":100},
        {"id":"divider","type":"line","x":50,"y":0,"width":0,"height":100,"points":[[0,0],[0,100]]}
    ]));
    let left = compute_bucket_fill(&scene, [25.0, 50.0], BucketFillOptions::default()).unwrap();
    let right = compute_bucket_fill(&scene, [75.0, 50.0], BucketFillOptions::default()).unwrap();
    assert!(left.boundary_element_ids.iter().any(|id| id == "divider"));
    assert!(right.boundary_element_ids.iter().any(|id| id == "divider"));
    assert!((area(&left.scene_points) - 5_000.0).abs() < 10.0);
    assert!((area(&right.scene_points) - 5_000.0).abs() < 10.0);
}

#[test]
fn preserves_disconnected_islands_as_keyhole_holes() {
    let scene = scene(json!([
        {"id":"outer","type":"rectangle","x":0,"y":0,"width":200,"height":160},
        {"id":"island","type":"rectangle","x":70,"y":50,"width":60,"height":60}
    ]));
    let result = compute_bucket_fill(&scene, [20.0, 20.0], BucketFillOptions::default()).unwrap();
    assert_eq!(result.owner_id.as_deref(), Some("outer"));
    assert!(result.boundary_element_ids.iter().any(|id| id == "island"));
    assert!(result.scene_points.len() > 8, "hole must be spliced through a keyhole path");
    let net = area(&result.scene_points);
    assert!((net - (200.0 * 160.0 - 60.0 * 60.0)).abs() < 25.0);
}
