use elephant_draw_contract::DrawingScene;
use serde_json::{json, Value};

#[test]
fn legacy_elements_gain_current_defaults_without_overwriting_existing_data() {
    let raw = serde_json::to_string(&json!({
        "type": "excalidraw",
        "elements": [
            {"id":"text","type":"text","x":1,"y":2,"text":"hello","customData":{"keep":true}},
            {"id":"free","type":"freedraw","x":3,"y":4,"points":[[0,0],[4,5]],"simulatePressure":false,"customField":"kept"},
            {"id":"arrow","type":"arrow","x":0,"y":0,"points":[[0,0],[10,10]],"elbowed":true,"fixedSegments":[{"index":1,"start":[0,0],"end":[10,0]}]},
            {"id":"frame","type":"frame","x":0,"y":0,"width":100,"height":80,"name":"Board"}
        ],
        "appState": {},
        "files": {}
    }))
    .unwrap();
    let mut scene = DrawingScene::from_json(&raw).expect("legacy scene");

    assert_eq!(scene.normalize_excalidraw_defaults(), 4);
    assert_eq!(scene.elements[0].extra["originalText"], "hello");
    assert_eq!(scene.elements[0].extra["customData"]["keep"], true);
    assert_eq!(scene.elements[1].extra["simulatePressure"], false);
    assert_eq!(scene.elements[1].extra["customField"], "kept");
    assert_eq!(scene.elements[2].extra["startIsSpecial"], Value::Null);
    assert_eq!(scene.elements[3].extra["name"], "Board");
    assert_eq!(scene.normalize_excalidraw_defaults(), 0);
}
