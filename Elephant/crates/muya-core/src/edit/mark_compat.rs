use crate::model::{Document, InlineKind, InlineMarkKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::{EditError, Operation, Transaction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkCommand {
  ToggleStrong,
  ToggleEmphasis,
  ToggleStrike,
}

#[derive(Clone, Copy)]
struct CrossEndpoint {
  point: SelectionPoint,
  block: NodeId,
  top_level: NodeId,
  top_index: usize,
}

impl MarkCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let fragment_kind = self.fragment_kind();
    if let Some(group) =
      super::mark_fragment_toggle::selected_group(document, selection, fragment_kind)?
    {
      if self == Self::ToggleStrike {
        return Ok(noop(selection));
      }
      return super::mark_fragment_toggle::build_unwrap_group(document, selection, group);
    }
    if let Some(wrapper) = selected_mark_ancestor(document, selection, self)? {
      if is_muya_nested_emphasis_noop(document, wrapper, self)? {
        return Ok(noop(selection));
      }
      return build_unwrap_ancestor(document, selection, wrapper);
    }
    if is_partial_cross_wrapper_selection(document, selection)? {
      return super::mark_fragments::build_partial_cross_wrapper_toggle(
        document,
        selection,
        fragment_kind,
      );
    }
    self.generic().build(document, selection)
  }

  fn generic(self) -> super::mark::MarkCommand {
    match self {
      Self::ToggleStrong => super::mark::MarkCommand::ToggleStrong,
      Self::ToggleEmphasis => super::mark::MarkCommand::ToggleEmphasis,
      Self::ToggleStrike => super::mark::MarkCommand::ToggleStrike,
    }
  }

  fn fragment_kind(self) -> InlineMarkKind {
    match self {
      Self::ToggleStrong => InlineMarkKind::Strong,
      Self::ToggleEmphasis => InlineMarkKind::Emphasis,
      Self::ToggleStrike => InlineMarkKind::Strike,
    }
  }
}

fn selected_mark_ancestor(
  document: &Document,
  selection: Selection,
  command: MarkCommand,
) -> Result<Option<NodeId>, EditError> {
  let Some((text, start, end)) = selection.ordered_same_node() else {
    return Ok(None);
  };
  if start == end {
    return Ok(None);
  }

  let mut current = text;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    let Some(parent) = node.parent else {
      return Ok(None);
    };
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match &parent_node.kind {
      NodeKind::Inline(kind) if matches_command(kind, command) => return Ok(Some(parent)),
      NodeKind::Inline(_) => current = parent,
      NodeKind::Block(_) | NodeKind::Document => return Ok(None),
    }
  }
}

fn matches_command(kind: &InlineKind, command: MarkCommand) -> bool {
  matches!(
    (kind, command),
    (InlineKind::Strong, MarkCommand::ToggleStrong)
      | (InlineKind::Emphasis, MarkCommand::ToggleEmphasis)
      | (InlineKind::Strike, MarkCommand::ToggleStrike)
  )
}

fn is_muya_nested_emphasis_noop(
  document: &Document,
  wrapper: NodeId,
  command: MarkCommand,
) -> Result<bool, EditError> {
  if command != MarkCommand::ToggleEmphasis {
    return Ok(false);
  }
  let parent = document
    .node(wrapper)
    .ok_or(EditError::NodeNotFound(wrapper))?
    .parent;
  Ok(matches!(
    parent
      .and_then(|id| document.node(id))
      .map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Strong))
  ))
}

fn is_partial_cross_wrapper_selection(
  document: &Document,
  selection: Selection,
) -> Result<bool, EditError> {
  if selection.ordered_same_node().is_some() {
    return Ok(false);
  }
  let anchor = cross_endpoint(document, selection.anchor)?;
  let focus = cross_endpoint(document, selection.focus)?;
  if anchor.block != focus.block || anchor.top_index == focus.top_index {
    return Ok(false);
  }
  let (start, end) = if anchor.top_index < focus.top_index {
    (anchor, focus)
  } else {
    (focus, anchor)
  };
  let end_length = text_value(document, end.point.node)?.encode_utf16().count() as u32;
  Ok(
    start.point.offset_utf16 != 0
      || end.point.offset_utf16 != end_length
      || first_text_descendant(document, start.top_level)? != start.point.node
      || last_text_descendant(document, end.top_level)? != end.point.node,
  )
}

fn cross_endpoint(document: &Document, point: SelectionPoint) -> Result<CrossEndpoint, EditError> {
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
        return Ok(CrossEndpoint {
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

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  let node = document.node(node).ok_or(EditError::NodeNotFound(node))?;
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node.id)),
  }
}

fn noop(selection: Selection) -> Transaction {
  Transaction {
    operations: Vec::new(),
    selection_before: selection,
    selection_after: selection,
  }
}

fn build_unwrap_ancestor(
  document: &Document,
  selection: Selection,
  wrapper: NodeId,
) -> Result<Transaction, EditError> {
  let wrapper_node = document
    .node(wrapper)
    .ok_or(EditError::NodeNotFound(wrapper))?;
  let parent = wrapper_node
    .parent
    .ok_or(EditError::UnsupportedStructure(wrapper))?;
  let index = document
    .child_index(parent, wrapper)
    .ok_or(EditError::UnsupportedStructure(wrapper))?;
  let children = wrapper_node.children.clone();

  let mut operations = children
    .into_iter()
    .enumerate()
    .map(|(offset, node)| Operation::MoveNode {
      node,
      new_parent: parent,
      new_index: index + offset,
    })
    .collect::<Vec<_>>();
  operations.push(Operation::RemoveNode { node: wrapper });

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::Node;
  use crate::{parse_markdown, to_markdown};

  fn text_with_value<'a>(document: &'a Document, expected: &str) -> &'a Node {
    document
      .nodes
      .values()
      .find(|node| {
        matches!(
          &node.kind,
          NodeKind::Inline(InlineKind::Text { value }) if value == expected
        )
      })
      .unwrap()
  }

  fn selection(document: &Document, start: u32, end: u32) -> Selection {
    let text = text_with_value(document, "alpha").id;
    Selection {
      anchor: SelectionPoint {
        node: text,
        offset_utf16: start,
      },
      focus: SelectionPoint {
        node: text,
        offset_utf16: end,
      },
    }
  }

  #[test]
  fn unwraps_the_whole_strong_ancestor_for_a_partial_selection() {
    let mut document = parse_markdown("**alpha**");
    let transaction = MarkCommand::ToggleStrong
      .build(&document, selection(&document, 1, 4))
      .unwrap();
    let inverse = transaction.apply(&mut document).unwrap();

    assert_eq!(to_markdown(&document), "alpha");
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**alpha**");
  }

  #[test]
  fn keeps_nested_emphasis_inside_strong_as_a_muya_noop() {
    let mut document = parse_markdown("***alpha***");
    let transaction = MarkCommand::ToggleEmphasis
      .build(&document, selection(&document, 0, 5))
      .unwrap();

    assert!(transaction.operations.is_empty());
    transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "***alpha***");
  }

  #[test]
  fn falls_back_to_the_generic_builder_for_plain_text() {
    let mut document = parse_markdown("alpha");
    MarkCommand::ToggleEmphasis
      .build(&document, selection(&document, 1, 4))
      .unwrap()
      .apply(&mut document)
      .unwrap();

    assert_eq!(to_markdown(&document), "a*lph*a");
  }

  #[test]
  fn routes_partial_cross_wrapper_selection_to_fragments() {
    let mut document = parse_markdown("**alpha** beta *gamma*");
    let selection = Selection {
      anchor: SelectionPoint {
        node: text_with_value(&document, "alpha").id,
        offset_utf16: 2,
      },
      focus: SelectionPoint {
        node: text_with_value(&document, "gamma").id,
        offset_utf16: 3,
      },
    };

    MarkCommand::ToggleStrike
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**al~~pha** beta *gam~~ma*");
  }

  #[test]
  fn keeps_a_complete_linked_strike_group_as_a_muya_noop() {
    let mut document = parse_markdown("**alpha** beta *gamma*");
    let selection = Selection {
      anchor: SelectionPoint {
        node: text_with_value(&document, "alpha").id,
        offset_utf16: 2,
      },
      focus: SelectionPoint {
        node: text_with_value(&document, "gamma").id,
        offset_utf16: 3,
      },
    };
    let apply = MarkCommand::ToggleStrike
      .build(&document, selection)
      .unwrap();
    let linked_selection = apply.selection_after;
    apply.apply(&mut document).unwrap();

    let repeated = MarkCommand::ToggleStrike
      .build(&document, linked_selection)
      .unwrap();
    assert!(repeated.operations.is_empty());
    repeated.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**al~~pha** beta *gam~~ma*");
  }
}
