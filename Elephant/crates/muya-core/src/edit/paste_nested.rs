use std::collections::BTreeMap;

use crate::model::{BlockKind, DetachedSubtree, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Copy)]
enum NestedPasteContext {
  ListItem,
  BlockQuote,
  TableCell,
}

pub(crate) fn build_nested_paste(
  document: &Document,
  selection: Selection,
  markdown: &str,
) -> Result<Option<Transaction>, EditError> {
  let Some((text, start, end)) = selection.ordered_same_node() else {
    return Ok(None);
  };
  let Some((container, context, text_index)) = nested_container(document, text)? else {
    return Ok(None);
  };
  if markdown.is_empty() {
    return Ok(Some(noop(selection)));
  }

  let value = text_value(document, text)?;
  utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let fragment = crate::parse_markdown(markdown);
  let blocks = fragment.children(fragment.root).collect::<Vec<_>>();
  if blocks.is_empty() {
    return Ok(Some(noop(selection)));
  }
  if blocks
    .iter()
    .any(|block| !matches!(block.kind, NodeKind::Block(BlockKind::Paragraph)))
  {
    return Err(EditError::UnsupportedStructure(container));
  }

  let suffix = value[end_byte..].to_string();
  let full_length = value.encode_utf16().count() as u32;
  let mut operations = vec![Operation::ReplaceText {
    node: text,
    range: Utf16Range::new(start, full_length),
    inserted: String::new(),
  }];
  let mut next_id = document.next_available_id().0;
  let mut groups = BTreeMap::new();
  let mut id_map = BTreeMap::new();
  let mut insertion_index = text_index + 1;
  let mut last_source_text = None;

  for (block_index, block) in blocks.iter().enumerate() {
    if block_index > 0 {
      for separator in separators(context, &mut next_id) {
        operations.push(Operation::InsertNode {
          parent: container,
          index: insertion_index,
          node: separator,
        });
        insertion_index += 1;
      }
    }

    for child in &block.children {
      let subtree = clone_subtree(&fragment, *child, &mut next_id, &mut groups, &mut id_map)?;
      operations.push(Operation::InsertSubtree {
        parent: container,
        index: insertion_index,
        subtree,
      });
      insertion_index += 1;
    }
    last_source_text = last_text_descendant(&fragment, block.id).or(last_source_text);
  }

  let (caret_node, caret_offset) = if let Some(source_text) = last_source_text {
    let target = *id_map
      .get(&source_text)
      .ok_or(EditError::UnsupportedStructure(source_text))?;
    let offset = text_value(&fragment, source_text)?.encode_utf16().count() as u32;
    (target, offset)
  } else {
    let id = NodeId(next_id);
    next_id = next_id.saturating_add(1);
    operations.push(Operation::InsertNode {
      parent: container,
      index: insertion_index,
      node: Node::new(
        id,
        NodeKind::Inline(InlineKind::Text {
          value: String::new(),
        }),
        None,
      ),
    });
    insertion_index += 1;
    (id, 0)
  };

  if !suffix.is_empty() {
    let suffix_id = NodeId(next_id);
    operations.push(Operation::InsertNode {
      parent: container,
      index: insertion_index,
      node: Node::new(
        suffix_id,
        NodeKind::Inline(InlineKind::Text { value: suffix }),
        None,
      ),
    });
  }

  Ok(Some(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret_node,
      offset_utf16: caret_offset,
    }),
  }))
}

fn nested_container(
  document: &Document,
  text: NodeId,
) -> Result<Option<(NodeId, NestedPasteContext, usize)>, EditError> {
  let node = document.node(text).ok_or(EditError::NodeNotFound(text))?;
  let parent = node.parent.ok_or(EditError::UnsupportedStructure(text))?;
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  let context = match &parent_node.kind {
    NodeKind::Block(BlockKind::BlockQuote) => NestedPasteContext::BlockQuote,
    NodeKind::Block(BlockKind::TableCell { .. }) => NestedPasteContext::TableCell,
    NodeKind::Block(BlockKind::Paragraph) => {
      let Some(grandparent) = parent_node.parent else {
        return Ok(None);
      };
      if !matches!(
        document.node(grandparent).map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::ListItem { .. }))
      ) {
        return Ok(None);
      }
      NestedPasteContext::ListItem
    }
    _ => return Ok(None),
  };
  let text_index = document
    .child_index(parent, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  Ok(Some((parent, context, text_index)))
}

fn separators(context: NestedPasteContext, next_id: &mut u64) -> Vec<Node> {
  match context {
    NestedPasteContext::BlockQuote => vec![
      allocate_inline(next_id, InlineKind::SoftBreak),
      allocate_inline(next_id, InlineKind::SoftBreak),
    ],
    NestedPasteContext::ListItem => vec![allocate_inline(
      next_id,
      InlineKind::InlineHtml {
        raw: "\n  \n  ".to_string(),
      },
    )],
    NestedPasteContext::TableCell => vec![allocate_inline(
      next_id,
      InlineKind::InlineHtml {
        raw: "<br/><br/>".to_string(),
      },
    )],
  }
}

fn allocate_inline(next_id: &mut u64, kind: InlineKind) -> Node {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  Node::new(id, NodeKind::Inline(kind), None)
}

fn clone_subtree(
  source: &Document,
  root: NodeId,
  next_id: &mut u64,
  groups: &mut BTreeMap<u64, u64>,
  id_map: &mut BTreeMap<NodeId, NodeId>,
) -> Result<DetachedSubtree, EditError> {
  let mut nodes = Vec::new();
  let cloned_root = clone_node(source, root, None, next_id, groups, id_map, &mut nodes)?;
  Ok(DetachedSubtree {
    root: cloned_root,
    nodes,
  })
}

fn clone_node(
  source: &Document,
  source_id: NodeId,
  parent: Option<NodeId>,
  next_id: &mut u64,
  groups: &mut BTreeMap<u64, u64>,
  id_map: &mut BTreeMap<NodeId, NodeId>,
  nodes: &mut Vec<Node>,
) -> Result<NodeId, EditError> {
  let source_node = source
    .node(source_id)
    .ok_or(EditError::NodeNotFound(source_id))?;
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  id_map.insert(source_id, id);

  let kind = remap_groups(&source_node.kind, next_id, groups);
  let index = nodes.len();
  let mut node = Node::new(id, kind, None);
  node.parent = parent;
  nodes.push(node);
  let children = source_node
    .children
    .iter()
    .copied()
    .map(|child| clone_node(source, child, Some(id), next_id, groups, id_map, nodes))
    .collect::<Result<Vec<_>, _>>()?;
  nodes[index].children = children;
  Ok(id)
}

fn remap_groups(kind: &NodeKind, next_id: &mut u64, groups: &mut BTreeMap<u64, u64>) -> NodeKind {
  let NodeKind::Inline(InlineKind::MarkFragment { mark, group, edge }) = kind else {
    return kind.clone();
  };
  let mapped = *groups.entry(*group).or_insert_with(|| {
    let value = *next_id;
    *next_id = next_id.saturating_add(1);
    value
  });
  NodeKind::Inline(InlineKind::MarkFragment {
    mark: *mark,
    group: mapped,
    edge: *edge,
  })
}

fn last_text_descendant(document: &Document, root: NodeId) -> Option<NodeId> {
  let node = document.node(root)?;
  if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
    return Some(root);
  }
  node
    .children
    .iter()
    .rev()
    .find_map(|child| last_text_descendant(document, *child))
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::to_markdown;

  fn text(document: &Document, expected: &str) -> NodeId {
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
      .id
  }

  fn apply(initial: &str, target: &str, pasted: &str) -> (Document, Transaction) {
    let mut document = crate::parse_markdown(initial);
    let node = text(&document, target);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 2,
    });
    let transaction = build_nested_paste(&document, selection, pasted)
      .unwrap()
      .unwrap();
    let inverse = transaction.apply(&mut document).unwrap();
    (document, inverse)
  }

  fn assert_nested(initial: &str, target: &str, pasted: &str, expected: &str) {
    let (mut document, inverse) = apply(initial, target, pasted);
    assert_eq!(to_markdown(&document), expected);
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), initial);
  }

  #[test]
  fn pastes_inside_a_list_item() {
    assert_nested("- alpha", "alpha", "XYZ", "- alXYZpha");
    assert_nested("- alpha", "alpha", "one\n\ntwo", "- alone\n  \n  twopha");
  }

  #[test]
  fn pastes_inside_a_blockquote() {
    assert_nested("> alpha", "alpha", "XYZ", "> alXYZpha");
    assert_nested("> alpha", "alpha", "one\n\ntwo", "> alone\n> \n> twopha");
  }

  #[test]
  fn pastes_inside_a_table_cell() {
    let table = "| A     | B    |\n| ----- | ---- |\n| alpha | beta |";
    assert_nested(
      table,
      "alpha",
      "XYZ",
      "| A        | B    |\n| -------- | ---- |\n| alXYZpha | beta |",
    );
    assert_nested(
      table,
      "alpha",
      "one\n\ntwo",
      "| A                     | B    |\n| --------------------- | ---- |\n| alone<br/><br/>twopha | beta |",
    );
  }
}
