use elephant_draw_contract::{create_element, ArrowEndpoint, Arrowhead, DrawingScene, DrawingTool};
use serde_json::Value;

fn arrow_scene() -> DrawingScene {
    let mut scene = DrawingScene::empty();
    let mut arrow = create_element(DrawingTool::Arrow, [10.0, 20.0], "arrow");
    arrow.points = vec![[0.0, 0.0], [120.0, 40.0]];
    scene.elements.push(arrow);
    scene
}

#[test]
fn current_excalidraw_arrowhead_ids_are_typed_and_round_trip() {
    for arrowhead in Arrowhead::ALL {
        assert_eq!(Arrowhead::from_id(arrowhead.id()), Some(arrowhead));
    }
    assert_eq!(Arrowhead::from_id("crowfoot_many"), None);

    let mut scene = arrow_scene();
    assert!(scene.set_arrowhead(
        "arrow",
        ArrowEndpoint::Start,
        Some(Arrowhead::CircleOutline),
    ));
    assert!(scene.set_arrowhead(
        "arrow",
        ArrowEndpoint::End,
        Some(Arrowhead::CardinalityZeroOrMany),
    ));
    assert_eq!(scene.elements[0].start_arrowhead.as_deref(), Some("circle_outline"));
    assert_eq!(
        scene.elements[0].end_arrowhead.as_deref(),
        Some("cardinality_zero_or_many")
    );

    let raw = scene.to_json().expect("serialize scene");
    let restored = DrawingScene::from_json(&raw).expect("restore scene");
    assert_eq!(
        restored.elements[0].start_arrowhead.as_deref(),
        Some("circle_outline")
    );
    assert_eq!(
        restored.elements[0].end_arrowhead.as_deref(),
        Some("cardinality_zero_or_many")
    );
}

#[test]
fn elbow_arrow_toggle_emits_current_specialized_metadata() {
    let mut scene = arrow_scene();
    assert!(scene.set_elbowed_arrow("arrow", true));
    let arrow = &scene.elements[0];
    assert_eq!(arrow.extra["elbowed"], true);
    assert_eq!(arrow.extra["fixedSegments"], Value::Null);
    assert_eq!(arrow.extra["startIsSpecial"], Value::Null);
    assert_eq!(arrow.extra["endIsSpecial"], Value::Null);
}
