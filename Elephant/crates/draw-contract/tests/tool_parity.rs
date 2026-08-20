use elephant_draw_contract::ToolType;

#[test]
fn current_excalidraw_tool_ids_are_complete_and_ordered() {
    let ids = ToolType::ALL.map(ToolType::id);
    assert_eq!(
        ids,
        [
            "selection",
            "lasso",
            "rectangle",
            "diamond",
            "ellipse",
            "arrow",
            "line",
            "freedraw",
            "text",
            "image",
            "eraser",
            "hand",
            "frame",
            "magicframe",
            "embeddable",
            "laser",
            "autoshape",
            "bucketfill",
        ]
    );
}

#[test]
fn current_excalidraw_transient_tool_shortcuts_match() {
    assert!(ToolType::Laser.matches_shortcut("k", false));
    assert!(ToolType::Autoshape.matches_shortcut("x", true));
    assert!(!ToolType::Autoshape.matches_shortcut("x", false));
    assert!(ToolType::BucketFill.matches_shortcut("b", false));
    assert!(ToolType::Selection.matches_shortcut("1", false));
    assert!(ToolType::FreeDraw.matches_shortcut("7", false));
    assert!(ToolType::Eraser.matches_shortcut("0", false));
}
