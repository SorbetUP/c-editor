use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::muya_compat::render_muya_html;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
  pub start: usize,
  pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditState {
  pub markdown: String,
  pub cursor: usize,
  pub selection: Option<Selection>,
  pub undo_stack: Vec<String>,
  pub redo_stack: Vec<String>,
}

impl EditState {
  pub fn new(markdown: impl Into<String>, cursor: usize) -> Self {
    let markdown = markdown.into();
    let cursor = clamp_boundary(&markdown, cursor);
    Self { markdown, cursor, selection: None, undo_stack: Vec::new(), redo_stack: Vec::new() }
  }
}

pub fn selected_markdown(markdown: &str, selection: Option<Selection>) -> String {
  let Some((start, end)) = normalized_selection(markdown, selection.as_ref()) else { return String::new(); };
  markdown[start..end].to_string()
}

pub fn selected_html(markdown: &str, selection: Option<Selection>) -> String {
  let selected = selected_markdown(markdown, selection);
  if selected.is_empty() { String::new() } else { render_muya_html(&selected) }
}

pub fn paste_text(mut state: EditState, text: &str) -> EditState {
  let previous = state.markdown.clone();
  let (start, end) = selection_or_cursor(&state);
  state.markdown.replace_range(start..end, text);
  state.cursor = clamp_boundary(&state.markdown, start + text.len());
  state.selection = None;
  state.undo_stack.push(previous);
  state.redo_stack.clear();
  state
}

pub fn backspace(mut state: EditState) -> EditState {
  let previous = state.markdown.clone();
  if let Some((start, end)) = normalized_selection(&state.markdown, state.selection.as_ref()) {
    state.markdown.replace_range(start..end, "");
    state.cursor = start;
    state.selection = None;
  } else if state.cursor > 0 {
    let end = clamp_boundary(&state.markdown, state.cursor);
    let start = previous_char_boundary(&state.markdown, end);
    state.markdown.replace_range(start..end, "");
    state.cursor = start;
  }
  if state.markdown != previous {
    state.undo_stack.push(previous);
    state.redo_stack.clear();
  }
  state
}

pub fn remove_next(mut state: EditState) -> EditState {
  let previous = state.markdown.clone();
  if let Some((start, end)) = normalized_selection(&state.markdown, state.selection.as_ref()) {
    state.markdown.replace_range(start..end, "");
    state.cursor = start;
    state.selection = None;
  } else if state.cursor < state.markdown.len() {
    let start = clamp_boundary(&state.markdown, state.cursor);
    let end = next_char_boundary(&state.markdown, start);
    state.markdown.replace_range(start..end, "");
    state.cursor = start;
  }
  if state.markdown != previous {
    state.undo_stack.push(previous);
    state.redo_stack.clear();
  }
  state
}

pub fn undo(mut state: EditState) -> EditState {
  if let Some(previous) = state.undo_stack.pop() {
    let current = state.markdown.clone();
    state.redo_stack.push(current);
    state.cursor = previous.len();
    state.markdown = previous;
    state.selection = None;
  }
  state
}

pub fn redo(mut state: EditState) -> EditState {
  if let Some(next) = state.redo_stack.pop() {
    let current = state.markdown.clone();
    state.undo_stack.push(current);
    state.cursor = next.len();
    state.markdown = next;
    state.selection = None;
  }
  state
}

pub fn clipboard_contract(markdown: &str, selection: Option<Selection>) -> Value {
  json!({
    "markdown": selected_markdown(markdown, selection.clone()),
    "html": selected_html(markdown, selection)
  })
}

fn normalized_selection(markdown: &str, selection: Option<&Selection>) -> Option<(usize, usize)> {
  let selection = selection?;
  let start = clamp_boundary(markdown, selection.start.min(selection.end));
  let end = clamp_boundary(markdown, selection.start.max(selection.end));
  if start == end { None } else { Some((start, end)) }
}

fn selection_or_cursor(state: &EditState) -> (usize, usize) {
  normalized_selection(&state.markdown, state.selection.as_ref()).unwrap_or_else(|| {
    let cursor = clamp_boundary(&state.markdown, state.cursor);
    (cursor, cursor)
  })
}

pub fn clamp_boundary(text: &str, mut index: usize) -> usize {
  if index >= text.len() { return text.len(); }
  while index > 0 && !text.is_char_boundary(index) { index -= 1; }
  index
}

pub fn previous_char_boundary(text: &str, index: usize) -> usize {
  let mut index = clamp_boundary(text, index);
  if index == 0 { return 0; }
  index -= 1;
  while index > 0 && !text.is_char_boundary(index) { index -= 1; }
  index
}

pub fn next_char_boundary(text: &str, index: usize) -> usize {
  let mut index = clamp_boundary(text, index);
  if index >= text.len() { return text.len(); }
  index += 1;
  while index < text.len() && !text.is_char_boundary(index) { index += 1; }
  index
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn copies_markdown_and_html() {
    let markdown = "# A\n\n**bold**";
    let selection = Some(Selection { start: 5, end: markdown.len() });
    assert_eq!(selected_markdown(markdown, selection.clone()), "**bold**");
    assert!(selected_html(markdown, selection).contains("strong"));
  }

  #[test]
  fn pastes_backspaces_removes_and_tracks_history() {
    let state = EditState::new("Hello", 5);
    let state = paste_text(state, " world");
    assert_eq!(state.markdown, "Hello world");
    let state = backspace(state);
    assert_eq!(state.markdown, "Hello worl");
    let state = undo(state);
    assert_eq!(state.markdown, "Hello world");
    let state = redo(state);
    assert_eq!(state.markdown, "Hello worl");
    let state = remove_next(EditState::new("abc", 1));
    assert_eq!(state.markdown, "ac");
  }

  #[test]
  fn replaces_selection_on_paste_and_backspace() {
    let mut state = EditState::new("abc", 1);
    state.selection = Some(Selection { start: 0, end: 2 });
    let state = paste_text(state, "x");
    assert_eq!(state.markdown, "xc");

    let mut state = EditState::new("abc", 1);
    state.selection = Some(Selection { start: 0, end: 2 });
    let state = backspace(state);
    assert_eq!(state.markdown, "c");
  }

  #[test]
  fn keeps_utf8_boundaries() {
    let state = EditState::new("Aé", 3);
    let state = backspace(state);
    assert_eq!(state.markdown, "A");
  }
}
