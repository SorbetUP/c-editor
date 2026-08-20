use crate::model::{BlockKind, Document, Node, NodeId, NodeKind, SourceRange};
use crate::syntax::block::blockquote;

pub(crate) fn apply(document: &mut Document, markdown: &str) {
  let root = document.root;
  let blocks = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for block in blocks {
    normalize_node(document, block, markdown);
  }
}

fn normalize_node(document: &mut Document, node_id: NodeId, markdown: &str) {
  let kind = document.node(node_id).map(|node| node.kind.clone());
  if matches!(kind, Some(NodeKind::Block(BlockKind::BlockQuote))) {
    normalize_blockquote(document, node_id, markdown);
    return;
  }

  let children = document
    .node(node_id)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  for child in children {
    normalize_node(document, child, markdown);
  }
}

fn normalize_blockquote(document: &mut Document, quote: NodeId, markdown: &str) {
  let Some(range) = document.node(quote).and_then(|node| node.source) else {
    return;
  };
  let Some(raw) = utf16_slice(markdown, range) else {
    return;
  };
  let Some(inner_markdown) = strip_quote_source(raw) else {
    return;
  };

  // Re-run the complete public parser pipeline on the quote body. This is
  // intentionally recursive: stripping one quote marker strictly reduces the
  // nesting depth, while lists, special blocks, diagrams and inline extensions
  // receive exactly the same treatment as top-level Markdown.
  let inner = crate::parse_markdown(&inner_markdown);

  clear_children(document, quote);
  let children = inner
    .node(inner.root)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  for (index, child) in children.into_iter().enumerate() {
    clone_subtree(&inner, child, document, quote, index);
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

fn clear_children(document: &mut Document, parent: NodeId) {
  let children = document
    .node(parent)
    .map(|node| node.children.clone())
    .unwrap_or_default();
  for child in children.into_iter().rev() {
    assert!(
      document.remove_subtree(child).is_some(),
      "blockquote child must be detachable"
    );
  }
}

fn strip_quote_source(raw: &str) -> Option<String> {
  raw
    .split('\n')
    .map(blockquote::strip_marker)
    .collect::<Option<Vec<_>>>()
    .map(|lines| lines.join("\n"))
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn strips_every_quote_marker_without_touching_inner_markdown() {
    assert_eq!(
      strip_quote_source("> # Heading\n>\n> - item\n>   - nested").as_deref(),
      Some("# Heading\n\n- item\n  - nested")
    );
    assert!(strip_quote_source("> quote\nplain").is_none());
  }

  #[test]
  fn blockquote_becomes_a_recursive_block_container() {
    let markdown = "> # Heading\n>\n> - item\n>   - nested\n>\n> ```rust\n> fn main() {}\n> ```";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);

    let quote = document.children(document.root).next().unwrap();
    assert!(matches!(quote.kind, NodeKind::Block(BlockKind::BlockQuote)));
    let children = document.children(quote.id).collect::<Vec<_>>();
    assert!(matches!(
      children.first().map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::Heading { level: 1 }))
    ));
    assert!(children.iter().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::List { .. })
    )));
    assert!(children.iter().any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::CodeBlock { .. })
    )));
  }

  #[test]
  fn nested_blockquotes_recurse_until_the_innermost_content() {
    let markdown = "> outer\n> > inner **bold**";
    let mut document = crate::parser::parse_markdown(markdown);
    apply(&mut document, markdown);

    let outer = document.children(document.root).next().unwrap();
    let nested = document
      .children(outer.id)
      .find(|node| matches!(node.kind, NodeKind::Block(BlockKind::BlockQuote)))
      .expect("nested quote");
    assert!(document.children(nested.id).any(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::Paragraph)
    )));
  }

  #[test]
  fn quote_lists_receive_the_same_multiline_normalization_as_root_lists() {
    let markdown = "> - item\n>   continuation\n>\n>   second paragraph";
    let document = crate::parse_markdown(markdown);
    let quote = document.children(document.root).next().unwrap();
    let list = document
      .children(quote.id)
      .find(|node| matches!(node.kind, NodeKind::Block(BlockKind::List { .. })))
      .expect("list inside quote");
    let item = document.children(list.id).next().unwrap();
    assert!(document.children(item.id).count() >= 2);
  }
}
