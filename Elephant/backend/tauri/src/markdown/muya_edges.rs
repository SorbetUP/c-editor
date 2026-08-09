use serde_json::{json, Value};

pub fn edge_contract(markdown: &str) -> Value {
  json!({
    "autolinks": autolink_contract(markdown),
    "escapes": escape_contract(markdown),
    "headingAttrs": heading_attr_contract(markdown),
    "codeFences": code_fence_contract(markdown)
  })
}

pub fn autolink_contract(markdown: &str) -> Value {
  let mut links = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let mut cursor = 0usize;
    while let Some(start_rel) = line[cursor..].find('<') {
      let start = cursor + start_rel;
      let Some(end_rel) = line[start + 1..].find('>') else { break; };
      let end = start + 1 + end_rel;
      let value = &line[start + 1..end];
      if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("mailto:") || looks_like_email(value) {
        links.push(json!({
          "text": value,
          "href": if looks_like_email(value) && !value.starts_with("mailto:") { format!("mailto:{}", value) } else { value.to_string() },
          "line": line_index + 1
        }));
      }
      cursor = end + 1;
    }
  }
  json!({ "items": links })
}

pub fn escape_contract(markdown: &str) -> Value {
  let mut escapes = Vec::new();
  let escapable = ['\\', '`', '*', '_', '{', '}', '[', ']', '<', '>', '(', ')', '#', '+', '-', '.', '!', '|', '~', '$'];
  for (line_index, line) in markdown.lines().enumerate() {
    let chars = line.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    while i + 1 < chars.len() {
      if chars[i] == '\\' && escapable.contains(&chars[i + 1]) {
        escapes.push(json!({ "char": chars[i + 1].to_string(), "line": line_index + 1 }));
        i += 2;
      } else {
        i += 1;
      }
    }
  }
  json!({ "items": escapes, "literal": render_escaped_literals(markdown) })
}

pub fn render_escaped_literals(markdown: &str) -> String {
  let mut out = String::new();
  let mut chars = markdown.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '\\' {
      if let Some(next) = chars.peek().copied() {
        out.push(next);
        chars.next();
      }
    } else {
      out.push(ch);
    }
  }
  out
}

pub fn heading_attr_contract(markdown: &str) -> Value {
  let mut headings = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let trimmed = line.trim();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) || trimmed.chars().nth(level) != Some(' ') {
      continue;
    }
    let text = trimmed[level..].trim();
    let (title, attrs) = if let Some(attr_start) = text.rfind(" {") {
      if text.ends_with('}') {
        (&text[..attr_start], parse_heading_attrs(&text[attr_start + 2..text.len() - 1]))
      } else {
        (text, json!({}))
      }
    } else {
      (text, json!({}))
    };
    headings.push(json!({ "line": line_index + 1, "level": level, "text": title.trim(), "attrs": attrs }));
  }
  json!({ "items": headings })
}

pub fn code_fence_contract(markdown: &str) -> Value {
  let mut fences = Vec::new();
  let lines = markdown.lines().collect::<Vec<_>>();
  let mut i = 0usize;
  while i < lines.len() {
    let trimmed = lines[i].trim();
    if let Some(rest) = trimmed.strip_prefix("```").or_else(|| trimmed.strip_prefix("~~~")) {
      let marker = if trimmed.starts_with("```") { "```" } else { "~~~" };
      let info = rest.trim();
      let language = info.split_whitespace().next().unwrap_or("");
      let start_line = i + 1;
      i += 1;
      let mut body = Vec::new();
      while i < lines.len() && !lines[i].trim().starts_with(marker) {
        body.push(lines[i]);
        i += 1;
      }
      fences.push(json!({ "line": start_line, "marker": marker, "info": info, "language": language, "text": body.join("\n") }));
    }
    i += 1;
  }
  json!({ "items": fences })
}

fn parse_heading_attrs(raw: &str) -> Value {
  let mut id = Value::Null;
  let mut classes = Vec::new();
  let mut attrs = Vec::new();
  for part in raw.split_whitespace() {
    if let Some(value) = part.strip_prefix('#') {
      id = json!(value);
    } else if let Some(value) = part.strip_prefix('.') {
      classes.push(json!(value));
    } else if let Some((key, value)) = part.split_once('=') {
      attrs.push(json!({ "key": key, "value": value.trim_matches('"') }));
    }
  }
  json!({ "id": id, "classes": classes, "attrs": attrs })
}

fn looks_like_email(value: &str) -> bool {
  value.contains('@') && value.contains('.') && !value.contains(' ')
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_autolinks() {
    let contract = autolink_contract("<https://example.com> and <me@example.com>");
    assert_eq!(contract["items"].as_array().unwrap().len(), 2);
    assert_eq!(contract["items"][1]["href"], "mailto:me@example.com");
  }

  #[test]
  fn detects_and_renders_escapes() {
    let contract = escape_contract("\\*literal\\* and \\# not heading");
    assert_eq!(contract["items"].as_array().unwrap().len(), 3);
    assert_eq!(contract["literal"], "*literal* and # not heading");
  }

  #[test]
  fn detects_heading_attrs() {
    let contract = heading_attr_contract("## Title {#id .a .b key=\"v\"}");
    assert_eq!(contract["items"][0]["level"], 2);
    assert_eq!(contract["items"][0]["attrs"]["id"], "id");
    assert_eq!(contract["items"][0]["attrs"]["classes"][1], "b");
  }

  #[test]
  fn detects_backtick_and_tilde_fences() {
    let contract = code_fence_contract("~~~python\nprint(1)\n~~~\n\n```rust\nlet x=1;\n```");
    assert_eq!(contract["items"].as_array().unwrap().len(), 2);
    assert_eq!(contract["items"][0]["language"], "python");
    assert_eq!(contract["items"][1]["language"], "rust");
  }
}
