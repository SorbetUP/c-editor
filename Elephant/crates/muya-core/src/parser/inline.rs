use crate::model::{Document, InlineKind, InlineMarkKind, NodeId, NodeKind, SourceRange};
use crate::selection::{Selection, SelectionPoint};
use crate::syntax::inline::{code_span, emphasis, escape, line_break, link, text};

use super::crossing_marks;

pub fn append_inlines(document: &mut Document, parent: NodeId, source: &str, base_utf16: u32) {
  if try_append_crossing_marks(document, parent, source, base_utf16) {
    return;
  }
  append_inlines_standard(document, parent, source, base_utf16);
}

fn try_append_crossing_marks(
  document: &mut Document,
  parent: NodeId,
  source: &str,
  base_utf16: u32,
) -> bool {
  let Some(crossing) = crossing_marks::detect(source) else {
    return false;
  };

  let mut candidate = document.clone();
  append_inlines_standard(&mut candidate, parent, &crossing.base, base_utf16);
  let mut flat_ranges = Vec::with_capacity(crossing.marks.len());
  for mark in &crossing.marks {
    let Some(anchor) = source_point(
      &candidate,
      parent,
      base_utf16.saturating_add(mark.start_utf16),
    ) else {
      return false;
    };
    let Some(focus) = source_point(
      &candidate,
      parent,
      base_utf16.saturating_add(mark.end_utf16),
    ) else {
      return false;
    };
    let Some(start) = flat_offset(&candidate, parent, anchor) else {
      return false;
    };
    let Some(end) = flat_offset(&candidate, parent, focus) else {
      return false;
    };
    flat_ranges.push((start, end));
  }

  let revision = candidate.revision;
  for (mark, (start, end)) in crossing.marks.iter().zip(flat_ranges) {
    let Some(anchor) = point_at_flat_offset(&candidate, parent, start) else {
      return false;
    };
    let Some(focus) = point_at_flat_offset(&candidate, parent, end) else {
      return false;
    };
    let selection = Selection { anchor, focus };
    let command = match mark.mark {
      InlineMarkKind::Strong => crate::edit::MarkCommand::ToggleStrong,
      InlineMarkKind::Emphasis => crate::edit::MarkCommand::ToggleEmphasis,
      InlineMarkKind::Strike => crate::edit::MarkCommand::ToggleStrike,
    };
    let Ok(transaction) = command.build(&candidate, selection) else {
      return false;
    };
    if transaction.apply(&mut candidate).is_err() {
      return false;
    }
    candidate.revision = revision;
  }

  *document = candidate;
  true
}

fn source_point(document: &Document, root: NodeId, target: u32) -> Option<SelectionPoint> {
  let mut stack = vec![root];
  while let Some(current) = stack.pop() {
    let node = document.node(current)?;
    if let NodeKind::Inline(InlineKind::Text { value }) = &node.kind {
      let range = node.source?;
      if range.start <= target && target <= range.end {
        let offset = target.saturating_sub(range.start);
        if offset <= value.encode_utf16().count() as u32 {
          return Some(SelectionPoint {
            node: current,
            offset_utf16: offset,
          });
        }
      }
    }
    stack.extend(node.children.iter().rev().copied());
  }
  None
}

fn flat_offset(document: &Document, root: NodeId, point: SelectionPoint) -> Option<u32> {
  let mut offset = 0u32;
  for node in text_nodes(document, root)? {
    let NodeKind::Inline(InlineKind::Text { value }) = &node.kind else {
      continue;
    };
    let length = value.encode_utf16().count() as u32;
    if node.id == point.node {
      return (point.offset_utf16 <= length).then_some(offset + point.offset_utf16);
    }
    offset = offset.saturating_add(length);
  }
  None
}

fn point_at_flat_offset(document: &Document, root: NodeId, target: u32) -> Option<SelectionPoint> {
  let mut offset = 0u32;
  let nodes = text_nodes(document, root)?;
  for node in &nodes {
    let NodeKind::Inline(InlineKind::Text { value }) = &node.kind else {
      continue;
    };
    let length = value.encode_utf16().count() as u32;
    if target <= offset.saturating_add(length) {
      return Some(SelectionPoint {
        node: node.id,
        offset_utf16: target.saturating_sub(offset),
      });
    }
    offset = offset.saturating_add(length);
  }
  let node = nodes.last()?;
  let NodeKind::Inline(InlineKind::Text { value }) = &node.kind else {
    return None;
  };
  (target == offset).then_some(SelectionPoint {
    node: node.id,
    offset_utf16: value.encode_utf16().count() as u32,
  })
}

fn text_nodes(document: &Document, root: NodeId) -> Option<Vec<&crate::model::Node>> {
  fn visit<'a>(
    document: &'a Document,
    node_id: NodeId,
    output: &mut Vec<&'a crate::model::Node>,
  ) -> Option<()> {
    let node = document.node(node_id)?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
      output.push(node);
    }
    for child in &node.children {
      visit(document, *child, output)?;
    }
    Some(())
  }

  let mut output = Vec::new();
  visit(document, root, &mut output)?;
  Some(output)
}

fn append_inlines_standard(document: &mut Document, parent: NodeId, source: &str, base_utf16: u32) {
  if source.is_empty() {
    append_empty_text(document, parent, base_utf16);
    return;
  }

  let mut byte_offset = 0usize;
  let mut utf16_offset = base_utf16;

  while byte_offset < source.len() {
    let remaining = &source[byte_offset..];

    if let Some(parsed) = escape::parse(remaining) {
      append_leaf(
        document,
        parent,
        InlineKind::Escaped {
          value: parsed.value,
        },
        utf16_offset,
        remaining,
        parsed.consumed,
      );
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    if let Some(parsed) = code_span::parse(remaining) {
      append_leaf(
        document,
        parent,
        InlineKind::CodeSpan {
          code: parsed.code.to_string(),
        },
        utf16_offset,
        remaining,
        parsed.consumed,
      );
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    if let Some(parsed) = link::parse_image(remaining) {
      append_leaf(
        document,
        parent,
        InlineKind::Image {
          source: parsed.destination.to_string(),
          title: parsed.title.map(str::to_string),
          alt: parsed.label.to_string(),
        },
        utf16_offset,
        remaining,
        parsed.consumed,
      );
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    if let Some(parsed) = link::parse_link(remaining) {
      let consumed_utf16 = utf16_len(&remaining[..parsed.consumed]);
      let node = document.allocate(
        NodeKind::Inline(InlineKind::Link {
          destination: parsed.destination.to_string(),
          title: parsed.title.map(str::to_string),
        }),
        Some(SourceRange::new(
          utf16_offset,
          utf16_offset + consumed_utf16,
        )),
      );
      document.append_child(parent, node);
      append_inlines(document, node, parsed.label, utf16_offset + 1);
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    if let Some(parsed) = emphasis::parse(remaining) {
      let consumed_utf16 = utf16_len(&remaining[..parsed.consumed]);
      let kind = match parsed.kind {
        emphasis::EmphasisKind::Emphasis => InlineKind::Emphasis,
        emphasis::EmphasisKind::Strong => InlineKind::Strong,
        emphasis::EmphasisKind::Strike => InlineKind::Strike,
      };
      let node = document.allocate(
        NodeKind::Inline(kind),
        Some(SourceRange::new(
          utf16_offset,
          utf16_offset + consumed_utf16,
        )),
      );
      document.append_child(parent, node);
      let delimiter_bytes = (parsed.consumed - parsed.content.len()) / 2;
      append_inlines(
        document,
        node,
        parsed.content,
        utf16_offset + delimiter_bytes as u32,
      );
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    if let Some(parsed) = line_break::parse(remaining) {
      let kind = match parsed.kind {
        line_break::BreakKind::Soft => InlineKind::SoftBreak,
        line_break::BreakKind::Hard => InlineKind::HardBreak,
      };
      append_leaf(
        document,
        parent,
        kind,
        utf16_offset,
        remaining,
        parsed.consumed,
      );
      advance(
        remaining,
        parsed.consumed,
        &mut byte_offset,
        &mut utf16_offset,
      );
      continue;
    }

    let plain = text::take(remaining);
    append_leaf(
      document,
      parent,
      InlineKind::Text {
        value: plain.to_string(),
      },
      utf16_offset,
      remaining,
      plain.len(),
    );
    advance(remaining, plain.len(), &mut byte_offset, &mut utf16_offset);
  }
}

pub fn append_literal(document: &mut Document, parent: NodeId, source: &str, base_utf16: u32) {
  if source.is_empty() {
    append_empty_text(document, parent, base_utf16);
    return;
  }
  let node = document.allocate(
    NodeKind::Inline(InlineKind::Text {
      value: source.to_string(),
    }),
    Some(SourceRange::new(base_utf16, base_utf16 + utf16_len(source))),
  );
  document.append_child(parent, node);
}

fn append_empty_text(document: &mut Document, parent: NodeId, offset: u32) {
  let node = document.allocate(
    NodeKind::Inline(InlineKind::Text {
      value: String::new(),
    }),
    Some(SourceRange::new(offset, offset)),
  );
  document.append_child(parent, node);
}

fn append_leaf(
  document: &mut Document,
  parent: NodeId,
  kind: InlineKind,
  start: u32,
  source: &str,
  consumed: usize,
) {
  let node = document.allocate(
    NodeKind::Inline(kind),
    Some(SourceRange::new(
      start,
      start + utf16_len(&source[..consumed]),
    )),
  );
  document.append_child(parent, node);
}

fn advance(source: &str, consumed: usize, byte_offset: &mut usize, utf16_offset: &mut u32) {
  *byte_offset += consumed;
  *utf16_offset += utf16_len(&source[..consumed]);
}

fn utf16_len(value: &str) -> u32 {
  value.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{BlockKind, MarkFragmentEdge};
  use crate::serializer::to_markdown;

  #[test]
  fn builds_nested_inline_nodes() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, paragraph);
    append_inlines(
      &mut document,
      paragraph,
      "A **bold** [link](https://example.com) and `code`.",
      0,
    );

    let kinds = document
      .children(paragraph)
      .map(|node| &node.kind)
      .collect::<Vec<_>>();
    assert!(kinds
      .iter()
      .any(|kind| matches!(kind, NodeKind::Inline(InlineKind::Strong))));
    assert!(kinds
      .iter()
      .any(|kind| matches!(kind, NodeKind::Inline(InlineKind::Link { .. }))));
    assert!(kinds
      .iter()
      .any(|kind| matches!(kind, NodeKind::Inline(InlineKind::CodeSpan { .. }))));
  }

  #[test]
  fn preserves_utf16_source_offsets() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    append_inlines(&mut document, paragraph, "😀 **x**", 10);
    let strong = document
      .children(paragraph)
      .find(|node| matches!(&node.kind, NodeKind::Inline(InlineKind::Strong)))
      .unwrap();
    assert_eq!(strong.source, Some(SourceRange::new(13, 18)));
  }

  #[test]
  fn creates_an_addressable_empty_text_node() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    append_inlines(&mut document, paragraph, "", 7);

    let children = document.children(paragraph).collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
    assert!(matches!(
      &children[0].kind,
      NodeKind::Inline(InlineKind::Text { value }) if value.is_empty()
    ));
    assert_eq!(children[0].source, Some(SourceRange::new(7, 7)));
  }

  #[test]
  fn creates_an_addressable_empty_literal_node() {
    let mut document = Document::new();
    let code = document.allocate(
      NodeKind::Block(BlockKind::CodeBlock {
        language: None,
        fenced: true,
      }),
      None,
    );
    append_literal(&mut document, code, "", 4);

    let text = document.children(code).next().unwrap();
    assert!(matches!(
      &text.kind,
      NodeKind::Inline(InlineKind::Text { value }) if value.is_empty()
    ));
    assert_eq!(text.source, Some(SourceRange::new(4, 4)));
  }

  #[test]
  fn reconstructs_crossing_strike_fragments_after_reparse() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, paragraph);
    let markdown = "**al~~pha** beta *gam~~ma*";
    append_inlines(&mut document, paragraph, markdown, 0);

    assert_eq!(to_markdown(&document), markdown);
    let edges = document
      .nodes
      .values()
      .filter_map(|node| match &node.kind {
        NodeKind::Inline(InlineKind::MarkFragment { edge, .. }) => Some(*edge),
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(
      edges,
      vec![
        MarkFragmentEdge::Start,
        MarkFragmentEdge::Middle,
        MarkFragmentEdge::End
      ]
    );
  }

  #[test]
  fn reconstructs_crossing_emphasis_fragments_after_reparse() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, paragraph);
    let markdown = "al*pha **be*ta** gamma";
    append_inlines(&mut document, paragraph, markdown, 0);

    assert_eq!(to_markdown(&document), markdown);
    assert_eq!(
      document
        .nodes
        .values()
        .filter(|node| matches!(node.kind, NodeKind::Inline(InlineKind::MarkFragment { .. })))
        .count(),
      2
    );
  }

  #[test]
  fn reconstructs_multiple_overlapping_crossing_groups_after_reparse() {
    let mut document = Document::new();
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
    document.append_child(document.root, paragraph);
    let markdown = "al*p~~ha **be*t~~a** gamma";
    append_inlines(&mut document, paragraph, markdown, 0);

    assert_eq!(to_markdown(&document), markdown);
    for (mark, expected) in [
      (
        InlineMarkKind::Emphasis,
        vec![MarkFragmentEdge::Start, MarkFragmentEdge::End],
      ),
      (
        InlineMarkKind::Strike,
        vec![
          MarkFragmentEdge::Start,
          MarkFragmentEdge::Middle,
          MarkFragmentEdge::End,
        ],
      ),
    ] {
      let edges = document
        .nodes
        .values()
        .filter_map(|node| match &node.kind {
          NodeKind::Inline(InlineKind::MarkFragment {
            mark: candidate,
            edge,
            ..
          }) if *candidate == mark => Some(*edge),
          _ => None,
        })
        .collect::<Vec<_>>();
      assert_eq!(edges, expected);
    }
  }
}
