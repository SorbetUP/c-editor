use crate::model::{Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkCommand {
  ToggleStrong,
  ToggleEmphasis,
  ToggleStrike,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MarkKind {
  Strong,
  Emphasis,
  Strike,
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
    let mark = match self {
      Self::ToggleStrong => MarkKind::Strong,
      Self::ToggleEmphasis => MarkKind::Emphasis,
      Self::ToggleStrike => MarkKind::Strike,
    };
    build_toggle_mark(document, selection, mark)
  }
}

impl MarkKind {
  fn inline(self) -> InlineKind {
    match self {
      Self::Strong => InlineKind::Strong,
      Self::Emphasis => InlineKind::Emphasis,
      Self::Strike => InlineKind::Strike,
    }
  }

  fn matches(self, kind: &NodeKind) -> bool {
    matches!(
      (self, kind),
      (Self::Strong, NodeKind::Inline(InlineKind::Strong))
        | (Self::Emphasis, NodeKind::Inline(InlineKind::Emphasis))
        | (Self::Strike, NodeKind::Inline(InlineKind::Strike))
    )
  }
}

fn build_toggle_mark(
  document: &Document,
  selection: Selection,
  mark: MarkKind,
) -> Result<Transaction, EditError> {
  if let Some((text, start, end)) = selection.ordered_same_node() {
    return build_same_text_toggle(document, selection, text, start, end, mark);
  }
  build_cross_wrapper_toggle(document, selection, mark)
}

fn build_same_text_toggle(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  mark: MarkKind,
) -> Result<Transaction, EditError> {
  if start == end {
    return Ok(Transaction {
      operations: Vec::new(),
      selection_before: selection,
      selection_after: selection,
    });
  }

  let value = text_value(document, text)?;
  let start_byte = utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let text_node = document.node(text).ok_or(EditError::NodeNotFound(text))?;
  let parent = text_node
    .parent
    .ok_or(EditError::UnsupportedStructure(text))?;
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;

  if mark.matches(&parent_node.kind) {
    return build_unwrap_mark(
      document, selection, text, parent, start, end, start_byte, end_byte,
    );
  }

  build_wrap_mark(
    document, selection, text, parent, start, end, start_byte, end_byte, mark,
  )
}

fn build_cross_wrapper_toggle(
  document: &Document,
  selection: Selection,
  mark: MarkKind,
) -> Result<Transaction, EditError> {
  let anchor = endpoint(document, selection.anchor)?;
  let focus = endpoint(document, selection.focus)?;
  if anchor.block != focus.block || anchor.top_index == focus.top_index {
    return Err(EditError::CrossNodeSelection);
  }

  let (start, end) = if anchor.top_index < focus.top_index {
    (anchor, focus)
  } else {
    (focus, anchor)
  };
  let end_length = text_value(document, end.point.node)?.encode_utf16().count() as u32;
  if start.point.offset_utf16 != 0 || end.point.offset_utf16 != end_length {
    return Err(EditError::UnsupportedStructure(start.point.node));
  }
  if first_text_descendant(document, start.top_level)? != start.point.node
    || last_text_descendant(document, end.top_level)? != end.point.node
  {
    return Err(EditError::UnsupportedStructure(start.block));
  }
  utf16_to_byte(
    text_value(document, start.point.node)?,
    start.point.node,
    start.point.offset_utf16,
  )?;
  utf16_to_byte(
    text_value(document, end.point.node)?,
    end.point.node,
    end.point.offset_utf16,
  )?;
  let block_node = document
    .node(start.block)
    .ok_or(EditError::NodeNotFound(start.block))?;
  let selected = block_node.children[start.top_index..=end.top_index].to_vec();
  if selected.iter().any(|node| {
    document
      .node(*node)
      .is_some_and(|candidate| mark.matches(&candidate.kind))
  }) {
    return Err(EditError::UnsupportedStructure(start.block));
  }

  let wrapper = document.next_available_id();
  let mut operations = vec![Operation::InsertNode {
    parent: start.block,
    index: start.top_index,
    node: Node::new(wrapper, NodeKind::Inline(mark.inline()), None),
  }];
  operations.extend(
    selected
      .into_iter()
      .enumerate()
      .map(|(index, node)| Operation::MoveNode {
        node,
        new_parent: wrapper,
        new_index: index,
      }),
  );

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn endpoint(document: &Document, point: SelectionPoint) -> Result<CrossEndpoint, EditError> {
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

#[allow(clippy::too_many_arguments)]
fn build_wrap_mark(
  document: &Document,
  selection: Selection,
  text: NodeId,
  parent: NodeId,
  start: u32,
  end: u32,
  start_byte: usize,
  end_byte: usize,
  mark: MarkKind,
) -> Result<Transaction, EditError> {
  let value = text_value(document, text)?;
  let full_length = value.encode_utf16().count() as u32;
  let parent_index = document
    .child_index(parent, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let selected = value[start_byte..end_byte].to_string();
  let suffix = value[end_byte..].to_string();
  let mut next_id = document.next_available_id().0;
  let wrapper_id = take_id(&mut next_id);
  let selected_id = take_id(&mut next_id);
  let suffix_id = take_id(&mut next_id);

  let mut operations = vec![
    Operation::ReplaceText {
      node: text,
      range: Utf16Range::new(start, full_length),
      inserted: String::new(),
    },
    Operation::InsertNode {
      parent,
      index: parent_index + 1,
      node: Node::new(wrapper_id, NodeKind::Inline(mark.inline()), None),
    },
    Operation::InsertNode {
      parent: wrapper_id,
      index: 0,
      node: text_node(selected_id, selected),
    },
  ];
  if !suffix.is_empty() {
    operations.push(Operation::InsertNode {
      parent,
      index: parent_index + 2,
      node: text_node(suffix_id, suffix),
    });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selected_selection(selected_id, end - start),
  })
}

#[allow(clippy::too_many_arguments)]
fn build_unwrap_mark(
  document: &Document,
  selection: Selection,
  text: NodeId,
  wrapper: NodeId,
  start: u32,
  end: u32,
  start_byte: usize,
  end_byte: usize,
) -> Result<Transaction, EditError> {
  let value = text_value(document, text)?;
  let full_length = value.encode_utf16().count() as u32;
  let wrapper_node = document
    .node(wrapper)
    .ok_or(EditError::NodeNotFound(wrapper))?;
  if wrapper_node.children.as_slice() != [text] {
    return Err(EditError::UnsupportedStructure(wrapper));
  }
  let grandparent = wrapper_node
    .parent
    .ok_or(EditError::UnsupportedStructure(wrapper))?;
  let wrapper_index = document
    .child_index(grandparent, wrapper)
    .ok_or(EditError::UnsupportedStructure(wrapper))?;

  if start == 0 && end == full_length {
    let detached_text = Node::new(
      text,
      document
        .node(text)
        .ok_or(EditError::NodeNotFound(text))?
        .kind
        .clone(),
      None,
    );
    return Ok(Transaction {
      operations: vec![
        Operation::RemoveNode { node: text },
        Operation::RemoveNode { node: wrapper },
        Operation::InsertNode {
          parent: grandparent,
          index: wrapper_index,
          node: detached_text,
        },
      ],
      selection_before: selection,
      selection_after: selection,
    });
  }

  let selected = value[start_byte..end_byte].to_string();
  let suffix = value[end_byte..].to_string();
  let mut next_id = document.next_available_id().0;
  let selected_id = take_id(&mut next_id);
  let selected_node = text_node(selected_id, selected);
  let mut operations = Vec::new();

  if start == 0 {
    operations.extend([
      Operation::ReplaceText {
        node: text,
        range: Utf16Range::new(0, end),
        inserted: String::new(),
      },
      Operation::InsertNode {
        parent: grandparent,
        index: wrapper_index,
        node: selected_node,
      },
    ]);
  } else {
    operations.extend([
      Operation::ReplaceText {
        node: text,
        range: Utf16Range::new(start, full_length),
        inserted: String::new(),
      },
      Operation::InsertNode {
        parent: grandparent,
        index: wrapper_index + 1,
        node: selected_node,
      },
    ]);

    if end < full_length {
      let suffix_wrapper_id = take_id(&mut next_id);
      let suffix_text_id = take_id(&mut next_id);
      operations.extend([
        Operation::InsertNode {
          parent: grandparent,
          index: wrapper_index + 2,
          node: Node::new(suffix_wrapper_id, wrapper_node.kind.clone(), None),
        },
        Operation::InsertNode {
          parent: suffix_wrapper_id,
          index: 0,
          node: text_node(suffix_text_id, suffix),
        },
      ]);
    }
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selected_selection(selected_id, end - start),
  })
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

fn selected_selection(node: NodeId, length: u32) -> Selection {
  Selection {
    anchor: SelectionPoint {
      node,
      offset_utf16: 0,
    },
    focus: SelectionPoint {
      node,
      offset_utf16: length,
    },
  }
}

fn text_node(id: NodeId, value: String) -> Node {
  Node::new(id, NodeKind::Inline(InlineKind::Text { value }), None)
}

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  match &document
    .node(node)
    .ok_or(EditError::NodeNotFound(node))?
    .kind
  {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node)),
  }
}

fn take_id(next_id: &mut u64) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  id
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn first_text(document: &Document, root: NodeId) -> NodeId {
    first_text_descendant(document, root).unwrap()
  }

  fn marked_text(document: &Document) -> NodeId {
    let block = document.children(document.root).next().unwrap().id;
    let wrapper = document.children(block).next().unwrap().id;
    first_text(document, wrapper)
  }

  fn selection(node: NodeId, start: u32, end: u32) -> Selection {
    Selection {
      anchor: SelectionPoint {
        node,
        offset_utf16: start,
      },
      focus: SelectionPoint {
        node,
        offset_utf16: end,
      },
    }
  }

  #[test]
  fn unwraps_the_middle_of_a_strong_mark_and_undoes_exactly() {
    let mut document = parse_markdown("**abcdef**");
    let text = marked_text(&document);
    let inverse = MarkCommand::ToggleStrong
      .build(&document, selection(text, 2, 5))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**ab**cde**f**");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**abcdef**");
  }

  #[test]
  fn unwraps_the_prefix_of_a_strong_mark() {
    let mut document = parse_markdown("**abcdef**");
    let text = marked_text(&document);
    MarkCommand::ToggleStrong
      .build(&document, selection(text, 0, 3))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "abc**def**");
  }

  #[test]
  fn unwraps_the_suffix_of_a_strong_mark() {
    let mut document = parse_markdown("**abcdef**");
    let text = marked_text(&document);
    MarkCommand::ToggleStrong
      .build(&document, selection(text, 3, 6))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**abc**def");
  }

  #[test]
  fn wraps_a_plain_text_selection() {
    let mut document = parse_markdown("abcdef");
    let block = document.children(document.root).next().unwrap().id;
    let text = first_text(&document, block);
    MarkCommand::ToggleEmphasis
      .build(&document, selection(text, 1, 4))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "a*bcd*ef");
  }

  #[test]
  fn wraps_complete_subtrees_across_different_inline_marks() {
    let mut document = parse_markdown("**bold** and *soft*");
    let block = document.children(document.root).next().unwrap().id;
    let children = document
      .children(block)
      .map(|node| node.id)
      .collect::<Vec<_>>();
    let strong_text = first_text(&document, children[0]);
    let emphasis_text = first_text(&document, children[2]);
    let selection = Selection {
      anchor: SelectionPoint {
        node: strong_text,
        offset_utf16: 0,
      },
      focus: SelectionPoint {
        node: emphasis_text,
        offset_utf16: 4,
      },
    };

    let inverse = MarkCommand::ToggleStrike
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "~~**bold** and *soft*~~");
    assert!(document.node(children[0]).unwrap().parent.is_some());
    assert!(document.node(children[2]).unwrap().parent.is_some());

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**bold** and *soft*");
    assert_eq!(document.node(children[0]).unwrap().parent, Some(block));
    assert_eq!(document.node(children[2]).unwrap().parent, Some(block));
  }

  #[test]
  fn accepts_a_reversed_cross_wrapper_selection() {
    let mut document = parse_markdown("**bold** and *soft*");
    let block = document.children(document.root).next().unwrap().id;
    let children = document
      .children(block)
      .map(|node| node.id)
      .collect::<Vec<_>>();
    let strong_text = first_text(&document, children[0]);
    let emphasis_text = first_text(&document, children[2]);
    let selection = Selection {
      anchor: SelectionPoint {
        node: emphasis_text,
        offset_utf16: 4,
      },
      focus: SelectionPoint {
        node: strong_text,
        offset_utf16: 0,
      },
    };

    MarkCommand::ToggleStrike
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "~~**bold** and *soft*~~");
  }
}
