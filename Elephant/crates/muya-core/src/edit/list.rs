use crate::model::{BlockKind, Document, InlineKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::{EditError, Operation, Transaction};

pub(crate) fn build_list_backspace(
  document: &Document,
  caret: SelectionPoint,
  selection: Selection,
) -> Result<Option<Transaction>, EditError> {
  let text = document
    .node(caret.node)
    .ok_or(EditError::NodeNotFound(caret.node))?;
  if !matches!(text.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    return Err(EditError::NotTextNode(caret.node));
  }
  let paragraph = text
    .parent
    .ok_or(EditError::UnsupportedStructure(caret.node))?;
  let paragraph_node = document
    .node(paragraph)
    .ok_or(EditError::NodeNotFound(paragraph))?;
  if !matches!(paragraph_node.kind, NodeKind::Block(BlockKind::Paragraph)) {
    return Ok(None);
  }
  if document.child_index(paragraph, caret.node) != Some(0) {
    return Ok(None);
  }

  let item = paragraph_node
    .parent
    .ok_or(EditError::UnsupportedStructure(paragraph))?;
  let item_node = document.node(item).ok_or(EditError::NodeNotFound(item))?;
  if !matches!(item_node.kind, NodeKind::Block(BlockKind::ListItem { .. })) {
    return Ok(None);
  }
  if item_node.children.as_slice() != [paragraph] {
    return Err(EditError::UnsupportedStructure(item));
  }

  let list = item_node
    .parent
    .ok_or(EditError::UnsupportedStructure(item))?;
  let list_node = document.node(list).ok_or(EditError::NodeNotFound(list))?;
  if !matches!(list_node.kind, NodeKind::Block(BlockKind::List { .. })) {
    return Err(EditError::UnsupportedStructure(list));
  }
  let item_index = document
    .child_index(list, item)
    .ok_or(EditError::UnsupportedStructure(item))?;

  if item_index == 0 {
    return build_unlist_first_item(document, list, item, paragraph, caret, selection).map(Some);
  }

  let previous_item = list_node.children[item_index - 1];
  let previous_item_node = document
    .node(previous_item)
    .ok_or(EditError::NodeNotFound(previous_item))?;
  if !matches!(
    previous_item_node.kind,
    NodeKind::Block(BlockKind::ListItem { .. })
  ) {
    return Err(EditError::UnsupportedStructure(previous_item));
  }
  let [previous_paragraph] = previous_item_node.children.as_slice() else {
    return Err(EditError::UnsupportedStructure(previous_item));
  };
  let previous_paragraph_node = document
    .node(*previous_paragraph)
    .ok_or(EditError::NodeNotFound(*previous_paragraph))?;
  if !matches!(
    previous_paragraph_node.kind,
    NodeKind::Block(BlockKind::Paragraph)
  ) {
    return Err(EditError::UnsupportedStructure(*previous_paragraph));
  }

  let previous_count = previous_paragraph_node.children.len();
  let current_children = paragraph_node.children.clone();
  let mut operations = current_children
    .into_iter()
    .enumerate()
    .map(|(offset, node)| Operation::MoveNode {
      node,
      new_parent: *previous_paragraph,
      new_index: previous_count + offset,
    })
    .collect::<Vec<_>>();
  operations.extend([
    Operation::RemoveNode { node: paragraph },
    Operation::RemoveNode { node: item },
  ]);

  Ok(Some(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  }))
}

fn build_unlist_first_item(
  document: &Document,
  list: NodeId,
  item: NodeId,
  paragraph: NodeId,
  caret: SelectionPoint,
  selection: Selection,
) -> Result<Transaction, EditError> {
  let list_node = document.node(list).ok_or(EditError::NodeNotFound(list))?;
  let parent = list_node
    .parent
    .ok_or(EditError::UnsupportedStructure(list))?;
  if parent != document.root {
    return Err(EditError::UnsupportedStructure(list));
  }
  let list_index = document
    .child_index(parent, list)
    .ok_or(EditError::UnsupportedStructure(list))?;
  let remove_empty_list = list_node.children.len() == 1;

  let mut operations = vec![
    Operation::MoveNode {
      node: paragraph,
      new_parent: parent,
      new_index: list_index,
    },
    Operation::RemoveNode { node: item },
  ];
  if remove_empty_list {
    operations.push(Operation::RemoveNode { node: list });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn first_text(document: &Document, item: NodeId) -> NodeId {
    let paragraph = document.children(item).next().unwrap().id;
    document.children(paragraph).next().unwrap().id
  }

  #[test]
  fn joins_an_item_with_its_previous_sibling() {
    let mut document = parse_markdown("- left**bold**\n- right*soft*");
    let list = document.children(document.root).next().unwrap().id;
    let first_item = document.children(list).next().unwrap().id;
    let second_item = document.children(list).nth(1).unwrap().id;
    let right = first_text(&document, second_item);
    let emphasis = {
      let paragraph = document.children(second_item).next().unwrap().id;
      document.children(paragraph).nth(1).unwrap().id
    };
    let selection = Selection::collapsed(SelectionPoint {
      node: right,
      offset_utf16: 0,
    });

    let inverse = build_list_backspace(&document, selection.focus, selection)
      .unwrap()
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- left**bold**right*soft*");
    assert_eq!(document.children(list).count(), 1);
    let first_paragraph = document.children(first_item).next().unwrap().id;
    assert_eq!(document.node(right).unwrap().parent, Some(first_paragraph));
    assert_eq!(
      document.node(emphasis).unwrap().parent,
      Some(first_paragraph)
    );

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- left**bold**\n- right*soft*");
    assert_eq!(document.children(list).count(), 2);
    assert_eq!(
      document.node(right).unwrap().parent,
      document.children(second_item).next().map(|n| n.id)
    );
  }

  #[test]
  fn removes_the_marker_from_the_first_item() {
    let mut document = parse_markdown("- first\n- second");
    let list = document.children(document.root).next().unwrap().id;
    let first_item = document.children(list).next().unwrap().id;
    let paragraph = document.children(first_item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = build_list_backspace(&document, selection.focus, selection)
      .unwrap()
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "first\n\n- second");
    assert_eq!(
      document.node(paragraph).unwrap().parent,
      Some(document.root)
    );

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- first\n- second");
    assert_eq!(document.node(paragraph).unwrap().parent, Some(first_item));
  }

  #[test]
  fn removes_an_empty_single_item_list_container() {
    let mut document = parse_markdown("- only");
    let list = document.children(document.root).next().unwrap().id;
    let item = document.children(list).next().unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = build_list_backspace(&document, selection.focus, selection)
      .unwrap()
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "only");
    assert!(document.node(list).is_none());

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- only");
    assert_eq!(document.node(paragraph).unwrap().parent, Some(item));
  }
}
