use crate::edit::{EditError, Operation, Transaction};
use crate::model::{BlockKind, Document, ListKind, Node, NodeId, NodeKind};
use crate::selection::Selection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListCommand {
  IndentItem,
  OutdentItem,
}

#[derive(Clone, Copy, Debug)]
struct ItemContext {
  item: NodeId,
  list: NodeId,
  item_index: usize,
  kind: ListKind,
  start: Option<u64>,
}

impl ListCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let context = item_context(document, selection)?;
    match self {
      Self::IndentItem => build_indent(document, selection, context),
      Self::OutdentItem => build_outdent(document, selection, context),
    }
  }
}

fn build_indent(
  document: &Document,
  selection: Selection,
  context: ItemContext,
) -> Result<Transaction, EditError> {
  if context.item_index == 0 {
    return Err(EditError::UnsupportedStructure(context.item));
  }

  let list_node = document
    .node(context.list)
    .ok_or(EditError::NodeNotFound(context.list))?;
  let previous_item = list_node.children[context.item_index - 1];
  let previous = document
    .node(previous_item)
    .ok_or(EditError::NodeNotFound(previous_item))?;
  if !matches!(previous.kind, NodeKind::Block(BlockKind::ListItem { .. })) {
    return Err(EditError::UnsupportedStructure(previous_item));
  }

  let reusable = previous.children.last().copied().filter(|candidate| {
    matches!(
      document.node(*candidate).map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::List { kind, .. })) if *kind == context.kind
    )
  });

  let mut operations = Vec::new();
  let (nested_list, nested_index) = if let Some(nested) = reusable {
    let count = document
      .node(nested)
      .ok_or(EditError::NodeNotFound(nested))?
      .children
      .len();
    (nested, count)
  } else {
    let nested = document.next_available_id();
    let nested_start = match context.kind {
      ListKind::Ordered => Some(context.start.unwrap_or(1) + context.item_index as u64),
      ListKind::Unordered | ListKind::Task => None,
    };
    operations.push(Operation::InsertNode {
      parent: previous_item,
      index: previous.children.len(),
      node: Node::new(
        nested,
        NodeKind::Block(BlockKind::List {
          kind: context.kind,
          start: nested_start,
        }),
        None,
      ),
    });
    (nested, 0)
  };

  operations.push(Operation::MoveNode {
    node: context.item,
    new_parent: nested_list,
    new_index: nested_index,
  });

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn build_outdent(
  document: &Document,
  selection: Selection,
  context: ItemContext,
) -> Result<Transaction, EditError> {
  let owner_item = document
    .node(context.list)
    .and_then(|list| list.parent)
    .ok_or(EditError::UnsupportedStructure(context.list))?;
  if !matches!(
    document.node(owner_item).map(|node| &node.kind),
    Some(NodeKind::Block(BlockKind::ListItem { .. }))
  ) {
    return Ok(Transaction {
      operations: Vec::new(),
      selection_before: selection,
      selection_after: selection,
    });
  }

  let outer_list = document
    .node(owner_item)
    .and_then(|item| item.parent)
    .ok_or(EditError::UnsupportedStructure(owner_item))?;
  if !matches!(
    document.node(outer_list).map(|node| &node.kind),
    Some(NodeKind::Block(BlockKind::List { .. }))
  ) {
    return Err(EditError::UnsupportedStructure(outer_list));
  }
  let owner_index = document
    .child_index(outer_list, owner_item)
    .ok_or(EditError::UnsupportedStructure(owner_item))?;
  let nested = document
    .node(context.list)
    .ok_or(EditError::NodeNotFound(context.list))?;
  let trailing_items = nested
    .children
    .iter()
    .skip(context.item_index + 1)
    .copied()
    .collect::<Vec<_>>();

  let mut operations = Vec::new();
  if !trailing_items.is_empty() {
    let continuation_list = document.next_available_id();
    let continuation_start = match context.kind {
      ListKind::Ordered => context
        .start
        .map(|start| start + context.item_index as u64 + 1),
      ListKind::Unordered | ListKind::Task => None,
    };
    let item_child_count = document
      .node(context.item)
      .ok_or(EditError::NodeNotFound(context.item))?
      .children
      .len();
    operations.push(Operation::InsertNode {
      parent: context.item,
      index: item_child_count,
      node: Node::new(
        continuation_list,
        NodeKind::Block(BlockKind::List {
          kind: context.kind,
          start: continuation_start,
        }),
        None,
      ),
    });
    operations.extend(trailing_items.into_iter().enumerate().map(|(index, item)| {
      Operation::MoveNode {
        node: item,
        new_parent: continuation_list,
        new_index: index,
      }
    }));
  }

  operations.push(Operation::MoveNode {
    node: context.item,
    new_parent: outer_list,
    new_index: owner_index + 1,
  });
  if context.item_index == 0 {
    operations.push(Operation::RemoveNode { node: context.list });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: selection,
  })
}

fn item_context(document: &Document, selection: Selection) -> Result<ItemContext, EditError> {
  let caret = selection.caret().ok_or(EditError::NonCollapsedSelection)?;
  let item = ancestor_matching(document, caret.node, |kind| {
    matches!(kind, NodeKind::Block(BlockKind::ListItem { .. }))
  })?;
  let list = document
    .node(item)
    .and_then(|node| node.parent)
    .ok_or(EditError::UnsupportedStructure(item))?;
  let (kind, start) = match document.node(list).map(|node| &node.kind) {
    Some(NodeKind::Block(BlockKind::List { kind, start })) => (*kind, *start),
    _ => return Err(EditError::UnsupportedStructure(list)),
  };
  let item_index = document
    .child_index(list, item)
    .ok_or(EditError::UnsupportedStructure(item))?;

  Ok(ItemContext {
    item,
    list,
    item_index,
    kind,
    start,
  })
}

fn ancestor_matching(
  document: &Document,
  start: NodeId,
  predicate: impl Fn(&NodeKind) -> bool,
) -> Result<NodeId, EditError> {
  let mut current = start;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if predicate(&node.kind) {
      return Ok(current);
    }
    current = node.parent.ok_or(EditError::UnsupportedStructure(start))?;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::selection::SelectionPoint;
  use crate::{parse_markdown, to_markdown};

  fn item_text(document: &Document, list: NodeId, item_index: usize) -> NodeId {
    let item = document.children(list).nth(item_index).unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    document.children(paragraph).next().unwrap().id
  }

  #[test]
  fn indents_an_item_under_its_previous_sibling_and_undoes() {
    let mut document = parse_markdown("- parent\n- child");
    let list = document.children(document.root).next().unwrap().id;
    let child_item = document.children(list).nth(1).unwrap().id;
    let text = item_text(&document, list, 1);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = ListCommand::IndentItem
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- parent\n  - child");
    assert_ne!(document.node(child_item).unwrap().parent, Some(list));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- parent\n- child");
    assert_eq!(document.node(child_item).unwrap().parent, Some(list));
  }

  #[test]
  fn reuses_a_compatible_nested_list() {
    let mut document = parse_markdown("- parent\n  - first child\n- second child");
    let list = document.children(document.root).next().unwrap().id;
    let text = item_text(&document, list, 1);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    ListCommand::IndentItem
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(
      to_markdown(&document),
      "- parent\n  - first child\n  - second child"
    );
    let parent = document.children(list).next().unwrap().id;
    assert_eq!(document.children(parent).count(), 2);
  }

  #[test]
  fn outdents_an_item_and_removes_an_empty_nested_container() {
    let mut document = parse_markdown("- parent\n  - child\n- sibling");
    let list = document.children(document.root).next().unwrap().id;
    let parent = document.children(list).next().unwrap().id;
    let nested = document.children(parent).nth(1).unwrap().id;
    let child_item = document.children(nested).next().unwrap().id;
    let paragraph = document.children(child_item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = ListCommand::OutdentItem
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- parent\n- child\n- sibling");
    assert!(document.node(nested).is_none());
    assert_eq!(document.node(child_item).unwrap().parent, Some(list));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "- parent\n  - child\n- sibling");
    assert_eq!(document.node(child_item).unwrap().parent, Some(nested));
  }

  #[test]
  fn keeps_a_nested_container_when_other_items_remain() {
    let mut document = parse_markdown("- parent\n  - first\n  - second");
    let list = document.children(document.root).next().unwrap().id;
    let parent = document.children(list).next().unwrap().id;
    let nested = document.children(parent).nth(1).unwrap().id;
    let second = document.children(nested).nth(1).unwrap().id;
    let paragraph = document.children(second).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    ListCommand::OutdentItem
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- parent\n  - first\n- second");
    assert!(document.node(nested).is_some());
  }

  #[test]
  fn outdents_a_middle_item_and_adopts_following_siblings_with_exact_undo() {
    let initial = "- parent\n  - first\n  - middle\n  - last";
    let mut document = parse_markdown(initial);
    let list = document.children(document.root).next().unwrap().id;
    let parent = document.children(list).next().unwrap().id;
    let nested = document.children(parent).nth(1).unwrap().id;
    let middle = document.children(nested).nth(1).unwrap().id;
    let last = document.children(nested).nth(2).unwrap().id;
    let paragraph = document.children(middle).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = ListCommand::OutdentItem
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(
      to_markdown(&document),
      "- parent\n  - first\n- middle\n  - last"
    );
    let continuation = document.children(middle).nth(1).unwrap().id;
    assert_eq!(document.node(last).unwrap().parent, Some(continuation));
    assert_eq!(document.node(middle).unwrap().parent, Some(list));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), initial);
    assert_eq!(document.node(middle).unwrap().parent, Some(nested));
    assert_eq!(document.node(last).unwrap().parent, Some(nested));
  }
}
