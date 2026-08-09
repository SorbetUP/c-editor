use crate::edit::{EditError, Operation, Transaction};
use crate::model::{
  Alignment, BlockKind, DetachedSubtree, Document, InlineKind, Node, NodeId, NodeKind,
};
use crate::selection::{Selection, SelectionPoint};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreateTable {
  pub rows: u16,
  pub columns: u16,
}

impl CreateTable {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    if self.rows == 0 || self.columns == 0 {
      return Err(EditError::UnsupportedStructure(selection.focus.node));
    }
    let anchor = root_block(document, selection.focus.node)?;
    let index = document
      .child_index(document.root, anchor)
      .ok_or(EditError::UnsupportedStructure(anchor))?;
    let (subtree, first_text) = table_subtree(
      document.next_available_id(),
      self.rows as usize,
      self.columns as usize,
    );
    let mut operations = vec![Operation::InsertSubtree {
      parent: document.root,
      index: index + 1,
      subtree,
    }];
    if removable_empty_anchor(document, anchor)? {
      operations.push(Operation::RemoveNode { node: anchor });
    }
    Ok(Transaction {
      operations,
      selection_before: selection,
      selection_after: Selection::collapsed(SelectionPoint {
        node: first_text,
        offset_utf16: 0,
      }),
    })
  }
}

fn root_block(document: &Document, mut node: NodeId) -> Result<NodeId, EditError> {
  loop {
    let current = document.node(node).ok_or(EditError::NodeNotFound(node))?;
    let parent = current
      .parent
      .ok_or(EditError::UnsupportedStructure(node))?;
    if parent == document.root {
      return Ok(node);
    }
    node = parent;
  }
}

fn removable_empty_anchor(document: &Document, anchor: NodeId) -> Result<bool, EditError> {
  let node = document
    .node(anchor)
    .ok_or(EditError::NodeNotFound(anchor))?;
  if !matches!(
    node.kind,
    NodeKind::Block(BlockKind::Paragraph | BlockKind::Heading { .. })
  ) {
    return Ok(false);
  }
  Ok(
    node
      .children
      .iter()
      .all(|child| empty_inline(document, *child)),
  )
}

fn empty_inline(document: &Document, node: NodeId) -> bool {
  let Some(node) = document.node(node) else {
    return false;
  };
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => value.is_empty(),
    NodeKind::Inline(_) => node
      .children
      .iter()
      .all(|child| empty_inline(document, *child)),
    _ => false,
  }
}

fn table_subtree(start: NodeId, rows: usize, columns: usize) -> (DetachedSubtree, NodeId) {
  let mut next = start.0;
  let table = take_id(&mut next);
  let mut nodes = vec![Node::new(table, NodeKind::Block(BlockKind::Table), None)];
  let mut row_ids = Vec::with_capacity(rows);
  let mut first_text = None;

  for row_index in 0..rows {
    let row = take_id(&mut next);
    row_ids.push(row);
    let mut row_node = Node::new(row, NodeKind::Block(BlockKind::TableRow), None);
    row_node.parent = Some(table);
    let mut cell_ids = Vec::with_capacity(columns);

    for _ in 0..columns {
      let cell = take_id(&mut next);
      let text = take_id(&mut next);
      first_text.get_or_insert(text);
      cell_ids.push(cell);

      let mut cell_node = Node::new(
        cell,
        NodeKind::Block(BlockKind::TableCell {
          alignment: Alignment::Default,
          header: row_index == 0,
        }),
        None,
      );
      cell_node.parent = Some(row);
      cell_node.children.push(text);

      let mut text_node = Node::new(
        text,
        NodeKind::Inline(InlineKind::Text {
          value: String::new(),
        }),
        None,
      );
      text_node.parent = Some(cell);
      nodes.extend([cell_node, text_node]);
    }
    row_node.children = cell_ids;
    nodes.push(row_node);
  }

  nodes[0].children = row_ids;
  (
    DetachedSubtree { root: table, nodes },
    first_text.expect("table dimensions are non-zero"),
  )
}

fn take_id(next: &mut u64) -> NodeId {
  let id = NodeId(*next);
  *next = next.saturating_add(1);
  id
}
