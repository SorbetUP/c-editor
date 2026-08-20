use muya_core::{
    model::{BlockKind, NodeKind},
    parse_markdown, to_markdown,
};

#[test]
fn tauri_special_blocks_become_structured_rust_nodes_and_round_trip() {
    let markdown = "$$\nx + y\n$$\n\n[^src]: cited **source**\n\n[openai]: https://openai.com \"Home\"\n\n<div>block</div>";
    let document = parse_markdown(markdown);

    assert_eq!(to_markdown(&document), markdown);
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::MathBlock)
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) if label == "src"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "openai"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::HtmlBlock)
    )));
}

#[test]
fn gitlab_math_is_recognized_but_normal_fences_stay_literal() {
    let markdown = "```math\nx^2\n```\n\n```rust\n$not_math$ :grinning: <kbd>x</kbd>\n```";
    let document = parse_markdown(markdown);

    assert_eq!(
        to_markdown(&document),
        "$$\nx^2\n$$\n\n```rust\n$not_math$ :grinning: <kbd>x</kbd>\n```"
    );
    let blocks = document.children(document.root).collect::<Vec<_>>();
    assert!(matches!(blocks[0].kind, NodeKind::Block(BlockKind::MathBlock)));
    assert!(matches!(
        blocks[1].kind,
        NodeKind::Block(BlockKind::CodeBlock { .. })
    ));
}

#[test]
fn inline_only_html_stays_inline_in_a_paragraph() {
    let markdown = "before <kbd>Ctrl</kbd> after";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
    assert!(matches!(
        document.children(document.root).next().unwrap().kind,
        NodeKind::Block(BlockKind::Paragraph)
    ));
}