use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::muya_clipboard::EditState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionState {
  pub markdown: String,
  pub cursor: usize,
  pub composing: bool,
  pub composition: String,
}

impl CompositionState {
  pub fn new(markdown: impl Into<String>, cursor: usize) -> Self {
    Self { markdown: markdown.into(), cursor, composing: false, composition: String::new() }
  }
}

pub fn table_insert_row(markdown: &str, row_index: usize) -> String {
  let mut lines = markdown.lines().map(str::to_string).collect::<Vec<_>>();
  let Some((start, end)) = find_first_table(&lines) else { return markdown.to_string(); };
  let columns = split_table_row(&lines[start]).len().max(1);
  let row = format!("| {} |", vec![""; columns].join(" | "));
  let insert_at = (start + 2 + row_index).min(end + 1);
  lines.insert(insert_at, row);
  lines.join("\n")
}

pub fn table_insert_column(markdown: &str, column_index: usize) -> String {
  let mut lines = markdown.lines().map(str::to_string).collect::<Vec<_>>();
  let Some((start, end)) = find_first_table(&lines) else { return markdown.to_string(); };
  for line in lines.iter_mut().take(end + 1).skip(start) {
    let mut cells = split_table_row(line);
    let insert_at = column_index.min(cells.len());
    let filler = if line.contains("---") || line.contains(":-") || line.contains("-:") { "---" } else { "" };
    cells.insert(insert_at, filler.to_string());
    *line = format!("| {} |", cells.join(" | "));
  }
  lines.join("\n")
}

pub fn table_contract(markdown: &str) -> Value {
  if let Some((start, end)) = find_first_table(&markdown.lines().map(str::to_string).collect::<Vec<_>>()) {
    let lines = markdown.lines().collect::<Vec<_>>();
    let columns = split_table_row(lines[start]).len();
    json!({ "found": true, "startLine": start + 1, "endLine": end + 1, "columns": columns, "rows": end.saturating_sub(start + 1) })
  } else {
    json!({ "found": false })
  }
}

pub fn image_selection(markdown: &str, cursor: usize) -> Option<Value> {
  let cursor = cursor.min(markdown.len());
  for (start, _) in markdown.match_indices("![") {
    let Some(alt_end_rel) = markdown[start + 2..].find(']') else { continue; };
    let alt_end = start + 2 + alt_end_rel;
    if markdown.as_bytes().get(alt_end + 1) != Some(&b'(') { continue; }
    let Some(url_end_rel) = markdown[alt_end + 2..].find(')') else { continue; };
    let end = alt_end + 2 + url_end_rel + 1;
    if cursor >= start && cursor <= end {
      let alt = &markdown[start + 2..alt_end];
      let url = &markdown[alt_end + 2..end - 1];
      return Some(json!({ "start": start, "end": end, "alt": alt, "url": url }));
    }
  }
  None
}

pub fn start_composition(mut state: CompositionState) -> CompositionState {
  state.composing = true;
  state.composition.clear();
  state
}

pub fn update_composition(mut state: CompositionState, text: &str) -> CompositionState {
  state.composing = true;
  state.composition = text.to_string();
  state
}

pub fn commit_composition(mut state: CompositionState) -> CompositionState {
  if !state.composing { return state; }
  let cursor = state.cursor.min(state.markdown.len());
  state.markdown.insert_str(cursor, &state.composition);
  state.cursor = cursor + state.composition.len();
  state.composition.clear();
  state.composing = false;
  state
}

pub fn cancel_composition(mut state: CompositionState) -> CompositionState {
  state.composing = false;
  state.composition.clear();
  state
}

pub fn editor_snapshot(state: &EditState) -> Value {
  json!({
    "markdown": state.markdown,
    "cursor": state.cursor,
    "selection": state.selection,
    "canUndo": !state.undo_stack.is_empty(),
    "canRedo": !state.redo_stack.is_empty()
  })
}

fn find_first_table(lines: &[String]) -> Option<(usize, usize)> {
  for i in 0..lines.len().saturating_sub(1) {
    if is_table_row(&lines[i]) && is_table_separator(&lines[i + 1]) {
      let mut end = i + 1;
      while end + 1 < lines.len() && is_table_row(&lines[end + 1]) {
        end += 1;
      }
      return Some((i, end));
    }
  }
  None
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

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::muya_clipboard::{paste_text, EditState};

  #[test]
  fn edits_table_rows_and_columns() {
    let table = "| A | B |\n| - | - |\n| 1 | 2 |";
    let with_row = table_insert_row(table, 1);
    assert_eq!(with_row.lines().count(), 4);
    let with_col = table_insert_column(table, 1);
    assert!(with_col.lines().next().unwrap().contains("A |  | B"));
    let contract = table_contract(table);
    assert_eq!(contract["found"], true);
    assert_eq!(contract["columns"], 2);
  }

  #[test]
  fn selects_image_under_cursor() {
    let markdown = "before ![Alt](pic.png) after";
    let image = image_selection(markdown, 10).unwrap();
    assert_eq!(image["alt"], "Alt");
    assert_eq!(image["url"], "pic.png");
  }

  #[test]
  fn handles_ime_lifecycle() {
    let state = CompositionState::new("A", 1);
    let state = start_composition(state);
    let state = update_composition(state, "é");
    assert!(state.composing);
    let state = commit_composition(state);
    assert_eq!(state.markdown, "Aé");
    assert!(!state.composing);
    let state = cancel_composition(start_composition(state));
    assert!(!state.composing);
  }

  #[test]
  fn exposes_editor_snapshot_flags() {
    let state = paste_text(EditState::new("A", 1), "B");
    let snapshot = editor_snapshot(&state);
    assert_eq!(snapshot["canUndo"], true);
    assert_eq!(snapshot["canRedo"], false);
  }
}
