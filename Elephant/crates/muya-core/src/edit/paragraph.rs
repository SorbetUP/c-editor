use crate::model::{BlockKind, Document, InlineKind, NodeId, NodeKind};
use crate::selection::Selection;

use super::{EditError, Operation, Transaction};

pub(crate) fn build_insert_paragraph(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  let preserved_task_state = middle_task_state(document, selection)?;
  let mut transaction = super::paragraph_engine::build_insert_paragraph(document, selection)?;

  if let Some(checked) = preserved_task_state {
    preserve_inserted_task_state(&mut transaction, checked);
  }
  Ok(transaction)
}

fn middle_task_state(document: &Document, selection: Selection) -> Result<Option<bool>, EditError> {
  let Some(caret) = selection.caret() else {
    return Ok(None);
  };
  let node = document
    .node(caret.node)
    .ok_or(EditError::NodeNotFound(caret.node))?;
  let NodeKind::Inline(InlineKind::Text { value }) = &node.kind else {
    return Err(EditError::NotTextNode(caret.node));
  };
  let length = value.encode_utf16().count() as u32;
  if caret.offset_utf16 == 0 || caret.offset_utf16 >= length {
    return Ok(None);
  }

  let mut current = caret.node;
  loop {
    let parent = document
      .node(current)
      .and_then(|node| node.parent)
      .ok_or(EditError::UnsupportedStructure(current))?;
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match &parent_node.kind {
      NodeKind::Block(BlockKind::Paragraph) => {
        let item = parent_node
          .parent
          .and_then(|id| document.node(id))
          .ok_or(EditError::UnsupportedStructure(parent))?;
        return match &item.kind {
          NodeKind::Block(BlockKind::ListItem {
            checked: Some(checked),
          }) => Ok(Some(*checked)),
          _ => Ok(None),
        };
      }
      NodeKind::Inline(_) => current = parent,
      _ => return Ok(None),
    }
  }
}

fn preserve_inserted_task_state(transaction: &mut Transaction, checked: bool) {
  for operation in &mut transaction.operations {
    let Operation::InsertNode { node, .. } = operation else {
      continue;
    };
    let NodeKind::Block(BlockKind::ListItem { checked: inserted }) = &mut node.kind else {
      continue;
    };
    if inserted.is_none() {
      continue;
    }
    *inserted = Some(checked);
    break;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::selection::SelectionPoint;
  use crate::{parse_markdown, to_markdown};

  fn first_text(document: &Document) -> NodeId {
    document
      .nodes
      .values()
      .find(|node| matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })))
      .unwrap()
      .id
  }

  #[test]
  fn preserves_checked_state_for_a_middle_task_split() {
    let mut document = parse_markdown("- [x] alpha");
    let text = first_text(&document);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 2,
    });
    let transaction = build_insert_paragraph(&document, selection).unwrap();
    let inverse = transaction.apply(&mut document).unwrap();

    assert_eq!(to_markdown(&document), "- [x] al\n- [x] pha");
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- [x] alpha");
  }

  #[test]
  fn creates_an_unchecked_task_when_splitting_at_the_end() {
    let mut document = parse_markdown("- [x] alpha");
    let text = first_text(&document);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 5,
    });
    build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();

    assert_eq!(to_markdown(&document), "- [x] alpha\n- [ ] ");
  }
}
