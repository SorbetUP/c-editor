//! Domain actions for the note editor tag rail.
//!
//! The view owns only the transient form state. This module owns the
//! persisted tag transition so add, edit, and delete all use one production
//! write path.

use super::ShellState;
use freya::prelude::State;

pub(super) fn persist_tags(
    mut state: State<ShellState>,
    tags: &[String],
    title: &str,
) -> Result<(), String> {
    state
        .write()
        .editor
        .as_mut()
        .ok_or_else(|| "cannot update tags without an open note".to_string())
        .and_then(|editor| {
            editor
                .update_tags(tags, title)
                .map_err(|error| error.to_string())
                .and_then(|_| editor.save().map_err(|error| error.to_string()))
        })
}

pub(super) fn delete_tag(
    mut state: State<ShellState>,
    current_tags: &[String],
    title: &str,
    index: usize,
) {
    if index >= current_tags.len() {
        return;
    }
    let mut next_tags = current_tags.to_vec();
    next_tags.remove(index);
    match persist_tags(state.clone(), &next_tags, title) {
        Ok(()) => eprintln!("[freya][editor] action=delete-tag status=complete index={index}"),
        Err(error) => {
            eprintln!("[freya][editor] action=delete-tag status=failure error={error}");
            state.write().error = Some(error);
        }
    }
}
