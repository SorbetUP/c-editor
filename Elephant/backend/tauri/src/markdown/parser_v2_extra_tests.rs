use super::parser_v2::*;

#[test]
fn handles_plain_documents() {
  let doc = parse_markdown_document("plain text");
  assert_eq!(doc.frontmatter, serde_json::json!({}));
  assert_eq!(doc.blocks[0].kind, "paragraph");
  assert!(doc.html.contains("plain text"));
}

#[test]
fn parses_boolean_frontmatter_values() {
  let (frontmatter, body) = split_frontmatter("---\ndraft: true\npinned: false\n---\nBody");
  assert_eq!(frontmatter["draft"], true);
  assert_eq!(frontmatter["pinned"], false);
  assert_eq!(body, "Body");
}

#[test]
fn ignores_invalid_frontmatter() {
  let (frontmatter, body) = split_frontmatter("---\ntitle: Missing end\n# Body");
  assert_eq!(frontmatter, serde_json::json!({}));
  assert!(body.contains("Missing end"));
}

#[test]
fn parses_common_blocks() {
  let blocks = parse_blocks("## Heading\n\n> quote\n\n1. first\n\n---\n\n- [ ] todo");
  assert!(blocks.iter().any(|block| block.kind == "heading" && block.level == Some(2)));
  assert!(blocks.iter().any(|block| block.kind == "blockquote"));
  assert!(blocks.iter().any(|block| block.kind == "ordered_list_item"));
  assert!(blocks.iter().any(|block| block.kind == "hr"));
  assert!(blocks.iter().any(|block| block.kind == "task" && block.checked == Some(false)));
}

#[test]
fn extracts_multiple_headings_and_links() {
  let markdown = "# A\n\n## B\n\n[one](a.md) and [two](b.md)";
  assert_eq!(extract_outline_cmark(markdown).len(), 2);
  assert_eq!(extract_links_cmark(markdown).len(), 2);
}
