use crate::model::{BlockKind, Document, NodeId, NodeKind, SourceRange};

pub(crate) fn apply(document: &mut Document, markdown: &str) {
  let root = document.root;
  let blocks = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for block in blocks {
    let kind = document.node(block).map(|node| node.kind.clone());
    match kind {
      Some(NodeKind::Block(BlockKind::CodeBlock { language, .. }))
        if language
          .as_deref()
          .is_some_and(|language| language.trim().eq_ignore_ascii_case("math")) =>
      {
        if let Some(node) = document.node_mut(block) {
          node.kind = NodeKind::Block(BlockKind::MathBlock);
        }
      }
      Some(NodeKind::Block(BlockKind::Paragraph)) => {
        let Some(range) = document.node(block).and_then(|node| node.source) else {
          continue;
        };
        let Some(raw) = utf16_slice(markdown, range) else {
          continue;
        };

        if let Some(body) = display_math_body(raw) {
          replace_with_literal(document, block, BlockKind::MathBlock, body, range.start + 3);
          continue;
        }

        if let Some((label, body, body_offset)) = footnote_definition(raw) {
          replace_with_inlines(
            document,
            block,
            BlockKind::FootnoteDefinition {
              label: label.to_string(),
            },
            body,
            range.start + utf16_len(&raw[..body_offset]),
          );
          continue;
        }

        if let Some((label, target, target_offset)) = reference_definition(raw) {
          replace_with_literal(
            document,
            block,
            BlockKind::ReferenceDefinition {
              label: label.to_string(),
            },
            target,
            range.start + utf16_len(&raw[..target_offset]),
          );
          continue;
        }

        if looks_like_html_block(raw) {
          replace_with_literal(document, block, BlockKind::HtmlBlock, raw, range.start);
        }
      }
      _ => {}
    }
  }
}

fn replace_with_literal(
  document: &mut Document,
  block: NodeId,
  kind: BlockKind,
  value: &str,
  source_start: u32,
) {
  clear_children(document, block);
  if let Some(node) = document.node_mut(block) {
    node.kind = NodeKind::Block(kind);
  }
  crate::parser::inline::append_literal(document, block, value, source_start);
}

fn replace_with_inlines(
  document: &mut Document,
  block: NodeId,
  kind: BlockKind,
  value: &str,
  source_start: u32,
) {
  clear_children(document, block);
  if let Some(node) = document.node_mut(block) {
    node.kind = NodeKind::Block(kind);
  }
  crate::parser::inline::append_inlines(document, block, value, source_start);
}

fn clear_children(document: &mut Document, block: NodeId) {
  let children = document
    .node(block)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  for child in children.into_iter().rev() {
    let removed = document.remove_subtree(child);
    assert!(removed.is_some(), "special block children must be detachable");
  }
}

fn display_math_body(raw: &str) -> Option<&str> {
  let body = raw.strip_prefix("$$\n")?.strip_suffix("\n$$")?;
  (!body.is_empty()).then_some(body)
}

fn footnote_definition(raw: &str) -> Option<(&str, &str, usize)> {
  let rest = raw.strip_prefix("[^")?;
  let label_end = rest.find("]: ").or_else(|| rest.find("]:"))?;
  let label = &rest[..label_end];
  if label.is_empty()
    || label
      .chars()
      .any(|character| character.is_whitespace() || matches!(character, '^' | '[' | ']'))
  {
    return None;
  }

  let marker_end = 2 + label_end + 2;
  let body_raw = &raw[marker_end..];
  let trimmed = body_raw.trim_start_matches(|character| matches!(character, ' ' | '\t'));
  if trimmed.is_empty() {
    return None;
  }
  let body_offset = raw.len() - trimmed.len();
  Some((label, trimmed, body_offset))
}

fn reference_definition(raw: &str) -> Option<(&str, &str, usize)> {
  let rest = raw.strip_prefix('[')?;
  if rest.starts_with('^') {
    return None;
  }
  let label_end = rest.find("]: ").or_else(|| rest.find("]:"))?;
  let label = &rest[..label_end];
  if label.is_empty() || label.chars().any(|character| matches!(character, '[' | ']')) {
    return None;
  }

  let marker_end = 1 + label_end + 2;
  let target_raw = &raw[marker_end..];
  let target = target_raw.trim_start_matches(|character| matches!(character, ' ' | '\t' | '\n'));
  if target.is_empty() || target.chars().next().is_some_and(char::is_whitespace) {
    return None;
  }
  let target_offset = raw.len() - target.len();
  Some((label, target, target_offset))
}

fn looks_like_html_block(raw: &str) -> bool {
  let source = raw.trim_start_matches(|character| matches!(character, ' ' | '\t'));
  if source.starts_with("<!--")
    || source.starts_with("<?")
    || source.starts_with("<![CDATA[")
    || (source.starts_with("<!") && source.chars().nth(2).is_some_and(|c| c.is_ascii_uppercase()))
  {
    return true;
  }

  let Some(tag) = html_tag_name(source) else {
    return false;
  };
  const BLOCK_TAGS: &[&str] = &[
    "address", "article", "aside", "base", "basefont", "blockquote", "body", "caption",
    "center", "col", "colgroup", "dd", "details", "dialog", "dir", "div", "dl", "dt",
    "fieldset", "figcaption", "figure", "footer", "form", "frame", "frameset", "h1", "h2",
    "h3", "h4", "h5", "h6", "head", "header", "hr", "html", "iframe", "legend", "li",
    "link", "main", "menu", "menuitem", "meta", "nav", "noframes", "ol", "optgroup",
    "option", "p", "param", "pre", "script", "section", "source", "style", "summary", "table",
    "tbody", "td", "tfoot", "th", "thead", "title", "tr", "track", "ul",
  ];
  BLOCK_TAGS.iter().any(|candidate| *candidate == tag)
}

fn html_tag_name(source: &str) -> Option<&str> {
  let source = source.strip_prefix('<')?;
  let source = source.strip_prefix('/').unwrap_or(source);
  let end = source
    .char_indices()
    .take_while(|(_, character)| character.is_ascii_alphanumeric() || *character == '-')
    .last()
    .map(|(index, character)| index + character.len_utf8())?;
  let tag = &source[..end];
  (!tag.is_empty()).then_some(tag)
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
  fn activates_display_math_reference_footnote_and_html_blocks() {
    let markdown = "$$\nx + y\n$$\n\n[^src]: cited **source**\n\n[openai]: https://openai.com \"Home\"\n\n<div>\nblock\n</div>";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);

    assert!(document.nodes.values().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::MathBlock)
    )));
    assert!(document.nodes.values().any(|node| matches!(
      &node.kind,
      NodeKind::Block(BlockKind::FootnoteDefinition { label }) if label == "src"
    )));
    assert!(document.nodes.values().any(|node| matches!(
      &node.kind,
      NodeKind::Block(BlockKind::ReferenceDefinition { label }) if label == "openai"
    )));
    assert!(document.nodes.values().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::HtmlBlock)
    )));
  }

  #[test]
  fn converts_gitlab_math_fences_without_touching_other_code() {
    let markdown = "```math\nx^2\n```\n\n```rust\n$x$\n```";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);
    let blocks = document.children(document.root).collect::<Vec<_>>();
    assert!(matches!(blocks[0].kind, NodeKind::Block(BlockKind::MathBlock)));
    assert!(matches!(
      blocks[1].kind,
      NodeKind::Block(BlockKind::CodeBlock { .. })
    ));
  }

  #[test]
  fn utf16_source_slicing_handles_non_bmp_text_before_special_blocks() {
    let source = "😀\n\n$$\nx\n$$";
    let range = SourceRange::new(4, 11);
    assert_eq!(utf16_slice(source, range), Some("$$\nx\n$$"));
  }

  #[test]
  fn html_detection_does_not_promote_inline_only_tags() {
    assert!(!looks_like_html_block("<kbd>Ctrl</kbd>"));
    assert!(looks_like_html_block("<div>hello</div>"));
    assert!(looks_like_html_block("<!-- comment -->"));
  }

  #[test]
  fn helpers_parse_tauri_reference_and_footnote_shapes() {
    assert_eq!(
      reference_definition("[id]: <https://example.com> \"title\"")
        .map(|(label, target, _)| (label, target)),
      Some(("id", "<https://example.com> \"title\""))
    );
    assert_eq!(
      footnote_definition("[^id]: content").map(|(label, body, _)| (label, body)),
      Some(("id", "content"))
    );
  }
}