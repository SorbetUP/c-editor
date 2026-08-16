//! Muya command routing for keyboard shortcuts and table navigation.

use freya::prelude::*;
use muya_core::{features::TableNavigationCommand, NodeId};

use crate::editor::EditorAction;

use super::super::super::{has_table_ancestor, ShellState};
use super::super::editor_clipboard_interactions;

pub(super) fn route_key(
    mut state: State<ShellState>,
    node_id: NodeId,
    key: &Key,
    modifiers: Modifiers,
) -> Option<Result<(), String>> {
    if let Some(result) = editor_clipboard_interactions::route_clipboard(state, key, modifiers) {
        return Some(result);
    }
    let result = match key {
        Key::Named(NamedKey::End) if modifiers.contains(Modifiers::ctrl_or_meta()) => Some(
            with_editor(state, "cannot move without an open note", |editor| {
                editor.move_caret_to_end_of_block(node_id).map(|_| ())
            }),
        ),
        Key::Named(NamedKey::Enter) if modifiers.is_empty() => Some(with_editor(
            state,
            "cannot split without an open note",
            |editor| editor.insert_paragraph().map(|_| ()),
        )),
        Key::Named(NamedKey::Backspace) if modifiers.is_empty() => Some(with_editor(
            state,
            "cannot delete without an open note",
            |editor| editor.delete_backward().map(|_| ()),
        )),
        Key::Named(NamedKey::Delete) if modifiers.is_empty() => Some(with_editor(
            state,
            "cannot delete without an open note",
            |editor| editor.delete_forward().map(|_| ()),
        )),
        Key::Named(NamedKey::Tab) if !modifiers.contains(Modifiers::ALT) => {
            let inside_table = state
                .read()
                .editor
                .as_ref()
                .is_some_and(|editor| has_table_ancestor(editor.session().document(), node_id));
            inside_table.then(|| {
                let command = if modifiers.contains(Modifiers::SHIFT) {
                    TableNavigationCommand::PreviousCell
                } else {
                    TableNavigationCommand::NextCell
                };
                with_editor(state, "cannot navigate without an open note", |editor| {
                    editor
                        .dispatch(EditorAction::TableNavigation(command))
                        .map(|_| ())
                })
            })
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && character.eq_ignore_ascii_case("s") =>
        {
            Some(
                state
                    .write()
                    .save_open_editor()
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
            )
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && character.eq_ignore_ascii_case("b") =>
        {
            Some(dispatch(
                state,
                EditorAction::ToggleStrong,
                "cannot format without an open note",
            ))
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && character.eq_ignore_ascii_case("i") =>
        {
            Some(dispatch(
                state,
                EditorAction::ToggleEmphasis,
                "cannot format without an open note",
            ))
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && modifiers.contains(Modifiers::SHIFT)
                && character.eq_ignore_ascii_case("x") =>
        {
            Some(dispatch(
                state,
                EditorAction::ToggleStrike,
                "cannot format without an open note",
            ))
        }
        Key::Character(character)
            if state
                .read()
                .editor
                .as_ref()
                .is_some_and(|editor| editor.snapshot().composition_active) =>
        {
            let text = character.clone();
            Some(with_editor(
                state,
                "cannot commit without an open note",
                |editor| {
                    editor.dispatch(EditorAction::UpdateComposition(text))?;
                    editor.dispatch(EditorAction::CommitComposition).map(|_| ())
                },
            ))
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && character.eq_ignore_ascii_case("z") =>
        {
            let redo = modifiers.contains(Modifiers::SHIFT);
            Some(with_editor(
                state,
                "cannot change history without an open note",
                |editor| {
                    if redo {
                        editor.redo().map(|_| ())
                    } else {
                        editor.undo().map(|_| ())
                    }
                },
            ))
        }
        Key::Character(character)
            if modifiers.contains(Modifiers::ctrl_or_meta())
                && character.eq_ignore_ascii_case("y") =>
        {
            Some(with_editor(
                state,
                "cannot change history without an open note",
                |editor| editor.redo().map(|_| ()),
            ))
        }
        _ => None,
    };
    result
}

pub(super) fn route_focused_text(
    state: State<ShellState>,
    key: &Key,
    modifiers: Modifiers,
) -> Option<Result<(), String>> {
    let Key::Character(character) = key else {
        return None;
    };
    if !modifiers.is_empty() {
        return None;
    }
    Some(with_editor(
        state,
        "cannot type without an open note",
        |editor| editor.dispatch_text(character.clone()).map(|_| ()),
    ))
}

fn dispatch(state: State<ShellState>, action: EditorAction, missing: &str) -> Result<(), String> {
    with_editor(state, missing, |editor| editor.dispatch(action).map(|_| ()))
}

fn with_editor<T>(
    mut state: State<ShellState>,
    missing: &str,
    action: impl FnOnce(&mut crate::editor::EditorDocument) -> Result<T, crate::editor::EditorError>,
) -> Result<T, String> {
    state
        .write()
        .editor
        .as_mut()
        .ok_or_else(|| missing.to_string())
        .and_then(|editor| action(editor).map_err(|error| error.to_string()))
}
