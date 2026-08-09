use unicode_segmentation::UnicodeSegmentation;

use crate::model::{Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::{Command, EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphemeCommand {
  DeleteBackward,
}

impl GraphemeCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    match self {
      Self::DeleteBackward => build_delete_backward(document, selection),
    }
  }
}

fn build_delete_backward(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  if !selection.is_collapsed() {
    return Command::InsertText(String::new()).build(document, selection);
  }

  let caret = selection
    .caret()
    .expect("collapsed selection must expose a caret");
  let value = text_value(document, caret.node)?;
  let byte_offset = utf16_to_byte(value, caret.node, caret.offset_utf16)?;
  if byte_offset == 0 {
    return Command::DeleteBackward.build(document, selection);
  }

  let prefix = &value[..byte_offset];
  let (start_byte, _) = prefix
    .grapheme_indices(true)
    .next_back()
    .ok_or(EditError::UnsupportedStructure(caret.node))?;
  let start_utf16 = prefix[..start_byte].encode_utf16().count() as u32;

  Ok(Transaction {
    operations: vec![Operation::ReplaceText {
      node: caret.node,
      range: Utf16Range::new(start_utf16, caret.offset_utf16),
      inserted: String::new(),
    }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret.node,
      offset_utf16: start_utf16,
    }),
  })
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

fn utf16_to_byte(value: &str, node: NodeId, target: u32) -> Result<usize, EditError> {
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
  use crate::{parse_markdown, to_markdown};

  fn first_text(document: &Document) -> NodeId {
    let paragraph = document.children(document.root).next().unwrap().id;
    document.children(paragraph).next().unwrap().id
  }

  fn delete_last_grapheme(markdown: &str) -> String {
    let mut document = parse_markdown(markdown);
    let text = first_text(&document);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: markdown.encode_utf16().count() as u32,
    });
    GraphemeCommand::DeleteBackward
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    to_markdown(&document)
  }

  #[test]
  fn deletes_a_combining_sequence_as_one_visible_character() {
    assert_eq!(delete_last_grapheme("caf\u{0065}\u{0301}"), "caf");
  }

  #[test]
  fn deletes_a_flag_as_one_grapheme() {
    assert_eq!(delete_last_grapheme("France 🇫🇷"), "France ");
  }

  #[test]
  fn deletes_a_zwj_family_as_one_grapheme() {
    assert_eq!(delete_last_grapheme("family 👨‍👩‍👧‍👦"), "family ");
  }

  #[test]
  fn delegates_structural_backspace_at_the_start_of_a_text_node() {
    let mut document = parse_markdown("one\n\ntwo");
    let second = document.children(document.root).nth(1).unwrap().id;
    let text = document.children(second).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    GraphemeCommand::DeleteBackward
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "onetwo");
  }
}
