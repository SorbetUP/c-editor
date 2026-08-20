use muya_core::{
    model::{BlockKind, NodeKind},
    parse_markdown, to_markdown,
};

#[test]
fn freya_dependency_preserves_nested_tauri_lists_as_structure() {
    let markdown = "- parent\n  - child\n    3. grandchild\n- sibling";
    let document = parse_markdown(markdown);
    let roots = document.children(document.root).collect::<Vec<_>>();

    assert_eq!(roots.len(), 1, "nested list must not leak root siblings");
    let first_item = document
        .children(roots[0].id)
        .next()
        .expect("first list item");
    assert!(document.children(first_item.id).any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::List { .. })
    )));

    let saved = to_markdown(&document);
    let reparsed = parse_markdown(&saved);
    let first_root = reparsed
        .children(reparsed.root)
        .next()
        .expect("reparsed root list");
    let reparsed_item = reparsed
        .children(first_root.id)
        .next()
        .expect("reparsed first item");
    assert!(reparsed.children(reparsed_item.id).any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::List { .. })
    )));
}

#[test]
fn freya_dependency_preserves_blockquote_block_children() {
    let markdown = "> # Heading\n>\n> - item\n>   - nested";
    let document = parse_markdown(markdown);
    let quote = document
        .children(document.root)
        .next()
        .expect("root blockquote");

    assert!(matches!(
        quote.kind,
        NodeKind::Block(BlockKind::BlockQuote)
    ));
    let children = document.children(quote.id).collect::<Vec<_>>();
    assert!(children.iter().any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::Heading { level: 1 })
    )));
    assert!(children.iter().any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::List { .. })
    )));

    let saved = to_markdown(&document);
    let reparsed = parse_markdown(&saved);
    let reparsed_quote = reparsed
        .children(reparsed.root)
        .next()
        .expect("reparsed blockquote");
    assert!(reparsed.children(reparsed_quote.id).any(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::Heading { level: 1 })
    )));
}

#[test]
fn freya_dependency_recognizes_tauri_indented_code() {
    let markdown = "    alpha\n    beta\n\nbody";
    let document = parse_markdown(markdown);
    let roots = document.children(document.root).collect::<Vec<_>>();

    assert!(matches!(
        roots.first().map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::CodeBlock {
            language: None,
            fenced: false
        }))
    ));
    assert!(to_markdown(&document).starts_with("    alpha\n    beta"));
}

#[test]
fn freya_dependency_promotes_only_tauri_diagram_languages() {
    for language in ["mermaid", "flowchart", "sequence", "plantuml", "vega-lite"] {
        let markdown = format!("```{language}\nA -> B\n```");
        let document = parse_markdown(&markdown);
        assert!(matches!(
            document
                .children(document.root)
                .next()
                .map(|node| &node.kind),
            Some(NodeKind::Block(BlockKind::Diagram { language: actual }))
                if actual == language
        ));
        assert_eq!(to_markdown(&document), markdown);
    }

    let rust = parse_markdown("```rust\nfn main() {}\n```");
    assert!(matches!(
        rust.children(rust.root).next().map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::CodeBlock { .. }))
    ));
}
