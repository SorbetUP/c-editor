use crate::features::CreateTable;
use crate::model::{BlockKind, Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};
use crate::to_markdown;

fn first_text(document: &Document, value: &str) -> NodeId {
  document
    .nodes
    .values()
    .find_map(|node| match &node.kind {
      NodeKind::Inline(InlineKind::Text { value: current }) if current == value => Some(node.id),
      _ => None,
    })
    .unwrap()
}

fn selection(document: &Document, value: &str, offset: u32) -> Selection {
  Selection::collapsed(SelectionPoint {
    node: first_text(document, value),
    offset_utf16: offset,
  })
}

#[test]
fn creates_a_two_by_two_table_after_non_empty_text() {
  let mut document = crate::parse_markdown("alpha");
  let before = selection(&document, "alpha", 2);
  let transaction = CreateTable {
    rows: 2,
    columns: 2,
  }
  .build(&document, before)
  .unwrap();
  let selected = transaction.selection_after.focus.node;
  let inverse = transaction.apply(&mut document).unwrap();

  assert_eq!(
    to_markdown(&document),
    "alpha\n\n|     |     |\n| --- | --- |\n|     |     |"
  );
  assert!(matches!(
    document.node(selected).map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Text { value })) if value.is_empty()
  ));

  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "alpha");
}

#[test]
fn replaces_an_empty_root_paragraph_with_the_table() {
  let session = crate::session::EditorSession::from_markdown("");
  let mut document = session.document().clone();
  let transaction = CreateTable {
    rows: 2,
    columns: 2,
  }
  .build(&document, session.selection())
  .unwrap();
  let inverse = transaction.apply(&mut document).unwrap();

  assert_eq!(
    to_markdown(&document),
    "|     |     |\n| --- | --- |\n|     |     |"
  );
  assert_eq!(document.children(document.root).count(), 1);
  assert!(matches!(
    document
      .children(document.root)
      .next()
      .map(|node| &node.kind),
    Some(NodeKind::Block(BlockKind::Table))
  ));

  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "");
}

#[test]
fn rejects_zero_sized_tables_without_mutation() {
  let document = crate::parse_markdown("alpha");
  let before = selection(&document, "alpha", 0);
  for command in [
    CreateTable {
      rows: 0,
      columns: 2,
    },
    CreateTable {
      rows: 2,
      columns: 0,
    },
  ] {
    assert!(command.build(&document, before).is_err());
    assert_eq!(to_markdown(&document), "alpha");
  }
}
