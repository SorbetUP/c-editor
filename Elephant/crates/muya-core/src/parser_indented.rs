use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind, SourceRange};

pub(crate) fn apply(document: &mut Document, markdown: &str) {
  let root = document.root;
  let blocks = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for block in blocks {
    let Some(node) = document.node(block) else {
      continue;
    };
    if !matches!(node.kind, NodeKind::Block(BlockKind::Paragraph)) {
      continue;
    }
    let Some(range) = node.source else {
      continue;
    };
    let Some(raw) = utf16_slice(markdown, range) else {
      continue;
    };
    let Some(split) = split_leading_indented_code(raw) else {
      continue;
    };
    replace_leading_code(document, block, range, split);
  }
}

struct IndentedSplit<'a> {
  code: String,
  code_end_utf16: u32,
  rest: &'a str,
  rest_start_utf16: u32,
}

fn split_leading_indented_code(raw: &str) -> Option<IndentedSplit<'_>> {
  let lines = raw.split_inclusive('\n').collect::<Vec<_>>();
  let first = lines.first()?.trim_end_matches('\n').trim_end_matches('\r');
  let first_body = first.strip_prefix("    ")?;
  if first_body.is_empty() {
    return None;
  }

  let mut code_lines = Vec::new();
  let mut pending_blank = Vec::new();
  let mut consumed = 0usize;
  let mut committed = 0usize;

  for segment in lines {
    let line = segment.trim_end_matches('\n').trim_end_matches('\r');
    if line.is_empty() {
      pending_blank.push(String::new());
      consumed += segment.len();
      continue;
    }
    let Some(body) = line.strip_prefix("    ") else {
      break;
    };
    if body.is_empty() {
      break;
    }
    code_lines.append(&mut pending_blank);
    code_lines.push(body.to_string());
    consumed += segment.len();
    committed = consumed;
  }

  if code_lines.is_empty() {
    return None;
  }

  let rest_with_blanks = &raw[committed..];
  let rest = rest_with_blanks.trim_start_matches(|character| matches!(character, '\r' | '\n'));
  let skipped = rest_with_blanks.len().saturating_sub(rest.len());

  Some(IndentedSplit {
    code: code_lines.join("\n"),
    code_end_utf16: utf16_len(&raw[..committed]),
    rest,
    rest_start_utf16: utf16_len(&raw[..committed + skipped]),
  })
}

fn replace_leading_code(
  document: &mut Document,
  original: NodeId,
  range: SourceRange,
  split: IndentedSplit<'_>,
) {
  let Some((_, parent, index)) = document.remove_subtree(original) else {
    return;
  };

  let code = document.next_available_id();
  let code_end = range.start + split.code_end_utf16;
  let code_node = Node::new(
    code,
    NodeKind::Block(BlockKind::CodeBlock {
      language: None,
      fenced: false,
    }),
    Some(SourceRange::new(range.start, code_end)),
  );
  assert!(document.insert_detached_node(parent, index, code_node));

  let text = document.next_available_id();
  let text_node = Node::new(
    text,
    NodeKind::Inline(InlineKind::Text {
      value: split.code,
    }),
    None,
  );
  assert!(document.insert_detached_node(code, 0, text_node));

  if split.rest.is_empty() {
    return;
  }

  let paragraph_start = range.start + split.rest_start_utf16;
  let paragraph = document.next_available_id();
  let paragraph_node = Node::new(
    paragraph,
    NodeKind::Block(BlockKind::Paragraph),
    Some(SourceRange::new(paragraph_start, range.end)),
  );
  assert!(document.insert_detached_node(parent, index + 1, paragraph_node));
  crate::parser::inline::append_inlines(document, paragraph, split.rest, paragraph_start);
}

fn utf16_slice(source: &str, range: SourceRange) -> Option<&str> {
  let start = byte_index_at_utf16(source, range.start)?;
  let end = byte_index_at_utf16(source, range.end)?;
  source.get(start..end)
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
  fn splits_only_a_leading_four_space_code_run() {
    let split = split_leading_indented_code("    alpha\n    beta\nbody\n    continuation")
      .expect("leading code");
    assert_eq!(split.code, "alpha\nbeta");
    assert_eq!(split.rest, "body\n    continuation");
    assert_eq!(split.code_end_utf16, 19);
    assert_eq!(split.rest_start_utf16, 19);
    assert!(split_leading_indented_code("body\n    continuation").is_none());
    assert!(split_leading_indented_code("   only-three").is_none());
  }

  #[test]
  fn blank_lines_between_indented_lines_stay_inside_the_code_block() {
    let split = split_leading_indented_code("    alpha\n\n    beta\nbody").expect("leading code");
    assert_eq!(split.code, "alpha\n\nbeta");
    assert_eq!(split.rest, "body");
  }
}
