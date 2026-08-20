use muya_core::{
    model::{InlineKind, NodeKind, ReferenceStyle},
    parse_markdown, to_markdown,
};

#[test]
fn tauri_inline_extensions_become_structured_rust_nodes_and_round_trip() {
    let markdown = "<https://example.com> <dev@example.com> https://bare.example www.example.com bare@example.com :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1]";
    let document = parse_markdown(markdown);

    assert_eq!(to_markdown(&document), markdown);
    assert!(document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::AutoLink { .. })
    )));
    assert_eq!(
        document
            .nodes
            .values()
            .filter(|node| matches!(node.kind, NodeKind::Inline(InlineKind::BareAutoLink { .. })))
            .count(),
        3
    );
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::BareAutoLink { destination, text })
            if destination == "http://www.example.com" && text == "www.example.com"
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::BareAutoLink { destination, text })
            if destination == "mailto:bare@example.com" && text == "bare@example.com"
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
        &node.kind,
        NodeKind::Inline(InlineKind::ReferenceLink { destination, title, reference, style })
            if destination == "https://example.com"
                && title.as_deref() == Some("Home")
                && reference == "Target ID"
                && *style == ReferenceStyle::Full
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::ReferenceLink { style, .. })
            if *style == ReferenceStyle::Collapsed
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::ReferenceLink { style, .. })
            if *style == ReferenceStyle::Shortcut
    )));
    assert!(document.nodes.values().any(|node| matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::ReferenceImage { source, .. }) if source == "asset.png"
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
    assert!(!document.nodes.values().any(|node| matches!(
        node.kind,
        NodeKind::Inline(InlineKind::ReferenceLink { .. } | InlineKind::ReferenceImage { .. })
    )));
}

#[test]
fn code_and_links_keep_precedence_over_extension_markers() {
    let markdown = "`$literal$ https://literal.example` [https://label.example](https://example.com?q=$x$) ![alt](image^2^.png)";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
    assert_eq!(
        document
            .nodes
            .values()
            .filter(|node| matches!(node.kind, NodeKind::Inline(InlineKind::BareAutoLink { .. })))
            .count(),
        0
    );
}
