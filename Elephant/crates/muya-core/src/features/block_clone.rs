use std::collections::BTreeMap;

use crate::edit::EditError;
use crate::model::{DetachedSubtree, Document, InlineKind, Node, NodeId, NodeKind};

pub(super) struct ClonedBlock {
  pub subtree: DetachedSubtree,
  pub first_text: Option<NodeId>,
}

pub(super) fn clone_block(document: &Document, root: NodeId) -> Result<ClonedBlock, EditError> {
  let mut next_node = document.next_available_id().0;
  let mut next_group = next_mark_group(document);
  let mut groups = BTreeMap::new();
  let mut ids = BTreeMap::new();
  let mut nodes = Vec::new();
  let cloned_root = clone_node(
    document,
    root,
    None,
    &mut next_node,
    &mut next_group,
    &mut groups,
    &mut ids,
    &mut nodes,
  )?;
  let first_text = first_text_descendant(document, root).and_then(|id| ids.get(&id).copied());
  Ok(ClonedBlock {
    subtree: DetachedSubtree {
      root: cloned_root,
      nodes,
    },
    first_text,
  })
}

#[allow(clippy::too_many_arguments)]
fn clone_node(
  document: &Document,
  source: NodeId,
  parent: Option<NodeId>,
  next_node: &mut u64,
  next_group: &mut u64,
  groups: &mut BTreeMap<u64, u64>,
  ids: &mut BTreeMap<NodeId, NodeId>,
  output: &mut Vec<Node>,
) -> Result<NodeId, EditError> {
  let source_node = document
    .node(source)
    .ok_or(EditError::NodeNotFound(source))?;
  let id = NodeId(*next_node);
  *next_node = next_node.saturating_add(1);
  ids.insert(source, id);

  let kind = remap_kind(&source_node.kind, next_group, groups);
  let index = output.len();
  let mut node = Node::new(id, kind, None);
  node.parent = parent;
  output.push(node);
  let children = source_node
    .children
    .iter()
    .copied()
    .map(|child| {
      clone_node(
        document,
        child,
        Some(id),
        next_node,
        next_group,
        groups,
        ids,
        output,
      )
    })
    .collect::<Result<Vec<_>, _>>()?;
  output[index].children = children;
  Ok(id)
}

fn remap_kind(kind: &NodeKind, next_group: &mut u64, groups: &mut BTreeMap<u64, u64>) -> NodeKind {
  let NodeKind::Inline(InlineKind::MarkFragment { mark, group, edge }) = kind else {
    return kind.clone();
  };
  let mapped = *groups.entry(*group).or_insert_with(|| {
    let value = *next_group;
    *next_group = next_group.saturating_add(1);
    value
  });
  NodeKind::Inline(InlineKind::MarkFragment {
    mark: *mark,
    group: mapped,
    edge: *edge,
  })
}

fn next_mark_group(document: &Document) -> u64 {
  document
    .nodes
    .values()
    .filter_map(|node| match node.kind {
      NodeKind::Inline(InlineKind::MarkFragment { group, .. }) => Some(group),
      _ => None,
    })
    .max()
    .unwrap_or(0)
    .saturating_add(1)
}

pub(super) fn first_text_descendant(document: &Document, root: NodeId) -> Option<NodeId> {
  let node = document.node(root)?;
  if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    return Some(root);
  }
  node
    .children
    .iter()
    .find_map(|child| first_text_descendant(document, *child))
}

pub(super) fn last_text_descendant(document: &Document, root: NodeId) -> Option<NodeId> {
  let node = document.node(root)?;
  if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    return Some(root);
  }
  node
    .children
    .iter()
    .rev()
    .find_map(|child| last_text_descendant(document, *child))
}

pub(super) fn text_length(document: &Document, node: NodeId) -> u32 {
  match document.node(node).map(|node| &node.kind) {
    Some(NodeKind::Inline(InlineKind::Text { value })) => value.encode_utf16().count() as u32,
    _ => 0,
  }
}
