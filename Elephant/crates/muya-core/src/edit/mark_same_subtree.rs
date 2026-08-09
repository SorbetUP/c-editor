use crate::model::{
  DetachedSubtree, Document, InlineKind, InlineMarkKind, MarkFragmentEdge, Node, NodeId, NodeKind,
};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction};

#[derive(Clone, Copy)]
struct Endpoint {
  point: SelectionPoint,
  block: NodeId,
  top_level: NodeId,
  top_index: usize,
}

#[derive(Clone, Copy)]
enum BoundaryRole {
  Start,
  End,
}

struct DraftNode {
  kind: NodeKind,
  children: Vec<DraftNode>,
  role: Option<BoundaryRole>,
}

#[derive(Default)]
struct MaterializedBoundaries {
  start: Option<NodeId>,
  end: Option<(NodeId, u32)>,
}

pub(crate) fn build_partial_same_top_level_toggle(
  document: &Document,
  selection: Selection,
  mark: InlineMarkKind,
) -> Result<Option<Transaction>, EditError> {
  if selection.ordered_same_node().is_some() {
    return Ok(None);
  }
  let anchor = endpoint(document, selection.anchor)?;
  let focus = endpoint(document, selection.focus)?;
  if anchor.block != focus.block
    || anchor.top_level != focus.top_level
    || anchor.point.node == focus.point.node
  {
    return Ok(None);
  }
  if subtree_contains_mark(document, anchor.top_level, mark) {
    return Ok(None);
  }

  let anchor_path = path_to_node(document, anchor.top_level, anchor.point.node)
    .ok_or(EditError::UnsupportedStructure(anchor.top_level))?;
  let focus_path = path_to_node(document, focus.top_level, focus.point.node)
    .ok_or(EditError::UnsupportedStructure(focus.top_level))?;
  let (start, end) = if anchor_path < focus_path {
    (anchor, focus)
  } else {
    (focus, anchor)
  };
  validate_boundary(document, start.point)?;
  validate_boundary(document, end.point)?;

  let start_length = text_value(document, start.point.node)?
    .encode_utf16()
    .count() as u32;
  if start.point.offset_utf16 >= start_length || end.point.offset_utf16 == 0 {
    return Ok(None);
  }

  let mut next_id = document.next_available_id().0;
  let group = next_id;
  next_id = next_id.saturating_add(1);
  let draft = split_between(
    document,
    start.top_level,
    start.point,
    end.point,
    mark,
    group,
  )?;
  let mut boundaries = MaterializedBoundaries::default();
  let subtree = materialize_root(draft, &mut next_id, &mut boundaries);
  let start_point = SelectionPoint {
    node: boundaries
      .start
      .ok_or(EditError::UnsupportedStructure(start.point.node))?,
    offset_utf16: 0,
  };
  let (end_node, end_offset) = boundaries
    .end
    .ok_or(EditError::UnsupportedStructure(end.point.node))?;
  let end_point = SelectionPoint {
    node: end_node,
    offset_utf16: end_offset,
  };
  let selection_after = if selection.anchor == start.point {
    Selection {
      anchor: start_point,
      focus: end_point,
    }
  } else {
    Selection {
      anchor: end_point,
      focus: start_point,
    }
  };

  Ok(Some(Transaction {
    operations: vec![
      Operation::RemoveNode {
        node: start.top_level,
      },
      Operation::InsertSubtree {
        parent: start.block,
        index: start.top_index,
        subtree,
      },
    ],
    selection_before: selection,
    selection_after,
  }))
}

fn split_between(
  document: &Document,
  node_id: NodeId,
  start: SelectionPoint,
  end: SelectionPoint,
  mark: InlineMarkKind,
  group: u64,
) -> Result<DraftNode, EditError> {
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  let start_index = node
    .children
    .iter()
    .position(|child| contains_node(document, *child, start.node))
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  let end_index = node
    .children
    .iter()
    .position(|child| contains_node(document, *child, end.node))
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  if start_index > end_index {
    return Err(EditError::UnsupportedStructure(node_id));
  }

  let mut children = node.children[..start_index]
    .iter()
    .map(|child| clone_draft(document, *child))
    .collect::<Result<Vec<_>, _>>()?;
  if start_index == end_index {
    children.push(split_between(
      document,
      node.children[start_index],
      start,
      end,
      mark,
      group,
    )?);
  } else {
    children.extend(split_start_boundary(
      document,
      node.children[start_index],
      start.node,
      start.offset_utf16,
      mark,
      group,
    )?);
    for child in &node.children[start_index + 1..end_index] {
      children.push(fragment(
        mark,
        group,
        MarkFragmentEdge::Middle,
        vec![clone_draft(document, *child)?],
      ));
    }
    children.extend(split_end_boundary(
      document,
      node.children[end_index],
      end.node,
      end.offset_utf16,
      mark,
      group,
    )?);
  }
  children.extend(
    node.children[end_index + 1..]
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?,
  );

  Ok(DraftNode {
    kind: node.kind.clone(),
    children,
    role: None,
  })
}

fn split_start_boundary(
  document: &Document,
  node_id: NodeId,
  target: NodeId,
  offset: u32,
  mark: InlineMarkKind,
  group: u64,
) -> Result<Vec<DraftNode>, EditError> {
  if node_id == target {
    let value = text_value(document, node_id)?;
    let byte = utf16_to_byte(value, node_id, offset)?;
    let mut output = Vec::new();
    if byte > 0 {
      output.push(text_draft(value[..byte].to_string(), None));
    }
    output.push(fragment(
      mark,
      group,
      MarkFragmentEdge::Start,
      vec![text_draft(
        value[byte..].to_string(),
        Some(BoundaryRole::Start),
      )],
    ));
    return Ok(output);
  }

  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  let path_index = node
    .children
    .iter()
    .position(|child| contains_node(document, *child, target))
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  let mut children = node.children[..path_index]
    .iter()
    .map(|child| clone_draft(document, *child))
    .collect::<Result<Vec<_>, _>>()?;
  children.extend(split_start_boundary(
    document,
    node.children[path_index],
    target,
    offset,
    mark,
    group,
  )?);
  if path_index + 1 < node.children.len() {
    let trailing = node.children[path_index + 1..]
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?;
    children.push(fragment(mark, group, MarkFragmentEdge::Middle, trailing));
  }

  Ok(vec![DraftNode {
    kind: node.kind.clone(),
    children,
    role: None,
  }])
}

fn split_end_boundary(
  document: &Document,
  node_id: NodeId,
  target: NodeId,
  offset: u32,
  mark: InlineMarkKind,
  group: u64,
) -> Result<Vec<DraftNode>, EditError> {
  if node_id == target {
    let value = text_value(document, node_id)?;
    let byte = utf16_to_byte(value, node_id, offset)?;
    let mut output = vec![fragment(
      mark,
      group,
      MarkFragmentEdge::End,
      vec![text_draft(
        value[..byte].to_string(),
        Some(BoundaryRole::End),
      )],
    )];
    if byte < value.len() {
      output.push(text_draft(value[byte..].to_string(), None));
    }
    return Ok(output);
  }

  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  let path_index = node
    .children
    .iter()
    .position(|child| contains_node(document, *child, target))
    .ok_or(EditError::UnsupportedStructure(node_id))?;
  let mut children = Vec::new();
  if path_index > 0 {
    let leading = node.children[..path_index]
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?;
    children.push(fragment(mark, group, MarkFragmentEdge::Middle, leading));
  }
  children.extend(split_end_boundary(
    document,
    node.children[path_index],
    target,
    offset,
    mark,
    group,
  )?);
  children.extend(
    node.children[path_index + 1..]
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?,
  );

  Ok(vec![DraftNode {
    kind: node.kind.clone(),
    children,
    role: None,
  }])
}

fn endpoint(document: &Document, point: SelectionPoint) -> Result<Endpoint, EditError> {
  text_value(document, point.node)?;
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
          top_level: current,
          top_index,
        });
      }
      NodeKind::Inline(_) => current = parent,
      NodeKind::Document => return Err(EditError::UnsupportedStructure(parent)),
    }
  }
}

fn path_to_node(document: &Document, root: NodeId, target: NodeId) -> Option<Vec<usize>> {
  if root == target {
    return Some(Vec::new());
  }
  let node = document.node(root)?;
  for (index, child) in node.children.iter().copied().enumerate() {
    if let Some(mut path) = path_to_node(document, child, target) {
      path.insert(0, index);
      return Some(path);
    }
  }
  None
}

fn fragment(
  mark: InlineMarkKind,
  group: u64,
  edge: MarkFragmentEdge,
  children: Vec<DraftNode>,
) -> DraftNode {
  DraftNode {
    kind: NodeKind::Inline(InlineKind::MarkFragment { mark, group, edge }),
    children,
    role: None,
  }
}

fn text_draft(value: String, role: Option<BoundaryRole>) -> DraftNode {
  DraftNode {
    kind: NodeKind::Inline(InlineKind::Text { value }),
    children: Vec::new(),
    role,
  }
}

fn clone_draft(document: &Document, node_id: NodeId) -> Result<DraftNode, EditError> {
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  Ok(DraftNode {
    kind: node.kind.clone(),
    children: node
      .children
      .iter()
      .map(|child| clone_draft(document, *child))
      .collect::<Result<Vec<_>, _>>()?,
    role: None,
  })
}

fn materialize_root(
  draft: DraftNode,
  next_id: &mut u64,
  boundaries: &mut MaterializedBoundaries,
) -> DetachedSubtree {
  let mut nodes = Vec::new();
  let root = materialize_node(draft, None, next_id, &mut nodes, boundaries);
  DetachedSubtree { root, nodes }
}

fn materialize_node(
  draft: DraftNode,
  parent: Option<NodeId>,
  next_id: &mut u64,
  nodes: &mut Vec<Node>,
  boundaries: &mut MaterializedBoundaries,
) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  let text_length = match &draft.kind {
    NodeKind::Inline(InlineKind::Text { value }) => Some(value.encode_utf16().count() as u32),
    _ => None,
  };
  let index = nodes.len();
  let mut node = Node::new(id, draft.kind, None);
  node.parent = parent;
  nodes.push(node);
  let children = draft
    .children
    .into_iter()
    .map(|child| materialize_node(child, Some(id), next_id, nodes, boundaries))
    .collect::<Vec<_>>();
  nodes[index].children = children;

  match draft.role {
    Some(BoundaryRole::Start) => boundaries.start = Some(id),
    Some(BoundaryRole::End) => boundaries.end = Some((id, text_length.unwrap_or(0))),
    None => {}
  }
  id
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

fn contains_node(document: &Document, root: NodeId, target: NodeId) -> bool {
  root == target
    || document.node(root).is_some_and(|node| {
      node
        .children
        .iter()
        .any(|child| contains_node(document, *child, target))
    })
}

fn subtree_contains_mark(document: &Document, root: NodeId, mark: InlineMarkKind) -> bool {
  let Some(node) = document.node(root) else {
    return false;
  };
  let matches = matches!(
    (&node.kind, mark),
    (
      NodeKind::Inline(InlineKind::Emphasis),
      InlineMarkKind::Emphasis
    ) | (NodeKind::Inline(InlineKind::Strong), InlineMarkKind::Strong)
      | (NodeKind::Inline(InlineKind::Strike), InlineMarkKind::Strike)
  ) || matches!(
    &node.kind,
    NodeKind::Inline(InlineKind::MarkFragment { mark: existing, .. }) if *existing == mark
  );
  matches
    || node
      .children
      .iter()
      .any(|child| subtree_contains_mark(document, *child, mark))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn text(document: &Document, value: &str) -> NodeId {
    document
      .nodes
      .values()
      .find(|node| {
        matches!(&node.kind, NodeKind::Inline(InlineKind::Text { value: candidate }) if candidate == value)
      })
      .unwrap()
      .id
  }

  #[test]
  fn marks_a_partial_range_inside_one_strong_subtree_and_undoes() {
    let mut document = parse_markdown("**alpha *beta* gamma**");
    let selection = Selection {
      anchor: SelectionPoint {
        node: text(&document, "alpha "),
        offset_utf16: 2,
      },
      focus: SelectionPoint {
        node: text(&document, " gamma"),
        offset_utf16: 4,
      },
    };

    let inverse = build_partial_same_top_level_toggle(&document, selection, InlineMarkKind::Strike)
      .unwrap()
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**al~~pha *beta* gam~~ma**");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**alpha *beta* gamma**");
  }
}
