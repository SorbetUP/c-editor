use serde::{Deserialize, Serialize};

const MAX_HISTORY_ENTRIES: usize = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaSelection {
    pub anchor: usize,
    pub focus: usize,
}

impl MuyaSelection {
    pub fn collapsed(offset: usize) -> Self {
        Self {
            anchor: offset,
            focus: offset,
        }
    }

    pub fn start(self) -> usize {
        self.anchor.min(self.focus)
    }

    pub fn end(self) -> usize {
        self.anchor.max(self.focus)
    }

    pub fn is_collapsed(self) -> bool {
        self.anchor == self.focus
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaEditorSnapshot {
    pub markdown: String,
    pub selection: MuyaSelection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaEditorState {
    pub markdown: String,
    pub selection: MuyaSelection,
    pub revision: u64,
    #[serde(default)]
    pub undo_stack: Vec<MuyaEditorSnapshot>,
    #[serde(default)]
    pub redo_stack: Vec<MuyaEditorSnapshot>,
}

impl MuyaEditorState {
    pub fn new(markdown: String) -> Self {
        let cursor = utf16_len(&markdown);
        Self {
            markdown,
            selection: MuyaSelection::collapsed(cursor),
            revision: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn snapshot(&self) -> MuyaEditorSnapshot {
        MuyaEditorSnapshot {
            markdown: self.markdown.clone(),
            selection: self.selection,
        }
    }

    fn restore(&mut self, snapshot: MuyaEditorSnapshot) {
        self.markdown = snapshot.markdown;
        self.selection = clamp_selection(&self.markdown, snapshot.selection);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MuyaEditorCommand {
    InsertText { text: String },
    ReplaceSelection { text: String },
    DeleteBackward,
    DeleteForward,
    SetSelection { anchor: usize, focus: usize },
    ToggleInline { marker: String },
    TransformBlock { kind: String },
    InsertLineBreak,
    Undo,
    Redo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaEditorTransaction {
    pub state: MuyaEditorState,
    pub document_changed: bool,
    pub selection_changed: bool,
}

pub fn apply_command(
    mut state: MuyaEditorState,
    command: MuyaEditorCommand,
) -> Result<MuyaEditorTransaction, String> {
    state.selection = clamp_selection(&state.markdown, state.selection);
    let before_markdown = state.markdown.clone();
    let before_selection = state.selection;

    match command {
        MuyaEditorCommand::Undo => apply_undo(&mut state),
        MuyaEditorCommand::Redo => apply_redo(&mut state),
        MuyaEditorCommand::SetSelection { anchor, focus } => {
            state.selection = clamp_selection(&state.markdown, MuyaSelection { anchor, focus });
        }
        command => {
            let snapshot = state.snapshot();
            apply_mutation(&mut state, command)?;
            if state.markdown != before_markdown {
                push_history(&mut state.undo_stack, snapshot);
                state.redo_stack.clear();
                state.revision = state.revision.saturating_add(1);
            }
        }
    }

    let document_changed = state.markdown != before_markdown;
    let selection_changed = state.selection != before_selection;
    Ok(MuyaEditorTransaction {
        state,
        document_changed,
        selection_changed,
    })
}

pub fn apply_commands(
    mut state: MuyaEditorState,
    commands: Vec<MuyaEditorCommand>,
) -> Result<MuyaEditorTransaction, String> {
    let initial_markdown = state.markdown.clone();
    let initial_selection = state.selection;
    for command in commands {
        state = apply_command(state, command)?.state;
    }
    Ok(MuyaEditorTransaction {
        document_changed: state.markdown != initial_markdown,
        selection_changed: state.selection != initial_selection,
        state,
    })
}

fn apply_mutation(state: &mut MuyaEditorState, command: MuyaEditorCommand) -> Result<(), String> {
    match command {
        MuyaEditorCommand::InsertText { text } | MuyaEditorCommand::ReplaceSelection { text } => {
            replace_selection(state, &text);
        }
        MuyaEditorCommand::DeleteBackward => delete_backward(state),
        MuyaEditorCommand::DeleteForward => delete_forward(state),
        MuyaEditorCommand::ToggleInline { marker } => toggle_inline(state, &marker)?,
        MuyaEditorCommand::TransformBlock { kind } => transform_current_block(state, &kind)?,
        MuyaEditorCommand::InsertLineBreak => replace_selection(state, "\n"),
        MuyaEditorCommand::SetSelection { .. }
        | MuyaEditorCommand::Undo
        | MuyaEditorCommand::Redo => {}
    }
    Ok(())
}

fn apply_undo(state: &mut MuyaEditorState) {
    let Some(previous) = state.undo_stack.pop() else {
        return;
    };
    let current = state.snapshot();
    push_history(&mut state.redo_stack, current);
    state.restore(previous);
    state.revision = state.revision.saturating_add(1);
}

fn apply_redo(state: &mut MuyaEditorState) {
    let Some(next) = state.redo_stack.pop() else {
        return;
    };
    let current = state.snapshot();
    push_history(&mut state.undo_stack, current);
    state.restore(next);
    state.revision = state.revision.saturating_add(1);
}

fn push_history(stack: &mut Vec<MuyaEditorSnapshot>, snapshot: MuyaEditorSnapshot) {
    while stack.len() >= MAX_HISTORY_ENTRIES {
        stack.remove(0);
    }
    stack.push(snapshot);
}

fn replace_selection(state: &mut MuyaEditorState, replacement: &str) {
    let selection = clamp_selection(&state.markdown, state.selection);
    let start_utf16 = selection.start();
    let start_byte = utf16_to_byte_index(&state.markdown, start_utf16);
    let end_byte = utf16_to_byte_index(&state.markdown, selection.end());
    state
        .markdown
        .replace_range(start_byte..end_byte, replacement);
    state.selection = MuyaSelection::collapsed(start_utf16 + utf16_len(replacement));
}

fn delete_backward(state: &mut MuyaEditorState) {
    if !state.selection.is_collapsed() {
        replace_selection(state, "");
        return;
    }

    let cursor_utf16 = state.selection.focus.min(utf16_len(&state.markdown));
    let cursor_byte = utf16_to_byte_index(&state.markdown, cursor_utf16);
    if cursor_byte == 0 {
        return;
    }
    let previous_byte = state.markdown[..cursor_byte]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
        .unwrap_or(0);
    state.markdown.replace_range(previous_byte..cursor_byte, "");
    state.selection = MuyaSelection::collapsed(byte_to_utf16_index(&state.markdown, previous_byte));
}

fn delete_forward(state: &mut MuyaEditorState) {
    if !state.selection.is_collapsed() {
        replace_selection(state, "");
        return;
    }

    let cursor_utf16 = state.selection.focus.min(utf16_len(&state.markdown));
    let cursor_byte = utf16_to_byte_index(&state.markdown, cursor_utf16);
    let Some(character) = state.markdown[cursor_byte..].chars().next() else {
        return;
    };
    let next_byte = cursor_byte + character.len_utf8();
    state.markdown.replace_range(cursor_byte..next_byte, "");
    state.selection = MuyaSelection::collapsed(cursor_utf16);
}

fn toggle_inline(state: &mut MuyaEditorState, marker: &str) -> Result<(), String> {
    if !matches!(marker, "**" | "*" | "~~" | "`" | "==") {
        return Err(format!("unsupported Muya inline marker: {marker}"));
    }

    let selection = clamp_selection(&state.markdown, state.selection);
    let start_utf16 = selection.start();
    let start_byte = utf16_to_byte_index(&state.markdown, start_utf16);
    let end_byte = utf16_to_byte_index(&state.markdown, selection.end());
    let marker_utf16 = utf16_len(marker);
    let marker_start = start_byte.saturating_sub(marker.len());
    let marker_end = end_byte.saturating_add(marker.len());

    if selection.is_collapsed() {
        let pair = format!("{marker}{marker}");
        state.markdown.insert_str(start_byte, &pair);
        state.selection = MuyaSelection::collapsed(start_utf16 + marker_utf16);
        return Ok(());
    }

    if start_byte >= marker.len()
        && marker_end <= state.markdown.len()
        && state.markdown.is_char_boundary(marker_start)
        && state.markdown.is_char_boundary(marker_end)
        && &state.markdown[marker_start..start_byte] == marker
        && &state.markdown[end_byte..marker_end] == marker
    {
        state.markdown.replace_range(end_byte..marker_end, "");
        state.markdown.replace_range(marker_start..start_byte, "");
        state.selection = MuyaSelection {
            anchor: start_utf16.saturating_sub(marker_utf16),
            focus: selection.end().saturating_sub(marker_utf16),
        };
        return Ok(());
    }

    let selected = state.markdown[start_byte..end_byte].to_string();
    let wrapped = format!("{marker}{selected}{marker}");
    state.markdown.replace_range(start_byte..end_byte, &wrapped);
    state.selection = MuyaSelection {
        anchor: start_utf16 + marker_utf16,
        focus: start_utf16 + marker_utf16 + utf16_len(&selected),
    };
    Ok(())
}

fn transform_current_block(state: &mut MuyaEditorState, kind: &str) -> Result<(), String> {
    let cursor_byte = utf16_to_byte_index(&state.markdown, state.selection.focus);
    let line_start = state.markdown[..cursor_byte]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = state.markdown[cursor_byte..]
        .find('\n')
        .map_or(state.markdown.len(), |offset| cursor_byte + offset);
    let original_line = state.markdown[line_start..line_end].to_string();
    let (indentation, content) = split_indentation(&original_line);
    let clean = strip_block_prefix(content);
    let prefix = block_prefix(kind)?;
    let replacement = format!("{indentation}{prefix}{clean}");

    let original_cursor_in_line = cursor_byte.saturating_sub(line_start);
    let old_clean_start = indentation.len() + content.len().saturating_sub(clean.len());
    let cursor_in_clean = original_cursor_in_line
        .saturating_sub(old_clean_start)
        .min(clean.len());
    let requested_cursor = line_start + indentation.len() + prefix.len() + cursor_in_clean;

    state
        .markdown
        .replace_range(line_start..line_end, &replacement);
    let new_cursor_byte = previous_char_boundary(&state.markdown, requested_cursor);
    let cursor_utf16 = byte_to_utf16_index(&state.markdown, new_cursor_byte);
    state.selection = MuyaSelection::collapsed(cursor_utf16);
    Ok(())
}

fn split_indentation(line: &str) -> (&str, &str) {
    let index = line
        .char_indices()
        .find(|(_, character)| !matches!(character, ' ' | '\t'))
        .map_or(line.len(), |(index, _)| index);
    line.split_at(index)
}

fn strip_block_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();

    if let Some(rest) = trimmed.strip_prefix('>') {
        return rest.trim_start();
    }

    let heading_markers = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if (1..=6).contains(&heading_markers) {
        if let Some(rest) = trimmed.get(heading_markers..) {
            if rest.chars().next().is_some_and(char::is_whitespace) {
                return rest.trim_start();
            }
        }
    }

    for prefix in ["- [ ] ", "- [x] ", "- [X] ", "- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest;
        }
    }

    if let Some((number, rest)) = trimmed.split_once(". ") {
        if !number.is_empty() && number.chars().all(|character| character.is_ascii_digit()) {
            return rest;
        }
    }

    trimmed
}

fn block_prefix(kind: &str) -> Result<&'static str, String> {
    match kind {
        "paragraph" => Ok(""),
        "heading1" => Ok("# "),
        "heading2" => Ok("## "),
        "heading3" => Ok("### "),
        "heading4" => Ok("#### "),
        "heading5" => Ok("##### "),
        "heading6" => Ok("###### "),
        "bullet" => Ok("- "),
        "ordered" => Ok("1. "),
        "task" => Ok("- [ ] "),
        "quote" => Ok("> "),
        _ => Err(format!("unsupported Muya block kind: {kind}")),
    }
}

pub fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

pub fn utf16_to_byte_index(value: &str, utf16_index: usize) -> usize {
    if utf16_index == 0 {
        return 0;
    }

    let mut consumed_utf16 = 0usize;
    for (byte_index, character) in value.char_indices() {
        let next_utf16 = consumed_utf16 + character.len_utf16();
        if utf16_index < next_utf16 {
            return byte_index;
        }
        if utf16_index == next_utf16 {
            return byte_index + character.len_utf8();
        }
        consumed_utf16 = next_utf16;
    }
    value.len()
}

pub fn byte_to_utf16_index(value: &str, byte_index: usize) -> usize {
    let safe_byte_index = previous_char_boundary(value, byte_index);
    value[..safe_byte_index].encode_utf16().count()
}

fn previous_char_boundary(value: &str, byte_index: usize) -> usize {
    let mut safe = byte_index.min(value.len());
    while safe > 0 && !value.is_char_boundary(safe) {
        safe -= 1;
    }
    safe
}

fn clamp_selection(markdown: &str, selection: MuyaSelection) -> MuyaSelection {
    let maximum = utf16_len(markdown);
    MuyaSelection {
        anchor: selection.anchor.min(maximum),
        focus: selection.focus.min(maximum),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(state: MuyaEditorState, command: MuyaEditorCommand) -> MuyaEditorState {
        apply_command(state, command)
            .expect("command should succeed")
            .state
    }

    #[test]
    fn converts_offsets_at_start_middle_and_end() {
        let text = "A😀B";
        assert_eq!(utf16_to_byte_index(text, 0), 0);
        assert_eq!(utf16_to_byte_index(text, 1), 1);
        assert_eq!(utf16_to_byte_index(text, 3), 5);
        assert_eq!(utf16_to_byte_index(text, 4), 6);
        assert_eq!(byte_to_utf16_index(text, 0), 0);
        assert_eq!(byte_to_utf16_index(text, 5), 3);
        assert_eq!(byte_to_utf16_index(text, text.len()), 4);
    }

    #[test]
    fn inserts_at_javascript_utf16_offsets() {
        let mut state = MuyaEditorState::new("A😀B".to_string());
        state.selection = MuyaSelection::collapsed(3);
        let state = apply(
            state,
            MuyaEditorCommand::InsertText {
                text: "!".to_string(),
            },
        );
        assert_eq!(state.markdown, "A😀!B");
        assert_eq!(state.selection, MuyaSelection::collapsed(4));
    }

    #[test]
    fn backspace_removes_a_complete_unicode_scalar() {
        let mut state = MuyaEditorState::new("A😀B".to_string());
        state.selection = MuyaSelection::collapsed(3);
        let state = apply(state, MuyaEditorCommand::DeleteBackward);
        assert_eq!(state.markdown, "AB");
        assert_eq!(state.selection, MuyaSelection::collapsed(1));
    }

    #[test]
    fn forward_delete_removes_a_complete_unicode_scalar() {
        let mut state = MuyaEditorState::new("A😀B".to_string());
        state.selection = MuyaSelection::collapsed(1);
        let state = apply(state, MuyaEditorCommand::DeleteForward);
        assert_eq!(state.markdown, "AB");
        assert_eq!(state.selection, MuyaSelection::collapsed(1));
    }

    #[test]
    fn replaces_forward_or_backward_selection() {
        let mut state = MuyaEditorState::new("hello world".to_string());
        state.selection = MuyaSelection {
            anchor: 11,
            focus: 6,
        };
        let state = apply(
            state,
            MuyaEditorCommand::ReplaceSelection {
                text: "Muya".to_string(),
            },
        );
        assert_eq!(state.markdown, "hello Muya");
        assert_eq!(state.selection, MuyaSelection::collapsed(10));
    }

    #[test]
    fn supports_real_undo_and_redo_history() {
        let state = apply(
            MuyaEditorState::new("hello".to_string()),
            MuyaEditorCommand::InsertText {
                text: "!".to_string(),
            },
        );
        assert_eq!(state.undo_stack.len(), 1);
        let state = apply(state, MuyaEditorCommand::Undo);
        assert_eq!(state.markdown, "hello");
        assert_eq!(state.redo_stack.len(), 1);
        let state = apply(state, MuyaEditorCommand::Redo);
        assert_eq!(state.markdown, "hello!");
    }

    #[test]
    fn toggles_inline_markers_around_selection() {
        let mut state = MuyaEditorState::new("hello".to_string());
        state.selection = MuyaSelection {
            anchor: 0,
            focus: 5,
        };
        let state = apply(
            state,
            MuyaEditorCommand::ToggleInline {
                marker: "**".to_string(),
            },
        );
        assert_eq!(state.markdown, "**hello**");
        assert_eq!(
            state.selection,
            MuyaSelection {
                anchor: 2,
                focus: 7
            }
        );
        let state = apply(
            state,
            MuyaEditorCommand::ToggleInline {
                marker: "**".to_string(),
            },
        );
        assert_eq!(state.markdown, "hello");
        assert_eq!(
            state.selection,
            MuyaSelection {
                anchor: 0,
                focus: 5
            }
        );
    }

    #[test]
    fn marker_detection_does_not_slice_inside_an_emoji() {
        let mut state = MuyaEditorState::new("😀hello".to_string());
        state.selection = MuyaSelection {
            anchor: 2,
            focus: 7,
        };
        let state = apply(
            state,
            MuyaEditorCommand::ToggleInline {
                marker: "**".to_string(),
            },
        );
        assert_eq!(state.markdown, "😀**hello**");
    }

    #[test]
    fn inserts_empty_inline_pair_with_cursor_inside() {
        let mut state = MuyaEditorState::new("text".to_string());
        state.selection = MuyaSelection::collapsed(4);
        let state = apply(
            state,
            MuyaEditorCommand::ToggleInline {
                marker: "`".to_string(),
            },
        );
        assert_eq!(state.markdown, "text``");
        assert_eq!(state.selection, MuyaSelection::collapsed(5));
    }

    #[test]
    fn transforms_current_line_without_losing_indentation() {
        let mut state = MuyaEditorState::new("before\n  old title\nafter".to_string());
        state.selection = MuyaSelection::collapsed(12);
        let state = apply(
            state,
            MuyaEditorCommand::TransformBlock {
                kind: "heading2".to_string(),
            },
        );
        assert_eq!(state.markdown, "before\n  ## old title\nafter");
    }

    #[test]
    fn replaces_existing_block_prefix_instead_of_stacking_it() {
        let mut state = MuyaEditorState::new("- [ ] task".to_string());
        state.selection = MuyaSelection::collapsed(4);
        let state = apply(
            state,
            MuyaEditorCommand::TransformBlock {
                kind: "quote".to_string(),
            },
        );
        assert_eq!(state.markdown, "> task");
    }

    #[test]
    fn keeps_end_cursor_at_end_after_block_transform() {
        let state = apply(
            MuyaEditorState::new("Title".to_string()),
            MuyaEditorCommand::TransformBlock {
                kind: "heading1".to_string(),
            },
        );
        assert_eq!(state.markdown, "# Title");
        assert_eq!(state.selection, MuyaSelection::collapsed(7));
    }

    #[test]
    fn rejects_unknown_formatting_commands() {
        let state = MuyaEditorState::new("hello".to_string());
        let error = apply_command(
            state,
            MuyaEditorCommand::ToggleInline {
                marker: "<script>".to_string(),
            },
        )
        .expect_err("unsafe arbitrary markers must be rejected");
        assert!(error.contains("unsupported"));
    }

    #[test]
    fn batch_application_is_deterministic() {
        let transaction = apply_commands(
            MuyaEditorState::new(String::new()),
            vec![
                MuyaEditorCommand::InsertText {
                    text: "Title".to_string(),
                },
                MuyaEditorCommand::TransformBlock {
                    kind: "heading1".to_string(),
                },
                MuyaEditorCommand::InsertLineBreak,
                MuyaEditorCommand::InsertText {
                    text: "Body".to_string(),
                },
            ],
        )
        .expect("batch should succeed");
        assert_eq!(transaction.state.markdown, "# Title\nBody");
        assert!(transaction.document_changed);
    }
}

