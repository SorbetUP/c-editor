use muya_core::{
    model::{InlineKind, NodeKind},
    parse_markdown, to_markdown,
};

#[test]
fn tauri_inline_extensions_become_structured_rust_nodes_and_round_trip() {
    let markdown = "<https://example.com> <dev@example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1]";
    let document = parse_markdown(markdown);

    assert_eq!(to_markdown(&document), markdown);
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::AutoLink { .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::Emoji { shortcode, value })
            if shortcode == "grinning" && value == "😀"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::InlineHtml { .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::InlineMath { .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::Superscript)
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::Subscript)
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::FootnoteReference { .. })
    )));
}

#[test]
fn code_and_links_keep_precedence_over_extension_markers() {
    let markdown = "`$literal$` [label](https://example.com?q=$x$) ![alt](image^2^.png)";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
}