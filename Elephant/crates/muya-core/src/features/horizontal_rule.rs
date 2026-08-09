use crate::edit::{EditError, Operation, Transaction};
use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::Selection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InsertHorizontalRule;

impl InsertHorizontalRule {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let paragraph = paragraph_ancestor(document, selection.focus.node)?;
    if !paragraph_is_empty(document, paragraph)? {
      return Ok(Transaction {
        operations: Vec::new(),
        selection_before: selection,
        selection_after: selection,
      });
    }
    let parent = document
      .node(paragraph)
      .and_then(|node| node.parent)
      .ok_or(EditError::UnsupportedStructure(paragraph))?;
    if parent != document.root {
      return Err(EditError::UnsupportedStructure(paragraph));
    }
    let index = document
      .child_index(parent, paragraph)
      .ok_or(EditError::UnsupportedStructure(paragraph))?;
    let rule = document.next_available_id();
    Ok(Transaction {
      operations: vec![Operation::InsertNode {
        parent,
        index,
        node: Node::new(rule, NodeKind::Block(BlockKind::ThematicBreak), None),
      }],
      selection_before: selection,
      selection_after: selection,
    })
  }
}

fn paragraph_ancestor(document: &Document, mut node: NodeId) -> Result<NodeId, EditError> {
  loop {
    let current = document.node(node).ok_or(EditError::NodeNotFound(node))?;
    if matches!(current.kind, NodeKind::Block(BlockKind::Paragraph)) {
      return Ok(node);
    }
    node = current
      .parent
      .ok_or(EditError::UnsupportedStructure(node))?;
  }
}

fn paragraph_is_empty(document: &Document, paragraph: NodeId) -> Result<bool, EditError> {
  let node = document
    .node(paragraph)
    .ok_or(EditError::NodeNotFound(paragraph))?;
  Ok(node.children.iter().all(|child| {
    matches!(
      document.node(*child).map(|node| &node.kind),
      Some(NodeKind::Inline(InlineKind::Text { value })) if value.is_empty()
    )
  }))
}
