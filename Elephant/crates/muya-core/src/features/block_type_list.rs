use crate::edit::{EditError, Operation, Transaction};
use crate::model::{BlockKind, Document, ListKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::block_type::text_block;

pub(super) fn set_list_kind(
  document: &Document,
  selection: Selection,
  target: ListKind,
) -> Result<Transaction, EditError> {
  if let Some(list) = ancestor_list(document, selection.focus.node) {
    let current = list_kind(document, list)?;
    return if current == target {
      unwrap_list(document, selection, list)
    } else {
      convert_list(document, selection, list, target)
    };
  }
  wrap_paragraph(document, selection, target)
}

fn wrap_paragraph(
  document: &Document,
  selection: Selection,
  kind: ListKind,
) -> Result<Transaction, EditError> {
  let paragraph = text_block(document, selection.focus.node)?;
  let node = document
    .node(paragraph)
    .ok_or(EditError::NodeNotFound(paragraph))?;
  if !matches!(node.kind, NodeKind::Block(BlockKind::Paragraph))
    || node.parent != Some(document.root)
  {
    return Err(EditError::UnsupportedStructure(paragraph));
  }
  let index = document
    .child_index(document.root, paragraph)
    .ok_or(EditError::UnsupportedStructure(paragraph))?;
  let list = document.next_available_id();
  let item = NodeId(list.0.saturating_add(1));
  let checked = (kind == ListKind::Task).then_some(false);
  Ok(Transaction {
    operations: vec![
      Operation::InsertNode {
        parent: document.root,
        index,
        node: Node::new(
          list,
          NodeKind::Block(BlockKind::List {
            kind,
            start: ordered_start(kind),
          }),
          None,
        ),
      },
      Operation::InsertNode {
        parent: list,
        index: 0,
        node: Node::new(item, NodeKind::Block(BlockKind::ListItem { checked }), None),
      },
      Operation::MoveNode {
        node: paragraph,
        new_parent: item,
        new_index: 0,
      },
    ],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: selection.focus.node,
      offset_utf16: 0,
    }),
  })
}

fn convert_list(
  document: &Document,
  selection: Selection,
  list: NodeId,
  kind: ListKind,
) -> Result<Transaction, EditError> {
  let node = document.node(list).ok_or(EditError::NodeNotFound(list))?;
  let mut operations = vec![Operation::SetBlockKind {
    node: list,
    kind: BlockKind::List {
      kind,
      start: ordered_start(kind),
    },
  }];
  for item in &node.children {
    operations.push(Operation::SetBlockKind {
      node: *item,
      kind: BlockKind::ListItem {
        checked: (kind == ListKind::Task).then_some(false),
      },
    });
  }
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn unwrap_list(
  document: &Document,
  selection: Selection,
  list: NodeId,
) -> Result<Transaction, EditError> {
  let index = document
    .child_index(document.root, list)
    .ok_or(EditError::UnsupportedStructure(list))?;
  let list_node = document.node(list).ok_or(EditError::NodeNotFound(list))?;
  let mut operations = Vec::new();
  let mut offset = 0;
  for item in &list_node.children {
    let item_node = document.node(*item).ok_or(EditError::NodeNotFound(*item))?;
    for child in &item_node.children {
      operations.push(Operation::MoveNode {
        node: *child,
        new_parent: document.root,
        new_index: index + offset,
      });
      offset += 1;
    }
  }
  operations.push(Operation::RemoveNode { node: list });
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn ancestor_list(document: &Document, mut node: NodeId) -> Option<NodeId> {
  loop {
    let current = document.node(node)?;
    if matches!(current.kind, NodeKind::Block(BlockKind::List { .. })) {
      return Some(node);
    }
    node = current.parent?;
  }
}

fn list_kind(document: &Document, list: NodeId) -> Result<ListKind, EditError> {
  match &document
    .node(list)
    .ok_or(EditError::NodeNotFound(list))?
    .kind
  {
    NodeKind::Block(BlockKind::List { kind, .. }) => Ok(*kind),
    _ => Err(EditError::UnsupportedStructure(list)),
  }
}

fn ordered_start(kind: ListKind) -> Option<u64> {
  (kind == ListKind::Ordered).then_some(1)
}
