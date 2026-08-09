use crate::features::BlockCommand;
use crate::model::{Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};
use crate::to_markdown;

fn text(document: &Document, value: &str) -> NodeId {
  document
    .nodes
    .values()
    .find_map(|node| match &node.kind {
      NodeKind::Inline(InlineKind::Text { value: current }) if current == value => Some(node.id),
      _ => None,
    })
    .unwrap()
}

fn selection(document: &Document, value: &str) -> Selection {
  Selection::collapsed(SelectionPoint {
    node: text(document, value),
    offset_utf16: 1,
  })
}

fn apply(
  initial: &str,
  target: &str,
  command: BlockCommand,
) -> (Document, crate::edit::Transaction) {
  let mut document = crate::parse_markdown(initial);
  let transaction = command
    .build(&document, selection(&document, target))
    .unwrap();
  let inverse = transaction.apply(&mut document).unwrap();
  (document, inverse)
}

#[test]
fn duplicates_paragraph_heading_and_nested_list_roots() {
  let (document, _) = apply("alpha\n\nbeta", "alpha", BlockCommand::Duplicate);
  assert_eq!(to_markdown(&document), "alpha\n\nalpha\n\nbeta");

  let (document, _) = apply("# alpha\n\nbeta", "alpha", BlockCommand::Duplicate);
  assert_eq!(to_markdown(&document), "# alpha\n\n# alpha\n\nbeta");

  let (document, _) = apply(
    "- parent\n  - child\n\nafter",
    "child",
    BlockCommand::Duplicate,
  );
  assert_eq!(
    to_markdown(&document),
    "- parent\n  - child\n\n- parent\n  - child\n\nafter"
  );
}

#[test]
fn deletes_middle_final_and_only_blocks() {
  let (document, _) = apply("before\n\nalpha\n\nafter", "alpha", BlockCommand::Delete);
  assert_eq!(to_markdown(&document), "before\n\nafter");

  let (document, _) = apply("before\n\nalpha", "alpha", BlockCommand::Delete);
  assert_eq!(to_markdown(&document), "before");

  let (document, _) = apply("alpha", "alpha", BlockCommand::Delete);
  assert_eq!(to_markdown(&document), "");
  assert_eq!(document.children(document.root).count(), 1);
}

#[test]
fn inserts_empty_root_paragraphs_after_text_and_nested_lists() {
  let (document, _) = apply("alpha\n\nbeta", "alpha", BlockCommand::InsertParagraphAfter);
  assert_eq!(to_markdown(&document), "alpha\n\n\n\nbeta");

  let (document, _) = apply(
    "- parent\n  - child\n\nafter",
    "child",
    BlockCommand::InsertParagraphAfter,
  );
  assert_eq!(to_markdown(&document), "- parent\n  - child\n\n\n\nafter");
}

#[test]
fn every_block_command_has_an_exact_inverse() {
  for (initial, target, command) in [
    ("alpha\n\nbeta", "alpha", BlockCommand::Duplicate),
    ("before\n\nalpha\n\nafter", "alpha", BlockCommand::Delete),
    ("alpha\n\nbeta", "alpha", BlockCommand::InsertParagraphAfter),
  ] {
    let (mut document, inverse) = apply(initial, target, command);
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), initial);
  }
}

#[test]
fn duplicated_linked_mark_groups_do_not_link_back_to_the_source() {
  let (document, _) = apply(
    "al*pha **be*ta** gamma\n\nafter",
    "al",
    BlockCommand::Duplicate,
  );
  let groups = document
    .nodes
    .values()
    .filter_map(|node| match node.kind {
      NodeKind::Inline(InlineKind::MarkFragment { group, .. }) => Some(group),
      _ => None,
    })
    .collect::<std::collections::BTreeSet<_>>();
  assert_eq!(groups.len(), 2);
}
