use muya_core::{
    model::{InlineKind, InlineSyntax, NodeKind, ReferenceStyle},
    parse_markdown, to_markdown,
};

fn is_bare_link(node: &muya_core::model::Node) -> bool {
    matches!(node.kind, NodeKind::Inline(InlineKind::Link { .. }))
        && matches!(node.inline_syntax, Some(InlineSyntax::BareAutoLink { .. }))
}

fn is_reference(node: &muya_core::model::Node) -> bool {
    matches!(
        node.kind,
        NodeKind::Inline(InlineKind::Link { .. } | InlineKind::Image { .. })
    ) && matches!(node.inline_syntax, Some(InlineSyntax::Reference { .. }))
}

#[test]
fn tauri_inline_extensions_become_structured_rust_nodes_and_round_trip() {
    let markdown = "<https://example.com> <dev@example.com> https://bare.example www.example.com bare@example.com :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1]";
    let document = parse_markdown(markdown);

    assert_eq!(to_markdown(&document), markdown);
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::AutoLink { .. })
    )));
    assert_eq!(document.nodes.values().filter(|node| is_bare_link(node)).count(), 3);
    assert!(document.nodes.values().any(|node| matches!(
        (&node.kind, &node.inline_syntax),
        (
            NodeKind::Inline(InlineKind::Link { destination, .. }),
            Some(InlineSyntax::BareAutoLink { text })
        ) if destination == "http://www.example.com" && text == "www.example.com"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        (&node.kind, &node.inline_syntax),
        (
            NodeKind::Inline(InlineKind::Link { destination, .. }),
            Some(InlineSyntax::BareAutoLink { text })
        ) if destination == "mailto:bare@example.com" && text == "bare@example.com"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::FootnoteReference { .. })
    )));
}

#[test]
fn tauri_reference_links_and_images_resolve_without_losing_source_form() {
    let markdown = "[full **label**][Target ID] [collapsed][] [shortcut] ![logo][image]\n\n[target   id]: https://example.com \"Home\"\n\n[collapsed]: https://collapsed.example\n\n[shortcut]: https://shortcut.example\n\n[image]: asset.png";
    let document = parse_markdown(markdown);

    assert_eq!(to_markdown(&document), markdown);
    assert!(document.nodes.values().any(|node| matches!(
        (&node.kind, &node.inline_syntax),
        (
            NodeKind::Inline(InlineKind::Link { destination, title }),
            Some(InlineSyntax::Reference { reference, style })
        ) if destination == "https://example.com"
            && title.as_deref() == Some("Home")
            && reference == "Target ID"
            && *style == ReferenceStyle::Full
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.inline_syntax,
        Some(InlineSyntax::Reference { style: ReferenceStyle::Collapsed, .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.inline_syntax,
        Some(InlineSyntax::Reference { style: ReferenceStyle::Shortcut, .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
        (&node.kind, &node.inline_syntax),
        (
            NodeKind::Inline(InlineKind::Image { source, .. }),
            Some(InlineSyntax::Reference { .. })
        ) if source == "asset.png"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::Strong)
    )));
}

#[test]
fn unresolved_reference_syntax_is_preserved_as_text() {
    let markdown = "[missing][id] ![missing][image]";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
    assert!(!document.nodes.values().any(is_reference));
}

#[test]
fn code_and_links_keep_precedence_over_extension_markers() {
    let markdown = "`$literal$ https://literal.example` [https://label.example](https://example.com?q=$x$) ![alt](image^2^.png)";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
    assert_eq!(document.nodes.values().filter(|node| is_bare_link(node)).count(), 0);
}
