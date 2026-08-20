use muya_core::{model::{BlockKind, NodeKind}, EditorSession};

#[test]
fn consecutive_tauri_reference_definitions_stay_independent_and_round_trip() {
    let markdown = "[one]: https://one.example\n[two]: <https://two.example> \"Two\"";
    let session = EditorSession::from_markdown(markdown);
    let blocks = session
        .document()
        .children(session.document().root)
        .collect::<Vec<_>>();

    assert_eq!(blocks.len(), 2);
    assert!(matches!(
        &blocks[0].kind,
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "one"
    ));
    assert!(matches!(
        &blocks[1].kind,
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "two"
    ));
    assert_eq!(
        session.snapshot().markdown,
        "[one]: https://one.example\n\n[two]: <https://two.example> \"Two\""
    );
}

#[test]
fn reference_definition_continuation_title_matches_tauri_canonical_serialization() {
    let session = EditorSession::from_markdown(
        "[target]:\n  https://reference.example\n  \"Reference title\"",
    );

    assert_eq!(
        session.snapshot().markdown,
        "[target]: https://reference.example \"Reference title\""
    );
}

#[test]
fn footnote_definitions_are_not_reclassified_as_link_reference_definitions() {
    let session = EditorSession::from_markdown("[^src]: cited source");
    let block = session
        .document()
        .children(session.document().root)
        .next()
        .expect("footnote block");

    assert!(matches!(
        &block.kind,
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) if label == "src"
    ));
}
