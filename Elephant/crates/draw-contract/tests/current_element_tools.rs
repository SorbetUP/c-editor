use elephant_draw_contract::{create_element, DrawingTool};

#[test]
fn magic_frame_and_embeddable_match_current_excalidraw_constructors() {
    let magic = create_element(DrawingTool::MagicFrame, [12.0, 34.0], "magic");
    assert_eq!(magic.kind, "magicframe");
    assert_eq!(magic.extra.get("name"), Some(&serde_json::Value::Null));
    assert_eq!(magic.extra["roughness"], 0);
    assert_eq!(magic.stroke_color, "#bbb");

    let embed = create_element(DrawingTool::Embeddable, [20.0, 30.0], "embed");
    assert_eq!(embed.kind, "embeddable");
    assert_eq!((embed.x, embed.y), (20.0, 30.0));
    for key in ["version", "versionNonce", "groupIds", "frameId", "boundElements", "link"] {
        assert!(embed.extra.contains_key(key), "missing Excalidraw base field {key}");
    }

    assert_eq!(DrawingTool::from_id("iframe"), DrawingTool::Embeddable);
}
