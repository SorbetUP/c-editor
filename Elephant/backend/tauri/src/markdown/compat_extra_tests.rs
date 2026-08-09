use super::muya_compat::{parse_muya_document, render_muya_html, tokenize_muya};
use super::muya_extras::{collect_list_item_meta, collect_muya_extras};
use super::parser_v4::{extract_images, extract_links, extract_outline, parse_blocks, parse_markdown_document, split_frontmatter};
use super::renderer_v2::{normalize_editor_html, render_html, render_plain_text, slugify, strip_inline_markdown};

#[test]
fn muya_parse_returns_tokens_and_document_fields() {
  let doc = parse_muya_document("---\ntitle: \"Demo\"\n---\n# Demo\n\n- [x] done");
  assert_eq!(doc["frontmatter"]["title"], "Demo");
  assert_eq!(doc["outline"].as_array().unwrap().len(), 1);
  assert_eq!(doc["tasks"].as_array().unwrap().len(), 1);
  assert!(doc["html"].as_str().unwrap().contains("task-list"));
  assert!(doc["tokens"].as_array().unwrap().iter().any(|token| token["kind"] == "task_marker"));
}

#[test]
fn muya_tokenizes_lists_tables_images_footnotes_and_breaks() {
  let markdown = "1. first\n2. second\n\n- bullet\n\n> quote\n\n![alt](pic.png)\n\n[^a]: footnote\n\nline  \nbreak\n\n---\n\n| A | B |\n| :- | -: |\n| 1 | 2 |";
  let tokens = tokenize_muya(markdown);
  assert!(tokens.iter().any(|token| token.kind == "ordered_list" && token.entering));
  assert!(tokens.iter().any(|token| token.kind == "bullet_list" && token.entering));
  assert!(tokens.iter().any(|token| token.kind == "blockquote" && token.entering));
  assert!(tokens.iter().any(|token| token.kind == "image" && token.attrs["url"] == "pic.png"));
  assert!(tokens.iter().any(|token| token.kind == "footnote_definition" && token.entering));
  assert!(tokens.iter().any(|token| token.kind == "hard_break"));
  assert!(tokens.iter().any(|token| token.kind == "hr"));
  assert!(tokens.iter().any(|token| token.kind == "table" && token.entering));
  assert!(tokens.iter().any(|token| token.kind == "table_cell"));
}

#[test]
fn muya_tokenizes_code_and_html() {
  let markdown = "```rust\nlet x = 1;\n```\n\n<span>raw</span>\n\n`inline`";
  let tokens = tokenize_muya(markdown);
  assert!(tokens.iter().any(|token| token.kind == "code_block" && token.attrs["kind"].as_str().unwrap().contains("rust")));
  assert!(tokens.iter().any(|token| token.kind == "html" || token.kind == "inline_html"));
  assert!(tokens.iter().any(|token| token.kind == "inline_code" && token.text == "inline"));
}

#[test]
fn muya_collects_reference_links_images_and_definitions() {
  let extras = collect_muya_extras("[ref]: https://example.com \"Title\"\n\nUse [label][ref] and ![alt][ref].");
  assert!(extras.iter().any(|extra| extra.kind == "reference_definition" && extra.attrs["url"] == "https://example.com"));
  assert!(extras.iter().any(|extra| extra.kind == "reference_link" && extra.attrs["reference"] == "ref"));
  assert!(extras.iter().any(|extra| extra.kind == "reference_image" && extra.text == "alt"));
}

#[test]
fn muya_collects_nested_list_item_metadata() {
  let items = collect_list_item_meta("- root\n  - child\n    1. ordered\n  - [x] checked");
  assert!(items.iter().any(|item| item.attrs["depth"] == 0 && item.attrs["text"] == "root"));
  assert!(items.iter().any(|item| item.attrs["depth"] == 1 && item.attrs["text"] == "child"));
  assert!(items.iter().any(|item| item.attrs["ordered"] == true && item.attrs["marker"] == "1."));
  assert!(items.iter().any(|item| item.attrs["checked"] == true));
}

#[test]
fn renderer_normalizes_task_html_and_keeps_non_task_html() {
  let task_html = normalize_editor_html("<ul>\n<li><input disabled type=\"checkbox\"> done</li>\n</ul>\n");
  assert!(task_html.contains("task-list"));
  assert!(task_html.contains("task-list-item"));

  let paragraph_html = normalize_editor_html("<p>Hello</p>\n");
  assert_eq!(paragraph_html, "<p>Hello</p>\n");
}

#[test]
fn renderer_helpers_cover_plain_text_slug_and_inline_strip() {
  assert_eq!(slugify("Hello, World!"), "hello-world");
  assert_eq!(slugify("!!!"), "section");
  assert_eq!(strip_inline_markdown("**bold** ~~old~~ `code`"), "bold old code");

  let blocks = parse_blocks("# Title\n\n**bold** text\n\n---");
  let text = render_plain_text(&blocks);
  assert!(text.contains("Title"));
  assert!(text.contains("bold text"));
}

#[test]
fn parser_handles_frontmatter_variants_and_common_blocks() {
  let (frontmatter, body) = split_frontmatter("---\ndraft: true\ntags: [one, #two]\n---\nBody");
  assert_eq!(frontmatter["draft"], true);
  assert_eq!(frontmatter["tags"][1], "two");
  assert_eq!(body, "Body");

  let (missing_frontmatter, missing_body) = split_frontmatter("---\ntitle: missing end\nBody");
  assert_eq!(missing_frontmatter, serde_json::json!({}));
  assert!(missing_body.contains("missing end"));

  let blocks = parse_blocks("## Heading\n\n> quote\n\n1. ordered\n\n- [ ] todo\n\n```\ncode\n```");
  assert!(blocks.iter().any(|block| block.kind == "heading" && block.level == Some(2)));
  assert!(blocks.iter().any(|block| block.kind == "blockquote"));
  assert!(blocks.iter().any(|block| block.kind == "ordered_list_item"));
  assert!(blocks.iter().any(|block| block.kind == "task" && block.checked == Some(false)));
  assert!(blocks.iter().any(|block| block.kind == "code"));
}

#[test]
fn parser_extracts_outline_links_and_images_with_cmark() {
  let markdown = "# A\n\n## B\n\n[one](a.md) and [two](b.md)\n\n![pic](p.png)";
  assert_eq!(extract_outline(markdown).len(), 2);
  assert_eq!(extract_links(markdown).len(), 2);
  assert_eq!(extract_images(markdown).len(), 1);
}

#[test]
fn markdown_document_renders_gfm_features() {
  let doc = parse_markdown_document("# Title\n\n| A | B |\n| - | - |\n| 1 | 2 |\n\n~~deleted~~\n\n- [x] task");
  assert!(doc.html.contains("table"));
  assert!(doc.html.contains("del"));
  assert!(doc.html.contains("task-list"));
  assert_eq!(doc.tasks.len(), 1);
}

#[test]
fn muya_html_entrypoint_matches_renderer_contract() {
  let html = render_muya_html("# A\n\n- [x] done");
  assert!(html.contains("<h1"));
  assert!(html.contains("task-list"));
  assert!(render_html("~~old~~").contains("del"));
}
