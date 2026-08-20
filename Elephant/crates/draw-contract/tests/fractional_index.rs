use elephant_draw_contract::{
    create_element, generate_key_between, generate_n_keys_between, validate_order_key, DrawingScene,
    DrawingTool,
};
use serde_json::Value;

fn persisted_index(scene: &DrawingScene, id: &str) -> String {
    scene
        .element_by_id(id)
        .and_then(|element| element.extra.get("index"))
        .and_then(Value::as_str)
        .expect("persisted fractional index")
        .to_owned()
}

#[test]
fn generated_keys_match_excalidraw_order_contract() {
    let between = generate_key_between(Some("a0"), Some("a1")).expect("key between");
    assert!(validate_order_key(&between));
    assert!("a0" < between.as_str() && between.as_str() < "a1");

    let keys = generate_n_keys_between(None, None, 205).expect("205 ordered keys");
    assert_eq!(keys.len(), 205);
    assert!(keys.iter().all(|key| validate_order_key(key)));
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn layer_reordering_keeps_array_and_persisted_indices_in_sync() {
    let mut scene = DrawingScene::empty();
    for id in ["back", "middle", "front"] {
        scene
            .elements
            .push(create_element(DrawingTool::Rectangle, [0.0, 0.0], id));
    }
    assert_eq!(scene.sync_fractional_indices(), 3);
    assert_eq!(persisted_index(&scene, "back"), "a0");
    assert!(scene.bring_element_to_front("back"));
    assert_eq!(
        scene
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>(),
        vec!["middle", "front", "back"]
    );
    let indices = scene
        .elements
        .iter()
        .map(|element| element.extra["index"].as_str().expect("index"))
        .collect::<Vec<_>>();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}
