use crate::model::{Document, InlineKind, InlineMarkKind, MarkFragmentEdge, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::{EditError, Operation, Transaction};

#[derive(Clone, Copy)]
struct FragmentBoundary {
  node: NodeId,
  group: u64,
  edge: MarkFragmentEdge,
}

struct FragmentContext {
  node: NodeId,
  parent: NodeId,
  index: usize,
  children: Vec<NodeId>,
}

pub(crate) fn selected_group(
  document: &Document,
  selection: Selection,
  mark: InlineMarkKind,
) -> Result<Option<u64>, EditError> {
  let Some(anchor) = fragment_boundary(document, selection.anchor.node, mark)? else {
    return Ok(None);
  };
  let Some(focus) = fragment_boundary(document, selection.focus.node, mark)? else {
    return Ok(None);
  };
  if anchor.group != focus.group {
    return Ok(None);
  }

  let (start, start_point, end, end_point) = match (anchor.edge, focus.edge) {
    (MarkFragmentEdge::Start, MarkFragmentEdge::End) => {
      (anchor, selection.anchor, focus, selection.focus)
    }
    (MarkFragmentEdge::End, MarkFragmentEdge::Start) => {
      (focus, selection.focus, anchor, selection.anchor)
    }
    _ => return Ok(None),
  };

  if first_text_descendant(document, start.node)? != start_point.node
    || start_point.offset_utf16 != 0
  {
    return Ok(None);
  }
  let end_text = last_text_descendant(document, end.node)?;
  if end_text != end_point.node || end_point.offset_utf16 != text_len(document, end_text)? {
    return Ok(None);
  }

  Ok(Some(anchor.group))
}

pub(crate) fn build_unwrap_group(
  document: &Document,
  selection: Selection,
  group: u64,
) -> Result<Transaction, EditError> {
  let mut fragments = document
    .nodes
    .values()
    .filter_map(|node| match &node.kind {
      NodeKind::Inline(InlineKind::MarkFragment {
        group: candidate, ..
      }) if *candidate == group => {
        let parent = node.parent?;
        let index = document.child_index(parent, node.id)?;
        Some(FragmentContext {
          node: node.id,
          parent,
          index,
          children: node.children.clone(),
        })
      }
      _ => None,
    })
    .collect::<Vec<_>>();
  if fragments.len() < 2 {
    return Err(EditError::UnsupportedStructure(NodeId(group)));
  }

  fragments.sort_by(|left, right| {
    left
      .parent
      .cmp(&right.parent)
      .then_with(|| right.index.cmp(&left.index))
  });
  let mut operations = Vec::new();
  for fragment in fragments {
    operations.extend(
      fragment
        .children
        .into_iter()
        .enumerate()
        .map(|(offset, node)| Operation::MoveNode {
          node,
          new_parent: fragment.parent,
          new_index: fragment.index + offset,
        }),
    );
    operations.push(Operation::RemoveNode {
      node: fragment.node,
    });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn fragment_boundary(
  document: &Document,
  start: NodeId,
  mark: InlineMarkKind,
) -> Result<Option<FragmentBoundary>, EditError> {
  let mut current = start;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if let NodeKind::Inline(InlineKind::MarkFragment {
      mark: candidate,
      group,
      edge,
    }) = &node.kind
    {
      return Ok((*candidate == mark).then_some(FragmentBoundary {
        node: current,
        group: *group,
        edge: *edge,
      }));
    }
    let Some(parent) = node.parent else {
      return Ok(None);
    };
    match document.node(parent).map(|node| &node.kind) {
      Some(NodeKind::Inline(_)) => current = parent,
      Some(NodeKind::Block(_)) | Some(NodeKind::Document) | None => return Ok(None),
    }
  }
}

fn first_text_descendant(document: &Document, root: NodeId) -> Result<NodeId, EditError> {
  let mut stack = vec![root];
  while let Some(current) = stack.pop() {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
      return Ok(current);
    }
    stack.extend(node.children.iter().rev().copied());
  }
  Err(EditError::UnsupportedStructure(root))
}

fn last_text_descendant(document: &Document, root: NodeId) -> Result<NodeId, EditError> {
  let mut stack = vec![root];
  while let Some(current) = stack.pop() {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
      return Ok(current);
    }
    stack.extend(node.children.iter().copied());
  }
  Err(EditError::UnsupportedStructure(root))
}

fn text_len(document: &Document, node: NodeId) -> Result<u32, EditError> {
  match &document
    .node(node)
    .ok_or(EditError::NodeNotFound(node))?
    .kind
  {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value.encode_utf16().count() as u32),
    _ => Err(EditError::NotTextNode(node)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::edit::MarkCommand;
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
  fn unwraps_a_complete_linked_strike_group_and_undoes() {
    let mut document = parse_markdown("**alpha** beta *gamma*");
    let initial_selection = Selection {
      anchor: SelectionPoint {
        node: text(&document, "alpha"),
        offset_utf16: 2,
      },
      focus: SelectionPoint {
        node: text(&document, "gamma"),
        offset_utf16: 3,
      },
    };
    let apply = MarkCommand::ToggleStrike
      .build(&document, initial_selection)
      .unwrap();
    let linked_selection = apply.selection_after;
    apply.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**al~~pha** beta *gam~~ma*");

    let group = selected_group(&document, linked_selection, InlineMarkKind::Strike)
      .unwrap()
      .unwrap();
    let inverse = build_unwrap_group(&document, linked_selection, group)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**alpha** beta *gamma*");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**al~~pha** beta *gam~~ma*");
  }
}
