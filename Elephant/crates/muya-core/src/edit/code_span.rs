use unicode_segmentation::UnicodeSegmentation;

use crate::model::{Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction, Utf16Range};

pub(crate) fn build_insert_text(
  document: &Document,
  selection: Selection,
  inserted: &str,
) -> Result<Option<Transaction>, EditError> {
  let Some((node, start, end)) = selection.ordered_same_node() else {
    return Ok(None);
  };
  let Some(value) = code_value(document, node)? else {
    return Ok(None);
  };
  utf16_to_byte(value, node, start)?;
  utf16_to_byte(value, node, end)?;

  Ok(Some(Transaction {
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
  }))
}

pub(crate) fn build_delete_backward(
  document: &Document,
  selection: Selection,
) -> Result<Option<Transaction>, EditError> {
  if !selection.is_collapsed() {
    return build_insert_text(document, selection, "");
  }

  let caret = selection
    .caret()
    .expect("collapsed selection must expose a caret");
  let Some(value) = code_value(document, caret.node)? else {
    return Ok(None);
  };
  let byte_offset = utf16_to_byte(value, caret.node, caret.offset_utf16)?;
  if byte_offset == 0 {
    return Ok(None);
  }

  let prefix = &value[..byte_offset];
  let (start_byte, _) = prefix
    .grapheme_indices(true)
    .next_back()
    .ok_or(EditError::UnsupportedStructure(caret.node))?;
  let start_utf16 = prefix[..start_byte].encode_utf16().count() as u32;

  Ok(Some(Transaction {
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
  }))
}

fn code_value(document: &Document, node: NodeId) -> Result<Option<&str>, EditError> {
  let node = document.node(node).ok_or(EditError::NodeNotFound(node))?;
  match &node.kind {
    NodeKind::Inline(InlineKind::CodeSpan { code }) => Ok(Some(code)),
    _ => Ok(None),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn code_span(document: &Document) -> NodeId {
    document
      .nodes
      .values()
      .find_map(|node| match &node.kind {
        NodeKind::Inline(InlineKind::CodeSpan { .. }) => Some(node.id),
        _ => None,
      })
      .unwrap()
  }

  #[test]
  fn inserts_and_replaces_text_inside_inline_code() {
    let mut document = parse_markdown("`alpha`");
    let node = code_span(&document);
    let selection = Selection {
      anchor: SelectionPoint {
        node,
        offset_utf16: 1,
      },
      focus: SelectionPoint {
        node,
        offset_utf16: 4,
      },
    };
    let transaction = build_insert_text(&document, selection, "X")
      .unwrap()
      .unwrap();
    let inverse = transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "`aXa`");
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "`alpha`");
  }

  #[test]
  fn deletes_one_grapheme_inside_inline_code() {
    let mut document = parse_markdown("`A🇫🇷B`");
    let node = code_span(&document);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 5,
    });
    let transaction = build_delete_backward(&document, selection)
      .unwrap()
      .unwrap();
    transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "`AB`");
  }
}
