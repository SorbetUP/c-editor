use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind, SourceRange};
use crate::syntax::inline::{emoji, extended};

#[derive(Debug)]
enum PieceKind {
  Text(String),
  Wrapper {
    kind: InlineKind,
    content: String,
    child_source: Option<(usize, usize)>,
  },
}

#[derive(Debug)]
struct Piece {
  kind: PieceKind,
  start: usize,
  end: usize,
}

pub(crate) fn apply(document: &mut Document) {
  let root = document.root;
  rewrite_container(document, root);
}

fn rewrite_container(document: &mut Document, parent: NodeId) {
  seed_atomic_payload_child(document, parent);
  if is_literal_container(document, parent) || is_atomic_editable_container(document, parent) {
    return;
  }

  let children = document
    .node(parent)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  let mut index = 0usize;
  while index < children.len() {
    if is_text(document, children[index]) {
      let start = index;
      while index < children.len() && is_text(document, children[index]) {
        index += 1;
      }
      rewrite_text_run(document, parent, &children[start..index]);
    } else {
      rewrite_container(document, children[index]);
      index += 1;
    }
  }
}

fn seed_atomic_payload_child(document: &mut Document, node_id: NodeId) {
  let payload = document.node(node_id).and_then(|node| {
    if !node.children.is_empty() {
      return None;
    }
    match &node.kind {
      NodeKind::Inline(InlineKind::Image { alt, .. }) => Some(alt.clone()),
      NodeKind::Inline(InlineKind::Escaped { value }) => Some(value.to_string()),
      NodeKind::Inline(InlineKind::SoftBreak | InlineKind::HardBreak) => Some("\n".to_string()),
      _ => None,
    }
  });
  let Some(payload) = payload else {
    return;
  };
  let child = document.next_available_id();
  assert!(document.insert_detached_node(
    node_id,
    0,
    Node::new(
      child,
      NodeKind::Inline(InlineKind::Text { value: payload }),
      None,
    ),
  ));
}

fn is_atomic_editable_container(document: &Document, node: NodeId) -> bool {
  document.node(node).is_some_and(|node| {
    matches!(
      node.kind,
      NodeKind::Inline(
        InlineKind::Escaped { .. }
          | InlineKind::Image { .. }
          | InlineKind::AutoLink { .. }
          | InlineKind::InlineHtml { .. }
          | InlineKind::InlineMath { .. }
          | InlineKind::Emoji { .. }
          | InlineKind::FootnoteReference { .. }
          | InlineKind::SoftBreak
          | InlineKind::HardBreak
      )
    )
  })
}

fn is_literal_container(document: &Document, node: NodeId) -> bool {
  document.node(node).is_some_and(|node| {
    matches!(
      node.kind,
      NodeKind::Block(
        BlockKind::CodeBlock { .. }
          | BlockKind::FrontMatter { .. }
          | BlockKind::HtmlBlock
          | BlockKind::MathBlock
          | BlockKind::ReferenceDefinition { .. }
          | BlockKind::Diagram { .. }
      )
    )
  })
}

fn is_text(document: &Document, node: NodeId) -> bool {
  document.node(node).is_some_and(|node| {
    matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. }))
  })
}

fn rewrite_text_run(document: &mut Document, parent: NodeId, run: &[NodeId]) {
  if run.is_empty() {
    return;
  }
  let value = run
    .iter()
    .filter_map(|id| match &document.node(*id)?.kind {
      NodeKind::Inline(InlineKind::Text { value }) => Some(value.as_str()),
      _ => None,
    })
    .collect::<String>();
  let pieces = parse_pieces(&value);
  if !pieces
    .iter()
    .any(|piece| !matches!(&piece.kind, PieceKind::Text(_)))
  {
    return;
  }

  let insert_at = document.child_index(parent, run[0]).unwrap_or(0);
  let source_start = run
    .first()
    .and_then(|id| document.node(*id))
    .and_then(|node| node.source)
    .map(|source| source.start);

  for id in run.iter().rev() {
    let removed = document.remove_leaf_node(*id);
    assert!(removed.is_some(), "inline extension pass only rewrites text leaves");
  }

  let mut index = insert_at;
  for piece in pieces {
    insert_piece(document, parent, index, piece, &value, source_start);
    index += 1;
  }
}

fn insert_piece(
  document: &mut Document,
  parent: NodeId,
  index: usize,
  piece: Piece,
  source: &str,
  source_start: Option<u32>,
) {
  let range = source_start.map(|base| {
    SourceRange::new(
      base + utf16_len(&source[..piece.start]),
      base + utf16_len(&source[..piece.end]),
    )
  });

  match piece.kind {
    PieceKind::Text(value) => {
      let id = document.next_available_id();
      assert!(document.insert_detached_node(
        parent,
        index,
        Node::new(id, NodeKind::Inline(InlineKind::Text { value }), range),
      ));
    }
    PieceKind::Wrapper {
      kind,
      content,
      child_source,
    } => {
      let wrapper = document.next_available_id();
      assert!(document.insert_detached_node(
        parent,
        index,
        Node::new(wrapper, NodeKind::Inline(kind), range),
      ));
      let child = document.next_available_id();
      let child_range = source_start.and_then(|base| {
        let (start, end) = child_source?;
        Some(SourceRange::new(
          base + utf16_len(&source[..piece.start + start]),
          base + utf16_len(&source[..piece.start + end]),
        ))
      });
      assert!(document.insert_detached_node(
        wrapper,
        0,
        Node::new(
          child,
          NodeKind::Inline(InlineKind::Text { value: content }),
          child_range,
        ),
      ));
    }
  }
}

fn parse_pieces(source: &str) -> Vec<Piece> {
  let mut pieces = Vec::new();
  let mut cursor = 0usize;
  let mut plain_start = 0usize;

  while cursor < source.len() {
    let remaining = &source[cursor..];
    let parsed = if let Some(parsed) = extended::parse_autolink(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::AutoLink {
            destination: parsed.value.to_string(),
          },
          content: parsed.value.to_string(),
          child_source: Some((1, parsed.consumed - 1)),
        },
      ))
    } else if let Some(parsed) = emoji::parse(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::Emoji {
            shortcode: parsed.shortcode,
            value: parsed.value.clone(),
          },
          content: parsed.value,
          child_source: None,
        },
      ))
    } else if let Some(parsed) = extended::parse_inline_math(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::InlineMath {
            source: parsed.value.to_string(),
          },
          content: parsed.value.to_string(),
          child_source: Some((1, parsed.consumed - 1)),
        },
      ))
    } else if let Some(parsed) = extended::parse_inline_html(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::InlineHtml {
            raw: parsed.value.to_string(),
          },
          content: parsed.value.to_string(),
          child_source: Some((0, parsed.consumed)),
        },
      ))
    } else if let Some(parsed) = extended::parse_superscript(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::Superscript,
          content: parsed.content.to_string(),
          child_source: Some((1, parsed.consumed - 1)),
        },
      ))
    } else if let Some(parsed) = extended::parse_subscript(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::Subscript,
          content: parsed.content.to_string(),
          child_source: Some((1, parsed.consumed - 1)),
        },
      ))
    } else if let Some(parsed) = extended::parse_footnote_reference(remaining) {
      Some((
        parsed.consumed,
        PieceKind::Wrapper {
          kind: InlineKind::FootnoteReference {
            label: parsed.value.to_string(),
          },
          content: parsed.value.to_string(),
          child_source: Some((2, parsed.consumed - 1)),
        },
      ))
    } else {
      None
    };

    if let Some((consumed, kind)) = parsed {
      if plain_start < cursor {
        pieces.push(Piece {
          kind: PieceKind::Text(source[plain_start..cursor].to_string()),
          start: plain_start,
          end: cursor,
        });
      }
      pieces.push(Piece {
        kind,
        start: cursor,
        end: cursor + consumed,
      });
      cursor += consumed;
      plain_start = cursor;
      continue;
    }

    let character = remaining
      .chars()
      .next()
      .expect("cursor is inside the source string");
    cursor += character.len_utf8();
  }

  if plain_start < source.len() {
    pieces.push(Piece {
      kind: PieceKind::Text(source[plain_start..].to_string()),
      start: plain_start,
      end: source.len(),
    });
  }
  pieces
}

fn utf16_len(value: &str) -> u32 {
  value.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::serializer::to_markdown;

  #[test]
  fn rewrites_only_tauri_extension_tokens() {
    let mut document = crate::parser::parse_markdown(
      "a <https://example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1] z",
    );
    apply(&mut document);
    assert_eq!(
      to_markdown(&document),
      "a <https://example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1] z"
    );
  }

  #[test]
  fn every_visible_atomic_node_exposes_real_editable_text() {
    let mut document = crate::parser::parse_markdown(
      "\\* <https://example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ [^source-1] ![alt](image.png)\nnext",
    );
    apply(&mut document);

    for node in document.nodes.values().filter(|node| {
      matches!(
        node.kind,
        NodeKind::Inline(
          InlineKind::Escaped { .. }
            | InlineKind::AutoLink { .. }
            | InlineKind::Emoji { .. }
            | InlineKind::InlineHtml { .. }
            | InlineKind::InlineMath { .. }
            | InlineKind::FootnoteReference { .. }
            | InlineKind::Image { .. }
            | InlineKind::SoftBreak
            | InlineKind::HardBreak
        )
      )
    }) {
      assert_eq!(node.children.len(), 1, "atomic node {:?} must be editable", node.kind);
      assert!(matches!(
        document.node(node.children[0]).map(|child| &child.kind),
        Some(NodeKind::Inline(InlineKind::Text { .. }))
      ));
    }
  }

  #[test]
  fn leaves_invalid_markers_as_plain_text() {
    let markdown = "price $5 and <not a link> and ^open";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    assert_eq!(to_markdown(&document), markdown);
  }

  #[test]
  fn never_interprets_literal_block_contents() {
    let markdown = "---\nraw: '$frontmatter$ :grinning: <kbd>x</kbd>'\n---\n\n```md\n$code$ :grinning: <kbd>x</kbd>\n```";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    assert_eq!(to_markdown(&document), markdown);
    assert!(!document.nodes.values().any(|node| matches!(
      node.kind,
      NodeKind::Inline(
        InlineKind::InlineMath { .. }
          | InlineKind::Emoji { .. }
          | InlineKind::InlineHtml { .. }
      )
    )));
  }
}