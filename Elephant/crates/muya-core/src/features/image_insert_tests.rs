use crate::features::InsertImage;
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

fn selection(document: &Document, value: &str, start: u32, end: u32) -> Selection {
  let node = text(document, value);
  Selection {
    anchor: SelectionPoint {
      node,
      offset_utf16: start,
    },
    focus: SelectionPoint {
      node,
      offset_utf16: end,
    },
  }
}

fn command(source: &str, alt: &str, title: Option<&str>) -> InsertImage {
  InsertImage {
    source: source.to_string(),
    alt: alt.to_string(),
    title: title.map(str::to_string),
  }
}

#[test]
fn infers_alt_and_encodes_local_paths() {
  let mut document = crate::parse_markdown("alpha");
  let before = selection(&document, "alpha", 2, 2);
  let transaction = command("/tmp/my image.png", "", None)
    .build(&document, before)
    .unwrap();
  let inverse = transaction.apply(&mut document).unwrap();
  assert_eq!(
    to_markdown(&document),
    "al![my image](/tmp/my%20image.png)pha"
  );
  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "alpha");
}

#[test]
fn uses_selected_text_as_alt_and_preserves_the_suffix() {
  let mut document = crate::parse_markdown("alpha");
  let transaction = command("https://example.com/a.png", "ignored", None)
    .build(&document, selection(&document, "alpha", 1, 4))
    .unwrap();
  transaction.apply(&mut document).unwrap();
  assert_eq!(
    to_markdown(&document),
    "a![lph](https://example.com/a.png)a"
  );
}

#[test]
fn inserts_at_the_focus_of_a_cross_block_selection() {
  let mut document = crate::parse_markdown("alpha\n\nbeta");
  let cross_block = Selection {
    anchor: SelectionPoint {
      node: text(&document, "alpha"),
      offset_utf16: 1,
    },
    focus: SelectionPoint {
      node: text(&document, "beta"),
      offset_utf16: 2,
    },
  };
  let transaction = command("/tmp/picture.png", "", None)
    .build(&document, cross_block)
    .unwrap();
  transaction.apply(&mut document).unwrap();
  assert_eq!(
    to_markdown(&document),
    "alpha\n\nbe![picture](/tmp/picture.png)ta"
  );
}

#[test]
fn preserves_explicit_alt_title_and_inline_marks() {
  let mut document = crate::parse_markdown("**alpha**");
  let transaction = command(
    "https://example.com/a b.png",
    "diagram",
    Some("Example image"),
  )
  .build(&document, selection(&document, "alpha", 2, 2))
  .unwrap();
  transaction.apply(&mut document).unwrap();
  assert_eq!(
    to_markdown(&document),
    "**al![diagram](https://example.com/a%20b.png \"Example image\")pha**"
  );
}

#[test]
fn rejects_image_insertion_inside_fenced_code_without_history_operations() {
  let document = crate::parse_markdown("```\nalpha\n```");
  let transaction = command("/tmp/picture.png", "", None)
    .build(&document, selection(&document, "alpha", 2, 2))
    .unwrap();
  assert!(transaction.operations.is_empty());
  assert_eq!(to_markdown(&document), "```\nalpha\n```");
}

#[test]
fn keeps_the_selection_on_an_editable_text_node_after_insertion() {
  let mut document = crate::parse_markdown("alpha");
  let transaction = command("/tmp/picture.png", "", None)
    .build(&document, selection(&document, "alpha", 5, 5))
    .unwrap();
  let caret = transaction.selection_after.focus;
  transaction.apply(&mut document).unwrap();
  assert!(matches!(
    document.node(caret.node).map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Text { value })) if value.is_empty()
  ));
  assert_eq!(to_markdown(&document), "alpha![picture](/tmp/picture.png)");
}

#[test]
fn inserts_image_markdown_literally_inside_inline_code() {
  let mut document = crate::parse_markdown("`alpha`");
  let code = document
    .nodes
    .values()
    .find_map(|node| match &node.kind {
      NodeKind::Inline(InlineKind::CodeSpan { code }) if code == "alpha" => Some(node.id),
      _ => None,
    })
    .unwrap();
  let before = Selection::collapsed(SelectionPoint {
    node: code,
    offset_utf16: 2,
  });
  let transaction = command("/tmp/picture.png", "", None)
    .build(&document, before)
    .unwrap();
  assert_eq!(transaction.selection_after.anchor.node, code);
  assert_eq!(transaction.selection_after.anchor.offset_utf16, 4);
  assert_eq!(transaction.selection_after.focus.offset_utf16, 11);
  let inverse = transaction.apply(&mut document).unwrap();
  assert_eq!(
    to_markdown(&document),
    "`al![picture](/tmp/picture.png)pha`"
  );
  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "`alpha`");
}
