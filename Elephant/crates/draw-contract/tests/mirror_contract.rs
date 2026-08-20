use elephant_draw_contract::{create_element, DrawingScene, DrawingTool, HistoryState};

#[test]
fn element_factory_keeps_excalidraw_reconciliation_metadata() {
    let rectangle = create_element(DrawingTool::Rectangle, [12.0, 34.0], "rect-1");
    assert_eq!(rectangle.kind, "rectangle");
    assert_eq!((rectangle.x, rectangle.y), (12.0, 34.0));
    assert_eq!(rectangle.stroke_color, "#1e1e1e");
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
    assert_eq!(image.stroke_color, "transparent");
    assert_eq!(
        image.extra.get("status").and_then(|value| value.as_str()),
        Some("pending")
    );
    assert!(image.extra.contains_key("fileId"));
    assert_eq!(image.extra.get("scale"), Some(&serde_json::json!([1, 1])));
    assert_eq!(image.extra.get("crop"), Some(&serde_json::Value::Null));

    let freedraw = create_element(DrawingTool::Freehand, [0.0, 0.0], "free-1");
    assert_eq!(freedraw.extra.get("pressures"), Some(&serde_json::json!([])));
    assert_eq!(
        freedraw
            .extra
            .get("simulatePressure")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        freedraw.extra.get("strokeOptions"),
        Some(&serde_json::json!({"variability": "variable", "streamline": 0.5}))
    );

    let line = create_element(DrawingTool::Line, [0.0, 0.0], "line-1");
    assert_eq!(line.extra.get("polygon"), Some(&serde_json::json!(false)));
    assert_eq!(line.extra.get("startBinding"), Some(&serde_json::Value::Null));
    assert_eq!(line.extra.get("endBinding"), Some(&serde_json::Value::Null));

    let arrow = create_element(DrawingTool::Arrow, [0.0, 0.0], "arrow-1");
    assert_eq!(arrow.extra.get("elbowed"), Some(&serde_json::json!(false)));
    assert_eq!(arrow.end_arrowhead.as_deref(), Some("arrow"));

    let frame = create_element(DrawingTool::Frame, [0.0, 0.0], "frame-1");
    assert_eq!(frame.kind, "frame");
    assert_eq!(frame.stroke_color, "#bbb");
    assert_eq!(frame.extra.get("name"), Some(&serde_json::Value::Null));
    assert_eq!(frame.extra.get("roughness"), Some(&serde_json::json!(0)));
    for key in ["version", "versionNonce", "index", "frameId", "locked"] {
        assert!(frame.extra.contains_key(key), "Frame missing {key}");
    }
}

#[test]
fn missing_fields_restore_with_the_same_defaults_as_draw() {
    let raw = r#"{"type":"excalidraw","elements":[{"id":"r","type":"rectangle","x":0,"y":0,"width":10,"height":10}],"files":{}}"#;
    let scene = DrawingScene::from_json(raw).expect("scene");
    let element = &scene.elements[0];
    assert_eq!(element.stroke_color, "#1e1e1e");
    assert_eq!(element.background_color, "transparent");
    assert_eq!(element.stroke_width, 2.0);
    assert_eq!(element.stroke_style, "solid");
    assert_eq!(element.fill_style, "solid");
    assert_eq!(element.font_size, 20.0);
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
    assert_eq!(DrawingTool::from_id("frame"), DrawingTool::Frame);
    assert_eq!(DrawingTool::Frame.id(), "frame");
}
