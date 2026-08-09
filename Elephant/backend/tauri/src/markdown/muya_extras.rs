use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaExtra {
  pub kind: String,
  pub text: String,
  pub attrs: Value,
  pub line: usize,
}

pub fn collect_muya_extras(markdown: &str) -> Vec<MuyaExtra> {
  let mut extras = Vec::new();
  extras.extend(collect_block_math(markdown));
  extras.extend(collect_inline_math(markdown));
  extras.extend(collect_diagrams(markdown));
  extras.extend(collect_reference_definitions(markdown));
  extras.extend(collect_reference_uses(markdown));
  extras.extend(collect_list_item_meta(markdown));
  extras.sort_by_key(|extra| extra.line);
  extras
}

pub fn render_muya_extras_html(markdown: &str) -> String {
  let mut out = String::new();
  let lines = markdown.lines().collect::<Vec<_>>();
  let mut i = 0;

  while i < lines.len() {
    let line = lines[i];
    let trimmed = line.trim();

    if trimmed == "$$" {
      let start_line = i + 1;
      let mut body = Vec::new();
      i += 1;
      while i < lines.len() && lines[i].trim() != "$$" {
        body.push(lines[i]);
        i += 1;
      }
      if i < lines.len() && lines[i].trim() == "$$" {
        out.push_str(&format!(
          "<div class=\"math-block\" data-muya-type=\"math-block\" data-line=\"{}\">{}</div>\n",
          start_line,
          escape_html(&body.join("\n"))
        ));
        i += 1;
        continue;
      }
      out.push_str(line);
      out.push('\n');
      continue;
    }

    if let Some(language) = fenced_language(trimmed) {
      if is_diagram_language(language) {
        let start_line = i + 1;
        let mut body = Vec::new();
        i += 1;
        while i < lines.len() && !lines[i].trim().starts_with("```") {
          body.push(lines[i]);
          i += 1;
        }
        if i < lines.len() {
          out.push_str(&format!(
            "<div class=\"diagram-block diagram-{}\" data-muya-type=\"diagram\" data-language=\"{}\" data-line=\"{}\"><pre><code>{}</code></pre></div>\n",
            escape_attr(language),
            escape_attr(language),
            start_line,
            escape_html(&body.join("\n"))
          ));
          i += 1;
          continue;
        }
      }
    }

    out.push_str(&render_inline_math_line(line));
    out.push('\n');
    i += 1;
  }

  out
}

pub fn collect_block_math(markdown: &str) -> Vec<MuyaExtra> {
  let lines = markdown.lines().collect::<Vec<_>>();
  let mut extras = Vec::new();
  let mut i = 0;

  while i < lines.len() {
    if lines[i].trim() == "$$" {
      let start_line = i + 1;
      let mut body = Vec::new();
      i += 1;
      while i < lines.len() && lines[i].trim() != "$$" {
        body.push(lines[i]);
        i += 1;
      }
      if i < lines.len() && lines[i].trim() == "$$" {
        extras.push(MuyaExtra {
          kind: "math_block".to_string(),
          text: body.join("\n"),
          attrs: json!({ "display": true }),
          line: start_line,
        });
      }
    }
    i += 1;
  }

  extras
}

pub fn collect_inline_math(markdown: &str) -> Vec<MuyaExtra> {
  let mut extras = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let mut in_math = false;
    let mut start = 0usize;

    for (index, ch) in line.char_indices() {
      if ch != '$' || is_escaped(line, index) || line[index..].starts_with("$$") {
        continue;
      }
      if !in_math {
        in_math = true;
        start = index + 1;
      } else {
        if start <= index {
          let text = line[start..index].trim();
          if !text.is_empty() {
            extras.push(MuyaExtra {
              kind: "inline_math".to_string(),
              text: text.to_string(),
              attrs: json!({ "display": false }),
              line: line_index + 1,
            });
          }
        }
        in_math = false;
      }
    }
  }
  extras
}

pub fn collect_diagrams(markdown: &str) -> Vec<MuyaExtra> {
  let lines = markdown.lines().collect::<Vec<_>>();
  let mut extras = Vec::new();
  let mut i = 0;

  while i < lines.len() {
    let trimmed = lines[i].trim();
    if let Some(language) = fenced_language(trimmed) {
      if is_diagram_language(language) {
        let start_line = i + 1;
        let mut body = Vec::new();
        i += 1;
        while i < lines.len() && !lines[i].trim().starts_with("```") {
          body.push(lines[i]);
          i += 1;
        }
        extras.push(MuyaExtra {
          kind: "diagram".to_string(),
          text: body.join("\n"),
          attrs: json!({ "language": language }),
          line: start_line,
        });
      }
    }
    i += 1;
  }

  extras
}

pub fn collect_reference_definitions(markdown: &str) -> Vec<MuyaExtra> {
  markdown
    .lines()
    .enumerate()
    .filter_map(|(line_index, line)| parse_reference_definition(line).map(|(label, url, title)| MuyaExtra {
      kind: "reference_definition".to_string(),
      text: url.clone(),
      attrs: json!({ "label": label, "url": url, "title": title }),
      line: line_index + 1,
    }))
    .collect()
}

pub fn collect_reference_uses(markdown: &str) -> Vec<MuyaExtra> {
  let mut extras = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    if parse_reference_definition(line).is_some() {
      continue;
    }
    extras.extend(parse_reference_uses_line(line, line_index + 1));
  }
  extras
}

pub fn collect_list_item_meta(markdown: &str) -> Vec<MuyaExtra> {
  markdown
    .lines()
    .enumerate()
    .filter_map(|(line_index, line)| parse_list_item_meta(line).map(|attrs| MuyaExtra {
      kind: "list_item_meta".to_string(),
      text: attrs.get("text").and_then(Value::as_str).unwrap_or_default().to_string(),
      attrs,
      line: line_index + 1,
    }))
    .collect()
}

pub fn is_diagram_language(language: &str) -> bool {
  matches!(language.to_ascii_lowercase().as_str(), "mermaid" | "flowchart" | "sequence" | "sequencechart" | "vega" | "vega-lite" | "vegalite" | "plantuml")
}

fn parse_reference_definition(line: &str) -> Option<(String, String, Option<String>)> {
  let trimmed = line.trim();
  let rest = trimmed.strip_prefix('[')?;
  let (label, after_label) = rest.split_once("]:")?;
  let label = label.trim();
  if label.is_empty() { return None; }
  let rest = after_label.trim();
  if rest.is_empty() { return None; }
  let (url, title) = split_url_title(rest);
  Some((label.to_string(), url, title))
}

fn split_url_title(input: &str) -> (String, Option<String>) {
  let mut parts = input.splitn(2, char::is_whitespace);
  let url = parts.next().unwrap_or_default().trim_matches('<').trim_matches('>').to_string();
  let title = parts.next().map(str::trim).filter(|value| !value.is_empty()).map(|value| value.trim_matches('"').trim_matches('\'').trim_matches('(').trim_matches(')').to_string());
  (url, title)
}

fn parse_reference_uses_line(line: &str, line_number: usize) -> Vec<MuyaExtra> {
  let mut extras = Vec::new();
  let bytes = line.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let is_image = bytes.get(i) == Some(&b'!') && bytes.get(i + 1) == Some(&b'[');
    let start = if is_image { i + 1 } else { i };
    if bytes.get(start) != Some(&b'[') {
      i += 1;
      continue;
    }
    let Some(label_end_rel) = line[start + 1..].find(']') else { i += 1; continue; };
    let label_end = start + 1 + label_end_rel;
    if bytes.get(label_end + 1) != Some(&b'[') {
      i += 1;
      continue;
    }
    let Some(reference_end_rel) = line[label_end + 2..].find(']') else { i += 1; continue; };
    let reference_end = label_end + 2 + reference_end_rel;
    let label = &line[start + 1..label_end];
    let reference = &line[label_end + 2..reference_end];
    if !label.is_empty() && !reference.is_empty() {
      extras.push(MuyaExtra {
        kind: if is_image { "reference_image" } else { "reference_link" }.to_string(),
        text: label.to_string(),
        attrs: json!({ "label": label, "reference": reference }),
        line: line_number,
      });
    }
    i = reference_end + 1;
  }
  extras
}

fn parse_list_item_meta(line: &str) -> Option<Value> {
  let indent_spaces = line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').map(|ch| if ch == '\t' { 4 } else { 1 }).sum::<usize>();
  let depth = indent_spaces / 2;
  let trimmed = line.trim_start();

  let (ordered, marker, rest) = if let Some((number, rest)) = trimmed.split_once(". ") {
    if number.chars().all(|ch| ch.is_ascii_digit()) {
      (true, format!("{}.", number), rest)
    } else {
      return None;
    }
  } else if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")).or_else(|| trimmed.strip_prefix("+ ")) {
    (false, trimmed.chars().next().unwrap_or('-').to_string(), rest)
  } else {
    return None;
  };

  let (checked, text) = if let Some(text) = rest.strip_prefix("[ ] ") {
    (Some(false), text)
  } else if let Some(text) = rest.strip_prefix("[x] ").or_else(|| rest.strip_prefix("[X] ")) {
    (Some(true), text)
  } else {
    (None, rest)
  };

  Some(json!({
    "depth": depth,
    "ordered": ordered,
    "marker": marker,
    "checked": checked,
    "text": text.trim()
  }))
}

fn fenced_language(line: &str) -> Option<&str> {
  let rest = line.strip_prefix("```")?.trim();
  if rest.is_empty() { None } else { rest.split_whitespace().next() }
}

fn render_inline_math_line(line: &str) -> String {
  let mut out = String::new();
  let mut last = 0usize;
  let mut start: Option<usize> = None;

  for (index, ch) in line.char_indices() {
    if ch != '$' || is_escaped(line, index) || line[index..].starts_with("$$") {
      continue;
    }

    if let Some(math_start) = start.take() {
      let text = line[math_start..index].trim();
      if !text.is_empty() {
        out.push_str(&line[last..math_start - 1]);
        out.push_str("<span class=\"math-inline\" data-muya-type=\"math-inline\">");
        out.push_str(&escape_html(text));
        out.push_str("</span>");
        last = index + 1;
      }
    } else {
      start = Some(index + 1);
    }
  }

  out.push_str(&line[last..]);
  out
}

fn is_escaped(line: &str, index: usize) -> bool {
  if index == 0 { return false; }
  let prefix = &line[..index];
  let backslashes = prefix.chars().rev().take_while(|ch| *ch == '\\').count();
  backslashes % 2 == 1
}

pub fn escape_html(input: &str) -> String {
  input
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

fn escape_attr(input: &str) -> String {
  input
    .chars()
    .map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn collects_inline_and_block_math() {
    let extras = collect_muya_extras("Euler $e^{i\\pi}+1=0$\n\n$$\na^2+b^2=c^2\n$$");
    assert!(extras.iter().any(|extra| extra.kind == "inline_math" && extra.text.contains("e^{i")));
    assert!(extras.iter().any(|extra| extra.kind == "math_block" && extra.text.contains("a^2")));
  }

  #[test]
  fn collects_diagram_fences() {
    let extras = collect_muya_extras("```mermaid\ngraph TD; A-->B;\n```");
    assert_eq!(extras[0].kind, "diagram");
    assert_eq!(extras[0].attrs["language"], "mermaid");
  }

  #[test]
  fn renders_math_and_diagram_placeholders_without_breaking_markdown() {
    let html = render_muya_extras_html("# Title\n\nText $x+1$\n\n$$\ny=x\n$$\n\n```mermaid\ngraph TD;\n```");
    assert!(html.contains("# Title"));
    assert!(html.contains("math-inline"));
    assert!(html.contains("math-block"));
    assert!(html.contains("diagram-block"));
    assert!(html.contains("data-language=\"mermaid\""));
  }

  #[test]
  fn ignores_escaped_inline_math_marker() {
    let extras = collect_inline_math("Price is \\$5 and math $x$.");
    assert_eq!(extras.len(), 1);
    assert_eq!(extras[0].text, "x");
  }

  #[test]
  fn collects_reference_definitions_and_uses() {
    let extras = collect_muya_extras("[ref]: https://example.com \"Title\"\n\nUse [label][ref] and ![alt][ref].");
    assert!(extras.iter().any(|extra| extra.kind == "reference_definition" && extra.attrs["label"] == "ref"));
    assert!(extras.iter().any(|extra| extra.kind == "reference_link" && extra.attrs["reference"] == "ref"));
    assert!(extras.iter().any(|extra| extra.kind == "reference_image" && extra.attrs["reference"] == "ref"));
  }

  #[test]
  fn collects_nested_list_item_metadata() {
    let extras = collect_list_item_meta("- root\n  - child\n    1. ordered\n  - [x] checked");
    assert!(extras.iter().any(|extra| extra.attrs["depth"] == 0 && extra.attrs["ordered"] == false));
    assert!(extras.iter().any(|extra| extra.attrs["depth"] == 1 && extra.attrs["text"] == "child"));
    assert!(extras.iter().any(|extra| extra.attrs["ordered"] == true && extra.attrs["marker"] == "1."));
    assert!(extras.iter().any(|extra| extra.attrs["checked"] == true));
  }
}
