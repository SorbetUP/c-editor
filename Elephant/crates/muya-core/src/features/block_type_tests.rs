use crate::features::BlockTypeCommand;
use crate::model::{Document, InlineKind, ListKind, NodeId, NodeKind};
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

fn apply(
  initial: &str,
  target: &str,
  command: BlockTypeCommand,
) -> (Document, crate::edit::Transaction) {
  let mut document = crate::parse_markdown(initial);
  let node = text(&document, target);
  let selection = Selection::collapsed(SelectionPoint {
    node,
    offset_utf16: 1,
  });
  let transaction = command.build(&document, selection).unwrap();
  let inverse = transaction.apply(&mut document).unwrap();
  (document, inverse)
}

#[test]
fn toggles_blockquotes_and_fenced_code() {
  for (initial, command, expected) in [
    ("alpha", BlockTypeCommand::ToggleBlockQuote, "> alpha"),
    ("> alpha", BlockTypeCommand::ToggleBlockQuote, "alpha"),
    (
      "alpha",
      BlockTypeCommand::ToggleCodeBlock,
      "```\nalpha\n```",
    ),
    (
      "```\nalpha\n```",
      BlockTypeCommand::ToggleCodeBlock,
      "alpha",
    ),
  ] {
    let (document, _) = apply(initial, "alpha", command);
    assert_eq!(to_markdown(&document), expected);
  }
}

#[test]
fn wraps_paragraphs_in_each_list_kind() {
  for (kind, expected) in [
    (ListKind::Unordered, "- alpha"),
    (ListKind::Task, "- [ ] alpha"),
    (ListKind::Ordered, "1. alpha"),
  ] {
    let (document, _) = apply("alpha", "alpha", BlockTypeCommand::SetListKind(kind));
    assert_eq!(to_markdown(&document), expected);
  }
}

#[test]
fn removes_a_list_when_the_same_kind_is_selected() {
  for (initial, kind) in [
    ("- alpha", ListKind::Unordered),
    ("- [x] alpha", ListKind::Task),
    ("1. alpha", ListKind::Ordered),
  ] {
    let (document, _) = apply(initial, "alpha", BlockTypeCommand::SetListKind(kind));
    assert_eq!(to_markdown(&document), "alpha");
  }
}

#[test]
fn converts_lists_and_resets_task_state() {
  for (initial, kind, expected) in [
    ("- alpha", ListKind::Task, "- [ ] alpha"),
    ("- [x] alpha", ListKind::Ordered, "1. alpha"),
    ("1. alpha", ListKind::Unordered, "- alpha"),
  ] {
    let (document, _) = apply(initial, "alpha", BlockTypeCommand::SetListKind(kind));
    assert_eq!(to_markdown(&document), expected);
  }
}

#[test]
fn every_block_type_transformation_has_an_exact_inverse() {
  for (initial, command) in [
    ("alpha", BlockTypeCommand::ToggleBlockQuote),
    ("alpha", BlockTypeCommand::ToggleCodeBlock),
    ("alpha", BlockTypeCommand::SetListKind(ListKind::Task)),
    (
      "- [x] alpha",
      BlockTypeCommand::SetListKind(ListKind::Ordered),
    ),
    (
      "- alpha",
      BlockTypeCommand::SetListKind(ListKind::Unordered),
    ),
  ] {
    let (mut document, inverse) = apply(initial, "alpha", command);
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), initial);
  }
}
