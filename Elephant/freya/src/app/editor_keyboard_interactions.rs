//! Keyboard event boundary for the native editor.

use freya::{
    prelude::*,
    text_edit::{EditableEvent, TextEditor, UseEditable},
};
use muya_core::NodeId;

use super::super::{
    apply_inline_delta, contains_node, sync_editable_from_muya, sync_muya_selection, ShellState,
};

#[path = "editor_key_routes.rs"]
mod editor_key_routes;

pub(crate) fn key_down_handler(
    mut state: State<ShellState>,
    node_id: NodeId,
    mut editable: UseEditable,
    previous_value: String,
    mut autosave_generation: State<u64>,
) -> impl FnMut(Event<KeyboardEventData>) {
    move |event: Event<KeyboardEventData>| {
        if event.key == Key::Named(NamedKey::End)
            && event.modifiers.contains(Modifiers::ctrl_or_meta())
        {
            // Keep the shared Tauri shortcut from being reinterpreted by the
            // parent ScrollView as a jump-to-end command.
            event.stop_propagation();
        }
        let stale_focus = state.read().editor.as_ref().is_some_and(|editor| {
            editor
                .focus_target()
                .is_some_and(|target| !contains_node(editor.session().document(), node_id, target))
        });
        let composition_active = state
            .read()
            .editor
            .as_ref()
            .is_some_and(|editor| editor.snapshot().composition_active);
        if !stale_focus && !composition_active {
            if let Err(error) = sync_muya_selection(state, node_id, &editable) {
                state.write().error = Some(error.clone());
                eprintln!("[freya][editor] action:failure action=selection error={error}");
                return;
            }
        }

        let result = stale_focus
            .then(|| editor_key_routes::route_focused_text(state, &event.key, event.modifiers))
            .flatten()
            .or_else(|| editor_key_routes::route_key(state, node_id, &event.key, event.modifiers))
            .unwrap_or_else(|| {
                editable.process_event(EditableEvent::KeyDown {
                    key: &event.key,
                    modifiers: event.modifiers,
                });
                let next_value = editable.editor().read().committed_text();
                if next_value == previous_value {
                    sync_muya_selection(state, node_id, &editable)
                } else {
                    apply_inline_delta(state, node_id, &previous_value, &next_value)
                }
            });

        match result {
            Ok(()) => {
                *autosave_generation.write() += 1;
                sync_editable_from_muya(state, node_id, &mut editable);
                eprintln!(
                    "[freya][editor] action:complete action=keyboard node={:?} key={}",
                    node_id, event.key
                );
            }
            Err(error) => {
                let error = error.to_string();
                state.write().error = Some(error.clone());
                editable.editor_mut().write().set(&previous_value);
                eprintln!("[freya][editor] action:failure action=keyboard error={error}");
            }
        }
    }
}
