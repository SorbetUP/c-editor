use std::collections::BTreeMap;

use crate::model::{
  BlockKind, DetachedSubtree, Document, InlineKind, ListKind, Node, NodeId, NodeKind,
};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{Command, EditError, Operation, Transaction, Utf16Range};

#[derive(Clone, Copy)]
enum StructuredContext {
  ListItem {
    paragraph: NodeId,
    item: NodeId,
    list: NodeId,
  },
  BlockQuote {
    quote: NodeId,
  },
  TableCell {
    cell: NodeId,
  },
}

pub(crate) fn build_nested_structured_paste(
  document: &Document,
  selection: Selection,
  markdown: &str,
) -> Result<Option<Transaction>, EditError> {
  let Some((text, start, end)) = selection.ordered_same_node() else {
    return Ok(None);
  };
  let Some(context) = structured_context(document, text)? else {
    return Ok(None);
  };
  let fragment = crate::parse_markdown(markdown);
  let blocks = fragment.children(fragment.root).collect::<Vec<_>>();
  if blocks.is_empty()
    || blocks
      .iter()
      .all(|block| matches!(block.kind, NodeKind::Block(BlockKind::Paragraph)))
  {
    return Ok(None);
  }

  match context {
    StructuredContext::TableCell { cell } => {
      build_table_cell_paste(document, selection, text, start, end, cell, markdown).map(Some)
    }
    StructuredContext::BlockQuote { quote } => build_blockquote_paste(
      document, selection, text, start, end, quote, markdown, &blocks,
    ),
    StructuredContext::ListItem {
      paragraph,
      item,
      list,
    } => build_list_item_paste(
      document, selection, text, start, end, paragraph, item, list, &fragment, &blocks,
    ),
  }
}

fn build_table_cell_paste(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  cell: NodeId,
  markdown: &str,
) -> Result<Transaction, EditError> {
  let value = text_value(document, text)?;
  utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let suffix = value[end_byte..].to_string();
  let raw = markdown
    .trim_end_matches(['\r', '\n'])
    .replace('\n', "<br/>");
  let raw_id = document.next_available_id();
  let suffix_id = NodeId(raw_id.0 + 1);
  let text_index = document
    .child_index(cell, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let mut operations = vec![
    Operation::ReplaceText {
      node: text,
      range: Utf16Range::new(start, value.encode_utf16().count() as u32),
      inserted: String::new(),
    },
    Operation::InsertNode {
      parent: cell,
      index: text_index + 1,
      node: text_node(raw_id, raw.clone()),
    },
  ];
  if !suffix.is_empty() {
    operations.push(Operation::InsertNode {
      parent: cell,
      index: text_index + 2,
      node: text_node(suffix_id, suffix),
    });
  }
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: raw_id,
      offset_utf16: raw.encode_utf16().count() as u32,
    }),
  })
}

fn build_blockquote_paste(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  quote: NodeId,
  markdown: &str,
  blocks: &[&Node],
) -> Result<Option<Transaction>, EditError> {
  if blocks.len() == 1 && matches!(blocks[0].kind, NodeKind::Block(BlockKind::Heading { .. })) {
    return Command::InsertText(markdown.to_string())
      .build(document, selection)
      .map(Some);
  }
  if blocks.len() != 1 || !matches!(blocks[0].kind, NodeKind::Block(BlockKind::List { .. })) {
    return Ok(None);
  }

  let value = text_value(document, text)?;
  utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let suffix = value[end_byte..].to_string();
  let lines = markdown
    .trim_end_matches(['\r', '\n'])
    .lines()
    .map(str::to_string)
    .collect::<Vec<_>>();
  if lines.is_empty() {
    return Ok(None);
  }
  let text_index = document
    .child_index(quote, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let mut next_id = document.next_available_id().0;
  let mut insertion_index = text_index + 1;
  let mut operations = vec![Operation::ReplaceText {
    node: text,
    range: Utf16Range::new(start, value.encode_utf16().count() as u32),
    inserted: String::new(),
  }];
  for _ in 0..2 {
    operations.push(Operation::InsertNode {
      parent: quote,
      index: insertion_index,
      node: inline_node(&mut next_id, InlineKind::SoftBreak),
    });
    insertion_index += 1;
  }

  let mut caret = None;
  for (line_index, line) in lines.iter().enumerate() {
    if line_index > 0 {
      operations.push(Operation::InsertNode {
        parent: quote,
        index: insertion_index,
        node: inline_node(&mut next_id, InlineKind::SoftBreak),
      });
      insertion_index += 1;
    }
    let id = NodeId(next_id);
    next_id = next_id.saturating_add(1);
    operations.push(Operation::InsertNode {
      parent: quote,
      index: insertion_index,
      node: text_node(id, line.clone()),
    });
    insertion_index += 1;
    caret = Some((id, line.encode_utf16().count() as u32));
  }
  if !suffix.is_empty() {
    let id = NodeId(next_id);
    operations.push(Operation::InsertNode {
      parent: quote,
      index: insertion_index,
      node: text_node(id, suffix),
    });
  }
  let (caret_node, caret_offset) = caret.ok_or(EditError::UnsupportedStructure(quote))?;
  Ok(Some(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret_node,
      offset_utf16: caret_offset,
    }),
  }))
}

#[allow(clippy::too_many_arguments)]
fn build_list_item_paste(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  paragraph: NodeId,
  item: NodeId,
  list: NodeId,
  fragment: &Document,
  blocks: &[&Node],
) -> Result<Option<Transaction>, EditError> {
  if blocks.len() != 1 {
    return Ok(None);
  }
  match &blocks[0].kind {
    NodeKind::Block(BlockKind::List { kind, .. }) => build_list_in_list_item(
      document,
      selection,
      text,
      start,
      end,
      paragraph,
      item,
      list,
      fragment,
      blocks[0].id,
      *kind,
    )
    .map(Some),
    NodeKind::Block(BlockKind::CodeBlock { .. }) => build_code_in_list_item(
      document,
      selection,
      text,
      start,
      end,
      paragraph,
      item,
      fragment,
      blocks[0].id,
    )
    .map(Some),
    _ => Ok(None),
  }
}

#[allow(clippy::too_many_arguments)]
fn build_list_in_list_item(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  paragraph: NodeId,
  item: NodeId,
  list: NodeId,
  fragment: &Document,
  pasted_list: NodeId,
  pasted_kind: ListKind,
) -> Result<Transaction, EditError> {
  let NodeKind::Block(BlockKind::List {
    kind: current_kind, ..
  }) = &document
    .node(list)
    .ok_or(EditError::NodeNotFound(list))?
    .kind
  else {
    return Err(EditError::UnsupportedStructure(list));
  };
  if *current_kind != pasted_kind {
    return Err(EditError::UnsupportedStructure(pasted_list));
  }
  let pasted_items = fragment
    .node(pasted_list)
    .ok_or(EditError::NodeNotFound(pasted_list))?
    .children
    .clone();
  let first_item = *pasted_items
    .first()
    .ok_or(EditError::UnsupportedStructure(pasted_list))?;
  let first_paragraph = first_paragraph(fragment, first_item)?;
  let value = text_value(document, text)?;
  utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let suffix = value[end_byte..].to_string();
  let text_index = document
    .child_index(paragraph, text)
    .ok_or(EditError::UnsupportedStructure(text))?;
  let item_index = document
    .child_index(list, item)
    .ok_or(EditError::UnsupportedStructure(item))?;
  let mut operations = vec![Operation::ReplaceText {
    node: text,
    range: Utf16Range::new(start, value.encode_utf16().count() as u32),
    inserted: String::new(),
  }];
  let mut next_id = document.next_available_id().0;
  let mut groups = BTreeMap::new();
  let mut id_map = BTreeMap::new();

  let first_children = fragment
    .node(first_paragraph)
    .ok_or(EditError::NodeNotFound(first_paragraph))?
    .children
    .clone();
  for (offset, child) in first_children.iter().copied().enumerate() {
    operations.push(Operation::InsertSubtree {
      parent: paragraph,
      index: text_index + 1 + offset,
      subtree: clone_subtree(fragment, child, &mut next_id, &mut groups, &mut id_map)?,
    });
  }

  for (offset, pasted_item) in pasted_items.iter().copied().skip(1).enumerate() {
    operations.push(Operation::InsertSubtree {
      parent: list,
      index: item_index + 1 + offset,
      subtree: clone_subtree(
        fragment,
        pasted_item,
        &mut next_id,
        &mut groups,
        &mut id_map,
      )?,
    });
  }

  let last_item = *pasted_items.last().unwrap();
  let source_text =
    last_text_descendant(fragment, last_item).ok_or(EditError::UnsupportedStructure(last_item))?;
  let caret_node = *id_map
    .get(&source_text)
    .ok_or(EditError::UnsupportedStructure(source_text))?;
  let caret_offset = text_value(fragment, source_text)?.encode_utf16().count() as u32;
  let (tail_parent, tail_index) = if pasted_items.len() == 1 {
    (paragraph, text_index + 1 + first_children.len())
  } else {
    let source_parent = last_inline_container(fragment, last_item)
      .ok_or(EditError::UnsupportedStructure(last_item))?;
    let target_parent = *id_map
      .get(&source_parent)
      .ok_or(EditError::UnsupportedStructure(source_parent))?;
    let child_count = fragment
      .node(source_parent)
      .ok_or(EditError::NodeNotFound(source_parent))?
      .children
      .len();
    (target_parent, child_count)
  };
  if !suffix.is_empty() {
    let suffix_id = NodeId(next_id);
    operations.push(Operation::InsertNode {
      parent: tail_parent,
      index: tail_index,
      node: text_node(suffix_id, suffix),
    });
  }
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret_node,
      offset_utf16: caret_offset,
    }),
  })
}

#[allow(clippy::too_many_arguments)]
fn build_code_in_list_item(
  document: &Document,
  selection: Selection,
  text: NodeId,
  start: u32,
  end: u32,
  paragraph: NodeId,
  item: NodeId,
  fragment: &Document,
  code: NodeId,
) -> Result<Transaction, EditError> {
  let value = text_value(document, text)?;
  utf16_to_byte(value, text, start)?;
  let end_byte = utf16_to_byte(value, text, end)?;
  let suffix = value[end_byte..].to_string();
  let paragraph_index = document
    .child_index(item, paragraph)
    .ok_or(EditError::UnsupportedStructure(paragraph))?;
  let mut next_id = document.next_available_id().0;
  let mut groups = BTreeMap::new();
  let mut id_map = BTreeMap::new();
  let subtree = clone_subtree(fragment, code, &mut next_id, &mut groups, &mut id_map)?;
  let source_text =
    last_text_descendant(fragment, code).ok_or(EditError::UnsupportedStructure(code))?;
  let caret_node = *id_map
    .get(&source_text)
    .ok_or(EditError::UnsupportedStructure(source_text))?;
  let caret_offset = text_value(fragment, source_text)?.encode_utf16().count() as u32;
  let target_code = *id_map
    .get(&code)
    .ok_or(EditError::UnsupportedStructure(code))?;
  let mut operations = vec![
    Operation::ReplaceText {
      node: text,
      range: Utf16Range::new(start, value.encode_utf16().count() as u32),
      inserted: String::new(),
    },
    Operation::InsertSubtree {
      parent: item,
      index: paragraph_index + 1,
      subtree,
    },
  ];
  if !suffix.is_empty() {
    let suffix_id = NodeId(next_id);
    let child_count = fragment
      .node(code)
      .ok_or(EditError::NodeNotFound(code))?
      .children
      .len();
    operations.push(Operation::InsertNode {
      parent: target_code,
      index: child_count,
      node: text_node(suffix_id, suffix),
    });
  }
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: caret_node,
      offset_utf16: caret_offset,
    }),
  })
}

fn structured_context(
  document: &Document,
  text: NodeId,
) -> Result<Option<StructuredContext>, EditError> {
  let parent = document
    .node(text)
    .ok_or(EditError::NodeNotFound(text))?
    .parent
    .ok_or(EditError::UnsupportedStructure(text))?;
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  match parent_node.kind {
    NodeKind::Block(BlockKind::TableCell { .. }) => {
      Ok(Some(StructuredContext::TableCell { cell: parent }))
    }
    NodeKind::Block(BlockKind::BlockQuote) => {
      Ok(Some(StructuredContext::BlockQuote { quote: parent }))
    }
    NodeKind::Block(BlockKind::Paragraph) => {
      let item = parent_node
        .parent
        .ok_or(EditError::UnsupportedStructure(parent))?;
      if !matches!(
        document.node(item).map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::ListItem { .. }))
      ) {
        return Ok(None);
      }
      let list = document
        .node(item)
        .and_then(|node| node.parent)
        .ok_or(EditError::UnsupportedStructure(item))?;
      if !matches!(
        document.node(list).map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::List { .. }))
      ) {
        return Ok(None);
      }
      Ok(Some(StructuredContext::ListItem {
        paragraph: parent,
        item,
        list,
      }))
    }
    _ => Ok(None),
  }
}

fn first_paragraph(document: &Document, item: NodeId) -> Result<NodeId, EditError> {
  document
    .node(item)
    .ok_or(EditError::NodeNotFound(item))?
    .children
    .iter()
    .copied()
    .find(|child| {
      matches!(
        document.node(*child).map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::Paragraph))
      )
    })
    .ok_or(EditError::UnsupportedStructure(item))
}

fn text_node(id: NodeId, value: String) -> Node {
  Node::new(id, NodeKind::Inline(InlineKind::Text { value }), None)
}

fn inline_node(next_id: &mut u64, kind: InlineKind) -> Node {
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

fn last_inline_container(document: &Document, root: NodeId) -> Option<NodeId> {
  let node = document.node(root)?;
  for child in node.children.iter().rev() {
    if let Some(container) = last_inline_container(document, *child) {
      return Some(container);
    }
  }
  matches!(
    node.kind,
    NodeKind::Block(
      BlockKind::Paragraph
        | BlockKind::Heading { .. }
        | BlockKind::BlockQuote
        | BlockKind::TableCell { .. }
        | BlockKind::CodeBlock { .. }
    )
  )
  .then_some(root)
}

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  let node = document.node(node).ok_or(EditError::NodeNotFound(node))?;
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node.id)),
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

  fn assert_structured(initial: &str, pasted: &str, expected: &str) {
    let mut document = crate::parse_markdown(initial);
    let node = text(&document, "alpha");
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 2,
    });
    let transaction = build_nested_structured_paste(&document, selection, pasted)
      .unwrap()
      .unwrap();
    let inverse = transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), expected);
    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), initial);
  }

  #[test]
  fn pastes_lists_and_code_inside_list_items() {
    assert_structured("- alpha", "- one\n- two", "- alone\n- twopha");
    assert_structured(
      "- alpha",
      "```js\nconsole.log(1)\n```",
      "- al\n  \n  ```js\n  console.log(1)pha\n  ```",
    );
  }

  #[test]
  fn pastes_structured_markdown_inside_table_cells() {
    let table = "| A     | B    |\n| ----- | ---- |\n| alpha | beta |";
    assert_structured(
      table,
      "- one\n- two",
      "| A                    | B    |\n| -------------------- | ---- |\n| al- one<br/>- twopha | beta |",
    );
    assert_structured(
      table,
      "```js\nconsole.log(1)\n```",
      "| A                                     | B    |\n| ------------------------------------- | ---- |\n| al```js<br/>console.log(1)<br/>```pha | beta |",
    );
  }

  #[test]
  fn pastes_lists_and_literal_headings_inside_blockquotes() {
    assert_structured("> alpha", "- one\n- two", "> al\n> \n> - one\n> - twopha");
    assert_structured("> alpha", "# Title", "> al# Titlepha");
  }
}
