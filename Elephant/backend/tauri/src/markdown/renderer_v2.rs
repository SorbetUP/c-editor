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

pub fn render_html(markdown: &str) -> String {
  let parser = Parser::new_ext(markdown, markdown_options());
  let mut html_out = String::new();
  html::push_html(&mut html_out, parser);
  normalize_editor_html(&html_out)
}

pub fn normalize_editor_html(html: &str) -> String {
  let mut out = html.to_string();
  if out.contains("checkbox") {
    out = out.replace("<ul>\n<li><input", "<ul class=\"task-list\">\n<li class=\"task-list-item\"><input");
    out = out.replace("\n<li><input", "\n<li class=\"task-list-item\"><input");
  }
  out
}

pub fn render_blocks_html(blocks: &[MarkdownBlock]) -> String {
  let markdown = blocks.iter().map(|block| block.raw.clone()).collect::<Vec<_>>().join("\n\n");
  render_html(&markdown)
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

pub fn render_plain_text(blocks: &[MarkdownBlock]) -> String {
  blocks
    .iter()
    .filter(|block| block.kind != "hr")
    .map(|block| strip_inline_markdown(&block.text))
    .filter(|text| !text.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n")
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn renders_task_list_classes() {
    let html = render_html("- [x] done");
    assert!(html.contains("task-list"));
    assert!(html.contains("task-list-item"));
  }

  #[test]
  fn renders_gfm_features() {
    let html = render_html("| A | B |\n| - | - |\n| 1 | 2 |\n\n~~old~~");
    assert!(html.contains("table"));
    assert!(html.contains("del"));
  }
}
