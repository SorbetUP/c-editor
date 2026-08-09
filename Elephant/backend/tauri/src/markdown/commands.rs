use serde_json::{json, Value};

use super::{parse_markdown_document, render_html, render_plain_text};
use super::muya_clipboard::{backspace, clipboard_contract, paste_text, redo, remove_next, selected_html, selected_markdown, undo, EditState, Selection};
use super::muya_compat::{parse_muya_document, render_muya_html, tokenize_muya};
use super::muya_extras::collect_muya_extras;
use super::muya_interactions::{commit_composition, editor_snapshot, image_selection, start_composition, table_contract, table_insert_column, table_insert_row, update_composition, cancel_composition, CompositionState};
use super::muya_navigation::{detect_input_rule, move_cursor};
use super::parser_v4::{extract_images, extract_links, parse_blocks, split_frontmatter};

#[tauri::command]
pub fn tauri_markdown_parse(markdown: String) -> Value {
  json!(parse_markdown_document(&markdown))
}

#[tauri::command]
pub fn tauri_markdown_render_html(markdown: String) -> Value {
  json!({ "html": render_html(&markdown) })
}

#[tauri::command]
pub fn tauri_markdown_to_text(markdown: String) -> Value {
  let blocks = parse_blocks(&markdown);
  json!({ "text": render_plain_text(&blocks) })
}

#[tauri::command]
pub fn tauri_markdown_extract_frontmatter(markdown: String) -> Value {
  let (frontmatter, body) = split_frontmatter(&markdown);
  json!({ "frontmatter": frontmatter, "body": body })
}

#[tauri::command]
pub fn tauri_markdown_extract_links(markdown: String) -> Value {
  json!({ "links": extract_links(&markdown), "images": extract_images(&markdown) })
}

#[tauri::command]
pub fn tauri_muya_parse(markdown: String) -> Value {
  parse_muya_document(&markdown)
}

#[tauri::command]
pub fn tauri_muya_render_html(markdown: String) -> Value {
  json!({ "html": render_muya_html(&markdown) })
}

#[tauri::command]
pub fn tauri_muya_tokens(markdown: String) -> Value {
  json!({ "tokens": tokenize_muya(&markdown) })
}

#[tauri::command]
pub fn tauri_muya_extras(markdown: String) -> Value {
  json!({ "extras": collect_muya_extras(&markdown) })
}

#[tauri::command]
pub fn tauri_muya_clipboard(markdown: String, selection: Option<Selection>) -> Value {
  clipboard_contract(&markdown, selection)
}

#[tauri::command]
pub fn tauri_muya_copy_markdown(markdown: String, selection: Option<Selection>) -> Value {
  json!({ "markdown": selected_markdown(&markdown, selection) })
}

#[tauri::command]
pub fn tauri_muya_copy_html(markdown: String, selection: Option<Selection>) -> Value {
  json!({ "html": selected_html(&markdown, selection) })
}

#[tauri::command]
pub fn tauri_muya_paste(state: EditState, text: String) -> Value {
  json!(paste_text(state, &text))
}

#[tauri::command]
pub fn tauri_muya_backspace(state: EditState) -> Value {
  json!(backspace(state))
}

#[tauri::command]
pub fn tauri_muya_remove_next(state: EditState) -> Value {
  json!(remove_next(state))
}

#[tauri::command]
pub fn tauri_muya_undo(state: EditState) -> Value {
  json!(undo(state))
}

#[tauri::command]
pub fn tauri_muya_redo(state: EditState) -> Value {
  json!(redo(state))
}

#[tauri::command]
pub fn tauri_muya_move_cursor(markdown: String, cursor: usize, direction: String, extend: bool, anchor: Option<usize>) -> Value {
  move_cursor(&markdown, cursor, &direction, extend, anchor)
}

#[tauri::command]
pub fn tauri_muya_input_rule(line_before_cursor: String) -> Value {
  json!({ "rule": detect_input_rule(&line_before_cursor) })
}

#[tauri::command]
pub fn tauri_muya_table_insert_row(markdown: String, row_index: usize) -> Value {
  json!({ "markdown": table_insert_row(&markdown, row_index) })
}

#[tauri::command]
pub fn tauri_muya_table_insert_column(markdown: String, column_index: usize) -> Value {
  json!({ "markdown": table_insert_column(&markdown, column_index) })
}

#[tauri::command]
pub fn tauri_muya_table_contract(markdown: String) -> Value {
  table_contract(&markdown)
}

#[tauri::command]
pub fn tauri_muya_image_selection(markdown: String, cursor: usize) -> Value {
  json!({ "image": image_selection(&markdown, cursor) })
}

#[tauri::command]
pub fn tauri_muya_start_composition(state: CompositionState) -> Value {
  json!(start_composition(state))
}

#[tauri::command]
pub fn tauri_muya_update_composition(state: CompositionState, text: String) -> Value {
  json!(update_composition(state, &text))
}

#[tauri::command]
pub fn tauri_muya_commit_composition(state: CompositionState) -> Value {
  json!(commit_composition(state))
}

#[tauri::command]
pub fn tauri_muya_cancel_composition(state: CompositionState) -> Value {
  json!(cancel_composition(state))
}

#[tauri::command]
pub fn tauri_muya_editor_snapshot(state: EditState) -> Value {
  editor_snapshot(&state)
}
