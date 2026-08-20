use elephant_draw_contract::{create_element, font_family_css, layout_text, DrawingTool};
use serde_json::json;

#[test]
fn multiline_text_respects_excalidraw_line_height_and_alignment() {
    let mut text = create_element(DrawingTool::Text, [0.0, 0.0], "text");
    text.text = "alpha\nbeta".to_owned();
    text.font_size = 20.0;
    text.width = 200.0;
    text.height = 100.0;
    text.extra.insert("autoResize".to_owned(), json!(false));
    text.extra.insert("lineHeight".to_owned(), json!(1.25));
    text.extra.insert("textAlign".to_owned(), json!("center"));
    text.extra.insert("verticalAlign".to_owned(), json!("middle"));

    let layout = layout_text(&text);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.line_height_px, 25.0);
    assert!(layout.lines[0].x > 0.0);
    assert!(layout.lines[0].baseline_y > 20.0);
}

#[test]
fn current_excalidraw_font_ids_have_stable_css_fallbacks() {
    for family in [1_u64, 2, 3, 5, 6, 7, 8, 9, 10] {
        assert!(!font_family_css(family).is_empty());
    }
}
