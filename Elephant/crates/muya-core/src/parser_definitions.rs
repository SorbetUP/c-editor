use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind, SourceRange};

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedDefinition {
  label: String,
  payload: String,
  start: u32,
  end: u32,
}

#[derive(Clone, Debug)]
struct SourceLine<'a> {
  text: &'a str,
  start: u32,
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
    let Some(raw) = utf16_slice(markdown, range) else {
      continue;
    };
    let Some(definitions) = parse_definition_sequence(raw) else {
      continue;
    };
    replace_paragraph(document, candidate, range.start, definitions);
  }
}

fn parse_definition_sequence(raw: &str) -> Option<Vec<ParsedDefinition>> {
  let lines = source_lines(raw);
  if lines.is_empty() {
    return None;
  }

  let mut definitions = Vec::new();
  let mut index = 0usize;
  while index < lines.len() {
    let line = &lines[index];
    let Some((label, payload, _)) = parse_definition_head(line.text) else {
      return None;
    };

    let mut payload = payload.to_string();
    let mut end = line.end;
    index += 1;

    if payload.is_empty() {
      let continuation = lines.get(index)?;
      let trimmed = continuation.text.trim_start_matches(' ');
      let indentation = continuation.text.len().saturating_sub(trimmed.len());
      if indentation == 0 || parse_definition_head(continuation.text).is_some() || trimmed.is_empty() {
        return None;
      }
      payload = trimmed.to_string();
      end = continuation.end;
      index += 1;
    }

    if let Some(title_line) = lines.get(index) {
      let trimmed = title_line.text.trim_start_matches(' ');
      let indentation = title_line.text.len().saturating_sub(trimmed.len());
      if indentation > 0 && is_title_only(trimmed) {
        payload.push(' ');
        payload.push_str(trimmed);
        end = title_line.end;
        index += 1;
      }
    }

    if !valid_reference_payload(&payload) {
      return None;
    }

    definitions.push(ParsedDefinition {
      label: label.to_string(),
      payload,
      start: line.start,
      end,
    });
  }

  (!definitions.is_empty()).then_some(definitions)
}

fn parse_definition_head(line: &str) -> Option<(&str, &str, usize)> {
  let leading = line.chars().take_while(|character| *character == ' ').count();
  if leading > 3 {
    return None;
  }
  let source = &line[leading..];
  if !source.starts_with('[') || source.starts_with("[^") {
    return None;
  }

  let close = find_unescaped_closing_bracket(source)?;
  if source.get(close + 1..close + 2)? != ":" {
    return None;
  }
  let label = &source[1..close];
  if label.trim().is_empty() {
    return None;
  }

  let after_colon = leading + close + 2;
  let tail = &line[after_colon..];
  let payload = tail.trim_start_matches(' ');
  let payload_byte = line.len().saturating_sub(payload.len());
  Some((label, payload, payload_byte))
}

fn find_unescaped_closing_bracket(source: &str) -> Option<usize> {
  let mut escaped = false;
  for (index, character) in source.char_indices().skip(1) {
    if escaped {
      escaped = false;
      continue;
    }
    if character == '\\' {
      escaped = true;
      continue;
    }
    if character == ']' {
      return Some(index);
    }
    if character == '[' {
      return None;
    }
  }
  None
}

fn valid_reference_payload(payload: &str) -> bool {
  let payload = payload.trim();
  if payload.is_empty() {
    return false;
  }
  if let Some(rest) = payload.strip_prefix('<') {
    return rest.find('>').is_some_and(|end| end > 0);
  }
  payload
    .split_whitespace()
    .next()
    .is_some_and(|destination| !destination.is_empty())
}

fn is_title_only(value: &str) -> bool {
  let value = value.trim();
  (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
    || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
    || (value.starts_with('(') && value.ends_with(')') && value.len() >= 2)
}

fn replace_paragraph(
  document: &mut Document,
  original: NodeId,
  source_start: u32,
  definitions: Vec<ParsedDefinition>,
) {
  let Some((_, parent, index)) = document.remove_subtree(original) else {
    return;
  };

  for (offset, definition) in definitions.into_iter().enumerate() {
    let block = document.next_available_id();
    assert!(document.insert_detached_node(
      parent,
      index + offset,
      Node::new(
        block,
        NodeKind::Block(BlockKind::ReferenceDefinition {
          label: definition.label,
        }),
        Some(SourceRange::new(
          source_start + definition.start,
          source_start + definition.end,
        )),
      ),
    ));
    let text = document.next_available_id();
    assert!(document.insert_detached_node(
      block,
      0,
      Node::new(
        text,
        NodeKind::Inline(InlineKind::Text {
          value: definition.payload,
        }),
        None,
      ),
    ));
  }
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
      let start = offset;
      let end = start + utf16_len(text);
      offset += utf16_len(segment);
      SourceLine { text, start, end }
    })
    .collect()
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
  use crate::serializer::to_markdown;

  #[test]
  fn splits_multiple_definitions_from_one_paragraph() {
    let markdown = "[one]: https://one.example\n[two]: <https://two.example> \"Two\"";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);
    let blocks = document.children(document.root).collect::<Vec<_>>();
    assert_eq!(blocks.len(), 2);
    assert!(matches!(
      &blocks[0].kind,
      NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "one"
    ));
    assert!(matches!(
      &blocks[1].kind,
      NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "two"
    ));
  }

  #[test]
  fn accepts_destination_and_title_continuation_lines() {
    let raw = "[id]:\n  https://example.com\n  \"Title\"";
    let definitions = parse_definition_sequence(raw).expect("definition sequence");
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].payload, "https://example.com \"Title\"");
  }

  #[test]
  fn non_definition_paragraph_is_not_consumed() {
    assert!(parse_definition_sequence("[link](target)\nplain").is_none());
    assert!(parse_definition_sequence("[^footnote]: body").is_none());
  }

  #[test]
  fn public_pipeline_keeps_reference_nodes_independent() {
    let markdown = "[one]: https://one.example\n[two]: https://two.example";
    let document = crate::parse_markdown(markdown);
    assert_eq!(document.children(document.root).count(), 2);
    assert_eq!(
      to_markdown(&document),
      "[one]: https://one.example\n\n[two]: https://two.example"
    );
  }
}
