use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::muya_engine::MuyaSelection;

use super::muya_complete::{apply_complete_command, MuyaCompleteCommand};
use super::muya_engine::{
    apply_command, utf16_len, utf16_to_byte_index, MuyaEditorCommand, MuyaEditorState,
    MuyaEditorTransaction,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellPosition {
    pub row: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MuyaAdvancedCommand {
    SmartInput {
        #[serde(default)]
        text: String,
        #[serde(default = "default_true")]
        auto_pair_bracket: bool,
        #[serde(default = "default_true")]
        auto_pair_markdown_syntax: bool,
    },
    SmartEnter {
        #[serde(default)]
        shift_key: bool,
    },
    SmartDeleteBackward,
    SmartDeleteForward,
    ReplaceCurrentWord {
        #[serde(default)]
        text: String,
    },
    ReorderTable {
        start: usize,
        end: usize,
        axis: String,
        from: usize,
        to: usize,
    },
    ClearTableCells {
        start: usize,
        end: usize,
        cells: Vec<TableCellPosition>,
        #[serde(default)]
        cut: bool,
    },
    InsertBlock {
        kind: String,
        #[serde(default)]
        language: String,
        #[serde(default)]
        content: String,
    },
}

fn default_true() -> bool {
    true
}

pub fn apply_advanced_command(
    state: MuyaEditorState,
    command: MuyaAdvancedCommand,
) -> Result<MuyaEditorTransaction, String> {
    match command {
        MuyaAdvancedCommand::SmartInput {
            text,
            auto_pair_bracket,
            auto_pair_markdown_syntax,
        } => smart_input(state, &text, auto_pair_bracket, auto_pair_markdown_syntax),
        MuyaAdvancedCommand::SmartEnter { shift_key } => smart_enter(state, shift_key),
        MuyaAdvancedCommand::SmartDeleteBackward => smart_delete_backward(state),
        MuyaAdvancedCommand::SmartDeleteForward => smart_delete_forward(state),
        MuyaAdvancedCommand::ReplaceCurrentWord { text } => replace_current_word(state, &text),
        MuyaAdvancedCommand::ReorderTable {
            start,
            end,
            axis,
            from,
            to,
        } => reorder_table(state, start, end, &axis, from, to),
        MuyaAdvancedCommand::ClearTableCells {
            start,
            end,
            cells,
            cut,
        } => clear_table_cells(state, start, end, &cells, cut),
        MuyaAdvancedCommand::InsertBlock {
            kind,
            language,
            content,
        } => insert_block(state, &kind, &language, &content),
    }
}

fn replace_range(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    text: &str,
) -> Result<MuyaEditorTransaction, String> {
    apply_complete_command(
        state,
        MuyaCompleteCommand::ReplaceRange {
            start,
            end,
            text: text.to_string(),
        },
    )
}

fn set_selection(
    state: MuyaEditorState,
    anchor: usize,
    focus: usize,
) -> Result<MuyaEditorTransaction, String> {
    apply_command(state, MuyaEditorCommand::SetSelection { anchor, focus })
}

fn next_char(markdown: &str, offset_utf16: usize) -> Option<char> {
    let byte = utf16_to_byte_index(markdown, offset_utf16.min(utf16_len(markdown)));
    markdown[byte..].chars().next()
}

fn previous_char(markdown: &str, offset_utf16: usize) -> Option<char> {
    let byte = utf16_to_byte_index(markdown, offset_utf16.min(utf16_len(markdown)));
    markdown[..byte].chars().next_back()
}

fn pair_for(character: char, bracket: bool, markdown: bool) -> Option<char> {
    if bracket {
        match character {
            '(' => return Some(')'),
            '[' => return Some(']'),
            '{' => return Some('}'),
            '"' => return Some('"'),
            '\'' => return Some('\''),
            _ => {}
        }
    }
    if markdown {
        match character {
            '`' => return Some('`'),
            '*' => return Some('*'),
            '_' => return Some('_'),
            '~' => return Some('~'),
            '=' => return Some('='),
            '$' => return Some('$'),
            _ => {}
        }
    }
    None
}

fn is_closing_pair(character: char) -> bool {
    matches!(
        character,
        ')' | ']' | '}' | '"' | '\'' | '`' | '*' | '_' | '~' | '=' | '$'
    )
}

fn smart_input(
    state: MuyaEditorState,
    text: &str,
    auto_pair_bracket: bool,
    auto_pair_markdown_syntax: bool,
) -> Result<MuyaEditorTransaction, String> {
    if text.is_empty() {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }
    let start = state.selection.start();
    let end = state.selection.end();
    if start == end && text.chars().count() == 1 {
        let character = text.chars().next().unwrap_or_default();
        if is_closing_pair(character) && next_char(&state.markdown, start) == Some(character) {
            return set_selection(
                state,
                start + character.len_utf16(),
                start + character.len_utf16(),
            );
        }
        if let Some(close) = pair_for(character, auto_pair_bracket, auto_pair_markdown_syntax) {
            let selected = String::new();
            let replacement = format!("{character}{selected}{close}");
            let mut transaction = replace_range(state, start, end, &replacement)?;
            let cursor = start + character.len_utf16();
            let document_changed = transaction.document_changed;
            transaction = set_selection(transaction.state, cursor, cursor)?;
            transaction.document_changed = document_changed;
            return Ok(transaction);
        }
    }

    if start != end && text.chars().count() == 1 {
        let character = text.chars().next().unwrap_or_default();
        if let Some(close) = pair_for(character, auto_pair_bracket, auto_pair_markdown_syntax) {
            let start_byte = utf16_to_byte_index(&state.markdown, start);
            let end_byte = utf16_to_byte_index(&state.markdown, end);
            let selected = state.markdown[start_byte..end_byte].to_string();
            let replacement = format!("{character}{selected}{close}");
            let mut transaction = replace_range(state, start, end, &replacement)?;
            let anchor = start + character.len_utf16();
            let focus = anchor + utf16_len(&selected);
            let document_changed = transaction.document_changed;
            transaction = set_selection(transaction.state, anchor, focus)?;
            transaction.document_changed = document_changed;
            return Ok(transaction);
        }
    }

    replace_range(state, start, end, text)
}

fn current_line(markdown: &str, cursor_utf16: usize) -> (usize, usize, usize, String) {
    let cursor_byte = utf16_to_byte_index(markdown, cursor_utf16.min(utf16_len(markdown)));
    let start = markdown[..cursor_byte]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let end = markdown[cursor_byte..]
        .find('\n')
        .map_or(markdown.len(), |offset| cursor_byte + offset);
    (start, end, cursor_byte, markdown[start..end].to_string())
}

fn line_prefix(line: &str) -> Option<(String, String)> {
    let indentation = line
        .char_indices()
        .find(|(_, character)| !matches!(character, ' ' | '\t'))
        .map_or(line.len(), |(index, _)| index);
    let indent = &line[..indentation];
    let body = &line[indentation..];

    for marker in ["- [ ] ", "- [x] ", "- [X] "] {
        if let Some(content) = body.strip_prefix(marker) {
            return Some((format!("{indent}- [ ] "), content.to_string()));
        }
    }
    for marker in ["- ", "* ", "+ ", "> "] {
        if let Some(content) = body.strip_prefix(marker) {
            return Some((format!("{indent}{marker}"), content.to_string()));
        }
    }
    for delimiter in [". ", ") "] {
        if let Some(index) = body.find(delimiter) {
            let number = &body[..index];
            if let Ok(number) = number.parse::<u64>() {
                let content = body[index + delimiter.len()..].to_string();
                return Some((
                    format!("{indent}{}{delimiter}", number.saturating_add(1)),
                    content,
                ));
            }
        }
    }
    None
}

fn leading_whitespace(line: &str) -> &str {
    let end = line
        .char_indices()
        .find(|(_, character)| !matches!(character, ' ' | '\t'))
        .map_or(line.len(), |(index, _)| index);
    &line[..end]
}

fn smart_enter(state: MuyaEditorState, shift_key: bool) -> Result<MuyaEditorTransaction, String> {
    if state.selection.anchor != state.selection.focus {
        let replacement = if shift_key { "  \n" } else { "\n" };
        return replace_range(
            state.clone(),
            state.selection.start(),
            state.selection.end(),
            replacement,
        );
    }

    let cursor = state.selection.focus;
    let (_line_start, line_end, cursor_byte, line) = current_line(&state.markdown, cursor);
    let at_line_end = cursor_byte == line_end;
    if at_line_end {
        if let Some((continuation, content)) = line_prefix(&line) {
            if content.trim().is_empty() {
                let marker_len = utf16_len(&line) - utf16_len(content.trim_start());
                let line_start_utf16 = cursor.saturating_sub(utf16_len(&line));
                return replace_range(
                    state,
                    line_start_utf16,
                    line_start_utf16 + marker_len,
                    leading_whitespace(&line),
                );
            }
            return replace_range(state, cursor, cursor, &format!("\n{continuation}"));
        }
    }

    let indent = leading_whitespace(&line);
    let replacement = if shift_key {
        "  \n".to_string()
    } else if at_line_end && !indent.is_empty() {
        format!("\n{indent}")
    } else {
        "\n".to_string()
    };
    replace_range(state, cursor, cursor, &replacement)
}

fn matching_pair(open: char, close: char) -> bool {
    matches!(
        (open, close),
        ('(', ')')
            | ('[', ']')
            | ('{', '}')
            | ('"', '"')
            | ('\'', '\'')
            | ('`', '`')
            | ('*', '*')
            | ('_', '_')
            | ('~', '~')
            | ('=', '=')
            | ('$', '$')
    )
}

fn markdown_prefix_range(markdown: &str, cursor_utf16: usize) -> Option<(usize, usize)> {
    let (line_start, _, cursor_byte, line) = current_line(markdown, cursor_utf16);
    let relative = cursor_byte.saturating_sub(line_start);
    if relative > line.len() {
        return None;
    }
    let before = &line[..relative];
    let trimmed = before.trim_start_matches([' ', '\t']);
    let indentation = before.len() - trimmed.len();
    let markers = ["- [ ] ", "- [x] ", "- [X] ", "- ", "* ", "+ ", "> "];
    for marker in markers {
        if trimmed == marker {
            let start = utf16_len(&markdown[..line_start + indentation]);
            return Some((start, cursor_utf16));
        }
    }
    for delimiter in [". ", ") "] {
        if let Some(number) = trimmed.strip_suffix(delimiter) {
            if number.parse::<u64>().is_ok() {
                let start = utf16_len(&markdown[..line_start + indentation]);
                return Some((start, cursor_utf16));
            }
        }
    }
    if trimmed.starts_with('#')
        && trimmed.ends_with(' ')
        && trimmed.trim_end().chars().all(|character| character == '#')
    {
        let start = utf16_len(&markdown[..line_start + indentation]);
        return Some((start, cursor_utf16));
    }
    None
}

fn smart_delete_backward(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    if state.selection.anchor != state.selection.focus {
        return replace_range(
            state.clone(),
            state.selection.start(),
            state.selection.end(),
            "",
        );
    }
    let cursor = state.selection.focus;
    if cursor == 0 {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }
    if let (Some(open), Some(close)) = (
        previous_char(&state.markdown, cursor),
        next_char(&state.markdown, cursor),
    ) {
        if matching_pair(open, close) {
            let open_len = open.len_utf16();
            let close_len = close.len_utf16();
            return replace_range(
                state,
                cursor.saturating_sub(open_len),
                cursor + close_len,
                "",
            );
        }
    }
    if let Some((start, end)) = markdown_prefix_range(&state.markdown, cursor) {
        return replace_range(state, start, end, "");
    }
    apply_command(state, MuyaEditorCommand::DeleteBackward)
}

fn smart_delete_forward(state: MuyaEditorState) -> Result<MuyaEditorTransaction, String> {
    if state.selection.anchor != state.selection.focus {
        return replace_range(
            state.clone(),
            state.selection.start(),
            state.selection.end(),
            "",
        );
    }
    let cursor = state.selection.focus;
    if let (Some(open), Some(close)) = (
        previous_char(&state.markdown, cursor),
        next_char(&state.markdown, cursor),
    ) {
        if matching_pair(open, close) {
            return replace_range(state, cursor, cursor + close.len_utf16(), "");
        }
    }
    apply_command(state, MuyaEditorCommand::DeleteForward)
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn replace_current_word(
    state: MuyaEditorState,
    text: &str,
) -> Result<MuyaEditorTransaction, String> {
    if state.selection.anchor != state.selection.focus {
        return replace_range(
            state.clone(),
            state.selection.start(),
            state.selection.end(),
            text,
        );
    }
    let cursor_byte = utf16_to_byte_index(&state.markdown, state.selection.focus);
    let start = state.markdown[..cursor_byte]
        .char_indices()
        .rev()
        .find(|(_, character)| !is_word(*character))
        .map_or(0, |(index, character)| index + character.len_utf8());
    let end = state.markdown[cursor_byte..]
        .char_indices()
        .find(|(_, character)| !is_word(*character))
        .map_or(state.markdown.len(), |(index, _)| cursor_byte + index);
    let start_utf16 = utf16_len(&state.markdown[..start]);
    let end_utf16 = utf16_len(&state.markdown[..end]);
    replace_range(state, start_utf16, end_utf16, text)
}

#[derive(Clone, Debug)]
struct TableRow {
    cells: Vec<String>,
}

fn split_table_row(line: &str) -> Result<TableRow, String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return Err("invalid Markdown table row".to_string());
    }
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    let mut code_ticks = 0usize;
    for character in trimmed[1..trimmed.len() - 1].chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            current.push(character);
            escaped = true;
            continue;
        }
        if character == '`' {
            code_ticks ^= 1;
            current.push(character);
            continue;
        }
        if character == '|' && code_ticks == 0 {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(character);
        }
    }
    cells.push(current.trim().to_string());
    Ok(TableRow { cells })
}

fn render_table_row(row: &TableRow) -> String {
    format!("| {} |", row.cells.join(" | "))
}

fn table_range(
    state: &MuyaEditorState,
    start: usize,
    end: usize,
) -> Result<(usize, usize, Vec<TableRow>), String> {
    let maximum = utf16_len(&state.markdown);
    let start_utf16 = start.min(end).min(maximum);
    let end_utf16 = start.max(end).min(maximum);
    let start_byte = utf16_to_byte_index(&state.markdown, start_utf16);
    let end_byte = utf16_to_byte_index(&state.markdown, end_utf16);
    let rows = state.markdown[start_byte..end_byte]
        .lines()
        .map(split_table_row)
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() < 2 {
        return Err("Markdown table must contain a header and separator".to_string());
    }
    Ok((start_utf16, end_utf16, rows))
}

fn reorder_table(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    axis: &str,
    from: usize,
    to: usize,
) -> Result<MuyaEditorTransaction, String> {
    let (start, end, mut rows) = table_range(&state, start, end)?;
    if from == to {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }
    match axis {
        "row" => {
            if from >= rows.len() || to >= rows.len() {
                return Err("table row index is out of bounds".to_string());
            }
            if from == 1 || to == 1 {
                return Err("the Markdown table separator cannot be reordered".to_string());
            }
            let row = rows.remove(from);
            rows.insert(to, row);
        }
        "column" => {
            let columns = rows.first().map_or(0, |row| row.cells.len());
            if from >= columns || to >= columns {
                return Err("table column index is out of bounds".to_string());
            }
            for row in &mut rows {
                if from >= row.cells.len() || to >= row.cells.len() {
                    return Err("table rows have inconsistent column counts".to_string());
                }
                let cell = row.cells.remove(from);
                row.cells.insert(to, cell);
            }
        }
        other => return Err(format!("unsupported table reorder axis: {other}")),
    }
    let markdown = rows
        .iter()
        .map(render_table_row)
        .collect::<Vec<_>>()
        .join("\n");
    replace_range(state, start, end, &markdown)
}

fn clear_table_cells(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    cells: &[TableCellPosition],
    cut: bool,
) -> Result<MuyaEditorTransaction, String> {
    let (start, end, mut rows) = table_range(&state, start, end)?;
    if cells.is_empty() {
        return Ok(MuyaEditorTransaction {
            state,
            document_changed: false,
            selection_changed: false,
        });
    }
    let all_cells = rows
        .iter()
        .enumerate()
        .filter(|(row, _)| *row != 1)
        .flat_map(|(row, value)| (0..value.cells.len()).map(move |column| (row, column)))
        .collect::<Vec<_>>();
    let selection = cells
        .iter()
        .map(|cell| (cell.row, cell.column))
        .collect::<std::collections::HashSet<_>>();
    if cut && all_cells.iter().all(|cell| selection.contains(cell)) {
        return replace_range(state, start, end, "");
    }
    for cell in cells {
        if cell.row == 1 {
            continue;
        }
        let row = rows
            .get_mut(cell.row)
            .ok_or_else(|| "selected table row is out of bounds".to_string())?;
        let value = row
            .cells
            .get_mut(cell.column)
            .ok_or_else(|| "selected table column is out of bounds".to_string())?;
        value.clear();
    }
    let markdown = rows
        .iter()
        .map(render_table_row)
        .collect::<Vec<_>>()
        .join("\n");
    replace_range(state, start, end, &markdown)
}

fn insert_block(
    state: MuyaEditorState,
    kind: &str,
    language: &str,
    content: &str,
) -> Result<MuyaEditorTransaction, String> {
    if language.contains(['\r', '\n', '`', '~']) {
        return Err("invalid block language".to_string());
    }
    let markdown = match kind {
        "paragraph" => content.to_string(),
        "heading" => format!("# {content}"),
        "blockquote" => content
            .lines()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        "bullet-list" => format!("- {content}"),
        "ordered-list" => format!("1. {content}"),
        "task-list" => format!("- [ ] {content}"),
        "code" => format!("```{}\n{}\n```", language.trim(), content),
        "math" => format!("$$\n{content}\n$$"),
        "html" => content.to_string(),
        "frontmatter" => format!("---\n{content}\n---"),
        "thematic-break" => "---".to_string(),
        other => return Err(format!("unsupported Muya block kind: {other}")),
    };
    replace_range(
        state.clone(),
        state.selection.start(),
        state.selection.end(),
        &markdown,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(markdown: &str, cursor: usize) -> MuyaEditorState {
        let mut state = MuyaEditorState::new(markdown.to_string());
        state.selection = MuyaSelection::collapsed(cursor);
        state
    }

    #[test]
    fn inserts_pairs_and_skips_existing_closing_character() {
        let inserted = apply_advanced_command(
            state("", 0),
            MuyaAdvancedCommand::SmartInput {
                text: "(".to_string(),
                auto_pair_bracket: true,
                auto_pair_markdown_syntax: true,
            },
        )
        .unwrap();
        assert_eq!(inserted.state.markdown, "()");
        assert_eq!(inserted.state.selection.focus, 1);
        let skipped = apply_advanced_command(
            inserted.state,
            MuyaAdvancedCommand::SmartInput {
                text: ")".to_string(),
                auto_pair_bracket: true,
                auto_pair_markdown_syntax: true,
            },
        )
        .unwrap();
        assert_eq!(skipped.state.markdown, "()");
        assert_eq!(skipped.state.selection.focus, 2);
    }

    #[test]
    fn enter_continues_and_exits_lists() {
        let continued = apply_advanced_command(
            state("- item", 6),
            MuyaAdvancedCommand::SmartEnter { shift_key: false },
        )
        .unwrap();
        assert_eq!(continued.state.markdown, "- item\n- ");
        let exited = apply_advanced_command(
            state("- ", 2),
            MuyaAdvancedCommand::SmartEnter { shift_key: false },
        )
        .unwrap();
        assert_eq!(exited.state.markdown, "");
    }

    #[test]
    fn backspace_removes_pairs_and_empty_markers_atomically() {
        let pair = apply_advanced_command(state("()", 1), MuyaAdvancedCommand::SmartDeleteBackward)
            .unwrap();
        assert_eq!(pair.state.markdown, "");
        let marker =
            apply_advanced_command(state("- ", 2), MuyaAdvancedCommand::SmartDeleteBackward)
                .unwrap();
        assert_eq!(marker.state.markdown, "");
    }

    #[test]
    fn reorders_columns_and_clears_selected_cells() {
        let markdown = "| A | B |\n| - | - |\n| 1 | 2 |";
        let reordered = apply_advanced_command(
            state(markdown, 0),
            MuyaAdvancedCommand::ReorderTable {
                start: 0,
                end: utf16_len(markdown),
                axis: "column".to_string(),
                from: 0,
                to: 1,
            },
        )
        .unwrap();
        assert!(reordered.state.markdown.starts_with("| B | A |"));
        let cleared = apply_advanced_command(
            reordered.state,
            MuyaAdvancedCommand::ClearTableCells {
                start: 0,
                end: utf16_len(markdown),
                cells: vec![TableCellPosition { row: 2, column: 0 }],
                cut: false,
            },
        )
        .unwrap();
        assert!(cleared.state.markdown.ends_with("|  | 1 |"));
    }

    #[test]
    fn replaces_unicode_current_word() {
        let replaced = apply_advanced_command(
            state("été chaud", 2),
            MuyaAdvancedCommand::ReplaceCurrentWord {
                text: "hiver".to_string(),
            },
        )
        .unwrap();
        assert_eq!(replaced.state.markdown, "hiver chaud");
    }
}

