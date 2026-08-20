use crate::model::{BlockKind, Document, ListKind, Node, NodeId, NodeKind, SourceRange};
use crate::syntax::block::list;

#[derive(Clone, Debug)]
struct SourceLine<'a> {
  text: &'a str,
  end: u32,
}

#[derive(Clone, Debug)]
struct ParsedItem {
  checked: Option<bool>,
  body: String,
}

#[derive(Clone, Debug)]
struct ParsedList {
  kind: ListKind,
  start: Option<u64>,
  items: Vec<ParsedItem>,
  end: u32,
}

pub(crate) fn apply(document: &mut Document, markdown: &str) {
  let root = document.root;
  let candidates = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for candidate in candidates {
    let source = document.node(candidate).and_then(|node| match &node.kind {
      NodeKind::Block(BlockKind::List { .. }) => node.source,
      _ => None,
    });
    let Some(source) = source else {
      continue;
    };
    let Some(byte_start) = byte_index_at_utf16(markdown, source.start) else {
      continue;
    };
    let suffix = &markdown[byte_start..];
    let lines = source_lines(suffix);
    let Some(first_line) = lines.first() else {
      continue;
    };
    let Some(first_marker) = list::parse(first_line.text) else {
      continue;
    };
    let Some(parsed) = parse_list_source(&lines, first_marker.indent, first_marker.kind) else {
      continue;
    };

    replace_root_list(document, candidate, source.start, parsed);
  }
}

fn parse_list_source(lines: &[SourceLine<'_>], indent: usize, kind: ListKind) -> Option<ParsedList> {
  let first = list::parse(lines.first()?.text)?;
  if first.indent != indent || first.kind != kind {
    return None;
  }

  let mut items = Vec::new();
  let mut index = 0usize;
  let mut end = lines[0].end;

  while index < lines.len() {
    let Some(marker) = list::parse(lines[index].text) else {
      break;
    };
    if marker.indent != indent || marker.kind != kind {
      break;
    }

    let (item, next, item_end) = parse_item(lines, index, indent, kind, marker.content.len());
    items.push(ParsedItem {
      checked: marker.checked,
      body: item,
    });
    end = item_end;
    index = next;
  }

  (!items.is_empty()).then_some(ParsedList {
    kind,
    start: first.start,
    items,
    end,
  })
}

fn parse_item(
  lines: &[SourceLine<'_>],
  start_index: usize,
  list_indent: usize,
  list_kind: ListKind,
  content_len: usize,
) -> (String, usize, u32) {
  let first = &lines[start_index];
  let marker = list::parse(first.text).expect("list item marker");
  let content_prefix = first.text.len().saturating_sub(content_len);
  let mut body = vec![marker.content.to_string()];
  let mut index = start_index + 1;
  let mut end = first.end;
  let mut blank_start = None;
  let mut blank_count = 0usize;

  while index < lines.len() {
    let line = &lines[index];

    if line.text.trim().is_empty() {
      blank_start.get_or_insert(index);
      blank_count += 1;
      index += 1;
      continue;
    }

    if let Some(next_marker) = list::parse(line.text) {
      if next_marker.indent < list_indent {
        if let Some(blank) = blank_start {
          index = blank;
        }
        break;
      }
      if next_marker.indent == list_indent {
        if next_marker.kind == list_kind {
          break;
        }
        if let Some(blank) = blank_start {
          index = blank;
        }
        break;
      }
    }

    let leading = leading_spaces(line.text);
    if blank_count > 0 && leading <= list_indent {
      index = blank_start.unwrap_or(index);
      break;
    }

    for _ in 0..blank_count {
      body.push(String::new());
    }
    blank_start = None;
    blank_count = 0;

    body.push(outdent_continuation(line.text, content_prefix));
    end = line.end;
    index += 1;
  }

  (body.join("\n"), index, end)
}

fn replace_root_list(
  document: &mut Document,
  original: NodeId,
  source_start: u32,
  parsed: ParsedList,
) {
  let root = document.root;
  let Some(insert_at) = document.child_index(root, original) else {
    return;
  };
  let absolute_end = source_start + parsed.end;

  let overlapping = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default()
    .into_iter()
    .skip(insert_at)
    .take_while(|id| {
      document
        .node(*id)
        .and_then(|node| node.source)
        .is_some_and(|range| range.start < absolute_end)
    })
    .collect::<Vec<_>>();

  for id in overlapping.into_iter().rev() {
    assert!(
      document.remove_subtree(id).is_some(),
      "list normalization must remove complete root subtrees"
    );
  }

  let list_node = document.next_available_id();
  assert!(document.insert_detached_node(
    root,
    insert_at,
    Node::new(
      list_node,
      NodeKind::Block(BlockKind::List {
        kind: parsed.kind,
        start: parsed.start,
      }),
      Some(SourceRange::new(source_start, absolute_end)),
    ),
  ));

  for item in parsed.items {
    append_item(document, list_node, item);
  }
}

fn append_item(document: &mut Document, list_node: NodeId, parsed: ParsedItem) {
  let item = document.next_available_id();
  let item_index = document
    .node(list_node)
    .map(|node| node.children.len())
    .unwrap_or_default();
  assert!(document.insert_detached_node(
    list_node,
    item_index,
    Node::new(
      item,
      NodeKind::Block(BlockKind::ListItem {
        checked: parsed.checked,
      }),
      None,
    ),
  ));

  if parsed.body.is_empty() {
    let paragraph = document.next_available_id();
    assert!(document.insert_detached_node(
      item,
      0,
      Node::new(paragraph, NodeKind::Block(BlockKind::Paragraph), None),
    ));
    return;
  }

  let inner = crate::parse_markdown(&parsed.body);
  let children = inner
    .node(inner.root)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  if children.is_empty() {
    let paragraph = document.next_available_id();
    assert!(document.insert_detached_node(
      item,
      0,
      Node::new(paragraph, NodeKind::Block(BlockKind::Paragraph), None),
    ));
    return;
  }

  for (index, child) in children.into_iter().enumerate() {
    clone_subtree(&inner, child, document, item, index);
  }
}

fn clone_subtree(
  source: &Document,
  source_id: NodeId,
  target: &mut Document,
  parent: NodeId,
  index: usize,
) -> NodeId {
  let source_node = source
    .node(source_id)
    .expect("source subtree node must exist");
  let id = target.next_available_id();
  assert!(target.insert_detached_node(
    parent,
    index,
    Node::new(id, source_node.kind.clone(), None),
  ));

  for (child_index, child) in source_node.children.iter().copied().enumerate() {
    clone_subtree(source, child, target, id, child_index);
  }
  id
}

fn source_lines(source: &str) -> Vec<SourceLine<'_>> {
  let mut offset = 0u32;
  source
    .split_inclusive('\n')
    .map(|segment| {
      let text = segment
        .strip_suffix('\n')
        .unwrap_or(segment)
        .trim_end_matches('\r');
      let end = offset + text.encode_utf16().count() as u32;
      offset += segment.encode_utf16().count() as u32;
      SourceLine { text, end }
    })
    .collect()
}

fn leading_spaces(value: &str) -> usize {
  value.chars().take_while(|character| *character == ' ').count()
}

fn outdent_continuation(value: &str, prefix: usize) -> String {
  let removable = leading_spaces(value).min(prefix);
  value[removable..].to_string()
}

fn byte_index_at_utf16(source: &str, target: u32) -> Option<usize> {
  let mut units = 0u32;
  for (index, character) in source.char_indices() {
    if units == target {
      return Some(index);
    }
    units += character.len_utf16() as u32;
    if units > target {
      return None;
    }
  }
  (units == target).then_some(source.len())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::serializer::to_markdown;

  #[test]
  fn multiline_item_body_is_reparsed_as_markdown_blocks() {
    let source = "- first line\n  continuation with **bold**\n\n  second paragraph\n  > quote\n- sibling";
    let mut document = crate::parser::parse_markdown(source);
    apply(&mut document, source);

    let list = document.children(document.root).next().unwrap();
    let first = document.children(list.id).next().unwrap();
    let blocks = document.children(first.id).collect::<Vec<_>>();
    assert!(blocks.len() >= 3);
    assert!(matches!(blocks[0].kind, NodeKind::Block(BlockKind::Paragraph)));
    assert!(blocks.iter().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::BlockQuote)
    )));
  }

  #[test]
  fn nested_lists_are_item_children_not_root_siblings() {
    let source = "- parent\n  - child\n    3. grandchild\n- sibling";
    let mut document = crate::parser::parse_markdown(source);
    apply(&mut document, source);

    let root_blocks = document.children(document.root).collect::<Vec<_>>();
    assert_eq!(root_blocks.len(), 1);
    let first_item = document.children(root_blocks[0].id).next().unwrap();
    assert!(document.children(first_item.id).any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::List { .. })
    )));
  }

  #[test]
  fn lazy_continuation_stays_inside_item_until_a_blank_top_level_break() {
    let source = "- item\nlazy continuation\n\noutside";
    let mut document = crate::parser::parse_markdown(source);
    apply(&mut document, source);

    let root_blocks = document.children(document.root).collect::<Vec<_>>();
    assert_eq!(root_blocks.len(), 2);
    assert!(matches!(root_blocks[0].kind, NodeKind::Block(BlockKind::List { .. })));
    assert!(matches!(root_blocks[1].kind, NodeKind::Block(BlockKind::Paragraph)));
    assert!(to_markdown(&document).contains("lazy continuation"));
  }

  #[test]
  fn source_line_offsets_remain_utf16_safe() {
    let lines = source_lines("😀\nnext");
    assert_eq!(lines[0].end, 2);
    assert_eq!(lines[1].end, 7);
  }
}
