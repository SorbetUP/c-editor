use elephant_draw_contract::{DrawingScene, SelectionSet};
use serde_json::json;

fn fixture() -> DrawingScene {
    DrawingScene::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[
                {"id":"frame","type":"frame","x":10,"y":10,"width":250,"height":180,"groupIds":[],"boundElements":[]},
                {"id":"box","type":"rectangle","x":30,"y":30,"width":90,"height":60,"frameId":"frame","groupIds":["g1"],"boundElements":[{"type":"text","id":"label"},{"type":"arrow","id":"arrow"}]},
                {"id":"label","type":"text","x":40,"y":45,"width":60,"height":20,"text":"hello","containerId":"box","frameId":"frame","groupIds":["g1"]},
                {"id":"arrow","type":"arrow","x":120,"y":60,"points":[[0,0],[90,40]],"frameId":"frame","startBinding":{"elementId":"box","focus":0,"gap":1,"fixedPoint":[0.5,0.5],"mode":"orbit"},"endBinding":{"elementId":"outside","focus":0,"gap":1,"fixedPoint":[0.5,0.5],"mode":"orbit"}},
                {"id":"image","type":"image","x":40,"y":110,"width":50,"height":40,"frameId":"frame","fileId":"file-a"},
                {"id":"outside","type":"ellipse","x":400,"y":30,"width":50,"height":50,"boundElements":[{"type":"arrow","id":"arrow"}]}
            ],
            "appState":{},
            "files":{"file-a":{"id":"file-a","mimeType":"image/png","dataURL":"data:image/png;base64,AAAA"}}
        }))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn copying_a_frame_includes_children_bound_text_and_binary_files() {
    let scene = fixture();
    let selection = SelectionSet::from_ids(["frame".to_owned()]);
    let fragment = scene.copy_selection_fragment(&selection);
    let ids = fragment
        .elements
        .iter()
        .map(|element| element.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["frame", "box", "label", "arrow", "image"]);
    assert!(fragment.files.get("file-a").is_some());
    assert!(fragment.elements.iter().all(|element| element.id != "outside"));
}

#[test]
fn paste_remaps_internal_references_drops_external_bindings_and_offsets_geometry() {
    let mut scene = fixture();
    let fragment = scene.copy_selection_fragment(&SelectionSet::from_ids(["frame".to_owned()]));
    let pasted = scene.paste_fragment(&fragment, [20.0, 30.0], "paste1");
    assert_eq!(pasted.len(), 5);
    assert_eq!(scene.element_by_id("paste1-frame").map(|e| (e.x, e.y)), Some((30.0, 40.0)));
    let box_element = scene.element_by_id("paste1-box").unwrap();
    assert_eq!(box_element.extra["frameId"], "paste1-frame");
    let arrow = scene.element_by_id("paste1-arrow").unwrap();
    assert_eq!(arrow.extra["startBinding"]["elementId"], "paste1-box");
    assert!(arrow.extra["endBinding"].is_null());
    assert_eq!(scene.files["file-a"]["mimeType"], "image/png");
}

#[test]
fn repeated_paste_with_unique_namespaces_never_reuses_element_or_group_ids() {
    let mut scene = fixture();
    let fragment = scene.copy_selection_fragment(&SelectionSet::from_ids(["box".to_owned()]));
    let first = scene.paste_fragment(&fragment, [10.0, 10.0], "copy1");
    let second = scene.paste_fragment(&fragment, [20.0, 20.0], "copy2");
    assert!(first.ids().all(|id| !second.contains(id)));
    let first_group = scene.element_by_id("copy1-box").unwrap().extra["groupIds"][0]
        .as_str()
        .unwrap();
    let second_group = scene.element_by_id("copy2-box").unwrap().extra["groupIds"][0]
        .as_str()
        .unwrap();
    assert_ne!(first_group, second_group);
}

#[test]
fn fragment_json_round_trip_preserves_unknown_excalidraw_fields() {
    let scene = fixture();
    let fragment = scene.copy_selection_fragment(&SelectionSet::from_ids(["arrow".to_owned()]));
    let raw = serde_json::to_string(&fragment).unwrap();
    let decoded: elephant_draw_contract::SceneFragment = serde_json::from_str(&raw).unwrap();
    assert_eq!(decoded, fragment);
    assert_eq!(decoded.elements[0].extra["startBinding"]["mode"], "orbit");
}
