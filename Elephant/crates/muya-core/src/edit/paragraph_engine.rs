use crate::model::{BlockKind, Document, InlineKind, ListKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Copy)]
enum SplitDestination {
  Document {
    parent: NodeId,
    paragraph_index: usize,
  },
  List {
    list: NodeId,
    item: NodeId,
    item_index: usize,
    new_checked: Option<bool>,
  },
}

struct SplitPath {
  paragraph: NodeId,
  destination: SplitDestination,
  wrappers_top_down: Vec<NodeId>,
}

pub(crate) fn build_insert_paragraph(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  let caret = selection.caret().ok_or(EditError::NonCollapsedSelection)?;
  let value = text_value(document, caret.node)?;
  let byte_offset = utf16_to_byte(value, caret.node, caret.offset_utf16)?;
  let path = split_path(document, caret.node)?;

  if value.is_empty()
    && path.wrappers_top_down.is_empty()
    && document
      .node(path.paragraph)
      .is_some_and(|paragraph| paragraph.children.as_slice() == [caret.node])
  {
    if let SplitDestination::List { .. } = path.destination {
      return build_exit_empty_list_item(document, selection, caret, &path);
    }
  }

  if !path.wrappers_top_down.is_empty()
    && (caret.offset_utf16 == 0 || caret.offset_utf16 == value.encode_utf16().count() as u32)
  {
    return Err(EditError::UnsupportedStructure(caret.node));
  }

  build_split(document, selection, caret, value, byte_offset, path)
}

fn build_split(
  document: &Document,
  selection: Selection,
  caret: SelectionPoint,
  value: &str,
  byte_offset: usize,
  path: SplitPath,
) -> Result<Transaction, EditError> {
  let suffix = value[byte_offset..].to_string();
  let mut next_id = document.next_available_id().0;
  let new_item_id = match path.destination {
    SplitDestination::List { .. } => Some(take_id(&mut next_id)),
    SplitDestination::Document { .. } => None,
  };
  let new_paragraph_id = take_id(&mut next_id);
  let cloned_wrapper_ids = path
    .wrappers_top_down
    .iter()
    .map(|_| take_id(&mut next_id))
    .collect::<Vec<_>>();
  let new_text_id = take_id(&mut next_id);

  let mut operations = vec![Operation::ReplaceText {
    node: caret.node,
    range: Utf16Range::new(caret.offset_utf16, value.encode_utf16().count() as u32),
    inserted: String::new(),
  }];

  match path.destination {
    SplitDestination::Document {
      parent,
      paragraph_index,
    } => operations.push(Operation::InsertNode {
      parent,
      index: paragraph_index + 1,
      node: Node::new(
        new_paragraph_id,
        NodeKind::Block(BlockKind::Paragraph),
        None,
      ),
    }),
    SplitDestination::List {
      list,
      item_index,
      new_checked,
      ..
    } => {
      let new_item_id = new_item_id.expect("list split must allocate an item ID");
      operations.extend([
        Operation::InsertNode {
          parent: list,
          index: item_index + 1,
          node: Node::new(
            new_item_id,
            NodeKind::Block(BlockKind::ListItem {
              checked: new_checked,
            }),
            None,
          ),
        },
        Operation::InsertNode {
          parent: new_item_id,
          index: 0,
          node: Node::new(
            new_paragraph_id,
            NodeKind::Block(BlockKind::Paragraph),
            None,
          ),
        },
      ]);
    }
  }

  let mut destination_parent = new_paragraph_id;
  for (original, cloned) in path.wrappers_top_down.iter().zip(cloned_wrapper_ids.iter()) {
    let original_node = document
      .node(*original)
      .ok_or(EditError::NodeNotFound(*original))?;
    operations.push(Operation::InsertNode {
      parent: destination_parent,
      index: 0,
      node: Node::new(*cloned, original_node.kind.clone(), None),
    });
    destination_parent = *cloned;
  }

  operations.push(Operation::InsertNode {
    parent: destination_parent,
    index: 0,
    node: Node::new(
      new_text_id,
      NodeKind::Inline(InlineKind::Text { value: suffix }),
      None,
    ),
  });

  let wrappers = &path.wrappers_top_down;
  for depth in (0..wrappers.len()).rev() {
    let original_parent = wrappers[depth];
    let original_child = wrappers.get(depth + 1).copied().unwrap_or(caret.node);
    let target = cloned_wrapper_ids[depth];
    let following = following_siblings(document, original_parent, original_child)?;
    operations.extend(following.into_iter().enumerate().map(|(offset, node)| {
      Operation::MoveNode {
        node,
        new_parent: target,
        new_index: 1 + offset,
      }
    }));
  }

  let paragraph_child = wrappers.first().copied().unwrap_or(caret.node);
  let paragraph_following = following_siblings(document, path.paragraph, paragraph_child)?;
  operations.extend(
    paragraph_following
      .into_iter()
      .enumerate()
      .map(|(offset, node)| Operation::MoveNode {
        node,
        new_parent: new_paragraph_id,
        new_index: 1 + offset,
      }),
  );

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: new_text_id,
      offset_utf16: 0,
    }),
  })
}

fn build_exit_empty_list_item(
  document: &Document,
  selection: Selection,
  caret: SelectionPoint,
  path: &SplitPath,
) -> Result<Transaction, EditError> {
  let SplitDestination::List {
    list,
    item,
    item_index,
    ..
  } = path.destination
  else {
    return Err(EditError::UnsupportedStructure(path.paragraph));
  };

  let item_node = document.node(item).ok_or(EditError::NodeNotFound(item))?;
  if item_node.children.as_slice() != [path.paragraph] {
    return Err(EditError::UnsupportedStructure(item));
  }
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
  let item_count = list_node.children.len();
  if item_count == 0 || item_index >= item_count {
    return Err(EditError::UnsupportedStructure(item));
  }

  let mut operations = Vec::new();
  if item_count == 1 {
    operations.extend([
      Operation::MoveNode {
        node: path.paragraph,
        new_parent: parent,
        new_index: list_index,
      },
      Operation::RemoveNode { node: item },
      Operation::RemoveNode { node: list },
    ]);
  } else if item_index == 0 {
    operations.extend([
      Operation::MoveNode {
        node: path.paragraph,
        new_parent: parent,
        new_index: list_index,
      },
      Operation::RemoveNode { node: item },
    ]);
  } else if item_index + 1 == item_count {
    operations.extend([
      Operation::MoveNode {
        node: path.paragraph,
        new_parent: parent,
        new_index: list_index + 1,
      },
      Operation::RemoveNode { node: item },
    ]);
  } else {
    let right_items = list_node.children[item_index + 1..].to_vec();
    let new_list_id = document.next_available_id();
    let right_kind = right_list_kind(&list_node.kind, item_index + 1)?;
    operations.extend([
      Operation::MoveNode {
        node: path.paragraph,
        new_parent: parent,
        new_index: list_index + 1,
      },
      Operation::InsertNode {
        parent,
        index: list_index + 2,
        node: Node::new(new_list_id, NodeKind::Block(right_kind), None),
      },
    ]);
    operations.extend(right_items.into_iter().enumerate().map(|(index, node)| {
      Operation::MoveNode {
        node,
        new_parent: new_list_id,
        new_index: index,
      }
    }));
    operations.push(Operation::RemoveNode { node: item });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

fn right_list_kind(kind: &NodeKind, split_at: usize) -> Result<BlockKind, EditError> {
  let NodeKind::Block(BlockKind::List { kind, start }) = kind else {
    return Err(EditError::UnsupportedStructure(NodeId(0)));
  };
  let right_start = match kind {
    ListKind::Ordered => start.map(|value| value.saturating_add(split_at as u64)),
    ListKind::Unordered | ListKind::Task => *start,
  };
  Ok(BlockKind::List {
    kind: *kind,
    start: right_start,
  })
}

fn take_id(next_id: &mut u64) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = (*next_id).saturating_add(1);
  id
}

fn split_path(document: &Document, text: NodeId) -> Result<SplitPath, EditError> {
  text_value(document, text)?;
  let mut wrappers_bottom_up = Vec::new();
  let mut current = text;

  let paragraph = loop {
    let current_node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    let parent = current_node
      .parent
      .ok_or(EditError::UnsupportedStructure(current))?;
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match &parent_node.kind {
      NodeKind::Block(BlockKind::Paragraph | BlockKind::Heading { .. } | BlockKind::BlockQuote) => {
        break parent
      }
      NodeKind::Inline(InlineKind::Text { .. }) => {
        return Err(EditError::UnsupportedStructure(parent));
      }
      NodeKind::Inline(_) => {
        wrappers_bottom_up.push(parent);
        current = parent;
      }
      _ => return Err(EditError::UnsupportedStructure(parent)),
    }
  };

  let paragraph_node = document
    .node(paragraph)
    .ok_or(EditError::NodeNotFound(paragraph))?;
  let paragraph_parent = paragraph_node
    .parent
    .ok_or(EditError::UnsupportedStructure(paragraph))?;
  let parent_node = document
    .node(paragraph_parent)
    .ok_or(EditError::NodeNotFound(paragraph_parent))?;

  let destination = if paragraph_parent == document.root {
    SplitDestination::Document {
      parent: paragraph_parent,
      paragraph_index: document
        .child_index(paragraph_parent, paragraph)
        .ok_or(EditError::UnsupportedStructure(paragraph))?,
    }
  } else if let NodeKind::Block(BlockKind::ListItem { checked }) = &parent_node.kind {
    if parent_node.children.as_slice() != [paragraph] {
      return Err(EditError::UnsupportedStructure(paragraph_parent));
    }
    let list = parent_node
      .parent
      .ok_or(EditError::UnsupportedStructure(paragraph_parent))?;
    if !matches!(
      document.node(list).map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::List { .. }))
    ) {
      return Err(EditError::UnsupportedStructure(list));
    }
    SplitDestination::List {
      list,
      item: paragraph_parent,
      item_index: document
        .child_index(list, paragraph_parent)
        .ok_or(EditError::UnsupportedStructure(paragraph_parent))?,
      new_checked: checked.map(|_| false),
    }
  } else {
    return Err(EditError::UnsupportedStructure(paragraph_parent));
  };

  wrappers_bottom_up.reverse();
  Ok(SplitPath {
    paragraph,
    destination,
    wrappers_top_down: wrappers_bottom_up,
  })
}

fn following_siblings(
  document: &Document,
  parent: NodeId,
  child: NodeId,
) -> Result<Vec<NodeId>, EditError> {
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  let index = document
    .child_index(parent, child)
    .ok_or(EditError::UnsupportedStructure(child))?;
  Ok(parent_node.children[index + 1..].to_vec())
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

  fn item_text(document: &Document, list: NodeId, index: usize) -> NodeId {
    let item = document.children(list).nth(index).unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    document.children(paragraph).next().unwrap().id
  }

  #[test]
  fn splits_inside_nested_marks_and_restores_original_ids() {
    let mut document = parse_markdown("**beforeafter**");
    let paragraph = document.children(document.root).next().unwrap().id;
    let strong = document.children(paragraph).next().unwrap().id;
    let text = document.children(strong).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 6,
    });

    let inverse = build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "**before**\n\n**after**");
    assert_eq!(document.node(strong).unwrap().parent, Some(paragraph));

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "**beforeafter**");
    assert_eq!(document.node(text).unwrap().parent, Some(strong));
  }

  #[test]
  fn splits_plain_and_task_list_items() {
    let mut document = parse_markdown("- [x] beforeafter");
    let list = document.children(document.root).next().unwrap().id;
    let text = item_text(&document, list, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 6,
    });

    build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- [x] before\n- [ ] after");
  }

  #[test]
  fn exits_a_single_empty_list_item_and_undoes() {
    let mut document = parse_markdown("- ");
    let list = document.children(document.root).next().unwrap().id;
    let item = document.children(list).next().unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert!(document.node(list).is_none());
    assert_eq!(
      document.node(paragraph).unwrap().parent,
      Some(document.root)
    );
    assert_eq!(document.node(text).unwrap().parent, Some(paragraph));

    inverse.apply(&mut document).unwrap();
    assert_eq!(document.node(paragraph).unwrap().parent, Some(item));
    assert_eq!(to_markdown(&document), "- ");
  }

  #[test]
  fn exits_the_last_empty_item_after_the_list() {
    let mut document = parse_markdown("- first\n- ");
    let list = document.children(document.root).next().unwrap().id;
    let item = document.children(list).nth(1).unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(document.children(list).count(), 1);
    assert_eq!(
      document.children(document.root).nth(1).unwrap().id,
      paragraph
    );
  }

  #[test]
  fn splits_an_ordered_list_around_a_middle_empty_item() {
    let mut document = parse_markdown("3. first\n4. \n5. third");
    let list = document.children(document.root).next().unwrap().id;
    let empty_item = document.children(list).nth(1).unwrap().id;
    let right_item = document.children(list).nth(2).unwrap().id;
    let paragraph = document.children(empty_item).next().unwrap().id;
    let text = document.children(paragraph).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = build_insert_paragraph(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    let right_list = document.children(document.root).nth(2).unwrap().id;
    assert_eq!(document.children(list).count(), 1);
    assert_eq!(document.children(right_list).next().unwrap().id, right_item);
    assert!(matches!(
      document.node(right_list).map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::List {
        kind: ListKind::Ordered,
        start: Some(5)
      }))
    ));

    inverse.apply(&mut document).unwrap();
    assert_eq!(document.children(list).count(), 3);
    assert_eq!(document.children(list).nth(2).unwrap().id, right_item);
  }

  #[test]
  fn accepts_common_markdown_caret_boundaries() {
    let samples = [
      "# Heading\n\nPlain **bold** and *emphasis* with [a link](https://example.com).",
      "> quoted **text**\n\n- one\n- [x] checked",
      "Before `code` and ~~strike~~ after.\n\n![diagram](.assets/diagram.png)",
    ];

    for markdown in samples {
      let document = parse_markdown(markdown);
      let text_nodes = document
        .nodes
        .values()
        .filter_map(|node| match &node.kind {
          NodeKind::Inline(InlineKind::Text { value }) => Some((node.id, value.clone())),
          _ => None,
        })
        .collect::<Vec<_>>();

      for (node, value) in text_nodes {
        let length = value.encode_utf16().count() as u32;
        for offset in [0, length / 2, length] {
          let selection = Selection::collapsed(SelectionPoint {
            node,
            offset_utf16: offset,
          });
          let result = build_insert_paragraph(&document, selection).or_else(|error| {
            if matches!(error, EditError::UnsupportedStructure(_)) {
              super::super::paragraph_boundary::ParagraphBoundaryCommand::InsertParagraph
                .build(&document, selection)
            } else {
              Err(error)
            }
          });
          assert!(
            !matches!(result, Err(EditError::UnsupportedStructure(_))),
            "common markdown rejected at node {node:?}, offset {offset}: {markdown:?}"
          );
        }
      }
    }
  }
}
