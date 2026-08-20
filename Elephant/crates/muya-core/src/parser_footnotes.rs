use crate::model::{BlockKind, Document, Node, NodeId, NodeKind, SourceRange};

#[derive(Clone, Debug)]
struct SourceLine<'a> {
  text: &'a str,
  end: u32,
}

#[derive(Clone, Debug)]
struct ParsedFootnote {
  label: String,
  body: String,
  end: u32,
}

pub(crate) fn apply(document: &mut Document, markdown: &str) {
  let root = document.root;
  let candidates = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for candidate in candidates {
    let range = document.node(candidate).and_then(|node| match &node.kind {
      NodeKind::Block(BlockKind::Paragraph) => node.source,
      _ => None,
    });
    let Some(range) = range else {
      continue;
    };
    let Some(byte_start) = byte_index_at_utf16(markdown, range.start) else {
      continue;
    };
    let suffix = &markdown[byte_start..];
    let Some(parsed) = parse_footnote_source(suffix) else {
      continue;
    };
    replace_root_footnote(document, candidate, range.start, parsed);
  }
}

fn parse_footnote_source(source: &str) -> Option<ParsedFootnote> {
  let lines = source_lines(source);
  let first = lines.first()?;
  let (label, first_body) = parse_marker(first.text)?;
  let mut body = vec![first_body.to_string()];
  let mut index = 1usize;
  let mut end = first.end;
  let mut blank_count = 0usize;

  while index < lines.len() {
    let line = &lines[index];
    if line.text.trim().is_empty() {
      blank_count += 1;
      index += 1;
      continue;
    }

    let leading = leading_spaces(line.text);
    if blank_count > 0 && leading <= 3 {
      break;
    }

    for _ in 0..blank_count {
      body.push(String::new());
    }
    blank_count = 0;
    body.push(strip_footnote_indent(line.text).to_string());
    end = line.end;
    index += 1;
  }

  let body = body.join("\n").trim_end_matches('\n').to_string();
  if body.trim().is_empty() {
    return None;
  }

  Some(ParsedFootnote {
    label: label.to_string(),
    body,
    end,
  })
}

fn parse_marker(line: &str) -> Option<(&str, &str)> {
  let rest = line.strip_prefix("[^")?;
  let close = rest.find("]:" )?;
  let label = &rest[..close];
  if label.is_empty()
    || label
      .chars()
      .any(|character| character.is_whitespace() || matches!(character, '^' | '[' | ']'))
  {
    return None;
  }
  let body = rest[close + 2..].trim_start_matches(|character| matches!(character, ' ' | '\t'));
  Some((label, body))
}

fn replace_root_footnote(
  document: &mut Document,
  original: NodeId,
  source_start: u32,
  parsed: ParsedFootnote,
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
      "footnote normalization must remove complete root subtrees"
    );
  }

  let footnote = document.next_available_id();
  assert!(document.insert_detached_node(
    root,
    insert_at,
    Node::new(
      footnote,
      NodeKind::Block(BlockKind::FootnoteDefinition {
        label: parsed.label,
      }),
      Some(SourceRange::new(source_start, absolute_end)),
    ),
  ));

  let inner = crate::parse_markdown(&parsed.body);
  let children = inner
    .node(inner.root)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  for (index, child) in children.into_iter().enumerate() {
    clone_subtree(&inner, child, document, footnote, index);
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
      let end = offset + utf16_len(text);
      offset += utf16_len(segment);
      SourceLine { text, end }
    })
    .collect()
}

fn strip_footnote_indent(line: &str) -> &str {
  line.strip_prefix("    ").unwrap_or(line)
}

fn leading_spaces(value: &str) -> usize {
  value.chars().take_while(|character| *character == ' ').count()
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

fn utf16_len(value: &str) -> u32 {
  value.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_structured_footnote_until_blank_top_level_block() {
    let source = "[^id]: first **bold**\n\n    second paragraph\n\n    - nested\n\noutside";
    let parsed = parse_footnote_source(source).expect("footnote");
    assert_eq!(parsed.label, "id");
    assert!(parsed.body.contains("second paragraph"));
    assert!(parsed.body.contains("- nested"));
    assert!(!parsed.body.contains("outside"));
  }

  #[test]
  fn public_pipeline_builds_block_children() {
    let markdown = "[^id]: first\n\n    second paragraph\n\n    > quote\n\noutside";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);
    let footnote = document.children(document.root).next().unwrap();
    assert!(matches!(
      footnote.kind,
      NodeKind::Block(BlockKind::FootnoteDefinition { .. })
    ));
    let blocks = document.children(footnote.id).collect::<Vec<_>>();
    assert!(blocks.len() >= 3);
    assert!(blocks.iter().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::BlockQuote)
    )));
  }

  #[test]
  fn marker_rejects_invalid_labels() {
    assert!(parse_marker("[^good]: body").is_some());
    assert!(parse_marker("[^bad label]: body").is_none());
    assert!(parse_marker("[normal]: body").is_none());
  }
}
