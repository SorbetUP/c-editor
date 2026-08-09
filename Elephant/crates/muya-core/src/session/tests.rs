use super::helpers::first_text_descendant;
use super::*;
use crate::edit::{Command, EditError};
use crate::features::TableNavigationCommand;
use crate::model::NodeId;
use crate::selection::{Selection, SelectionPoint};

fn text_id(session: &EditorSession) -> NodeId {
  first_text_descendant(session.document(), session.document().root).unwrap()
}

#[test]
fn creates_an_editable_target_for_an_empty_document() {
  let session = EditorSession::from_markdown("");
  assert!(session
    .document()
    .node(session.selection().focus.node)
    .is_some());
  assert_eq!(session.snapshot().markdown, "");
}

#[test]
fn rejects_commands_from_a_stale_revision() {
  let mut session = EditorSession::from_markdown("abc");
  let error = session
    .dispatch(4, SessionCommand::Core(Command::InsertText("x".into())))
    .unwrap_err();
  assert_eq!(
    error,
    EditError::RevisionMismatch {
      expected: 4,
      actual: 0,
    }
  );
}

#[test]
fn dispatches_edits_and_exposes_forward_and_inverse_patches() {
  let mut session = EditorSession::from_markdown("abc");
  let text = text_id(&session);
  session
    .set_selection(
      0,
      Selection::collapsed(SelectionPoint {
        node: text,
        offset_utf16: 3,
      }),
    )
    .unwrap();
  let update = session
    .dispatch(0, SessionCommand::Core(Command::InsertText("x".into())))
    .unwrap();
  assert_eq!(update.revision, 1);
  assert_eq!(update.patches.len(), 1);
  assert_eq!(session.snapshot().markdown, "abcx");

  let undo = session.dispatch(1, SessionCommand::Undo).unwrap();
  assert_eq!(undo.revision, 2);
  assert_eq!(undo.patches.len(), 1);
  assert_eq!(session.snapshot().markdown, "abc");
}

#[test]
fn groups_composition_updates_and_undoes_them_once() {
  let mut session = EditorSession::from_markdown("x");
  let text = text_id(&session);
  session
    .set_selection(
      0,
      Selection::collapsed(SelectionPoint {
        node: text,
        offset_utf16: 1,
      }),
    )
    .unwrap();
  session
    .dispatch(0, SessionCommand::BeginComposition)
    .unwrap();
  let first = session
    .dispatch(0, SessionCommand::UpdateComposition("に".into()))
    .unwrap();
  let second = session
    .dispatch(
      first.revision,
      SessionCommand::UpdateComposition("ほ".into()),
    )
    .unwrap();
  let third = session
    .dispatch(
      second.revision,
      SessionCommand::UpdateComposition("ん".into()),
    )
    .unwrap();
  session
    .dispatch(third.revision, SessionCommand::CommitComposition)
    .unwrap();
  assert_eq!(session.snapshot().markdown, "xにほん");

  session
    .dispatch(third.revision, SessionCommand::Undo)
    .unwrap();
  assert_eq!(session.snapshot().markdown, "x");
}

#[test]
fn selection_only_navigation_does_not_increment_revision() {
  let mut session = EditorSession::from_markdown("| A | B |\n| --- | --- |\n| one | two |");
  let table = session
    .document()
    .children(session.document().root)
    .next()
    .unwrap()
    .id;
  let row = session.document().children(table).nth(1).unwrap().id;
  let cell = session.document().children(row).next().unwrap().id;
  let text = first_text_descendant(session.document(), cell).unwrap();
  session
    .set_selection(
      0,
      Selection::collapsed(SelectionPoint {
        node: text,
        offset_utf16: 0,
      }),
    )
    .unwrap();
  let update = session
    .dispatch(
      0,
      SessionCommand::TableNavigation(TableNavigationCommand::NextCell),
    )
    .unwrap();
  assert_eq!(update.revision, 0);
  assert!(update.patches.is_empty());
  assert_ne!(update.selection.focus.node, text);
}
