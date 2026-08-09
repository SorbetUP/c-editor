use crate::features::TaskCommand;
use crate::model::{BlockKind, Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};
use crate::to_markdown;

fn task_items(document: &Document) -> Vec<NodeId> {
  document
    .nodes
    .values()
    .filter_map(|node| match node.kind {
      NodeKind::Block(BlockKind::ListItem { checked: Some(_) }) => Some(node.id),
      _ => None,
    })
    .collect()
}

fn first_text(document: &Document, root: NodeId) -> NodeId {
  let node = document.node(root).unwrap();
  if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    return root;
  }
  node
    .children
    .iter()
    .find_map(|child| {
      let child_node = document.node(*child)?;
      if matches!(child_node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
        Some(*child)
      } else {
        Some(first_text(document, *child))
      }
    })
    .unwrap()
}

fn selection(document: &Document, item: NodeId) -> Selection {
  Selection::collapsed(SelectionPoint {
    node: first_text(document, item),
    offset_utf16: 0,
  })
}

#[test]
fn toggles_one_task_and_restores_it() {
  let mut document = crate::parse_markdown("- [ ] alpha");
  let item = task_items(&document)[0];
  let transaction = TaskCommand::SetChecked {
    item,
    checked: true,
    auto_check: false,
  }
  .build(&document, selection(&document, item))
  .unwrap();
  let inverse = transaction.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "- [x] alpha");
  inverse.apply(&mut document).unwrap();
  assert_eq!(to_markdown(&document), "- [ ] alpha");
}

#[test]
fn cascades_to_descendants_and_recomputes_parents() {
  let mut document = crate::parse_markdown("- [ ] parent\n  - [ ] child");
  let items = task_items(&document);
  let parent = items[0];
  let child = items[1];
  let selection = selection(&document, parent);

  TaskCommand::SetChecked {
    item: parent,
    checked: true,
    auto_check: true,
  }
  .build(&document, selection)
  .unwrap()
  .apply(&mut document)
  .unwrap();
  assert_eq!(to_markdown(&document), "- [x] parent\n  - [x] child");

  TaskCommand::SetChecked {
    item: child,
    checked: false,
    auto_check: true,
  }
  .build(&document, selection)
  .unwrap()
  .apply(&mut document)
  .unwrap();
  assert_eq!(to_markdown(&document), "- [ ] parent\n  - [ ] child");
}

#[test]
fn preserves_checked_siblings_when_recomputing_a_parent() {
  let mut document = crate::parse_markdown("- [ ] parent\n  - [x] first\n  - [ ] second");
  let items = task_items(&document);
  let parent = items[0];
  let second = items[2];
  TaskCommand::SetChecked {
    item: second,
    checked: true,
    auto_check: true,
  }
  .build(&document, selection(&document, parent))
  .unwrap()
  .apply(&mut document)
  .unwrap();
  assert_eq!(
    to_markdown(&document),
    "- [x] parent\n  - [x] first\n  - [x] second"
  );
}
