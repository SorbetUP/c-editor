use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::muya_extras::{collect_diagrams, collect_inline_math, collect_block_math, escape_html};
use super::muya_frontmatter::split_yaml_frontmatter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootnoteContract {
  pub label: String,
  pub text: String,
  pub line: usize,
}

pub fn deterministic_contract(markdown: &str) -> Value {
  let (frontmatter, body) = split_yaml_frontmatter(markdown);
  json!({
    "frontmatter": frontmatter,
    "footnotes": footnote_contract(body),
    "htmlPolicy": html_policy_contract(body),
    "math": math_contract(body),
    "diagrams": diagram_contract(body),
    "inlineMarks": inline_mark_contract(body),
    "tables": table_alignment_contract(body)
  })
}

pub fn footnote_contract(markdown: &str) -> Value {
  let definitions = extract_footnote_definitions(markdown);
  let references = extract_footnote_references(markdown);
  let html = if definitions.is_empty() {
    String::new()
  } else {
    let items = definitions.iter().map(|footnote| format!(
      "<li id=\"fn:{}\"><p>{}</p></li>",
      escape_attr(&footnote.label),
      escape_html(&footnote.text)
    )).collect::<Vec<_>>().join("");
    format!("<section class=\"footnotes\"><ol>{}</ol></section>", items)
  };
  json!({ "definitions": definitions, "references": references, "html": html })
}

pub fn extract_footnote_definitions(markdown: &str) -> Vec<FootnoteContract> {
  markdown.lines().enumerate().filter_map(|(line_index, line)| {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("[^")?;
    let (label, text) = rest.split_once("]:")?;
    Some(FootnoteContract { label: label.trim().to_string(), text: text.trim().to_string(), line: line_index + 1 })
  }).collect()
}

pub fn extract_footnote_references(markdown: &str) -> Vec<Value> {
  let mut refs = Vec::new();
  for (line_index, line) in markdown.lines().enumerate() {
    let mut start_at = 0usize;
    while let Some(pos) = line[start_at..].find("[^") {
      let start = start_at + pos;
      if let Some(end_rel) = line[start + 2..].find(']') {
        let label = &line[start + 2..start + 2 + end_rel];
        if !label.is_empty() && !line[start..].contains("]:") {
          refs.push(json!({ "label": label, "line": line_index + 1 }));
        }
        start_at = start + 3 + end_rel;
      } else {
        break;
      }
    }
  }
  refs
}

pub fn html_policy_contract(markdown: &str) -> Value {
  let sanitized = sanitize_html_blocks(markdown);
  json!({
    "sanitized": sanitized,
    "removedDangerousTags": markdown.contains("<script") || markdown.contains("<style"),
    "escapesDangerousProtocols": markdown.contains("javascript:")
  })
}

pub fn sanitize_html_blocks(markdown: &str) -> String {
  let mut out = markdown.to_string();
  out = remove_tag_block(&out, "script");
  out = remove_tag_block(&out, "style");
  out = out.replace("javascript:", "blocked-javascript:");
  out = out.replace("onerror=", "data-blocked-onerror=");
  out = out.replace("onclick=", "data-blocked-onclick=");
  out
}

fn remove_tag_block(input: &str, tag: &str) -> String {
  let mut out = String::new();
  let mut rest = input;
  let open = format!("<{}", tag);
  let close = format!("</{}>", tag);
  while let Some(start) = rest.to_ascii_lowercase().find(&open) {
    out.push_str(&rest[..start]);
    let after_start = &rest[start..];
    if let Some(end_rel) = after_start.to_ascii_lowercase().find(&close) {
      rest = &after_start[end_rel + close.len()..];
    } else {
      rest = "";
      break;
    }
  }
  out.push_str(rest);
  out
}

pub fn math_contract(markdown: &str) -> Value {
  let inline = collect_inline_math(markdown);
  let block = collect_block_math(markdown);
  let html = inline.iter().map(|item| format!("<span class=\"math-inline katex\" data-latex=\"{}\">{}</span>", escape_attr(&item.text), escape_html(&item.text)))
    .chain(block.iter().map(|item| format!("<div class=\"math-block katex-display\" data-latex=\"{}\">{}</div>", escape_attr(&item.text), escape_html(&item.text))))
    .collect::<Vec<_>>();
  json!({ "inline": inline, "block": block, "html": html })
}

pub fn diagram_contract(markdown: &str) -> Value {
  let diagrams = collect_diagrams(markdown);
  let html = diagrams.iter().map(|item| {
    let language = item.attrs.get("language").and_then(Value::as_str).unwrap_or("diagram");
    format!("<div class=\"diagram-block diagram-{}\" data-diagram-language=\"{}\"><pre><code>{}</code></pre></div>", escape_attr(language), escape_attr(language), escape_html(&item.text))
  }).collect::<Vec<_>>();
  json!({ "items": diagrams, "html": html })
}

pub fn inline_mark_contract(markdown: &str) -> Value {
  json!({
    "strongCount": count_occurrences(markdown, "**") / 2 + count_occurrences(markdown, "__") / 2,
    "emphasisCount": count_single_star_emphasis(markdown),
    "strikeCount": count_occurrences(markdown, "~~") / 2,
    "codeSpanCount": count_occurrences(markdown, "`") / 2,
    "nestedExamples": collect_nested_inline_examples(markdown)
  })
}

fn collect_nested_inline_examples(markdown: &str) -> Vec<Value> {
  let mut out = Vec::new();
  if markdown.contains("***") || markdown.contains("**_") || markdown.contains("_**") {
    out.push(json!({ "kind": "strong_emphasis" }));
  }
  if markdown.contains("~~**") || markdown.contains("**~~") {
    out.push(json!({ "kind": "strike_strong" }));
  }
  if markdown.contains("[`") || markdown.contains("`](") {
    out.push(json!({ "kind": "link_code" }));
  }
  out
}

pub fn table_alignment_contract(markdown: &str) -> Value {
  let lines = markdown.lines().collect::<Vec<_>>();
  let mut tables = Vec::new();
  for i in 0..lines.len().saturating_sub(1) {
    if is_table_row(lines[i]) && is_table_separator(lines[i + 1]) {
      let headers = split_table_row(lines[i]);
      let alignments = split_table_row(lines[i + 1]).into_iter().map(|cell| alignment_from_separator(&cell)).collect::<Vec<_>>();
      let mut end = i + 1;
      while end + 1 < lines.len() && is_table_row(lines[end + 1]) { end += 1; }
      tables.push(json!({ "startLine": i + 1, "endLine": end + 1, "headers": headers, "alignments": alignments, "columns": alignments.len(), "rows": end.saturating_sub(i + 1) }));
    }
  }
  json!({ "tables": tables })
}

fn alignment_from_separator(cell: &str) -> &'static str {
  let trimmed = cell.trim();
  let left = trimmed.starts_with(':');
  let right = trimmed.ends_with(':');
  match (left, right) {
    (true, true) => "center",
    (true, false) => "left",
    (false, true) => "right",
    _ => "default",
  }
}

fn is_table_row(line: &str) -> bool {
  line.trim().starts_with('|') && line.trim().ends_with('|') && line.matches('|').count() >= 2
}

fn is_table_separator(line: &str) -> bool {
  is_table_row(line) && line.chars().all(|ch| matches!(ch, '|' | '-' | ':' | ' '))
}

fn split_table_row(line: &str) -> Vec<String> {
  line.trim().trim_matches('|').split('|').map(|cell| cell.trim().to_string()).collect()
}

fn count_occurrences(markdown: &str, needle: &str) -> usize {
  markdown.match_indices(needle).count()
}

fn count_single_star_emphasis(markdown: &str) -> usize {
  let mut count = 0usize;
  let bytes = markdown.as_bytes();
  let mut i = 0usize;
  while i < bytes.len() {
    if bytes[i] == b'*' && bytes.get(i + 1) != Some(&b'*') && (i == 0 || bytes.get(i - 1) != Some(&b'*')) {
      count += 1;
    }
    i += 1;
  }
  count / 2
}

fn escape_attr(input: &str) -> String {
  input.chars().map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ' ' | '+' | '=' | '^' | '/' | '\\') { ch } else { '-' }).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_footnote_definitions_refs_and_html() {
    let contract = footnote_contract("Text[^a]\n\n[^a]: Footnote text");
    assert_eq!(contract["definitions"][0]["label"], "a");
    assert_eq!(contract["references"][0]["label"], "a");
    assert!(contract["html"].as_str().unwrap().contains("footnotes"));
  }

  #[test]
  fn sanitizes_dangerous_html() {
    let contract = html_policy_contract("<script>alert(1)</script><img src=x onerror=bad><a href=\"javascript:x\">x</a>");
    let sanitized = contract["sanitized"].as_str().unwrap();
    assert!(!sanitized.contains("<script"));
    assert!(sanitized.contains("data-blocked-onerror"));
    assert!(sanitized.contains("blocked-javascript"));
  }

  #[test]
  fn creates_katex_like_math_contract() {
    let contract = math_contract("Inline $x+1$\n\n$$\ny=x\n$$");
    assert_eq!(contract["inline"].as_array().unwrap().len(), 1);
    assert_eq!(contract["block"].as_array().unwrap().len(), 1);
    assert!(contract["html"][0].as_str().unwrap().contains("katex"));
    assert!(contract["html"][1].as_str().unwrap().contains("katex-display"));
  }

  #[test]
  fn creates_diagram_contract() {
    let contract = diagram_contract("```mermaid\ngraph TD;\n```");
    assert_eq!(contract["items"][0]["attrs"]["language"], "mermaid");
    assert!(contract["html"][0].as_str().unwrap().contains("diagram-mermaid"));
  }

  #[test]
  fn detects_nested_inline_marks() {
    let contract = inline_mark_contract("***bold em*** and ~~**dead**~~ and [`code`](x)");
    assert!(contract["nestedExamples"].as_array().unwrap().iter().any(|item| item["kind"] == "strong_emphasis"));
    assert!(contract["nestedExamples"].as_array().unwrap().iter().any(|item| item["kind"] == "strike_strong"));
    assert!(contract["nestedExamples"].as_array().unwrap().iter().any(|item| item["kind"] == "link_code"));
  }

  #[test]
  fn detects_table_alignment() {
    let contract = table_alignment_contract("| A | B | C |\n| :- | :-: | -: |\n| 1 | 2 | 3 |");
    assert_eq!(contract["tables"][0]["alignments"][0], "left");
    assert_eq!(contract["tables"][0]["alignments"][1], "center");
    assert_eq!(contract["tables"][0]["alignments"][2], "right");
  }

  #[test]
  fn builds_full_deterministic_contract() {
    let contract = deterministic_contract("---\ntags:\n  - a\n---\n# A\n\nText[^n]\n\n[^n]: note\n\n$x$\n\n| A |\n| :- |\n| 1 |");
    assert_eq!(contract["frontmatter"]["tags"][0], "a");
    assert_eq!(contract["footnotes"]["definitions"][0]["label"], "n");
    assert_eq!(contract["math"]["inline"].as_array().unwrap().len(), 1);
    assert_eq!(contract["tables"]["tables"][0]["alignments"][0], "left");
  }
}
