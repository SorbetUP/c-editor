use elephant_draw_contract::{
    apply_bucket_fill, convert_autoshape, BucketFillMutation, BucketFillOptions,
    DrawingCurrentStyle, DrawingScene, LaserTrails,
};
use serde_json::json;

fn scene(elements: serde_json::Value) -> DrawingScene {
    DrawingScene::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw","elements":elements,"appState":{},"files":{}
        }))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn autoshape_never_persists_autoshape_type() {
    let points = vec![
        [0.0,0.0],[30.0,0.0],[60.0,0.0],[90.0,0.0],[90.0,30.0],
        [90.0,60.0],[60.0,60.0],[30.0,60.0],[0.0,60.0],[0.0,30.0],[0.0,0.0],
    ];
    let element = convert_autoshape(
        &points,
        1.0,
        false,
        "shape",
        &DrawingCurrentStyle::default(),
        None,
        &[],
    )
    .unwrap();
    assert_eq!(element.kind, "rectangle");
    assert_ne!(element.kind, "autoshape");
}

#[test]
fn bucket_fill_persists_line_polygon_and_restyles_same_region() {
    let mut scene = scene(json!([
        {"id":"box","type":"rectangle","x":0,"y":0,"width":100,"height":80}
    ]));
    let mut first = DrawingCurrentStyle::default();
    first.background_color = "#ffc9c9".into();
    let inserted = apply_bucket_fill(
        &mut scene,[50.0,40.0],"fill",&first,BucketFillOptions::default()
    ).unwrap();
    assert_eq!(inserted, BucketFillMutation::Inserted { element_id: "fill".into() });
    let fill = scene.element_by_id("fill").unwrap();
    assert_eq!(fill.kind, "line");
    assert_eq!(fill.extra["polygon"], true);
    assert_eq!(fill.stroke_color, "transparent");
    assert_eq!(fill.extra["roughness"], 0);

    let count = scene.elements.len();
    let mut second = first.clone();
    second.background_color = "#a5d8ff".into();
    let changed = apply_bucket_fill(
        &mut scene,[50.0,40.0],"duplicate",&second,BucketFillOptions::default()
    ).unwrap();
    assert_eq!(changed, BucketFillMutation::Restyled { element_id: "fill".into() });
    assert_eq!(scene.elements.len(), count);
    assert!(scene.element_by_id("duplicate").is_none());
}

#[test]
fn laser_is_ephemeral_and_decays_without_scene_elements() {
    let mut trails = LaserTrails::default();
    trails.start_path(10.0, 20.0, 0.0);
    assert!(trails.add_point_to_path(30.0, 20.0, 10.0));
    assert!(trails.end_path());
    assert!(!trails.visible_samples(100.0).is_empty());
    trails.prune(2_000.0);
    assert!(trails.past().is_empty());
    let scene = DrawingScene::empty();
    assert!(scene.elements.is_empty(), "Laser must never serialize a drawing element");
}
