use crate::model::{DetachedSubtree, Document, InlineKind, InlineSyntax, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction};

#[derive(Clone, Copy)]
struct Endpoint {
  point: SelectionPoint,
  block: NodeId,
  top_index: usize,
  text_index: usize,
}

#[derive(Clone)]
struct DraftNode {
  kind: NodeKind,
  inline_syntax: Option<InlineSyntax>,
  children: Vec<DraftNode>,
  caret_offset: Option<u32>,
}

enum DraftPart {
  Node(DraftNode),
  Caret,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Phase {
  Before,
  Selected,
  After,
}

struct Transform<'a> {
  start: SelectionPoint,
  end: SelectionPoint,
  inserted: &'a str,
  phase: Phase,
}

#[derive(Default)]
struct Materialized {
  caret: Option<SelectionPoint>,
}

pub(crate) fn build_cross_wrapper_replace(
  document: &Document,
  selection: Selection,
  inserted: &str,
) -> Result<Transaction, EditError> {
  let text_order = ordered_text_nodes(document, selection.anchor.node, selection.focus.node)?;
  let anchor = endpoint(document, selection.anchor, &text_order)?;
  let focus = endpoint(document, selection.focus, &text_order)?;
  if anchor.block != focus.block || anchor.point.node == focus.point.node {
    return Err(EditError::CrossNodeSelection);
  }

  let (start, end) = if anchor.text_index < focus.text_index {
    (anchor, focus)
  } else {
    (focus, anchor)
  };
  validate_boundary(document, start.point)?;
  validate_boundary(document, end.point)?;

  let block = document
    .node(start.block)
    .ok_or(EditError::NodeNotFound(start.block))?;
  let selected_roots = block.children[start.top_index..=end.top_index].to_vec();
  let mut transform = Transform {
    start: start.point,
    end: end.point,
    inserted,
    phase: Phase::Before,
  };
  let mut parts = Vec::new();
  for root in &selected_roots {
    parts.extend(transform_node(document, *root, &mut transform)?);
  }
  if transform.phase != Phase::After {
    return Err(EditError::UnsupportedStructure(end.point.node));
  }

  let mut drafts = Vec::new();
  for part in parts {
    match part {
      DraftPart::Node(node) => drafts.push(node),
      DraftPart::Caret => drafts.push(DraftNode {
        kind: NodeKind::Inline(InlineKind::Text {
          value: String::new(),
        }),
        inline_syntax: None,
        children: Vec::new(),
        caret_offset: Some(0),
      }),
    }
  }
  if drafts.is_empty() {
    drafts.push(DraftNode {
      kind: NodeKind::Inline(InlineKind::Text {
        value: String::new(),
      }),
      inline_syntax: None,
      children: Vec::new(),
      caret_offset: Some(0),
    });
  }

  let mut next_id = document.next_available_id().0;
  let mut materialized = Materialized::default();
  let subtrees = drafts
    .into_iter()
    .map(|draft| materialize_root(draft, &mut next_id, &mut materialized))
    .collect::<Vec<_>>();
  let caret = materialized
    .caret
    .ok_or(EditError::UnsupportedStructure(start.point.node))?;

  let mut operations = selected_roots
    .into_iter()
    .map(|node| Operation::RemoveNode { node })
    .collect::<Vec<_>>();
  operations.extend(subtrees.into_iter().enumerate().map(|(offset, subtree)| {
    Operation::InsertSubtree {
      parent: start.block,
      index: start.top_index + offset,
      subtree,
    }
  }));

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

fn transform_node(
  document: &Document,
  node_id: NodeId,
  transform: &mut Transform<'_>,
) -> Result<Vec<DraftPart>, EditError> {
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;

  if let NodeKind::Inline(InlineKind::Text { value }) = &node.kind {
    return transform_text(node_id, value, transform);
  }

  if node.children.is_empty() {
    return match transform.phase {
      Phase::Before | Phase::After => Ok(vec![DraftPart::Node(clone_draft(document, node_id)?)]),
      Phase::Selected => Ok(Vec::new()),
    };
  }

  let mut child_parts = Vec::new();
  for child in &node.children {
    child_parts.extend(transform_node(document, *child, transform)?);
  }

  if !matches!(node.kind, NodeKind::Inline(_)) {
    return Ok(child_parts);
  }
  wrap_inline(node, child_parts)
}

fn transform_text(
  node_id: NodeId,
  value: &str,
  transform: &mut Transform<'_>,
) -> Result<Vec<DraftPart>, EditError> {
  match transform.phase {
    Phase::Before => {
      if node_id != transform.start.node {
        return Ok(vec![DraftPart::Node(text_draft(value.to_string(), None))]);
      }
      let byte = utf16_to_byte(value, node_id, transform.start.offset_utf16)?;
      let mut kept = value[..byte].to_string();
      kept.push_str(transform.inserted);
      let caret = transform.start.offset_utf16 + transform.inserted.encode_utf16().count() as u32;
      transform.phase = Phase::Selected;
      if kept.is_empty() {
        Ok(vec![DraftPart::Caret])
      } else {
        Ok(vec![DraftPart::Node(text_draft(kept, Some(caret)))])
      }
    }
    Phase::Selected => {
      if node_id != transform.end.node {
        return Ok(Vec::new());
      }
      let byte = utf16_to_byte(value, node_id, transform.end.offset_utf16)?;
      transform.phase = Phase::After;
      if byte == value.len() {
        Ok(Vec::new())
      } else {
        Ok(vec![DraftPart::Node(text_draft(
          value[byte..].to_string(),
          None,
        ))])
      }
    }
    Phase::After => Ok(vec![DraftPart::Node(text_draft(value.to_string(), None))]),
  }
}

fn wrap_inline(node: &Node, parts: Vec<DraftPart>) -> Result<Vec<DraftPart>, EditError> {
  let caret_count = parts
    .iter()
    .filter(|part| matches!(part, DraftPart::Caret))
    .count();
  if caret_count > 1 {
    return Err(EditError::UnsupportedStructure(node.id));
  }

  let node_count = parts
    .iter()
    .filter(|part| matches!(part, DraftPart::Node(_)))
    .count();
  if node_count == 0 {
    return if caret_count == 1 {
      Ok(vec![DraftPart::Caret])
    } else {
      Ok(Vec::new())
    };
  }

  let children = parts
    .into_iter()
    .map(|part| match part {
      DraftPart::Node(node) => node,
      DraftPart::Caret => text_draft(String::new(), Some(0)),
    })
    .collect::<Vec<_>>();
  Ok(vec![DraftPart::Node(DraftNode {
    kind: node.kind.clone(),
    inline_syntax: node.inline_syntax.clone(),
    children,
    caret_offset: None,
  })])
}

fn text_draft(value: String, caret_offset: Option<u32>) -> DraftNode {
  DraftNode {
    kind: NodeKind::Inline(InlineKind::Text { value }),
    inline_syntax: None,
    children: Vec::new(),
    caret_offset,
  }
}

fn clone_draft(document: &Document, node_id: NodeId) -> Result<DraftNode, EditError> {
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  Ok(DraftNode {
    kind: node.kind.clone(),
    inline_syntax: node.inline_syntax.clone(),
    children: node
      .children
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?,
    caret_offset: None,
  })
}

fn materialize_root(
  draft: DraftNode,
  next_id: &mut u64,
  state: &mut Materialized,
) -> DetachedSubtree {
  let mut nodes = Vec::new();
  let root = materialize_node(draft, None, next_id, &mut nodes, state);
  DetachedSubtree { root, nodes }
}

fn materialize_node(
  draft: DraftNode,
  parent: Option<NodeId>,
  next_id: &mut u64,
  nodes: &mut Vec<Node>,
  state: &mut Materialized,
) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  let index = nodes.len();
  let mut node = Node::new(id, draft.kind, None);
  node.parent = parent;
  node.inline_syntax = draft.inline_syntax;
  nodes.push(node);
  let children = draft
    .children
    .into_iter()
    .map(|child| materialize_node(child, Some(id), next_id, nodes, state))
    .collect::<Vec<_>>();
  nodes[index].children = children;
  if let Some(offset_utf16) = draft.caret_offset {
    state.caret = Some(SelectionPoint {
      node: id,
      offset_utf16,
    });
  }
  id
}

fn endpoint(
  document: &Document,
  point: SelectionPoint,
  text_order: &[NodeId],
) -> Result<Endpoint, EditError> {
  text_value(document, point.node)?;
  let text_index = text_order
    .iter()
    .position(|node| *node == point.node)
    .ok_or(EditError::UnsupportedStructure(point.node))?;
  let mut current = point.node;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    let parent = node
      .parent
      .ok_or(EditError::UnsupportedStructure(current))?;
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match &parent_node.kind {
      NodeKind::Block(_) => {
        let top_index = document
          .child_index(parent, current)
          .ok_or(EditError::UnsupportedStructure(current))?;
        return Ok(Endpoint {
          point,
          block: parent,
          top_index,
          text_index,
        });
      }
      NodeKind::Inline(_) => current = parent,
      NodeKind::Document => return Err(EditError::UnsupportedStructure(parent)),
    }
  }
}

fn ordered_text_nodes(
  document: &Document,
  anchor: NodeId,
  focus: NodeId,
) -> Result<Vec<NodeId>, EditError> {
  let anchor_block = block_ancestor(document, anchor)?;
  let focus_block = block_ancestor(document, focus)?;
  if anchor_block != focus_block {
    return Err(EditError::CrossNodeSelection);
  }
  let mut nodes = Vec::new();
  collect_text_nodes(document, anchor_block, &mut nodes);
  Ok(nodes)
}

fn block_ancestor(document: &Document, start: NodeId) -> Result<NodeId, EditError> {
  let mut current = start;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    let parent = node
      .parent
      .ok_or(EditError::UnsupportedStructure(current))?;
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match parent_node.kind {
      NodeKind::Block(_) => return Ok(parent),
      NodeKind::Inline(_) => current = parent,
      NodeKind::Document => return Err(EditError::UnsupportedStructure(parent)),
    }
  }
}

fn collect_text_nodes(document: &Document, root: NodeId, output: &mut Vec<NodeId>) {
  let Some(node) = document.node(root) else {
    return;
  };
  if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    output.push(root);
    return;
  }
  for child in &node.children {
    collect_text_nodes(document, *child, output);
  }
}

fn validate_boundary(document: &Document, point: SelectionPoint) -> Result<(), EditError> {
  utf16_to_byte(
    text_value(document, point.node)?,
    point.node,
    point.offset_utf16,
  )
  .map(|_| ())
}

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  let node = document.node(node).ok_or(EditError::NodeNotFound(node))?;
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node.id)),
  }
}
