use pulldown_cmark::{html, Options, Parser};

use super::types::MarkdownBlock;

pub fn markdown_options() -> Options {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_FOOTNOTES);
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TASKLISTS);
  options.insert(Options::ENABLE_SMART_PUNCTUATION);
  options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
  options
}

pub fn escape_html(input: &str) -> String {
  input
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

pub fn slugify(input: &str) -> String {
  let mut out = String::new();
  let mut dash = false;
  for ch in input.trim().to_lowercase().chars() {
    if ch.is_ascii_alphanumeric() {
      out.push(ch);
      dash = false;
    } else if !dash {
      out.push('-');
      dash = true;
    }
  }
  let out = out.trim_matches('-').to_string();
  if out.is_empty() { "section".to_string() } else { out }
}

pub fn strip_inline_markdown(input: &str) -> String {
  input
    .replace("**", "")
    .replace("__", "")
    .replace("~~", "")
    .replace('`', "")
    .replace('*', "")
    .replace('_', "")
}

pub fn render_html(markdown: &str) -> String {
  let parser = Parser::new_ext(markdown, markdown_options());
  let mut out = String::new();
  html::push_html(&mut out, parser);
  out
}

pub fn render_blocks_html(blocks: &[MarkdownBlock]) -> String {
  let markdown = blocks.iter().map(|block| block.raw.clone()).collect::<Vec<_>>().join("\n\n");
  render_html(&markdown)
}

pub fn render_plain_text(blocks: &[MarkdownBlock]) -> String {
  blocks
    .iter()
    .filter(|block| block.kind != "hr")
    .map(|block| strip_inline_markdown(&block.text))
    .filter(|text| !text.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

pub fn render_inline(input: &str) -> String {
  escape_html(input)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn escapes_html() {
    assert_eq!(escape_html("a < b"), "a &lt; b");
  }

  #[test]
  fn slugifies_headings() {
    assert_eq!(slugify("Hello World!"), "hello-world");
  }

  #[test]
  fn renders_common_mark_and_gfm() {
    let html = render_html("# Title\n\n| A | B |\n| - | - |\n| 1 | 2 |\n\n- [x] done\n\n~~old~~");
    assert!(html.contains("Title"));
    assert!(html.contains("table"));
    assert!(html.contains("checked"));
    assert!(html.contains("del"));
  }
}
