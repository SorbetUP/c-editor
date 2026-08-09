use serde_json::{json, Value};

use super::muya_clipboard::{clamp_boundary, next_char_boundary, previous_char_boundary};

pub fn move_cursor(markdown: &str, cursor: usize, direction: &str, extend: bool, anchor: Option<usize>) -> Value {
  let cursor = clamp_boundary(markdown, cursor);
  let next = match direction {
    "left" => previous_char_boundary(markdown, cursor),
    "right" => next_char_boundary(markdown, cursor),
    "up" => vertical_move(markdown, cursor, -1),
    "down" => vertical_move(markdown, cursor, 1),
    "line_start" => line_bounds(markdown, cursor).0,
    "line_end" => line_bounds(markdown, cursor).1,
    "document_start" => 0,
    "document_end" => markdown.len(),
    _ => cursor,
  };
  let selection = if extend {
    let start = anchor.unwrap_or(cursor);
    Some(json!({ "start": start.min(next), "end": start.max(next), "anchor": start }))
  } else {
    None
  };
  json!({ "cursor": next, "selection": selection })
}

pub fn detect_input_rule(line_before_cursor: &str) -> Option<Value> {
  let line = line_before_cursor.trim_start();
  let indent = line_before_cursor.len().saturating_sub(line.len());
  if line == "---" || line == "***" || line == "___" {
    return Some(json!({ "kind": "hr", "indent": indent }));
  }
  if line == "```" {
    return Some(json!({ "kind": "code_fence", "language": "", "indent": indent }));
  }
  if let Some(language) = line.strip_prefix("```") {
    if !language.trim().is_empty() {
      return Some(json!({ "kind": "code_fence", "language": language.trim(), "indent": indent }));
    }
  }
  if line == "$$" {
    return Some(json!({ "kind": "math_block", "indent": indent }));
  }
  let level = line.chars().take_while(|ch| *ch == '#').count();
  if (1..=6).contains(&level) && line.chars().nth(level) == Some(' ') {
    return Some(json!({ "kind": "heading", "level": level, "indent": indent }));
  }
  if line == "> " || line == ">" {
    return Some(json!({ "kind": "blockquote", "indent": indent }));
  }
  for marker in ["-", "*", "+"] {
    if line == marker || line == format!("{} ", marker) {
      return Some(json!({ "kind": "bullet_list", "marker": marker, "indent": indent }));
    }
  }
  if line == "- [ ] " || line == "- [ ]" {
    return Some(json!({ "kind": "task_list", "checked": false, "indent": indent }));
  }
  if line == "- [x] " || line == "- [x]" || line == "- [X] " || line == "- [X]" {
    return Some(json!({ "kind": "task_list", "checked": true, "indent": indent }));
  }
  if let Some((number, rest)) = line.split_once(". ") {
    if number.chars().all(|ch| ch.is_ascii_digit()) && rest.is_empty() {
      return Some(json!({ "kind": "ordered_list", "number": number.parse::<usize>().unwrap_or(1), "indent": indent }));
    }
  }
  None
}

fn line_bounds(text: &str, cursor: usize) -> (usize, usize) {
  let cursor = clamp_boundary(text, cursor);
  let start = text[..cursor].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
  let end = text[cursor..].find('\n').map(|pos| cursor + pos).unwrap_or(text.len());
  (start, end)
}

fn vertical_move(text: &str, cursor: usize, delta: isize) -> usize {
  let (line_start, _) = line_bounds(text, cursor);
  let column = cursor.saturating_sub(line_start);
  if delta < 0 {
    if line_start == 0 { return cursor; }
    let previous_end = line_start - 1;
    let previous_start = text[..previous_end].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    (previous_start + column).min(previous_end)
  } else {
    let (_, line_end) = line_bounds(text, cursor);
    if line_end >= text.len() { return cursor; }
    let next_start = line_end + 1;
    let next_end = text[next_start..].find('\n').map(|pos| next_start + pos).unwrap_or(text.len());
    (next_start + column).min(next_end)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn moves_cursor_in_all_directions() {
    let text = "abc\ndef";
    assert_eq!(move_cursor(text, 1, "right", false, None)["cursor"], 2);
    assert_eq!(move_cursor(text, 1, "left", false, None)["cursor"], 0);
    assert_eq!(move_cursor(text, 5, "up", false, None)["cursor"], 1);
    assert_eq!(move_cursor(text, 1, "down", false, None)["cursor"], 5);
    assert_eq!(move_cursor(text, 5, "line_start", false, None)["cursor"], 4);
    assert_eq!(move_cursor(text, 5, "line_end", false, None)["cursor"], 7);
  }

  #[test]
  fn extends_selection_when_requested() {
    let result = move_cursor("abc", 1, "right", true, Some(1));
    assert_eq!(result["selection"]["start"], 1);
    assert_eq!(result["selection"]["end"], 2);
  }

  #[test]
  fn detects_input_rules() {
    assert_eq!(detect_input_rule("# ").unwrap()["kind"], "heading");
    assert_eq!(detect_input_rule("- ").unwrap()["kind"], "bullet_list");
    assert_eq!(detect_input_rule("1. ").unwrap()["kind"], "ordered_list");
    assert_eq!(detect_input_rule("- [x] ").unwrap()["kind"], "task_list");
    assert_eq!(detect_input_rule("> ").unwrap()["kind"], "blockquote");
    assert_eq!(detect_input_rule("---").unwrap()["kind"], "hr");
    assert_eq!(detect_input_rule("```mermaid").unwrap()["language"], "mermaid");
    assert_eq!(detect_input_rule("$$").unwrap()["kind"], "math_block");
  }
}
