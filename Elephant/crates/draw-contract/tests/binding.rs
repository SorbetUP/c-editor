use elephant_draw_contract::{create_element, ArrowEndpoint, DrawingScene, DrawingTool};
use serde_json::Value;

fn element<'a>(scene: &'a DrawingScene, id: &str) -> &'a elephant_draw_contract::DrawingElement {
    scene.element_by_id(id).expect("element")
}

fn bound_contains(element: &elephant_draw_contract::DrawingElement, id: &str, kind: &str) -> bool {
    element
        .extra
        .get("boundElements")
        .and_then(Value::as_array)
        .is_some_and(|bound| {
            bound.iter().any(|entry| {
                entry.get("id").and_then(Value::as_str) == Some(id)
                    && entry.get("type").and_then(Value::as_str) == Some(kind)
            })
        })
}

#[test]
fn arrow_binding_is_bidirectional_and_roundtrips_current_excalidraw_metadata() {
    let mut scene = DrawingScene::empty();
    scene
        .elements
        .push(create_element(DrawingTool::Rectangle, [0.0, 0.0], "target"));
    scene
        .elements
        .push(create_element(DrawingTool::Arrow, [50.0, 50.0], "arrow"));

    assert!(scene.bind_arrow_endpoint(
        "arrow",
        ArrowEndpoint::End,
        "target",
        0.25,
        8.0,
        Some([0.5, 1.0]),
    ));
    assert!(bound_contains(element(&scene, "target"), "arrow", "arrow"));
    let binding = element(&scene, "arrow")
        .extra
        .get("endBinding")
        .and_then(Value::as_object)
        .expect("binding");
    assert_eq!(binding.get("elementId").and_then(Value::as_str), Some("target"));
    assert_eq!(binding.get("fixedPoint"), Some(&serde_json::json!([0.5, 1.0])));

    let roundtrip = DrawingScene::from_json(&scene.to_json().expect("json")).expect("scene");
    assert!(bound_contains(element(&roundtrip, "target"), "arrow", "arrow"));
}

#[test]
fn labeled_arrow_text_binding_is_reciprocal_and_unbinds_cleanly() {
    let mut scene = DrawingScene::empty();
    scene
        .elements
        .push(create_element(DrawingTool::Arrow, [0.0, 0.0], "arrow"));
    scene
        .elements
        .push(create_element(DrawingTool::Text, [20.0, 20.0], "label"));

    assert!(scene.bind_text_to_container("label", "arrow"));
    assert_eq!(
        element(&scene, "label")
            .extra
            .get("containerId")
            .and_then(Value::as_str),
        Some("arrow")
    );
    assert!(bound_contains(element(&scene, "arrow"), "label", "text"));

    assert!(scene.unbind_text("label"));
    assert_eq!(element(&scene, "label").extra.get("containerId"), Some(&Value::Null));
    assert!(!bound_contains(element(&scene, "arrow"), "label", "text"));
}
