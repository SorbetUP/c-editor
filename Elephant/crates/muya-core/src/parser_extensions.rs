use std::collections::BTreeMap;

use crate::model::{
  BlockKind, Document, InlineKind, InlineSyntax, Node, NodeId, NodeKind, ReferenceStyle, SourceRange,
};
use crate::syntax::inline::{emoji, extended};

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReferenceDefinition {
  destination: String,
  title: Option<String>,
}

#[derive(Debug)]
enum PieceKind {
  Text(String),
  Wrapper {
    kind: InlineKind,
    syntax: Option<InlineSyntax>,
    content: String,
    child_source: Option<(usize, usize)>,
    parse_content: bool,
  },
}

impl PieceKind {
  fn wrapper(
    kind: InlineKind,
    content: String,
    child_source: Option<(usize, usize)>,
    parse_content: bool,
  ) -> Self {
    Self::Wrapper {
      kind,
      syntax: None,
      content,
      child_source,
      parse_content,
    }
  }

  fn wrapper_with_syntax(
    kind: InlineKind,
    syntax: InlineSyntax,
    content: String,
    child_source: Option<(usize, usize)>,
    parse_content: bool,
  ) -> Self {
    Self::Wrapper {
      kind,
      syntax: Some(syntax),
      content,
      child_source,
      parse_content,
    }
  }
}

#[derive(Debug)]
struct Piece {
  kind: PieceKind,
  start: usize,
  end: usize,
}

pub(crate) fn apply(document: &mut Document) {
  let definitions = collect_reference_definitions(document);
  let root = document.root;
  rewrite_container(document, root, &definitions, false);
}

fn rewrite_container(
  document: &mut Document,
  parent: NodeId,
  definitions: &BTreeMap<String, ReferenceDefinition>,
  inside_link: bool,
) {
  seed_atomic_payload_child(document, parent);
  if is_literal_container(document, parent) || is_atomic_editable_container(document, parent) {
    return;
  }

  let child_inside_link = inside_link || is_link_container(document, parent);
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
      rewrite_text_run(
        document,
        parent,
        &children[start..index],
        definitions,
        child_inside_link,
      );
    } else {
      rewrite_container(document, children[index], definitions, child_inside_link);
      index += 1;
    }
  }
}

fn seed_atomic_payload_child(document: &mut Document, node_id: NodeId) {
  let payload = document.node(node_id).and_then(|node| {
    if !node.children.is_empty() {
      return None;
    }
    match (&node.kind, &node.inline_syntax) {
      (NodeKind::Inline(InlineKind::Image { alt, .. }), _) => Some(alt.clone()),
      (
        NodeKind::Inline(InlineKind::Link { .. }),
        Some(InlineSyntax::BareAutoLink { text }),
      ) => Some(text.clone()),
      (NodeKind::Inline(InlineKind::Escaped { value }), _) => Some(value.to_string()),
      (NodeKind::Inline(InlineKind::SoftBreak | InlineKind::HardBreak), _) => {
        Some("\n".to_string())
      }
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

fn is_atomic_editable_container(document: &Document, node_id: NodeId) -> bool {
  document.node(node_id).is_some_and(|node| {
    let bare_link = matches!(
      (&node.kind, &node.inline_syntax),
      (
        NodeKind::Inline(InlineKind::Link { .. }),
        Some(InlineSyntax::BareAutoLink { .. })
      )
    );
    bare_link
      || matches!(
        &node.kind,
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

fn is_link_container(document: &Document, node: NodeId) -> bool {
  document.node(node).is_some_and(|node| {
    matches!(&node.kind, NodeKind::Inline(InlineKind::Link { .. }))
  })
}

fn is_literal_container(document: &Document, node: NodeId) -> bool {
  document.node(node).is_some_and(|node| {
    matches!(
      &node.kind,
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
    matches!(&node.kind, NodeKind::Inline(InlineKind::Text { .. }))
  })
}

fn rewrite_text_run(
  document: &mut Document,
  parent: NodeId,
  run: &[NodeId],
  definitions: &BTreeMap<String, ReferenceDefinition>,
  inside_link: bool,
) {
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
  let pieces = parse_pieces(&value, definitions, !inside_link);
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
    insert_piece(
      document,
      parent,
      index,
      piece,
      &value,
      source_start,
      definitions,
      inside_link,
    );
    index += 1;
  }
}

#[allow(clippy::too_many_arguments)]
fn insert_piece(
  document: &mut Document,
  parent: NodeId,
  index: usize,
  piece: Piece,
  source: &str,
  source_start: Option<u32>,
  definitions: &BTreeMap<String, ReferenceDefinition>,
  inside_link: bool,
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
      syntax,
      content,
      child_source,
      parse_content,
    } => {
      let wrapper = document.next_available_id();
      let mut wrapper_node = Node::new(wrapper, NodeKind::Inline(kind), range);
      wrapper_node.inline_syntax = syntax;
      assert!(document.insert_detached_node(parent, index, wrapper_node));
      let child_start = source_start.and_then(|base| {
        let (start, _) = child_source?;
        Some(base + utf16_len(&source[..piece.start + start]))
      });
      if parse_content {
        crate::parser::inline::append_inlines(
          document,
          wrapper,
          &content,
          child_start.unwrap_or_default(),
        );
        rewrite_container(document, wrapper, definitions, inside_link);
      } else {
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
}

fn parse_pieces(
  source: &str,
  definitions: &BTreeMap<String, ReferenceDefinition>,
  allow_bare_autolink: bool,
) -> Vec<Piece> {
  let mut pieces = Vec::new();
  let mut cursor = 0usize;
  let mut plain_start = 0usize;

  while cursor < source.len() {
    let remaining = &source[cursor..];
    let parsed = if let Some(parsed) = extended::parse_reference_image(remaining) {
      resolve_reference(definitions, parsed.reference).map(|definition| {
        let label_start = 2;
        (
          parsed.consumed,
          PieceKind::wrapper_with_syntax(
            InlineKind::Image {
              source: definition.destination.clone(),
              title: definition.title.clone(),
              alt: parsed.label.to_string(),
            },
            InlineSyntax::Reference {
              reference: parsed.reference.to_string(),
              style: parsed.style,
            },
            parsed.label.to_string(),
            Some((label_start, label_start + parsed.label.len())),
            false,
          ),
        )
      })
    } else if let Some(parsed) = extended::parse_autolink(remaining) {
      Some((
        parsed.consumed,
        PieceKind::wrapper(
          InlineKind::AutoLink {
            destination: extended::autolink_destination(parsed.value),
          },
          parsed.value.to_string(),
          Some((1, parsed.consumed - 1)),
          false,
        ),
      ))
    } else if let Some(parsed) = extended::parse_footnote_reference(remaining) {
      Some((
        parsed.consumed,
        PieceKind::wrapper(
          InlineKind::FootnoteReference {
            label: parsed.value.to_string(),
          },
          parsed.value.to_string(),
          Some((2, parsed.consumed - 1)),
          false,
        ),
      ))
    } else if let Some(parsed) = extended::parse_reference_link(remaining) {
      resolve_reference(definitions, parsed.reference).map(|definition| {
        (
          parsed.consumed,
          PieceKind::wrapper_with_syntax(
            InlineKind::Link {
              destination: definition.destination.clone(),
              title: definition.title.clone(),
            },
            InlineSyntax::Reference {
              reference: parsed.reference.to_string(),
              style: parsed.style,
            },
            parsed.label.to_string(),
            Some((1, 1 + parsed.label.len())),
            true,
          ),
        )
      })
    } else if allow_bare_autolink && bare_autolink_boundary(source, cursor) {
      extended::parse_bare_autolink(remaining).map(|parsed| {
        let text = parsed.text.to_string();
        (
          parsed.consumed,
          PieceKind::wrapper_with_syntax(
            InlineKind::Link {
              destination: parsed.destination,
              title: None,
            },
            InlineSyntax::BareAutoLink { text: text.clone() },
            text,
            Some((0, parsed.consumed)),
            false,
          ),
        )
      })
    } else {
      None
    }
    .or_else(|| {
      emoji::parse(remaining).map(|parsed| {
        (
          parsed.consumed,
          PieceKind::wrapper(
            InlineKind::Emoji {
              shortcode: parsed.shortcode,
              value: parsed.value.clone(),
            },
            parsed.value,
            None,
            false,
          ),
        )
      })
    })
    .or_else(|| {
      extended::parse_inline_math(remaining).map(|parsed| {
        (
          parsed.consumed,
          PieceKind::wrapper(
            InlineKind::InlineMath {
              source: parsed.value.to_string(),
            },
            parsed.value.to_string(),
            Some((1, parsed.consumed - 1)),
            false,
          ),
        )
      })
    })
    .or_else(|| {
      extended::parse_inline_html(remaining).map(|parsed| {
        (
          parsed.consumed,
          PieceKind::wrapper(
            InlineKind::InlineHtml {
              raw: parsed.value.to_string(),
            },
            parsed.value.to_string(),
            Some((0, parsed.consumed)),
            false,
          ),
        )
      })
    })
    .or_else(|| {
      extended::parse_superscript(remaining).map(|parsed| {
        (
          parsed.consumed,
          PieceKind::wrapper(
            InlineKind::Superscript,
            parsed.content.to_string(),
            Some((1, parsed.consumed - 1)),
            true,
          ),
        )
      })
    })
    .or_else(|| {
      extended::parse_subscript(remaining).map(|parsed| {
        (
          parsed.consumed,
          PieceKind::wrapper(
            InlineKind::Subscript,
            parsed.content.to_string(),
            Some((1, parsed.consumed - 1)),
            true,
          ),
        )
      })
    });

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

fn bare_autolink_boundary(source: &str, cursor: usize) -> bool {
  cursor == 0
    || source[..cursor]
      .chars()
      .next_back()
      .is_some_and(|character| matches!(character, '*' | ' ' | '_' | '~' | '('))
}

fn collect_reference_definitions(document: &Document) -> BTreeMap<String, ReferenceDefinition> {
  let mut definitions = BTreeMap::new();
  for node in document.children(document.root) {
    let NodeKind::Block(BlockKind::ReferenceDefinition { label }) = &node.kind else {
      continue;
    };
    let target = inline_text(document, node.id);
    let Some(definition) = parse_reference_definition_target(&target) else {
      continue;
    };
    definitions
      .entry(normalize_reference(label))
      .or_insert(definition);
  }
  definitions
}

fn resolve_reference<'a>(
  definitions: &'a BTreeMap<String, ReferenceDefinition>,
  reference: &str,
) -> Option<&'a ReferenceDefinition> {
  definitions.get(&normalize_reference(reference))
}

fn normalize_reference(value: &str) -> String {
  value
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase()
}

fn inline_text(document: &Document, node_id: NodeId) -> String {
  let Some(node) = document.node(node_id) else {
    return String::new();
  };
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => value.clone(),
    _ => node
      .children
      .iter()
      .map(|child| inline_text(document, *child))
      .collect::<String>(),
  }
}

fn parse_reference_definition_target(raw: &str) -> Option<ReferenceDefinition> {
  let raw = raw.trim();
  if raw.is_empty() {
    return None;
  }

  let (destination, rest) = if let Some(rest) = raw.strip_prefix('<') {
    let end = rest.find('>')?;
    (rest[..end].to_string(), rest[end + 1..].trim_start())
  } else {
    let end = raw
      .char_indices()
      .find_map(|(index, character)| character.is_whitespace().then_some(index))
      .unwrap_or(raw.len());
    (raw[..end].to_string(), raw[end..].trim_start())
  };
  if destination.is_empty() {
    return None;
  }

  let title = if rest.is_empty() {
    None
  } else {
    Some(parse_reference_title(rest)?)
  };
  Some(ReferenceDefinition { destination, title })
}

fn parse_reference_title(raw: &str) -> Option<String> {
  let raw = raw.trim();
  let (opening, closing) = match raw.chars().next()? {
    '"' => ('"', '"'),
    '\'' => ('\'', '\''),
    '(' => ('(', ')'),
    _ => return None,
  };
  let body = raw.strip_prefix(opening)?.strip_suffix(closing)?;
  Some(body.to_string())
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
      "a <https://example.com> <dev-team@example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1] z",
    );
    apply(&mut document);
    assert_eq!(
      to_markdown(&document),
      "a <https://example.com> <dev-team@example.com> :grinning: <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^source-1] z"
    );
    assert!(document.nodes.values().any(|node| matches!(
      &node.kind,
      NodeKind::Inline(InlineKind::AutoLink { destination })
        if destination == "mailto:dev-team@example.com"
    )));
  }

  #[test]
  fn resolves_reference_links_and_images_case_insensitively() {
    let markdown = "[shown][Target ID] ![logo][IMG]\n\n[target   id]: https://example.com \"Home\"\n\n[img]: image.png";
    let mut document = crate::parser::parse_markdown(markdown);
    crate::parser_blocks::apply(&mut document, markdown);
    apply(&mut document);

    assert!(document.nodes.values().any(|node| matches!(
      (&node.kind, &node.inline_syntax),
      (
        NodeKind::Inline(InlineKind::Link { destination, title }),
        Some(InlineSyntax::Reference { .. })
      ) if destination == "https://example.com" && title.as_deref() == Some("Home")
    )));
    assert!(document.nodes.values().any(|node| matches!(
      (&node.kind, &node.inline_syntax),
      (
        NodeKind::Inline(InlineKind::Image { source, .. }),
        Some(InlineSyntax::Reference { .. })
      ) if source == "image.png"
    )));
  }

  #[test]
  fn renderer_semantics_stay_on_existing_link_and_image_kinds() {
    let markdown = "[shown][id] ![logo][img] https://example.com\n\n[id]: https://target.example\n\n[img]: image.png";
    let mut document = crate::parser::parse_markdown(markdown);
    crate::parser_blocks::apply(&mut document, markdown);
    apply(&mut document);

    let references = document
      .nodes
      .values()
      .filter(|node| matches!(node.inline_syntax, Some(InlineSyntax::Reference { .. })))
      .collect::<Vec<_>>();
    assert_eq!(references.len(), 2);
    assert!(references.iter().any(|node| matches!(
      node.kind,
      NodeKind::Inline(InlineKind::Link { .. })
    )));
    assert!(references.iter().any(|node| matches!(
      node.kind,
      NodeKind::Inline(InlineKind::Image { .. })
    )));
    assert!(document.nodes.values().any(|node| matches!(
      (&node.kind, &node.inline_syntax),
      (
        NodeKind::Inline(InlineKind::Link { destination, .. }),
        Some(InlineSyntax::BareAutoLink { .. })
      ) if destination == "https://example.com"
    )));
  }

  #[test]
  fn unresolved_reference_syntax_stays_plain_text() {
    let markdown = "[missing][id] ![missing][image]";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    assert_eq!(to_markdown(&document), markdown);
  }

  #[test]
  fn bare_autolinks_require_the_same_top_level_boundary_as_tauri() {
    let markdown = "https://start.example abchttps://joined.example (https://paren.example) _https://underscore.example [https://label.example](https://target.example)";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    let destinations = document
      .nodes
      .values()
      .filter_map(|node| match (&node.kind, &node.inline_syntax) {
        (
          NodeKind::Inline(InlineKind::Link { destination, .. }),
          Some(InlineSyntax::BareAutoLink { .. }),
        ) => Some(destination.as_str()),
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(destinations.len(), 3);
    assert!(destinations.contains(&"https://start.example"));
    assert!(destinations.contains(&"https://paren.example"));
    assert!(destinations.contains(&"https://underscore.example"));
    assert!(!destinations.contains(&"https://joined.example"));
    assert!(!destinations.contains(&"https://label.example"));
  }

  #[test]
  fn every_visible_atomic_node_exposes_real_editable_text() {
    let mut document = crate::parser::parse_markdown(
      "\\* <https://example.com> https://example.org :grinning: <kbd>Ctrl</kbd> $x+y$ [^source-1] ![alt](image.png)\nnext",
    );
    apply(&mut document);

    for node in document.nodes.values().filter(|node| {
      let bare_link = matches!(
        (&node.kind, &node.inline_syntax),
        (
          NodeKind::Inline(InlineKind::Link { .. }),
          Some(InlineSyntax::BareAutoLink { .. })
        )
      );
      bare_link
        || matches!(
          &node.kind,
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
    let markdown = "price $5 and <not a link> and ^open and ftp://not-bare.example";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    assert_eq!(to_markdown(&document), markdown);
  }

  #[test]
  fn never_interprets_literal_block_contents() {
    let markdown = "---\nraw: '$frontmatter$ :grinning: <kbd>x</kbd>'\n---\n\n```md\n$code$ :grinning: <kbd>x</kbd> https://literal.example\n```";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document);
    assert_eq!(to_markdown(&document), markdown);
    assert!(!document.nodes.values().any(|node| {
      matches!(
        &node.kind,
        NodeKind::Inline(
          InlineKind::InlineMath { .. } | InlineKind::Emoji { .. } | InlineKind::InlineHtml { .. }
        )
      ) || matches!(
        (&node.kind, &node.inline_syntax),
        (
          NodeKind::Inline(InlineKind::Link { .. }),
          Some(InlineSyntax::BareAutoLink { .. })
        )
      )
    }));
  }
}
