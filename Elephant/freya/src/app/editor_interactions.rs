//! Focused interaction modules for the native Muya editor.

#[path = "editor_clipboard_interactions.rs"]
mod editor_clipboard_interactions;
#[path = "editor_composition_interactions.rs"]
mod editor_composition_interactions;
#[path = "editor_keyboard_interactions.rs"]
mod editor_keyboard_interactions;
#[path = "editor_task_interactions.rs"]
mod editor_task_interactions;

pub(super) use editor_composition_interactions::ime_preedit_handler;
pub(super) use editor_keyboard_interactions::key_down_handler;
pub(super) use editor_task_interactions::task_marker;
