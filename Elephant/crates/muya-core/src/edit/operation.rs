use serde::{Deserialize, Serialize};

use crate::model::{BlockKind, DetachedSubtree, Document, InlineKind, Node, NodeId, NodeKind};

use super::EditError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Utf16Range {
  pub start: u32,
  pub end: u32,
}

impl Utf16Range {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
  ReplaceText {
    node: NodeId,
    range: Utf16Range,
    inserted: String,
  },
  InsertNode {
    parent: NodeId,
    index: usize,
    node: Node,
  },
  InsertSubtree {
    parent: NodeId,
    index: usize,
    subtree: DetachedSubtree,
  },
  MoveNode {
    node: NodeId,
    new_parent: NodeId,
    new_index: usize,
  },
  RemoveNode {
    node: NodeId,
  },
  SetBlockKind {
    node: NodeId,
    kind: BlockKind,
  },
}

impl Operation {
  pub fn apply(&self, document: &mut Document) -> Result<Self, EditError> {
    match self {
      Self::ReplaceText {
        node,
        range,
        inserted,
      } => apply_replace_text(document, *node, *range, inserted),
      Self::InsertNode {
        parent,
        index,
        node,
      } => apply_insert_node(document, *parent, *index, node),
      Self::InsertSubtree {
        parent,
        index,
        subtree,
      } => apply_insert_subtree(document, *parent, *index, subtree),
      Self::MoveNode {
        node,
        new_parent,
        new_index,
      } => apply_move_node(document, *node, *new_parent, *new_index),
      Self::RemoveNode { node } => apply_remove_node(document, *node),
      Self::SetBlockKind { node, kind } => apply_set_block_kind(document, *node, kind),
    }
  }
}

fn apply_replace_text(
  document: &mut Document,
  node_id: NodeId,
  range: Utf16Range,
  inserted: &str,
) -> Result<Operation, EditError> {
  let deleted = {
    let node = document
      .node_mut(node_id)
      .ok_or(EditError::NodeNotFound(node_id))?;
    let value = match &mut node.kind {
      NodeKind::Inline(InlineKind::Text { value }) => value,
      NodeKind::Inline(InlineKind::CodeSpan { code }) => code,
      _ => return Err(EditError::NotTextNode(node_id)),
    };

    let utf16_length = value.encode_utf16().count() as u32;
    if range.start > range.end || range.end > utf16_length {
      return Err(EditError::RangeOutOfBounds {
        node: node_id,
        start: range.start,
        end: range.end,
      });
    }

    let start = utf16_to_byte(value, node_id, range.start)?;
    let end = utf16_to_byte(value, node_id, range.end)?;
    let deleted = value[start..end].to_string();
    value.replace_range(start..end, inserted);
    deleted
  };
  document.invalidate_source_chain(node_id);

  Ok(Operation::ReplaceText {
    node: node_id,
    range: Utf16Range::new(
      range.start,
      range.start + inserted.encode_utf16().count() as u32,
    ),
    inserted: deleted,
  })
}

fn apply_insert_node(
  document: &mut Document,
  parent: NodeId,
  index: usize,
  node: &Node,
) -> Result<Operation, EditError> {
  if document.node(node.id).is_some() {
    return Err(EditError::NodeAlreadyExists(node.id));
  }
  if !node.children.is_empty() {
    return Err(EditError::NodeHasChildren(node.id));
  }
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  if index > parent_node.children.len() {
    return Err(EditError::InvalidChildIndex { parent, index });
  }

  if !document.insert_detached_node(parent, index, node.clone()) {
    return Err(EditError::UnsupportedStructure(node.id));
  }
  document.invalidate_source_chain(parent);
  Ok(Operation::RemoveNode { node: node.id })
}

fn apply_insert_subtree(
  document: &mut Document,
  parent: NodeId,
  index: usize,
  subtree: &DetachedSubtree,
) -> Result<Operation, EditError> {
  if document.node(subtree.root).is_some() {
    return Err(EditError::NodeAlreadyExists(subtree.root));
  }
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  if index > parent_node.children.len() {
    return Err(EditError::InvalidChildIndex { parent, index });
  }
  if !document.insert_detached_subtree(parent, index, subtree.clone()) {
    return Err(EditError::UnsupportedStructure(subtree.root));
  }
  document.invalidate_source_chain(parent);
  Ok(Operation::RemoveNode { node: subtree.root })
}

fn apply_move_node(
  document: &mut Document,
  node_id: NodeId,
  new_parent: NodeId,
  new_index: usize,
) -> Result<Operation, EditError> {
  if node_id == document.root || node_id == new_parent {
    return Err(EditError::UnsupportedStructure(node_id));
  }
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  let old_parent = node
    .parent
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  let old_index = document
    .child_index(old_parent, node_id)
    .ok_or(EditError::UnsupportedStructure(node_id))?;

  let (subtree, removed_parent, removed_index) = document
    .remove_subtree(node_id)
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  debug_assert_eq!((removed_parent, removed_index), (old_parent, old_index));

  let target = document
    .node(new_parent)
    .ok_or(EditError::NodeNotFound(new_parent))?;
  if new_index > target.children.len() {
    return Err(EditError::InvalidChildIndex {
      parent: new_parent,
      index: new_index,
    });
  }
  if !document.insert_detached_subtree(new_parent, new_index, subtree) {
    return Err(EditError::UnsupportedStructure(node_id));
  }

  document.invalidate_source_chain(old_parent);
  if new_parent != old_parent {
    document.invalidate_source_chain(new_parent);
  }
  Ok(Operation::MoveNode {
    node: node_id,
    new_parent: old_parent,
    new_index: old_index,
  })
}

fn apply_remove_node(document: &mut Document, node_id: NodeId) -> Result<Operation, EditError> {
  let (subtree, parent, index) = document
    .remove_subtree(node_id)
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  document.invalidate_source_chain(parent);

  Ok(Operation::InsertSubtree {
    parent,
    index,
    subtree,
  })
}

fn apply_set_block_kind(
  document: &mut Document,
  node_id: NodeId,
  kind: &BlockKind,
) -> Result<Operation, EditError> {
  let previous = {
    let node = document
      .node_mut(node_id)
      .ok_or(EditError::NodeNotFound(node_id))?;
    let NodeKind::Block(previous) = &mut node.kind else {
      return Err(EditError::UnsupportedStructure(node_id));
    };
    std::mem::replace(previous, kind.clone())
  };
  document.invalidate_source_chain(node_id);
  Ok(Operation::SetBlockKind {
    node: node_id,
    kind: previous,
  })
}

pub(crate) fn utf16_to_byte(value: &str, node: NodeId, target: u32) -> Result<usize, EditError> {
  if target == 0 {
    return Ok(0);
  }

  let mut utf16_offset = 0u32;
  for (byte_offset, character) in value.char_indices() {
    if utf16_offset == target {
      return Ok(byte_offset);
    }
    utf16_offset += character.len_utf16() as u32;
    if utf16_offset > target {
      return Err(EditError::InvalidUtf16Boundary {
        node,
        offset: target,
      });
    }
  }

  if utf16_offset == target {
    Ok(value.len())
  } else {
    Err(EditError::RangeOutOfBounds {
      node,
      start: target,
      end: target,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{BlockKind, InlineKind, NodeKind};

  #[test]
  fn replaces_text_and_returns_its_inverse() {
    let mut document = Document::new();
    let node = document.allocate(
      NodeKind::Inline(InlineKind::Text {
        value: "A😀B".to_string(),
      }),
      None,
    );
    let operation = Operation::ReplaceText {
      node,
      range: Utf16Range::new(1, 3),
      inserted: "X".to_string(),
    };
    let inverse = operation.apply(&mut document).unwrap();
    assert!(matches!(
      &document.node(node).unwrap().kind,
      NodeKind::Inline(InlineKind::Text { value }) if value == "AXB"
    ));
    inverse.apply(&mut document).unwrap();
    assert!(matches!(
      &document.node(node).unwrap().kind,
      NodeKind::Inline(InlineKind::Text { value }) if value == "A😀B"
    ));
  }

  #[test]
  fn inserts_and_removes_nodes_with_stable_ids() {
    let mut document = Document::new();
    let paragraph = Node::new(
      document.next_available_id(),
      NodeKind::Block(BlockKind::Paragraph),
      None,
    );
    let remove = Operation::InsertNode {
      parent: document.root,
      index: 0,
      node: paragraph.clone(),
    }
    .apply(&mut document)
    .unwrap();
    assert!(document.node(paragraph.id).is_some());

    let insert = remove.apply(&mut document).unwrap();
    assert!(document.node(paragraph.id).is_none());
    insert.apply(&mut document).unwrap();
    assert!(document.node(paragraph.id).is_some());
  }

  #[test]
  fn removes_and_restores_nested_subtrees() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, paragraph);
    let strong = document.allocate(NodeKind::Inline(InlineKind::Strong), None);
    document.append_child(paragraph, strong);
    let text = document.allocate(
      NodeKind::Inline(InlineKind::Text {
        value: "bold".to_string(),
      }),
      None,
    );
    document.append_child(strong, text);

    let restore = Operation::RemoveNode { node: strong }
      .apply(&mut document)
      .unwrap();
    assert!(document.node(strong).is_none());
    assert!(document.node(text).is_none());

    restore.apply(&mut document).unwrap();
    assert_eq!(document.node(strong).unwrap().children, vec![text]);
    assert_eq!(document.node(text).unwrap().parent, Some(strong));
  }

  #[test]
  fn moves_nested_subtrees_and_restores_their_position() {
    let mut document = Document::new();
    let first = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    let second = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, first);
    document.append_child(document.root, second);
    let strong = document.allocate(NodeKind::Inline(InlineKind::Strong), None);
    document.append_child(first, strong);
    let text = document.allocate(
      NodeKind::Inline(InlineKind::Text {
        value: "bold".to_string(),
      }),
      None,
    );
    document.append_child(strong, text);

    let inverse = Operation::MoveNode {
      node: strong,
      new_parent: second,
      new_index: 0,
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.node(strong).unwrap().parent, Some(second));
    assert_eq!(document.node(text).unwrap().parent, Some(strong));

    inverse.apply(&mut document).unwrap();
    assert_eq!(document.node(strong).unwrap().parent, Some(first));
    assert_eq!(document.node(first).unwrap().children, vec![strong]);
  }

  #[test]
  fn changes_block_kind_and_restores_it() {
    let mut document = Document::new();
    let block = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    let inverse = Operation::SetBlockKind {
      node: block,
      kind: BlockKind::Heading { level: 2 },
    }
    .apply(&mut document)
    .unwrap();
    assert!(matches!(
      document.node(block).unwrap().kind,
      NodeKind::Block(BlockKind::Heading { level: 2 })
    ));
    inverse.apply(&mut document).unwrap();
    assert!(matches!(
      document.node(block).unwrap().kind,
      NodeKind::Block(BlockKind::Paragraph)
    ));
  }
}
