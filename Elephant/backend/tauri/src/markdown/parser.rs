use serde_json::{json, Map, Value};

use super::renderer::{render_html, render_plain_text, slugify, strip_inline_markdown};
use super::types::{MarkdownBlock, MarkdownDocument, MarkdownHeading, MarkdownImage, MarkdownLink, MarkdownTask};

pub fn parse_markdown_document(markdown: &str) -> MarkdownDocument {
  let (frontmatter, body) = split_frontmatter(markdown);
  let blocks = parse_blocks(body);
  let outline = extract_outline(&blocks);
  let links = extract_links(body);
  let images = extract_images(body);
  let tasks = extract_tasks(&blocks);
  let html = render_html(&blocks);
  let plain_text = render_plain_text(&blocks);

  MarkdownDocument {
    frontmatter,
    blocks,
    outline,
    links,
    images,
    tasks,
    plain_text,
    html,
  }
}

pub fn split_frontmatter(markdown: &str) -> (Value, &str) {
  let Some(rest) = markdown.strip_prefix("---\n") else {
    return (json!({}), markdown);
  };

  let Some(end) = rest.find("\n---") else {
    return (json!({}), markdown);
  };

  let raw = &rest[..end];
  let body = rest[end + 4..].trim_start_matches('\n');
  (parse_frontmatter(raw), body)
}

pub fn parse_frontmatter(raw: &str) -> Value {
  let mut map = Map::new();
  for line in raw.lines() {
    let Some((key, value)) = line.split_once(':') else {
      continue;
    };
    let key = key.trim();
    if key.is_empty() {
      continue;
    }
    map.insert(key.to_string(), parse_frontmatter_value(value));
  }
  Value::Object(map)
}

fn parse_frontmatter_value(raw: &str) -> Value {
  let value = raw.trim();
  if value.starts_with('[') && value.ends_with(']') {
    let items = value
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(',')
      .map(|item| Value::String(item.trim().trim_matches('"').trim_start_matches('#').to_string()))
      .filter(|item| item.as_str().is_some_and(|value| !value.is_empty()))
      .collect::<Vec<_>>();
    Value::Array(items)
  } else if value == "true" {
    Value::Bool(true)
  } else if value == "false" {
    Value::Bool(false)
  } else {
    Value::String(value.trim_matches('"').to_string())
  }
}

pub fn parse_blocks(markdown: &str) -> Vec<MarkdownBlock> {
  let mut blocks = Vec::new();
  let mut paragraph = Vec::<String>::new();
  let mut code_language: Option<String> = None;
  let mut code_lines = Vec::<String>::new();

  for line in markdown.lines() {
    let trimmed = line.trim_end();

    if let Some(language) = code_language.clone() {
      if trimmed.starts_with("```") {
        blocks.push(code_block(language, &code_lines));
        code_language = None;
        code_lines.clear();
      } else {
        code_lines.push(line.to_string());
      }
      continue;
    }

    if trimmed.starts_with("```") {
      flush_paragraph(&mut blocks, &mut paragraph);
      code_language = Some(trimmed.trim_start_matches("```").trim().to_string());
      continue;
    }

    if trimmed.is_empty() {
      flush_paragraph(&mut blocks, &mut paragraph);
      continue;
    }

    if is_hr(trimmed) {
      flush_paragraph(&mut blocks, &mut paragraph);
      blocks.push(MarkdownBlock::new("hr", trimmed, "", None));
      continue;
    }

    if let Some((level, title)) = parse_heading(trimmed) {
      flush_paragraph(&mut blocks, &mut paragraph);
      blocks.push(MarkdownBlock::new("heading", trimmed, title, Some(level)));
      continue;
    }

    if let Some((checked, text)) = parse_task(trimmed) {
      flush_paragraph(&mut blocks, &mut paragraph);
      let mut block = MarkdownBlock::new("task", trimmed, text, None);
      block.checked = Some(checked);
      blocks.push(block);
      continue;
    }

    if let Some(text) = parse_unordered_item(trimmed) {
      flush_paragraph(&mut blocks, &mut paragraph);
      blocks.push(MarkdownBlock::new("list_item", trimmed, text, None));
      continue;
    }

    if let Some(text) = parse_ordered_item(trimmed) {
      flush_paragraph(&mut blocks, &mut paragraph);
      blocks.push(MarkdownBlock::new("ordered_list_item", trimmed, text, None));
      continue;
    }

    if let Some(text) = trimmed.strip_prefix("> ") {
      flush_paragraph(&mut blocks, &mut paragraph);
      blocks.push(MarkdownBlock::new("blockquote", trimmed, text.trim(), None));
      continue;
    }

    paragraph.push(trimmed.to_string());
  }

  if let Some(language) = code_language {
    blocks.push(code_block(language, &code_lines));
  }
  flush_paragraph(&mut blocks, &mut paragraph);
  blocks
}

fn flush_paragraph(blocks: &mut Vec<MarkdownBlock>, paragraph: &mut Vec<String>) {
  if paragraph.is_empty() {
    return;
  }
  let raw = paragraph.join("\n");
  let text = paragraph.join(" ");
  blocks.push(MarkdownBlock::new("paragraph", raw, text, None));
  paragraph.clear();
}

fn code_block(language: String, code_lines: &[String]) -> MarkdownBlock {
  let text = code_lines.join("\n");
  let mut block = MarkdownBlock::new("code", text.clone(), text, None);
  if !language.is_empty() {
    block.language = Some(language);
  }
  block
}

fn parse_heading(line: &str) -> Option<(u8, String)> {
  let hashes = line.chars().take_while(|ch| *ch == '#').count();
  if hashes == 0 || hashes > 6 {
    return None;
  }
  let rest = line.get(hashes..)?.trim();
  if rest.is_empty() {
    return None;
  }
  Some((hashes as u8, strip_inline_markdown(rest)))
}

fn parse_task(line: &str) -> Option<(bool, String)> {
  for prefix in ["- [ ] ", "* [ ] ", "+ [ ] "] {
    if let Some(text) = line.strip_prefix(prefix) {
      return Some((false, text.trim().to_string()));
    }
  }
  for prefix in ["- [x] ", "- [X] ", "* [x] ", "* [X] ", "+ [x] ", "+ [X] "] {
    if let Some(text) = line.strip_prefix(prefix) {
      return Some((true, text.trim().to_string()));
    }
  }
  None
}

fn parse_unordered_item(line: &str) -> Option<String> {
  for prefix in ["- ", "* ", "+ "] {
    if let Some(text) = line.strip_prefix(prefix) {
      return Some(text.trim().to_string());
    }
  }
  None
}

fn parse_ordered_item(line: &str) -> Option<String> {
  let Some((number, text)) = line.split_once(". ") else {
    return None;
  };
  if number.chars().all(|ch| ch.is_ascii_digit()) {
    Some(text.trim().to_string())
  } else {
    None
  }
}

fn is_hr(line: &str) -> bool {
  matches!(line, "---" | "***" | "___")
}

pub fn extract_outline(blocks: &[MarkdownBlock]) -> Vec<MarkdownHeading> {
  blocks
    .iter()
    .enumerate()
    .filter_map(|(index, block)| {
      if block.kind == "heading" {
        let title = block.text.clone();
        Some(MarkdownHeading {
          level: block.level.unwrap_or(1),
          slug: slugify(&title),
          title,
          line: index + 1,
        })
      } else {
        None
      }
    })
    .collect()
}

pub fn extract_tasks(blocks: &[MarkdownBlock]) -> Vec<MarkdownTask> {
  blocks
    .iter()
    .enumerate()
    .filter_map(|(index, block)| {
      if block.kind == "task" {
        Some(MarkdownTask {
          text: block.text.clone(),
          checked: block.checked.unwrap_or(false),
          line: index + 1,
        })
      } else {
        None
      }
    })
    .collect()
}

pub fn extract_links(markdown: &str) -> Vec<MarkdownLink> {
  let mut links = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let mut rest = line;
    while let Some(start) = rest.find('[') {
      let is_image = start > 0 && rest[..start].ends_with('!');
      let candidate = &rest[start + 1..];
      let Some(label_end) = candidate.find(']') else { break; };
      let label = &candidate[..label_end];
      let after_label = &candidate[label_end + 1..];
      if after_label.starts_with('(') {
        if let Some(url_end) = after_label[1..].find(')') {
          let url = &after_label[1..1 + url_end];
          if !is_image && !url.is_empty() {
            links.push(MarkdownLink { label: label.to_string(), url: url.to_string(), line: line_index + 1 });
          }
          rest = &after_label[1 + url_end + 1..];
          continue;
        }
      }
      rest = &candidate[label_end + 1..];
    }
  }
  links
}

pub fn extract_images(markdown: &str) -> Vec<MarkdownImage> {
  let mut images = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let mut rest = line;
    while let Some(start) = rest.find("![") {
      let candidate = &rest[start + 2..];
      let Some(alt_end) = candidate.find(']') else { break; };
      let alt = &candidate[..alt_end];
      let after_alt = &candidate[alt_end + 1..];
      if after_alt.starts_with('(') {
        if let Some(url_end) = after_alt[1..].find(')') {
          let url = &after_alt[1..1 + url_end];
          images.push(MarkdownImage { alt: alt.to_string(), url: url.to_string(), line: line_index + 1 });
          rest = &after_alt[1 + url_end + 1..];
          continue;
        }
      }
      rest = &candidate[alt_end + 1..];
    }
  }
  images
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_frontmatter() {
    let (frontmatter, body) = split_frontmatter("---\ntitle: \"A\"\ntags: [one, two]\n---\n# A");
    assert_eq!(frontmatter["title"], "A");
    assert_eq!(frontmatter["tags"][0], "one");
    assert_eq!(body, "# A");
  }

  #[test]
  fn parses_blocks_outline_links_images_and_tasks() {
    let doc = parse_markdown_document("# Title\n\n- [x] Done\n- item\n![alt](a.png)\n[site](https://example.com)");
    assert_eq!(doc.outline[0].title, "Title");
    assert_eq!(doc.tasks[0].checked, true);
    assert_eq!(doc.images[0].url, "a.png");
    assert_eq!(doc.links.len(), 1);
    assert_eq!(doc.links[0].label, "site");
    assert!(doc.html.contains("task-list"));
  }

  #[test]
  fn parses_code_blocks() {
    let doc = parse_markdown_document("```rust\nfn main() {}\n```");
    assert_eq!(doc.blocks[0].kind, "code");
    assert_eq!(doc.blocks[0].language.as_deref(), Some("rust"));
  }
}
