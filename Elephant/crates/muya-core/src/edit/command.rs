use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
  InsertText(String),
  InsertParagraph,
  DeleteBackward,
  SetParagraph,
  SetHeading(u8),
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

impl Command {
  pub fn build(&self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    match self {
      Self::InsertText(inserted) => build_replace_selection(document, selection, inserted),
      Self::InsertParagraph => {
        if let (Some(anchor_block), Some(focus_block)) = (
          literal_newline_block_ancestor(document, selection.anchor.node),
          literal_newline_block_ancestor(document, selection.focus.node),
        ) {
          if anchor_block == focus_block {
            return build_replace_selection(document, selection, "\n");
          }
        }
        build_insert_paragraph(document, selection)
      }
      Self::DeleteBackward => build_delete_backward(document, selection),
      Self::SetParagraph => build_set_block_kind(document, selection, BlockKind::Paragraph),
      Self::SetHeading(level) => {
        if !(1..=6).contains(level) {
          return Err(EditError::InvalidHeadingLevel(*level));
        }
        build_set_block_kind(document, selection, BlockKind::Heading { level: *level })
      }
      Self::ToggleStrong => build_toggle_mark(document, selection, MarkKind::Strong),
      Self::ToggleEmphasis => build_toggle_mark(document, selection, MarkKind::Emphasis),
      Self::ToggleStrike => build_toggle_mark(document, selection, MarkKind::Strike),
    }
  }
}

fn literal_newline_block_ancestor(document: &Document, start: NodeId) -> Option<NodeId> {
  let mut current = start;
  while let Some(node) = document.node(current) {
    if matches!(
      node.kind,
      NodeKind::Block(BlockKind::CodeBlock { .. } | BlockKind::TableCell { .. })
    ) {
      return Some(current);
    }
    current = node.parent?;
  }
  None
}

fn build_replace_selection(
  document: &Document,
  selection: Selection,
  inserted: &str,
) -> Result<Transaction, EditError> {
  if let Some((node, start, end)) = selection.ordered_same_node() {
    let value = text_value(document, node)?;
    utf16_to_byte(value, node, start)?;
    utf16_to_byte(value, node, end)?;
    return Ok(Transaction {
      operations: vec![Operation::ReplaceText {
        node,
        range: Utf16Range::new(start, end),
        inserted: inserted.to_string(),
      }],
      selection_before: selection,
      selection_after: Selection::collapsed(SelectionPoint {
        node,
        offset_utf16: start + inserted.encode_utf16().count() as u32,
      }),
    });
  }

  let (start, end, parent, start_index, end_index) =
    ordered_sibling_text_selection(document, selection)?;
  let start_value = text_value(document, start.node)?;
  let end_value = text_value(document, end.node)?;
  let end_byte = utf16_to_byte(end_value, end.node, end.offset_utf16)?;
  let suffix = &end_value[end_byte..];
  let replacement = format!("{inserted}{suffix}");
  let start_length = start_value.encode_utf16().count() as u32;
  let covered = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?
    .children[start_index + 1..=end_index]
    .to_vec();

  let mut operations = vec![Operation::ReplaceText {
    node: start.node,
    range: Utf16Range::new(start.offset_utf16, start_length),
    inserted: replacement,
  }];
  operations.extend(
    covered
      .into_iter()
      .map(|node| Operation::RemoveNode { node }),
  );

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: start.node,
      offset_utf16: start.offset_utf16 + inserted.encode_utf16().count() as u32,
    }),
  })
}

fn ordered_sibling_text_selection(
  document: &Document,
  selection: Selection,
) -> Result<(SelectionPoint, SelectionPoint, NodeId, usize, usize), EditError> {
  let anchor = document
    .node(selection.anchor.node)
    .ok_or(EditError::NodeNotFound(selection.anchor.node))?;
  let focus = document
    .node(selection.focus.node)
    .ok_or(EditError::NodeNotFound(selection.focus.node))?;
  text_value(document, anchor.id)?;
  text_value(document, focus.id)?;
  let parent = anchor.parent.ok_or(EditError::CrossNodeSelection)?;
  if focus.parent != Some(parent) {
    return Err(EditError::CrossNodeSelection);
  }
  let anchor_index = document
    .child_index(parent, anchor.id)
    .ok_or(EditError::UnsupportedStructure(anchor.id))?;
  let focus_index = document
    .child_index(parent, focus.id)
    .ok_or(EditError::UnsupportedStructure(focus.id))?;

  if anchor_index < focus_index {
    Ok((
      selection.anchor,
      selection.focus,
      parent,
      anchor_index,
      focus_index,
    ))
  } else if focus_index < anchor_index {
    Ok((
      selection.focus,
      selection.anchor,
      parent,
      focus_index,
      anchor_index,
    ))
  } else {
    Err(EditError::CrossNodeSelection)
  }
}

fn build_delete_backward(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  if !selection.is_collapsed() {
    return build_replace_selection(document, selection, "");
  }

  let caret = selection
    .caret()
    .expect("collapsed selection must expose a caret");
  let value = text_value(document, caret.node)?;
  let byte_offset = utf16_to_byte(value, caret.node, caret.offset_utf16)?;
  if byte_offset == 0 {
    return build_join_previous_paragraph(document, caret, selection);
  }

  let previous = value[..byte_offset]
    .chars()
    .next_back()
    .expect("non-zero byte offset must have a previous character");
  let start = caret.offset_utf16 - previous.len_utf16() as u32;

  Ok(Transaction {
    operations: vec![Operation::ReplaceText {
      node: caret.node,
      range: Utf16Range::new(start, caret.offset_utf16),
      inserted: String::new(),
    }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret.node,
      offset_utf16: start,
    }),
  })
}

fn build_insert_paragraph(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  super::paragraph::build_insert_paragraph(document, selection)
}

fn build_join_previous_paragraph(
  document: &Document,
  caret: SelectionPoint,
  selection: Selection,
) -> Result<Transaction, EditError> {
  if let Some(transaction) = super::list::build_list_backspace(document, caret, selection)? {
    return Ok(transaction);
  }

  let (current_block, parent, _, text_index) = direct_paragraph_for_text(document, caret.node)?;
  if text_index != 0 {
    return Ok(Transaction {
      operations: Vec::new(),
      selection_before: selection,
      selection_after: selection,
    });
  }

  let Some(previous_block) = document.previous_sibling(current_block) else {
    return Ok(Transaction {
      operations: Vec::new(),
      selection_before: selection,
      selection_after: selection,
    });
  };
  if !matches!(previous_block.kind, NodeKind::Block(BlockKind::Paragraph)) {
    return Ok(Transaction {
      operations: Vec::new(),
      selection_before: selection,
      selection_after: selection,
    });
  }
  if previous_block.parent != Some(parent) {
    return Err(EditError::UnsupportedStructure(current_block));
  }

  let previous_count = previous_block.children.len();
  let current_children = document
    .node(current_block)
    .ok_or(EditError::NodeNotFound(current_block))?
    .children
    .clone();
  let mut operations = current_children
    .into_iter()
    .enumerate()
    .map(|(offset, node)| Operation::MoveNode {
      node,
      new_parent: previous_block.id,
      new_index: previous_count + offset,
    })
    .collect::<Vec<_>>();
  operations.push(Operation::RemoveNode {
    node: current_block,
  });

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

fn build_set_block_kind(
  document: &Document,
  selection: Selection,
  kind: BlockKind,
) -> Result<Transaction, EditError> {
  let node = if selection.anchor.node == selection.focus.node {
    selection.focus.node
  } else {
    return Err(EditError::CrossNodeSelection);
  };
  let block = editable_text_block_for_text(document, node)?;
  Ok(Transaction {
    operations: vec![Operation::SetBlockKind { node: block, kind }],
    selection_before: selection,
    selection_after: selection,
  })
}

fn build_toggle_mark(
  document: &Document,
  selection: Selection,
  mark: MarkKind,
) -> Result<Transaction, EditError> {
  let (text, start, end) = selection
    .ordered_same_node()
    .ok_or(EditError::CrossNodeSelection)?;
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
  let full_length = value.encode_utf16().count() as u32;

  if mark.matches(&parent_node.kind) {
    if start != 0 || end != full_length || parent_node.children.as_slice() != [text] {
      return Err(EditError::UnsupportedStructure(parent));
    }
    let grandparent = parent_node
      .parent
      .ok_or(EditError::UnsupportedStructure(parent))?;
    let parent_index = document
      .child_index(grandparent, parent)
      .ok_or(EditError::UnsupportedStructure(parent))?;
    let detached_text = Node::new(text, text_node.kind.clone(), None);
    return Ok(Transaction {
      operations: vec![
        Operation::RemoveNode { node: text },
        Operation::RemoveNode { node: parent },
        Operation::InsertNode {
          parent: grandparent,
          index: parent_index,
          node: detached_text,
        },
      ],
      selection_before: selection,
      selection_after: selection,
    });
  }

  let parent_index = document
    .child_index(parent, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let selected = value[start_byte..end_byte].to_string();
  let suffix = value[end_byte..].to_string();
  let wrapper_id = document.next_available_id();
  let selected_id = NodeId(wrapper_id.0.saturating_add(1));
  let suffix_id = NodeId(wrapper_id.0.saturating_add(2));
  let wrapper = Node::new(wrapper_id, NodeKind::Inline(mark.inline()), None);
  let selected_node = Node::new(
    selected_id,
    NodeKind::Inline(InlineKind::Text { value: selected }),
    None,
  );

  let mut operations = vec![
    Operation::ReplaceText {
      node: text,
      range: Utf16Range::new(start, full_length),
      inserted: String::new(),
    },
    Operation::InsertNode {
      parent,
      index: parent_index + 1,
      node: wrapper,
    },
    Operation::InsertNode {
      parent: wrapper_id,
      index: 0,
      node: selected_node,
    },
  ];
  if !suffix.is_empty() {
    operations.push(Operation::InsertNode {
      parent,
      index: parent_index + 2,
      node: Node::new(
        suffix_id,
        NodeKind::Inline(InlineKind::Text { value: suffix }),
        None,
      ),
    });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection {
      anchor: SelectionPoint {
        node: selected_id,
        offset_utf16: 0,
      },
      focus: SelectionPoint {
        node: selected_id,
        offset_utf16: end - start,
      },
    },
  })
}

fn direct_paragraph_for_text(
  document: &Document,
  text: NodeId,
) -> Result<(NodeId, NodeId, usize, usize), EditError> {
  let text_node = document.node(text).ok_or(EditError::NodeNotFound(text))?;
  text_value(document, text)?;
  let block = text_node
    .parent
    .ok_or(EditError::UnsupportedStructure(text))?;
  let block_node = document.node(block).ok_or(EditError::NodeNotFound(block))?;
  if !matches!(block_node.kind, NodeKind::Block(BlockKind::Paragraph)) {
    return Err(EditError::UnsupportedStructure(block));
  }
  let text_index = document
    .child_index(block, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let parent = block_node
    .parent
    .ok_or(EditError::UnsupportedStructure(block))?;
  let block_index = document
    .child_index(parent, block)
    .ok_or(EditError::UnsupportedStructure(block))?;
  Ok((block, parent, block_index, text_index))
}

fn editable_text_block_for_text(document: &Document, text: NodeId) -> Result<NodeId, EditError> {
  let text_node = document.node(text).ok_or(EditError::NodeNotFound(text))?;
  text_value(document, text)?;
  let block = text_node
    .parent
    .ok_or(EditError::UnsupportedStructure(text))?;
  let block_node = document.node(block).ok_or(EditError::NodeNotFound(block))?;
  if !matches!(
    block_node.kind,
    NodeKind::Block(BlockKind::Paragraph | BlockKind::Heading { .. })
  ) || block_node.children.as_slice() != [text]
  {
    return Err(EditError::UnsupportedStructure(block));
  }
  Ok(block)
}

fn text_value(document: &Document, node_id: NodeId) -> Result<&str, EditError> {
  let node = document
    .node(node_id)
    .ok_or(EditError::NodeNotFound(node_id))?;
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node_id)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn block_text(document: &Document, block_index: usize) -> NodeId {
    let block = document.children(document.root).nth(block_index).unwrap();
    document.children(block.id).next().unwrap().id
  }

  #[test]
  fn replaces_a_same_node_selection() {
    let mut document = parse_markdown("abcdef");
    let node = block_text(&document, 0);
    let selection = Selection {
      anchor: SelectionPoint {
        node,
        offset_utf16: 5,
      },
      focus: SelectionPoint {
        node,
        offset_utf16: 2,
      },
    };
    Command::InsertText("X".to_string())
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "abXf");
  }

  #[test]
  fn replaces_across_sibling_inlines_and_restores_subtrees() {
    let mut document = parse_markdown("a **bold** z");
    let paragraph = document.children(document.root).next().unwrap().id;
    let children = document
      .children(paragraph)
      .map(|node| node.id)
      .collect::<Vec<_>>();
    let selection = Selection {
      anchor: SelectionPoint {
        node: children[0],
        offset_utf16: 1,
      },
      focus: SelectionPoint {
        node: children[2],
        offset_utf16: 1,
      },
    };
    let inverse = Command::InsertText("X".to_string())
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "aXz");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "a **bold** z");
  }

  #[test]
  fn inserts_text_at_a_utf16_caret() {
    let mut document = parse_markdown("A😀B");
    let node = block_text(&document, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 3,
    });
    let transaction = Command::InsertText("X".to_string())
      .build(&document, selection)
      .unwrap();
    assert_eq!(transaction.selection_after.focus.offset_utf16, 4);
    transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "A😀XB");
  }

  #[test]
  fn splits_a_plain_paragraph_and_undoes_exactly() {
    let mut document = parse_markdown("hello");
    let node = block_text(&document, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 2,
    });
    let inverse = Command::InsertParagraph
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "he\n\nllo");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "hello");
  }

  #[test]
  fn splits_rich_paragraphs_and_preserves_moved_subtree_ids() {
    let mut document = parse_markdown("before**bold**after");
    let original_block = document.children(document.root).next().unwrap().id;
    let children = document
      .children(original_block)
      .map(|node| node.id)
      .collect::<Vec<_>>();
    let first_text = children[0];
    let strong = children[1];
    let strong_text = document.children(strong).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: first_text,
      offset_utf16: 6,
    });

    let inverse = Command::InsertParagraph
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "before\n\n**bold**after");
    let new_block = document.children(document.root).nth(1).unwrap().id;
    assert_eq!(document.node(strong).unwrap().parent, Some(new_block));
    assert_eq!(document.node(strong_text).unwrap().parent, Some(strong));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "before**bold**after");
    assert_eq!(document.node(strong).unwrap().parent, Some(original_block));
    assert_eq!(document.node(strong_text).unwrap().parent, Some(strong));
  }

  #[test]
  fn joins_rich_paragraphs_and_restores_them() {
    let mut document = parse_markdown("left**bold**\n\nright*soft*");
    let first_block = document.children(document.root).next().unwrap().id;
    let second_block = document.children(document.root).nth(1).unwrap().id;
    let second_children = document
      .children(second_block)
      .map(|node| node.id)
      .collect::<Vec<_>>();
    let right = second_children[0];
    let emphasis = second_children[1];
    let selection = Selection::collapsed(SelectionPoint {
      node: right,
      offset_utf16: 0,
    });

    let inverse = Command::DeleteBackward
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "left**bold**right*soft*");
    assert_eq!(document.node(right).unwrap().parent, Some(first_block));
    assert_eq!(document.node(emphasis).unwrap().parent, Some(first_block));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "left**bold**\n\nright*soft*");
    assert_eq!(document.node(right).unwrap().parent, Some(second_block));
    assert_eq!(document.node(emphasis).unwrap().parent, Some(second_block));
  }

  #[test]
  fn transforms_paragraphs_to_headings_and_back() {
    let mut document = parse_markdown("Title");
    let node = block_text(&document, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 5,
    });
    let inverse = Command::SetHeading(2)
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "## Title");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "Title");
  }

  #[test]
  fn wraps_and_unwraps_strong_marks() {
    let mut document = parse_markdown("abcdef");
    let node = block_text(&document, 0);
    let selection = Selection {
      anchor: SelectionPoint {
        node,
        offset_utf16: 2,
      },
      focus: SelectionPoint {
        node,
        offset_utf16: 5,
      },
    };
    Command::ToggleStrong
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "ab**cde**f");

    let paragraph = document.children(document.root).next().unwrap().id;
    let strong = document.children(paragraph).nth(1).unwrap().id;
    let selected = document.children(strong).next().unwrap().id;
    let full = Selection {
      anchor: SelectionPoint {
        node: selected,
        offset_utf16: 0,
      },
      focus: SelectionPoint {
        node: selected,
        offset_utf16: 3,
      },
    };
    Command::ToggleStrong
      .build(&document, full)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "abcdef");
  }

  #[test]
  fn delete_backward_removes_one_unicode_scalar() {
    let mut document = parse_markdown("A😀B");
    let node = block_text(&document, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 3,
    });
    let transaction = Command::DeleteBackward.build(&document, selection).unwrap();
    assert_eq!(transaction.selection_after.focus.offset_utf16, 1);
    transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "AB");
  }
}
