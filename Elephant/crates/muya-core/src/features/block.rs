use crate::edit::{EditError, Operation, Transaction};
use crate::model::{BlockKind, DetachedSubtree, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::block_clone::{clone_block, first_text_descendant, last_text_descendant, text_length};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockCommand {
  Duplicate,
  Delete,
  InsertParagraphAfter,
}

impl BlockCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let root = root_block(document, selection.focus.node)?;
    match self {
      Self::Duplicate => duplicate(document, selection, root),
      Self::Delete => delete(document, selection, root),
      Self::InsertParagraphAfter => insert_paragraph_after(document, selection, root),
    }
  }
}

fn duplicate(
  document: &Document,
  selection: Selection,
  root: NodeId,
) -> Result<Transaction, EditError> {
  let index = document
    .child_index(document.root, root)
    .ok_or(EditError::UnsupportedStructure(root))?;
  let source_text = first_text_descendant(document, root);
  let cloned = clone_block(document, root)?;
  let target = cloned
    .first_text
    .ok_or(EditError::UnsupportedStructure(root))?;
  let offset = source_text
    .map(|node| text_length(document, node))
    .unwrap_or(0);
  Ok(Transaction {
    operations: vec![Operation::InsertSubtree {
      parent: document.root,
      index: index + 1,
      subtree: cloned.subtree,
    }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: target,
      offset_utf16: offset,
    }),
  })
}

fn delete(
  document: &Document,
  selection: Selection,
  root: NodeId,
) -> Result<Transaction, EditError> {
  let index = document
    .child_index(document.root, root)
    .ok_or(EditError::UnsupportedStructure(root))?;
  let roots = document
    .node(document.root)
    .ok_or(EditError::NodeNotFound(document.root))?
    .children
    .clone();
  if let Some(next) = roots.get(index + 1).copied() {
    let text =
      first_text_descendant(document, next).ok_or(EditError::UnsupportedStructure(next))?;
    return Ok(remove_with_selection(document, selection, root, text));
  }
  if let Some(previous) = index
    .checked_sub(1)
    .and_then(|value| roots.get(value))
    .copied()
  {
    let text =
      last_text_descendant(document, previous).ok_or(EditError::UnsupportedStructure(previous))?;
    return Ok(remove_with_selection(document, selection, root, text));
  }

  let (subtree, text) = empty_paragraph(document.next_available_id());
  Ok(Transaction {
    operations: vec![
      Operation::InsertSubtree {
        parent: document.root,
        index: 1,
        subtree,
      },
      Operation::RemoveNode { node: root },
    ],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    }),
  })
}

fn remove_with_selection(
  document: &Document,
  selection: Selection,
  root: NodeId,
  text: NodeId,
) -> Transaction {
  Transaction {
    operations: vec![Operation::RemoveNode { node: root }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: text_length(document, text),
    }),
  }
}

fn insert_paragraph_after(
  document: &Document,
  selection: Selection,
  root: NodeId,
) -> Result<Transaction, EditError> {
  let index = document
    .child_index(document.root, root)
    .ok_or(EditError::UnsupportedStructure(root))?;
  let (subtree, text) = empty_paragraph(document.next_available_id());
  Ok(Transaction {
    operations: vec![Operation::InsertSubtree {
      parent: document.root,
      index: index + 1,
      subtree,
    }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    }),
  })
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

fn empty_paragraph(next: NodeId) -> (DetachedSubtree, NodeId) {
  let paragraph = next;
  let text = NodeId(next.0.saturating_add(1));
  let mut paragraph_node = Node::new(paragraph, NodeKind::Block(BlockKind::Paragraph), None);
  paragraph_node.children.push(text);
  let mut text_node = Node::new(
    text,
    NodeKind::Inline(InlineKind::Text {
      value: String::new(),
    }),
    None,
  );
  text_node.parent = Some(paragraph);
  (
    DetachedSubtree {
      root: paragraph,
      nodes: vec![paragraph_node, text_node],
    },
    text,
  )
}
