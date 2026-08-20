use crate::model::{BlockKind, Document, NodeKind};

const TAURI_DIAGRAM_LANGUAGES: &[&str] = &[
  "mermaid",
  "flowchart",
  "sequence",
  "plantuml",
  "vega-lite",
];

pub(crate) fn apply(document: &mut Document) {
  let root = document.root;
  let blocks = document
    .node(root)
    .map(|node| node.children.clone())
    .unwrap_or_default();

  for block in blocks {
    let language = document.node(block).and_then(|node| match &node.kind {
      NodeKind::Block(BlockKind::CodeBlock {
        language: Some(language),
        fenced: true,
      }) => Some(language.trim().to_ascii_lowercase()),
      _ => None,
    });
    let Some(language) = language else {
      continue;
    };
    if !TAURI_DIAGRAM_LANGUAGES.contains(&language.as_str()) {
      continue;
    }
    if let Some(node) = document.node_mut(block) {
      node.kind = NodeKind::Block(BlockKind::Diagram { language });
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn promotes_only_renderers_shipped_by_tauri_muya() {
    for language in TAURI_DIAGRAM_LANGUAGES {
      let markdown = format!("```{language}\nA -> B\n```");
      let mut document = crate::parser::parse_markdown(&markdown);
      apply(&mut document);
      assert!(matches!(
        document.children(document.root).next().map(|node| &node.kind),
        Some(NodeKind::Block(BlockKind::Diagram { language: actual }))
          if actual.as_str() == *language
      ));
    }

    let mut rust = crate::parser::parse_markdown("```rust\nfn main() {}\n```");
    apply(&mut rust);
    assert!(matches!(
      rust.children(rust.root).next().map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::CodeBlock { .. }))
    ));
  }
}
