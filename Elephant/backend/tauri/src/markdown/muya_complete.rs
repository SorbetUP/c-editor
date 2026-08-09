use serde::{Deserialize, Serialize};

use super::muya_engine::{
    apply_command, utf16_len, utf16_to_byte_index, MuyaEditorCommand, MuyaEditorSnapshot,
    MuyaEditorState, MuyaEditorTransaction, MuyaSelection,
};

const HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MuyaCompleteCommand {
    ReplaceRange {
        start: usize,
        end: usize,
        #[serde(default)]
        text: String,
    },
    DeleteBackward,
    DeleteForward,
    InsertParagraph {
        location: String,
        #[serde(default)]
        text: String,
    },
    DuplicateBlock,
    DeleteBlock,
    MoveBlock {
        from_start: usize,
        from_end: usize,
        target: usize,
    },
    IndentSelection {
        #[serde(default)]
        outdent: bool,
        #[serde(default = "default_indent_width")]
        width: usize,
    },
    ToggleTask,
    SetCodeLanguage {
        #[serde(default)]
        language: String,
    },
    InsertLink {
        url: String,
        #[serde(default)]
        title: String,
    },
    RemoveLink,
    SearchReplace {
        query: String,
        #[serde(default)]
        replacement: String,
        #[serde(default)]
        replace_all: bool,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(default)]
        whole_word: bool,
    },
    SelectAll,
}

fn default_indent_width() -> usize {
    2
}

pub fn apply_complete_command(
    state: MuyaEditorState,
    command: MuyaCompleteCommand,
) -> Result<MuyaEditorTransaction, String> {
    match command {
        MuyaCompleteCommand::ReplaceRange { start, end, text } => {
            replace_range(state, start, end, &text)
        }
        MuyaCompleteCommand::DeleteBackward => {
            apply_command(state, MuyaEditorCommand::DeleteBackward)
        }
        MuyaCompleteCommand::DeleteForward => {
            apply_command(state, MuyaEditorCommand::DeleteForward)
        }
        MuyaCompleteCommand::InsertParagraph { location, text } => {
            insert_paragraph(state, &location, &text)
        }
        MuyaCompleteCommand::DuplicateBlock => duplicate_selected_lines(state),
        MuyaCompleteCommand::DeleteBlock => delete_selected_lines(state),
        MuyaCompleteCommand::MoveBlock {
            from_start,
            from_end,
            target,
        } => move_range(state, from_start, from_end, target),
        MuyaCompleteCommand::IndentSelection { outdent, width } => {
            indent_selected_lines(state, outdent, width.clamp(1, 8))
        }
        MuyaCompleteCommand::ToggleTask => toggle_task(state),
        MuyaCompleteCommand::SetCodeLanguage { language } => set_code_language(state, &language),
        MuyaCompleteCommand::InsertLink { url, title } => insert_link(state, &url, &title),
        MuyaCompleteCommand::RemoveLink => remove_link(state),
        MuyaCompleteCommand::SearchReplace {
            query,
            replacement,
            replace_all,
            case_sensitive,
            whole_word,
        } => search_replace(
            state,
            &query,
            &replacement,
            replace_all,
            case_sensitive,
            whole_word,
        ),
        MuyaCompleteCommand::SelectAll => {
            let end = utf16_len(&state.markdown);
            apply_command(
                state,
                MuyaEditorCommand::SetSelection {
                    anchor: 0,
                    focus: end,
                },
            )
        }
    }
}

fn commit_document(
    mut state: MuyaEditorState,
    markdown: String,
    selection: MuyaSelection,
) -> MuyaEditorTransaction {
    let before_markdown = state.markdown.clone();
    let before_selection = state.selection;
    let maximum = utf16_len(&markdown);
    let next_selection = MuyaSelection {
        anchor: selection.anchor.min(maximum),
        focus: selection.focus.min(maximum),
    };

    if before_markdown != markdown {
        while state.undo_stack.len() >= HISTORY_LIMIT {
            state.undo_stack.remove(0);
        }
        state.undo_stack.push(MuyaEditorSnapshot {
            markdown: before_markdown.clone(),
            selection: before_selection,
        });
        state.redo_stack.clear();
        state.markdown = markdown;
        state.selection = next_selection;
        state.revision = state.revision.saturating_add(1);
    } else {
        state.selection = next_selection;
    }

    MuyaEditorTransaction {
        document_changed: state.markdown != before_markdown,
        selection_changed: state.selection != before_selection,
        state,
    }
}

fn replace_range(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    text: &str,
) -> Result<MuyaEditorTransaction, String> {
    let maximum = utf16_len(&state.markdown);
    let start_utf16 = start.min(end).min(maximum);
    let end_utf16 = start.max(end).min(maximum);
    let start_byte = utf16_to_byte_index(&state.markdown, start_utf16);
    let end_byte = utf16_to_byte_index(&state.markdown, end_utf16);
    let mut markdown = state.markdown.clone();
    markdown.replace_range(start_byte..end_byte, text);
    let cursor = start_utf16 + utf16_len(text);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection::collapsed(cursor),
    ))
}

fn byte_to_utf16(markdown: &str, byte_index: usize) -> usize {
    markdown[..byte_index.min(markdown.len())]
        .encode_utf16()
        .count()
}

fn selected_line_range(markdown: &str, selection: MuyaSelection) -> (usize, usize) {
    let maximum = utf16_len(markdown);
    let start = utf16_to_byte_index(markdown, selection.start().min(maximum));
    let end = utf16_to_byte_index(markdown, selection.end().min(maximum));
    let line_start = markdown[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = markdown[end..]
        .find('\n')
        .map_or(markdown.len(), |offset| end + offset);
    (line_start, line_end)
}

fn insert_paragraph(
    state: MuyaEditorState,
    location: &str,
    text: &str,
) -> Result<MuyaEditorTransaction, String> {
    let (line_start, line_end) = selected_line_range(&state.markdown, state.selection);
    let (insert_at, insertion, cursor_byte) = match location {
        "before" => {
            let insertion = format!("{text}\n");
            (line_start, insertion, line_start + text.len())
        }
        "after" => {
            let insertion = format!("\n{text}");
            (line_end, insertion, line_end + 1 + text.len())
        }
        other => return Err(format!("unsupported Muya paragraph location: {other}")),
    };
    let mut markdown = state.markdown.clone();
    markdown.insert_str(insert_at, &insertion);
    let cursor = byte_to_utf16(&markdown, cursor_byte);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection::collapsed(cursor),
    ))
}

fn duplicate_selected_lines(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    let (start, end) = selected_line_range(&state.markdown, state.selection);
    let block = state.markdown[start..end].to_string();
    let mut markdown = state.markdown.clone();
    let insertion = format!("\n{block}");
    markdown.insert_str(end, &insertion);
    let anchor = byte_to_utf16(&markdown, end + 1);
    let focus = anchor + utf16_len(&block);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection { anchor, focus },
    ))
}

fn delete_selected_lines(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    let (mut start, mut end) = selected_line_range(&state.markdown, state.selection);
    if end < state.markdown.len() {
        end += 1;
    } else if start > 0 {
        start -= 1;
    }
    let mut markdown = state.markdown.clone();
    markdown.replace_range(start..end, "");
    let cursor = byte_to_utf16(&markdown, start.min(markdown.len()));
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection::collapsed(cursor),
    ))
}

fn move_range(
    state: MuyaEditorState,
    from_start: usize,
    from_end: usize,
    target: usize,
) -> Result<MuyaEditorTransaction, String> {
    let maximum = utf16_len(&state.markdown);
    let start_utf16 = from_start.min(from_end).min(maximum);
    let end_utf16 = from_start.max(from_end).min(maximum);
    let start = utf16_to_byte_index(&state.markdown, start_utf16);
    let end = utf16_to_byte_index(&state.markdown, end_utf16);
    let target = utf16_to_byte_index(&state.markdown, target.min(maximum));
    if start == end || (target >= start && target <= end) {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }

    let fragment = state.markdown[start..end].to_string();
    let mut markdown = state.markdown.clone();
    markdown.replace_range(start..end, "");
    let adjusted_target = if target > end {
        target - (end - start)
    } else {
        target
    };
    markdown.insert_str(adjusted_target, &fragment);
    let anchor = byte_to_utf16(&markdown, adjusted_target);
    let focus = anchor + utf16_len(&fragment);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection { anchor, focus },
    ))
}

fn indent_selected_lines(
    state: MuyaEditorState,
    outdent: bool,
    width: usize,
) -> Result<MuyaEditorTransaction, String> {
    let (start, end) = selected_line_range(&state.markdown, state.selection);
    let selected = &state.markdown[start..end];
    let indent = " ".repeat(width);
    let transformed = selected
        .split('\n')
        .map(|line| {
            if outdent {
                let removable = line
                    .chars()
                    .take(width)
                    .take_while(|character| *character == ' ')
                    .count();
                line[removable..].to_string()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut markdown = state.markdown.clone();
    markdown.replace_range(start..end, &transformed);
    let anchor = byte_to_utf16(&markdown, start);
    let focus = anchor + utf16_len(&transformed);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection { anchor, focus },
    ))
}

fn toggle_task(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    let (start, end) = selected_line_range(&state.markdown, state.selection);
    let line = &state.markdown[start..end];
    let indentation = line.len()
        - line
            .trim_start_matches(|character| matches!(character, ' ' | '\t'))
            .len();
    let (prefix, body) = line.split_at(indentation);
    let replacement = if let Some(rest) = body.strip_prefix("- [ ] ") {
        format!("{prefix}- [x] {rest}")
    } else if let Some(rest) = body
        .strip_prefix("- [x] ")
        .or_else(|| body.strip_prefix("- [X] "))
    {
        format!("{prefix}- [ ] {rest}")
    } else {
        format!("{prefix}- [ ] {body}")
    };
    let mut markdown = state.markdown.clone();
    markdown.replace_range(start..end, &replacement);
    let cursor = byte_to_utf16(&markdown, start + replacement.len());
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection::collapsed(cursor),
    ))
}

fn set_code_language(
    state: MuyaEditorState,
    language: &str,
) -> Result<MuyaEditorTransaction, String> {
    if language.contains('\r') || language.contains('\n') || language.contains('`') {
        return Err("invalid fenced-code language".to_string());
    }
    let cursor = utf16_to_byte_index(&state.markdown, state.selection.focus);
    let lines = state.markdown.split_inclusive('\n').collect::<Vec<_>>();
    let mut offset = 0usize;
    let mut active: Option<(usize, &str)> = None;
    let mut target: Option<(usize, usize, &str)> = None;

    for line in lines {
        let line_end = offset + line.trim_end_matches('\n').len();
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let marker = if trimmed.starts_with("```") {
            Some("```")
        } else if trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        if let Some(marker) = marker {
            if active.is_some_and(|(_, open_marker)| open_marker == marker) {
                if let Some((header_start, open_marker)) = active.take() {
                    if cursor >= header_start && cursor <= line_end {
                        let header_end = state.markdown[header_start..]
                            .find('\n')
                            .map_or(state.markdown.len(), |relative| header_start + relative);
                        target = Some((header_start, header_end, open_marker));
                        break;
                    }
                }
            } else if active.is_none() {
                active = Some((offset + leading, marker));
            }
        }
        offset += line.len();
    }

    if target.is_none() {
        if let Some((header_start, marker)) = active {
            if cursor >= header_start {
                let header_end = state.markdown[header_start..]
                    .find('\n')
                    .map_or(state.markdown.len(), |relative| header_start + relative);
                target = Some((header_start, header_end, marker));
            }
        }
    }

    let (header_start, header_end, marker) =
        target.ok_or_else(|| "cursor is not inside a fenced code block".to_string())?;
    let header = format!("{marker}{}", language.trim());
    let mut markdown = state.markdown.clone();
    markdown.replace_range(header_start..header_end, &header);
    let cursor = byte_to_utf16(&markdown, header_start + header.len());
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection::collapsed(cursor),
    ))
}

fn insert_link(
    state: MuyaEditorState,
    url: &str,
    title: &str,
) -> Result<MuyaEditorTransaction, String> {
    if url.trim().is_empty() || url.contains('\r') || url.contains('\n') {
        return Err("Muya link URL must be a non-empty single line".to_string());
    }
    let start_utf16 = state.selection.start();
    let end_utf16 = state.selection.end();
    let start = utf16_to_byte_index(&state.markdown, start_utf16);
    let end = utf16_to_byte_index(&state.markdown, end_utf16);
    let label = if start == end {
        url.to_string()
    } else {
        state.markdown[start..end].to_string()
    };
    let suffix = if title.trim().is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", title.replace('"', "\\\""))
    };
    let replacement = format!("[{label}]({url}{suffix})");
    replace_range(state, start_utf16, end_utf16, &replacement)
}

fn remove_link(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    let cursor = utf16_to_byte_index(&state.markdown, state.selection.focus);
    let open = state.markdown[..cursor]
        .rfind('[')
        .ok_or_else(|| "no Markdown link at selection".to_string())?;
    let close_label = state.markdown[open..]
        .find("](")
        .map(|offset| open + offset)
        .ok_or_else(|| "no Markdown link at selection".to_string())?;
    let close = state.markdown[close_label + 2..]
        .find(')')
        .map(|offset| close_label + 2 + offset)
        .ok_or_else(|| "no Markdown link at selection".to_string())?;
    if cursor < open || cursor > close + 1 {
        return Err("no Markdown link at selection".to_string());
    }
    let label = state.markdown[open + 1..close_label].to_string();
    let start_utf16 = byte_to_utf16(&state.markdown, open);
    let end_utf16 = byte_to_utf16(&state.markdown, close + 1);
    replace_range(state, start_utf16, end_utf16, &label)
}

fn search_replace(
    state: MuyaEditorState,
    query: &str,
    replacement: &str,
    replace_all: bool,
    case_sensitive: bool,
    whole_word: bool,
) -> Result<MuyaEditorTransaction, String> {
    if query.is_empty() {
        return Err("Muya search query must not be empty".to_string());
    }
    let matches = find_matches(&state.markdown, query, case_sensitive, whole_word);
    let matches = if replace_all {
        matches
    } else {
        matches.into_iter().take(1).collect()
    };
    if matches.is_empty() {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }

    let mut markdown = state.markdown.clone();
    for (start, end) in matches.iter().rev() {
        markdown.replace_range(*start..*end, replacement);
    }
    let first_start = matches[0].0;
    let anchor = byte_to_utf16(&markdown, first_start);
    let focus = anchor + utf16_len(replacement);
    Ok(commit_document(
        state,
        markdown,
        MuyaSelection { anchor, focus },
    ))
}

pub fn find_matches(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let boundaries = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let query_chars = query.chars().count();
    if query_chars == 0 || boundaries.len() <= query_chars {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for index in 0..boundaries.len() - query_chars {
        let start = boundaries[index];
        let end = boundaries[index + query_chars];
        let candidate = &text[start..end];
        let equal = if case_sensitive {
            candidate == query
        } else {
            candidate.to_lowercase() == query.to_lowercase()
        };
        if equal && (!whole_word || (is_start_boundary(text, start) && is_end_boundary(text, end)))
        {
            matches.push((start, end));
        }
    }
    matches
}

fn is_start_boundary(text: &str, byte_index: usize) -> bool {
    text[..byte_index.min(text.len())]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_alphanumeric() && character != '_')
}

fn is_end_boundary(text: &str, byte_index: usize) -> bool {
    text[byte_index.min(text.len())..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric() && character != '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(markdown: &str, anchor: usize, focus: usize) -> MuyaEditorState {
        let mut state = MuyaEditorState::new(markdown.to_string());
        state.selection = MuyaSelection { anchor, focus };
        state
    }

    #[test]
    fn duplicates_and_deletes_complete_lines_atomically() {
        let duplicated = apply_complete_command(
            state("alpha\nbeta", 1, 1),
            MuyaCompleteCommand::DuplicateBlock,
        )
        .unwrap();
        assert_eq!(duplicated.state.markdown, "alpha\nalpha\nbeta");
        assert_eq!(duplicated.state.undo_stack.len(), 1);
        let deleted =
            apply_complete_command(duplicated.state, MuyaCompleteCommand::DeleteBlock).unwrap();
        assert_eq!(deleted.state.markdown, "alpha\nbeta");
    }

    #[test]
    fn indents_and_outdents_selected_lines() {
        let indented = apply_complete_command(
            state("a\nb", 0, 3),
            MuyaCompleteCommand::IndentSelection {
                outdent: false,
                width: 2,
            },
        )
        .unwrap();
        assert_eq!(indented.state.markdown, "  a\n  b");
        let outdented = apply_complete_command(
            indented.state,
            MuyaCompleteCommand::IndentSelection {
                outdent: true,
                width: 2,
            },
        )
        .unwrap();
        assert_eq!(outdented.state.markdown, "a\nb");
    }

    #[test]
    fn toggles_tasks_and_replaces_unicode_case_insensitively() {
        let task =
            apply_complete_command(state("item", 0, 0), MuyaCompleteCommand::ToggleTask).unwrap();
        assert_eq!(task.state.markdown, "- [ ] item");
        let replaced = apply_complete_command(
            state("Été été", 0, 0),
            MuyaCompleteCommand::SearchReplace {
                query: "été".to_string(),
                replacement: "hiver".to_string(),
                replace_all: true,
                case_sensitive: false,
                whole_word: true,
            },
        )
        .unwrap();
        assert_eq!(replaced.state.markdown, "hiver hiver");
    }

    #[test]
    fn inserts_and_removes_links() {
        let linked = apply_complete_command(
            state("hello", 0, 5),
            MuyaCompleteCommand::InsertLink {
                url: "https://example.com".to_string(),
                title: String::new(),
            },
        )
        .unwrap();
        assert_eq!(linked.state.markdown, "[hello](https://example.com)");
        let mut linked_state = linked.state;
        linked_state.selection = MuyaSelection::collapsed(3);
        let unlinked =
            apply_complete_command(linked_state, MuyaCompleteCommand::RemoveLink).unwrap();
        assert_eq!(unlinked.state.markdown, "hello");
    }

    #[test]
    fn moves_ranges_without_losing_utf16_offsets() {
        let moved = apply_complete_command(
            state("😀A B", 0, 0),
            MuyaCompleteCommand::MoveBlock {
                from_start: 0,
                from_end: 3,
                target: 5,
            },
        )
        .unwrap();
        assert_eq!(moved.state.markdown, " B😀A");
    }
}

