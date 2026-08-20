use muya_core::{
    model::{BlockKind, InlineKind, NodeKind},
    Command, EditorSession, GraphemeCommand, Selection, SelectionPoint, SessionCommand,
};

fn first_text(session: &EditorSession) -> (muya_core::NodeId, u32) {
    session
        .document()
        .nodes
        .values()
        .find_map(|node| match &node.kind {
            NodeKind::Inline(InlineKind::Text { value }) => {
                Some((node.id, value.encode_utf16().count() as u32))
            }
            _ => None,
        })
        .expect("fixture must contain editable text")
}

fn place_caret_at_end(session: &mut EditorSession) {
    let (node, end) = first_text(session);
    session
        .set_selection(
            session.snapshot().revision,
            Selection::collapsed(SelectionPoint {
                node,
                offset_utf16: end,
            }),
        )
        .expect("place caret at end");
}

#[test]
fn enter_then_typing_targets_the_new_muya_paragraph() {
    let mut session = EditorSession::from_markdown("alpha");
    place_caret_at_end(&mut session);

    let paragraph = session
        .dispatch(
            session.snapshot().revision,
            SessionCommand::Core(Command::InsertParagraph),
        )
        .expect("insert paragraph");

    let blocks = session
        .document()
        .children(session.document().root)
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 2, "Enter must create two top-level blocks");
    assert!(blocks.iter().all(|node| matches!(
        node.kind,
        NodeKind::Block(BlockKind::Paragraph)
    )));

    session
        .dispatch(
            paragraph.revision,
            SessionCommand::Core(Command::InsertText("beta".into())),
        )
        .expect("type into new paragraph");
    assert_eq!(session.snapshot().markdown, "alpha\n\nbeta");
}

#[test]
fn backspace_removes_one_visible_extended_grapheme() {
    let mut session = EditorSession::from_markdown("a👨‍👩‍👧‍👦");
    place_caret_at_end(&mut session);

    session
        .dispatch(
            session.snapshot().revision,
            SessionCommand::Grapheme(GraphemeCommand::DeleteBackward),
        )
        .expect("delete grapheme");

    assert_eq!(session.snapshot().markdown, "a");
}

#[test]
fn committed_ime_composition_can_be_undone() {
    let mut session = EditorSession::from_markdown("x");
    place_caret_at_end(&mut session);

    let revision = session.snapshot().revision;
    session
        .dispatch(revision, SessionCommand::BeginComposition)
        .expect("begin composition");
    let update = session
        .dispatch(revision, SessionCommand::UpdateComposition("にほん".into()))
        .expect("update composition");
    let committed = session
        .dispatch(update.revision, SessionCommand::CommitComposition)
        .expect("commit composition");
    assert_eq!(session.snapshot().markdown, "xにほん");
    assert!(committed.can_undo);

    session
        .dispatch(committed.revision, SessionCommand::Undo)
        .expect("undo composition");
    assert_eq!(session.snapshot().markdown, "x");
}
