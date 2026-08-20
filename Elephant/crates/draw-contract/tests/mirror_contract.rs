use elephant_draw_contract::{create_element, DrawingScene, DrawingTool, HistoryState};

#[test]
fn element_factory_keeps_excalidraw_reconciliation_metadata() {
    let rectangle = create_element(DrawingTool::Rectangle, [12.0, 34.0], "rect-1");
    assert_eq!(rectangle.kind, "rectangle");
    assert_eq!((rectangle.x, rectangle.y), (12.0, 34.0));
    for key in [
        "roughness",
        "seed",
        "version",
        "versionNonce",
        "index",
        "groupIds",
        "frameId",
        "boundElements",
        "updated",
        "link",
        "locked",
    ] {
        assert!(rectangle.extra.contains_key(key), "missing {key}");
    }
}

#[test]
fn specialized_factory_fields_match_draw() {
    let text = create_element(DrawingTool::Text, [0.0, 0.0], "text-1");
    assert_eq!(
        text.extra
            .get("originalText")
            .and_then(|value| value.as_str()),
        Some("")
    );
    let image = create_element(DrawingTool::Image, [0.0, 0.0], "image-1");
    assert_eq!(
        image.extra.get("status").and_then(|value| value.as_str()),
        Some("pending")
    );
    assert!(image.extra.contains_key("fileId"));
    assert!(image.extra.contains_key("scale"));
    assert!(image.extra.contains_key("crop"));
}

#[test]
fn history_has_real_undo_redo_semantics() {
    let empty = DrawingScene::empty();
    let mut one = empty.clone();
    one.elements
        .push(create_element(DrawingTool::Rectangle, [0.0, 0.0], "a"));
    let mut history = HistoryState::default();
    history.push(empty.clone());
    assert!(history.can_undo());
    let restored = history.undo(one.clone()).expect("undo scene");
    assert_eq!(restored, empty);
    assert!(history.can_redo());
    assert_eq!(history.redo(restored).expect("redo scene"), one);
}

#[test]
fn tool_aliases_match_the_canonical_draw_ids() {
    assert_eq!(DrawingTool::from_id("freedraw"), DrawingTool::Freehand);
    assert_eq!(DrawingTool::from_id("freehand"), DrawingTool::Freehand);
    assert_eq!(DrawingTool::from_id("pencil"), DrawingTool::Freehand);
    assert_eq!(DrawingTool::Arrow.id(), "arrow");
    assert_eq!(DrawingTool::Image.id(), "image");
}
