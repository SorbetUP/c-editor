use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MarkdownMeta {
  pub title: String,
  pub note_type: String,
  pub tags: Vec<String>,
  pub body: String,
}

pub fn parse_markdown(input: &str, fallback_title: &str) -> MarkdownMeta {
  let mut meta = MarkdownMeta {
    title: String::new(),
    note_type: "note".to_string(),
    tags: Vec::new(),
    body: input.to_string(),
  };

  if let Some(rest) = input.strip_prefix("---\n") {
    if let Some(end) = rest.find("\n---") {
      let header = &rest[..end];
      meta.body = rest[end + 4..].trim_start().to_string();
      for line in header.lines() {
        apply_header_line(&mut meta, line);
      }
    }
  }

  if meta.title.is_empty() {
    meta.title = heading_title(&meta.body).unwrap_or_else(|| fallback_title.trim_end_matches(".md").to_string());
  }
  meta
}

fn apply_header_line(meta: &mut MarkdownMeta, line: &str) {
  let Some((key, raw_value)) = line.split_once(':') else { return; };
  let value = raw_value.trim().trim_matches('"');
  match key.trim() {
    "title" => meta.title = value.to_string(),
    "type" => meta.note_type = value.to_string(),
    "tags" => meta.tags = parse_tags(raw_value),
    _ => {}
  }
}

pub fn parse_tags(raw: &str) -> Vec<String> {
  raw.trim()
    .trim_start_matches('[')
    .trim_end_matches(']')
    .split(',')
    .map(|item| item.trim().trim_matches('"').trim_start_matches('#').to_string())
    .filter(|item| !item.is_empty())
    .collect()
}

pub fn heading_title(body: &str) -> Option<String> {
  body.lines()
    .find_map(|line| line.strip_prefix("# ").map(|title| title.trim().to_string()))
    .filter(|title| !title.is_empty())
}

pub fn excerpt(body: &str, max_lines: usize) -> String {
  body.lines()
    .map(|line| line.trim().trim_start_matches('#').trim())
    .filter(|line| !line.is_empty())
    .take(max_lines)
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn render_note(title: &str, note_type: &str, tags: &[String], body: &str) -> String {
  let tags = tags.iter().map(|tag| format!("\"{}\"", tag)).collect::<Vec<_>>().join(", ");
  format!("---\ntitle: \"{}\"\ntype: \"{}\"\ntags: [{}]\n---\n\n{}\n", title.replace('"', "\\\""), note_type, tags, body.trim())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_frontmatter_title_type_and_tags() {
    let doc = parse_markdown("---\ntitle: \"Alpha\"\ntype: \"note\"\ntags: [\"one\", \"two\"]\n---\n\nBody", "fallback.md");
    assert_eq!(doc.title, "Alpha");
    assert_eq!(doc.note_type, "note");
    assert_eq!(doc.tags, vec!["one", "two"]);
    assert_eq!(doc.body, "Body");
  }

  #[test]
  fn uses_heading_when_title_is_missing() {
    let doc = parse_markdown("# Heading\n\nText", "fallback.md");
    assert_eq!(doc.title, "Heading");
  }

  #[test]
  fn creates_excerpt_from_visible_lines() {
    assert_eq!(excerpt("# Title\n\nFirst line\nSecond line", 2), "Title First line");
  }

  #[test]
  fn renders_roundtrip_note() {
    let text = render_note("Alpha", "note", &["tag".to_string()], "# Alpha");
    let doc = parse_markdown(&text, "fallback.md");
    assert_eq!(doc.title, "Alpha");
    assert_eq!(doc.tags, vec!["tag"]);
  }
}
