use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use super::muya_advanced::{apply_advanced_command, MuyaAdvancedCommand};
use super::muya_clipboard_commands::paste_clipboard;
use super::muya_complete::{apply_complete_command, MuyaCompleteCommand};
use super::muya_engine::{
    apply_command, apply_commands, MuyaEditorCommand, MuyaEditorSnapshot, MuyaEditorState,
    MuyaEditorTransaction, MuyaSelection,
};
use super::muya_parity::{apply_parity_command, MuyaParityCommand};
use super::muya_surface::{apply_surface_command, MuyaSurfaceCommand};
use super::muya_ui::{execute_ui_query, MuyaUiQuery};

const HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum MuyaSessionMutation {
    Complete(MuyaCompleteCommand),
    Surface(MuyaSurfaceCommand),
    Advanced(MuyaAdvancedCommand),
}

#[derive(Default)]
pub struct MuyaEngineSessions {
    sessions: Mutex<HashMap<String, MuyaEditorState>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaSessionState {
    pub markdown: String,
    pub selection: MuyaSelection,
    pub revision: u64,
    pub undo_depth: usize,
    pub redo_depth: usize,
}

impl From<&MuyaEditorState> for MuyaSessionState {
    fn from(state: &MuyaEditorState) -> Self {
        Self {
            markdown: state.markdown.clone(),
            selection: state.selection,
            revision: state.revision,
            undo_depth: state.undo_stack.len(),
            redo_depth: state.redo_stack.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaSessionTransaction {
    pub state: MuyaSessionState,
    pub document_changed: bool,
    pub selection_changed: bool,
}

impl From<&MuyaEditorTransaction> for MuyaSessionTransaction {
    fn from(transaction: &MuyaEditorTransaction) -> Self {
        Self {
            state: MuyaSessionState::from(&transaction.state),
            document_changed: transaction.document_changed,
            selection_changed: transaction.selection_changed,
        }
    }
}

fn validate_editor_id(editor_id: &str) -> Result<(), String> {
    let valid = !editor_id.is_empty()
        && editor_id.len() <= 128
        && editor_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        });
    if valid {
        Ok(())
    } else {
        Err("invalid Muya editor session id".to_string())
    }
}

fn lock_sessions(
    sessions: &MuyaEngineSessions,
) -> Result<std::sync::MutexGuard<'_, HashMap<String, MuyaEditorState>>, String> {
    sessions
        .sessions
        .lock()
        .map_err(|_| "Muya editor session store is unavailable".to_string())
}

fn session_transaction(
    sessions: &MuyaEngineSessions,
    editor_id: &str,
    operation: impl FnOnce(MuyaEditorState) -> Result<MuyaEditorTransaction, String>,
) -> Result<MuyaSessionTransaction, String> {
    validate_editor_id(editor_id)?;
    let mut store = lock_sessions(sessions)?;
    let current = store
        .get(editor_id)
        .cloned()
        .ok_or_else(|| format!("Muya editor session not found: {editor_id}"))?;
    let transaction = operation(current)?;
    store.insert(editor_id.to_string(), transaction.state.clone());
    Ok(MuyaSessionTransaction::from(&transaction))
}

fn sync_document(
    mut state: MuyaEditorState,
    markdown: String,
    selection: MuyaSelection,
    continue_group: bool,
) -> Result<MuyaEditorTransaction, String> {
    let before_markdown = state.markdown.clone();
    let before_selection = state.selection;
    if before_markdown == markdown {
        return apply_command(
            state,
            MuyaEditorCommand::SetSelection {
                anchor: selection.anchor,
                focus: selection.focus,
            },
        );
    }

    let current_snapshot = MuyaEditorSnapshot {
        markdown: before_markdown,
        selection: before_selection,
    };
    let grouped_snapshot = if continue_group {
        state.undo_stack.pop()
    } else {
        None
    };

    state.markdown = markdown;
    state = apply_command(
        state,
        MuyaEditorCommand::SetSelection {
            anchor: selection.anchor,
            focus: selection.focus,
        },
    )?
    .state;
    while state.undo_stack.len() >= HISTORY_LIMIT {
        state.undo_stack.remove(0);
    }
    state
        .undo_stack
        .push(grouped_snapshot.unwrap_or(current_snapshot));
    state.redo_stack.clear();
    state.revision = state.revision.saturating_add(1);

    Ok(MuyaEditorTransaction {
        selection_changed: state.selection != before_selection,
        state,
        document_changed: true,
    })
}

#[tauri::command]
pub fn tauri_muya_session_create(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    markdown: String,
) -> Result<MuyaSessionState, String> {
    validate_editor_id(&editor_id)?;
    let state = MuyaEditorState::new(markdown);
    let view = MuyaSessionState::from(&state);
    lock_sessions(&sessions)?.insert(editor_id, state);
    Ok(view)
}

#[tauri::command]
pub fn tauri_muya_session_sync_document(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    markdown: String,
    selection: MuyaSelection,
    continue_group: bool,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| {
        sync_document(state, markdown, selection, continue_group)
    })
}

#[tauri::command]
pub fn tauri_muya_session_apply(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    command: MuyaEditorCommand,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| apply_command(state, command))
}

#[tauri::command]
pub fn tauri_muya_session_apply_parity(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    command: MuyaParityCommand,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| {
        apply_parity_command(state, command)
    })
}

#[tauri::command]
pub fn tauri_muya_session_apply_complete(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    command: MuyaSessionMutation,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| match command {
        MuyaSessionMutation::Complete(command) => apply_complete_command(state, command),
        MuyaSessionMutation::Surface(command) => apply_surface_command(state, command),
        MuyaSessionMutation::Advanced(command) => apply_advanced_command(state, command),
    })
}

#[tauri::command]
pub fn tauri_muya_session_paste_clipboard(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    html: String,
    text: String,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| {
        paste_clipboard(state, html, text)
    })
}

#[tauri::command]
pub fn tauri_muya_session_commit_composition(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    selection: MuyaSelection,
    text: String,
) -> Result<MuyaSessionTransaction, String> {
    session_transaction(&sessions, &editor_id, |state| {
        apply_commands(
            state,
            vec![
                MuyaEditorCommand::SetSelection {
                    anchor: selection.anchor,
                    focus: selection.focus,
                },
                MuyaEditorCommand::ReplaceSelection { text },
            ],
        )
    })
}

#[tauri::command]
pub fn tauri_muya_session_query(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
    query: MuyaUiQuery,
) -> Result<Value, String> {
    validate_editor_id(&editor_id)?;
    let store = lock_sessions(&sessions)?;
    let state = store
        .get(&editor_id)
        .ok_or_else(|| format!("Muya editor session not found: {editor_id}"))?;
    execute_ui_query(Some(state), query)
}

#[tauri::command]
pub fn tauri_muya_session_close(
    sessions: State<'_, MuyaEngineSessions>,
    editor_id: String,
) -> Result<bool, String> {
    validate_editor_id(&editor_id)?;
    Ok(lock_sessions(&sessions)?.remove(&editor_id).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_sync_keeps_one_native_snapshot() {
        let state = MuyaEditorState::new(String::new());
        let first = sync_document(state, "a".to_string(), MuyaSelection::collapsed(1), false)
            .expect("first sync");
        let second = sync_document(
            first.state,
            "ab".to_string(),
            MuyaSelection::collapsed(2),
            true,
        )
        .expect("grouped sync");
        assert_eq!(second.state.undo_stack.len(), 1);
        assert_eq!(second.state.undo_stack[0].markdown, "");
    }

    #[test]
    fn session_view_never_serializes_history_arrays() {
        let mut state = MuyaEditorState::new("hello".to_string());
        state.undo_stack.push(MuyaEditorSnapshot {
            markdown: String::new(),
            selection: MuyaSelection::collapsed(0),
        });
        let json = serde_json::to_value(MuyaSessionState::from(&state)).unwrap();
        assert_eq!(json["undoDepth"], 1);
        assert!(json.get("undoStack").is_none());
    }

    #[test]
    fn advanced_mutations_share_the_same_native_session_history() {
        let state = MuyaEditorState::new("- item".to_string());
        let transaction =
            apply_advanced_command(state, MuyaAdvancedCommand::SmartEnter { shift_key: false })
                .unwrap();
        assert_eq!(transaction.state.markdown, "- item\n- ");
        assert_eq!(transaction.state.undo_stack.len(), 1);
    }

    #[test]
    fn editor_ids_reject_paths() {
        assert!(validate_editor_id("note-1:primary").is_ok());
        assert!(validate_editor_id("../../note").is_err());
    }
}

