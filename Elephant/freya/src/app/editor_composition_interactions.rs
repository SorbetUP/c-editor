//! IME preedit, commit and cancel routing.

use freya::{prelude::*, text_edit::UseEditable};
use muya_core::NodeId;

use crate::editor::EditorAction;

use super::super::{sync_editable_from_muya, sync_muya_selection, ShellState};

pub(crate) fn ime_preedit_handler(
    mut state: State<ShellState>,
    node_id: NodeId,
    mut editable: UseEditable,
    mut autosave_generation: State<u64>,
) -> impl FnMut(Event<ImePreeditEventData>) {
    move |event: Event<ImePreeditEventData>| {
        if let Err(error) = sync_muya_selection(state, node_id, &editable) {
            state.write().error = Some(error.clone());
            eprintln!("[freya][editor] action:failure action=ime-selection error={error}");
            return;
        }
        let result = {
            let mut shell = state.write();
            let editor = shell
                .editor
                .as_mut()
                .ok_or_else(|| "cannot compose without an open note".to_string());
            editor.and_then(|editor| {
                if event.text.is_empty() {
                    if editor.snapshot().composition_active {
                        editor
                            .dispatch(EditorAction::CancelComposition)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    } else {
                        Ok(())
                    }
                } else {
                    if !editor.snapshot().composition_active {
                        editor
                            .dispatch(EditorAction::BeginComposition)
                            .map_err(|error| error.to_string())?;
                    }
                    editor
                        .dispatch(EditorAction::UpdateComposition(event.text.clone()))
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                }
            })
        };
        match result {
            Ok(()) => {
                *autosave_generation.write() += 1;
                sync_editable_from_muya(state, node_id, &mut editable);
                eprintln!(
                    "[freya][editor] action:complete action=ime-preedit node={:?}",
                    node_id
                );
            }
            Err(error) => {
                eprintln!("[freya][editor] action:failure action=ime-preedit error={error}");
                state.write().error = Some(error);
            }
        }
    }
}
