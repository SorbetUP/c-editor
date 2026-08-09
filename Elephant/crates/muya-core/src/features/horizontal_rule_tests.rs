use crate::features::InsertHorizontalRule;
use crate::model::{Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};
use crate::to_markdown;

fn first_text(document: &Document) -> NodeId {
  document
    .nodes
    .values()
    .find_map(|node| match node.kind {
      NodeKind::Inline(InlineKind::Text { .. }) => Some(node.id),
      _ => None,
    })
    .unwrap()
}

fn selection(document: &Document) -> Selection {
  Selection::collapsed(SelectionPoint {
    node: first_text(document),
    offset_utf16: 0,
  })
}

#[test]
fn inserts_a_rule_before_an_empty_root_paragraph() {
  let session = crate::session::EditorSession::from_markdown("");
  let mut document = session.document().clone();
  let transaction = InsertHorizontalRule
    .build(&document, session.selection())
    .unwrap();
  let inverse = transaction.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "---\n\n");
  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "");
}

#[test]
fn keeps_non_empty_paragraphs_unchanged_without_history_operations() {
  let document = crate::parse_markdown("alpha");
  let transaction = InsertHorizontalRule
    .build(&document, selection(&document))
    .unwrap();
  assert!(transaction.operations.is_empty());
  assert_eq!(to_markdown(&document), "alpha");
}
