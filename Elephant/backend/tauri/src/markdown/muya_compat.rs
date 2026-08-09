use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::muya_extras::{collect_muya_extras, render_muya_extras_html};
use super::parser_v4::parse_markdown_document;
use super::renderer_v2::render_html;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaToken {
  pub kind: String,
  pub entering: bool,
  pub text: String,
  pub attrs: Value,
}

pub fn muya_options() -> Options {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_FOOTNOTES);
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TASKLISTS);
  options.insert(Options::ENABLE_SMART_PUNCTUATION);
  options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
  options
}

pub fn render_muya_html(markdown: &str) -> String {
  if collect_muya_extras(markdown).is_empty() {
    render_html(markdown)
  } else {
    render_html(&render_muya_extras_html(markdown))
  }
}

pub fn parse_muya_document(markdown: &str) -> Value {
  let doc = parse_markdown_document(markdown);
  let extras = collect_muya_extras(markdown);
  json!({
    "frontmatter": doc.frontmatter,
    "blocks": doc.blocks,
    "outline": doc.outline,
    "links": doc.links,
    "images": doc.images,
    "tasks": doc.tasks,
    "plainText": doc.plain_text,
    "html": render_muya_html(markdown),
    "tokens": tokenize_muya(markdown),
    "extras": extras
  })
}

pub fn tokenize_muya(markdown: &str) -> Vec<MuyaToken> {
  let parser = Parser::new_ext(markdown, muya_options());
  let mut tokens = Vec::new();

  for event in parser {
    match event {
      Event::Start(tag) => tokens.push(MuyaToken { kind: start_tag_kind(&tag), entering: true, text: String::new(), attrs: start_tag_attrs(&tag) }),
      Event::End(tag) => tokens.push(MuyaToken { kind: end_tag_kind(&tag), entering: false, text: String::new(), attrs: json!({}) }),
      Event::Text(text) => tokens.push(text_token("text", &text)),
      Event::Code(text) => tokens.push(text_token("inline_code", &text)),
      Event::Html(text) => tokens.push(text_token("html", &text)),
      Event::InlineHtml(text) => tokens.push(text_token("inline_html", &text)),
      Event::FootnoteReference(text) => tokens.push(text_token("footnote_reference", &text)),
      Event::SoftBreak => tokens.push(text_token("soft_break", "\n")),
      Event::HardBreak => tokens.push(text_token("hard_break", "\n")),
      Event::Rule => tokens.push(MuyaToken { kind: "hr".to_string(), entering: false, text: String::new(), attrs: json!({}) }),
      Event::TaskListMarker(checked) => tokens.push(MuyaToken { kind: "task_marker".to_string(), entering: false, text: String::new(), attrs: json!({ "checked": checked }) }),
      _ => tokens.push(MuyaToken { kind: "unknown_event".to_string(), entering: false, text: String::new(), attrs: json!({}) }),
    }
  }

  tokens.extend(collect_muya_extras(markdown).into_iter().map(|extra| MuyaToken {
    kind: extra.kind,
    entering: false,
    text: extra.text,
    attrs: json!({ "line": extra.line, "extra": extra.attrs }),
  }));

  tokens
}

fn text_token(kind: &str, text: &str) -> MuyaToken {
  MuyaToken { kind: kind.to_string(), entering: false, text: text.to_string(), attrs: json!({}) }
}

fn start_tag_kind(tag: &Tag<'_>) -> String {
  match tag {
    Tag::Paragraph => "paragraph",
    Tag::Heading { .. } => "heading",
    Tag::BlockQuote(_) => "blockquote",
    Tag::CodeBlock(_) => "code_block",
    Tag::HtmlBlock => "html_block",
    Tag::List(Some(_)) => "ordered_list",
    Tag::List(None) => "bullet_list",
    Tag::Item => "list_item",
    Tag::FootnoteDefinition(_) => "footnote_definition",
    Tag::Table(_) => "table",
    Tag::TableHead => "table_head",
    Tag::TableRow => "table_row",
    Tag::TableCell => "table_cell",
    Tag::Emphasis => "emphasis",
    Tag::Strong => "strong",
    Tag::Strikethrough => "strikethrough",
    Tag::Link { .. } => "link",
    Tag::Image { .. } => "image",
    Tag::MetadataBlock(_) => "metadata_block",
    _ => "unknown_block",
  }.to_string()
}

fn end_tag_kind(tag: &TagEnd) -> String {
  match tag {
    TagEnd::Paragraph => "paragraph",
    TagEnd::Heading(_) => "heading",
    TagEnd::BlockQuote(_) => "blockquote",
    TagEnd::CodeBlock => "code_block",
    TagEnd::HtmlBlock => "html_block",
    TagEnd::List(_) => "list",
    TagEnd::Item => "list_item",
    TagEnd::FootnoteDefinition => "footnote_definition",
    TagEnd::Table => "table",
    TagEnd::TableHead => "table_head",
    TagEnd::TableRow => "table_row",
    TagEnd::TableCell => "table_cell",
    TagEnd::Emphasis => "emphasis",
    TagEnd::Strong => "strong",
    TagEnd::Strikethrough => "strikethrough",
    TagEnd::Link => "link",
    TagEnd::Image => "image",
    TagEnd::MetadataBlock(_) => "metadata_block",
    _ => "unknown_block",
  }.to_string()
}

fn start_tag_attrs(tag: &Tag<'_>) -> Value {
  match tag {
    Tag::Heading { level, id, classes, attrs } => json!({
      "level": format!("{:?}", level),
      "id": id.as_ref().map(|value| value.to_string()),
      "classes": classes.iter().map(|value| value.to_string()).collect::<Vec<_>>(),
      "attrs": attrs.iter().map(|(key, value)| json!({ "key": key.to_string(), "value": value.as_ref().map(|v| v.to_string()) })).collect::<Vec<_>>()
    }),
    Tag::CodeBlock(kind) => json!({ "kind": format!("{:?}", kind) }),
    Tag::List(start) => json!({ "start": start }),
    Tag::Link { dest_url, title, .. } => json!({ "url": dest_url.to_string(), "title": title.to_string() }),
    Tag::Image { dest_url, title, .. } => json!({ "url": dest_url.to_string(), "title": title.to_string() }),
    Tag::Table(alignments) => json!({ "alignments": alignments.iter().map(|value| format!("{:?}", value)).collect::<Vec<_>>() }),
    Tag::FootnoteDefinition(label) => json!({ "label": label.to_string() }),
    _ => json!({})
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn renders_editor_task_classes() {
    let html = render_muya_html("- [x] done");
    assert!(html.contains("task-list"));
    assert!(html.contains("task-list-item"));
  }

  #[test]
  fn tokenizes_core_blocks_and_inline_marks() {
    let tokens = tokenize_muya("# A\n\n**bold** and [link](a.md)\n\n| A |\n| - |\n| 1 |");
    assert!(tokens.iter().any(|token| token.kind == "heading" && token.entering));
    assert!(tokens.iter().any(|token| token.kind == "strong" && token.entering));
    assert!(tokens.iter().any(|token| token.kind == "link" && token.entering));
    assert!(tokens.iter().any(|token| token.kind == "table" && token.entering));
  }

  #[test]
  fn includes_math_and_diagram_extras() {
    let doc = parse_muya_document("# A\n\n$x$\n\n$$\ny=x\n$$\n\n```mermaid\ngraph TD;\n```");
    let extras = doc["extras"].as_array().unwrap();
    assert!(extras.iter().any(|extra| extra["kind"] == "inline_math"));
    assert!(extras.iter().any(|extra| extra["kind"] == "math_block"));
    assert!(extras.iter().any(|extra| extra["kind"] == "diagram"));
    assert!(doc["html"].as_str().unwrap().contains("<h1"));
    assert!(doc["html"].as_str().unwrap().contains("math-inline"));
    assert!(doc["html"].as_str().unwrap().contains("diagram-block"));
  }
}
