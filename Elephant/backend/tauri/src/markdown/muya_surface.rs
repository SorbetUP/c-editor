use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::muya_engine::MuyaSelection;

use super::muya_complete::{apply_complete_command, MuyaCompleteCommand};
use super::muya_engine::{
    apply_command, utf16_len, utf16_to_byte_index, MuyaEditorCommand, MuyaEditorState,
    MuyaEditorTransaction,
};
use super::muya_parity::apply_table_command;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MuyaSurfaceCommand {
    FormatInline {
        format: String,
    },
    CreateTable {
        rows: usize,
        columns: usize,
    },
    TransformTable {
        start: usize,
        end: usize,
        action: String,
        #[serde(default)]
        index: usize,
    },
    InsertImage {
        #[serde(default)]
        alt: String,
        src: String,
        #[serde(default)]
        title: String,
    },
    UpdateImage {
        start: usize,
        end: usize,
        attribute: String,
        value: String,
    },
    UpdateLink {
        start: usize,
        end: usize,
        href: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        label: String,
    },
    RemoveFootnote {
        label: String,
    },
}

pub fn apply_surface_command(
    state: MuyaEditorState,
    command: MuyaSurfaceCommand,
) -> Result<MuyaEditorTransaction, String> {
    match command {
        MuyaSurfaceCommand::FormatInline { format } => format_inline(state, &format),
        MuyaSurfaceCommand::CreateTable { rows, columns } => create_table(state, rows, columns),
        MuyaSurfaceCommand::TransformTable {
            start,
            end,
            action,
            index,
        } => transform_table(state, start, end, &action, index),
        MuyaSurfaceCommand::InsertImage { alt, src, title } => {
            insert_image(state, &alt, &src, &title)
        }
        MuyaSurfaceCommand::UpdateImage {
            start,
            end,
            attribute,
            value,
        } => update_image(state, start, end, &attribute, &value),
        MuyaSurfaceCommand::UpdateLink {
            start,
            end,
            href,
            title,
            label,
        } => update_link(state, start, end, &href, &title, &label),
        MuyaSurfaceCommand::RemoveFootnote { label } => remove_footnote(state, &label),
    }
}

fn replace_range(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    text: String,
) -> Result<MuyaEditorTransaction, String> {
    apply_complete_command(
        state,
        MuyaCompleteCommand::ReplaceRange { start, end, text },
    )
}

fn replace_range_with_selection(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    text: String,
    anchor: usize,
    focus: usize,
) -> Result<MuyaEditorTransaction, String> {
    let mut transaction = replace_range(state, start, end, text)?;
    let document_changed = transaction.document_changed;
    let selection = apply_command(
        transaction.state,
        MuyaEditorCommand::SetSelection { anchor, focus },
    )?;
    transaction = selection;
    transaction.document_changed = document_changed;
    Ok(transaction)
}

fn selection_text(state: &MuyaEditorState) -> (usize, usize, String) {
    let start = state.selection.start().min(utf16_len(&state.markdown));
    let end = state.selection.end().min(utf16_len(&state.markdown));
    let start_byte = utf16_to_byte_index(&state.markdown, start);
    let end_byte = utf16_to_byte_index(&state.markdown, end);
    (start, end, state.markdown[start_byte..end_byte].to_string())
}

fn format_inline(state: MuyaEditorState, format: &str) -> Result<MuyaEditorTransaction, String> {
    let (start, end, selected) = selection_text(&state);
    if format == "clear" {
        let stripped = clear_inline_markup(&selected);
        return replace_range_with_selection(
            state,
            start,
            end,
            stripped.clone(),
            start,
            start + utf16_len(&stripped),
        );
    }

    let (open, close) = match format {
        "strong" => ("**", "**"),
        "em" => ("*", "*"),
        "del" => ("~~", "~~"),
        "inline_code" => ("`", "`"),
        "inline_math" => ("$", "$"),
        "highlight" | "mark" => ("==", "=="),
        "sub" => ("<sub>", "</sub>"),
        "sup" => ("<sup>", "</sup>"),
        "u" => ("<u>", "</u>"),
        other => return Err(format!("unsupported Muya inline format: {other}")),
    };

    let wrapper_start = start.saturating_sub(utf16_len(open));
    let wrapper_end = end.saturating_add(utf16_len(close));
    let maximum = utf16_len(&state.markdown);
    if wrapper_start < start && wrapper_end <= maximum {
        let start_byte = utf16_to_byte_index(&state.markdown, wrapper_start);
        let end_byte = utf16_to_byte_index(&state.markdown, wrapper_end);
        let wrapped = &state.markdown[start_byte..end_byte];
        if wrapped.starts_with(open) && wrapped.ends_with(close) {
            return replace_range_with_selection(
                state,
                wrapper_start,
                wrapper_end,
                selected.clone(),
                wrapper_start,
                wrapper_start + utf16_len(&selected),
            );
        }
    }

    let replacement = format!("{open}{selected}{close}");
    let content_start = start + utf16_len(open);
    let content_end = content_start + utf16_len(&selected);
    replace_range_with_selection(state, start, end, replacement, content_start, content_end)
}

fn clear_inline_markup(text: &str) -> String {
    let mut value = text.to_string();
    for (open, close) in [
        ("**", "**"),
        ("~~", "~~"),
        ("==", "=="),
        ("<sub>", "</sub>"),
        ("<sup>", "</sup>"),
        ("<u>", "</u>"),
        ("<mark>", "</mark>"),
        ("`", "`"),
        ("$", "$"),
        ("*", "*"),
        ("_", "_"),
    ] {
        if value.starts_with(open)
            && value.ends_with(close)
            && value.len() >= open.len() + close.len()
        {
            value = value[open.len()..value.len() - close.len()].to_string();
        }
    }
    value
}

fn create_table(
    state: MuyaEditorState,
    rows: usize,
    columns: usize,
) -> Result<MuyaEditorTransaction, String> {
    let columns = columns.clamp(1, 64);
    let rows = rows.clamp(1, 512);
    let header = format!("| {} |", vec![String::new(); columns].join(" | "));
    let separator = format!("| {} |", vec!["-"; columns].join(" | "));
    let body = format!("| {} |", vec![String::new(); columns].join(" | "));
    let mut lines = vec![header, separator];
    lines.extend(std::iter::repeat_n(body, rows));
    let table = lines.join("\n");
    replace_range(
        state.clone(),
        state.selection.start(),
        state.selection.end(),
        table,
    )
}

fn transform_table(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    action: &str,
    index: usize,
) -> Result<MuyaEditorTransaction, String> {
    let maximum = utf16_len(&state.markdown);
    let start = start.min(end).min(maximum);
    let end = start.max(end.min(maximum));
    let start_byte = utf16_to_byte_index(&state.markdown, start);
    let end_byte = utf16_to_byte_index(&state.markdown, end);
    let table = &state.markdown[start_byte..end_byte];
    let transformed = apply_table_command(table, action, index)?;
    replace_range(state, start, end, transformed)
}

fn validate_single_line(value: &str, name: &str) -> Result<(), String> {
    if value.contains('\r') || value.contains('\n') {
        Err(format!("Muya {name} must be a single line"))
    } else {
        Ok(())
    }
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace(']', "\\]")
}

fn escape_title(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn insert_image(
    state: MuyaEditorState,
    alt: &str,
    src: &str,
    title: &str,
) -> Result<MuyaEditorTransaction, String> {
    validate_single_line(alt, "image alt text")?;
    validate_single_line(src, "image source")?;
    validate_single_line(title, "image title")?;
    if src.trim().is_empty() {
        return Err("Muya image source must not be empty".to_string());
    }
    let title = if title.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", escape_title(title))
    };
    let markdown = format!("![{}]({}{title})", escape_label(alt), src.trim());
    replace_range(
        state.clone(),
        state.selection.start(),
        state.selection.end(),
        markdown,
    )
}

fn update_image(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    attribute: &str,
    value: &str,
) -> Result<MuyaEditorTransaction, String> {
    validate_single_line(value, "image attribute")?;
    let maximum = utf16_len(&state.markdown);
    let start = start.min(end).min(maximum);
    let end = start.max(end.min(maximum));
    let start_byte = utf16_to_byte_index(&state.markdown, start);
    let end_byte = utf16_to_byte_index(&state.markdown, end);
    let source = &state.markdown[start_byte..end_byte];
    let image = parse_image(source)?;
    let mut alt = image.alt;
    let mut src = image.src;
    let mut title = image.title;
    let mut width = image.width;
    let mut align = image.align;
    match attribute {
        "alt" => alt = value.to_string(),
        "src" => src = value.to_string(),
        "title" => title = value.to_string(),
        "width" => width = value.to_string(),
        "data-align" | "align" => align = value.to_string(),
        other => return Err(format!("unsupported Muya image attribute: {other}")),
    }
    let mut attrs = Vec::new();
    if !width.is_empty() {
        attrs.push(format!("width=\"{}\"", escape_html(&width)));
    }
    if !align.is_empty() {
        attrs.push(format!("data-align=\"{}\"", escape_html(&align)));
    }
    let replacement = if attrs.is_empty() {
        let title = if title.is_empty() {
            String::new()
        } else {
            format!(" \"{}\"", escape_title(&title))
        };
        format!("![{}]({}{title})", escape_label(&alt), src)
    } else {
        let title_attr = if title.is_empty() {
            String::new()
        } else {
            format!(" title=\"{}\"", escape_html(&title))
        };
        format!(
            "<img src=\"{}\" alt=\"{}\"{} {}>",
            escape_html(&src),
            escape_html(&alt),
            title_attr,
            attrs.join(" ")
        )
    };
    replace_range(state, start, end, replacement)
}

struct ParsedImage {
    alt: String,
    src: String,
    title: String,
    width: String,
    align: String,
}

fn parse_image(source: &str) -> Result<ParsedImage, String> {
    if let Some(rest) = source.strip_prefix("![") {
        let alt_end = rest
            .find("](")
            .ok_or_else(|| "invalid Markdown image".to_string())?;
        let alt = rest[..alt_end].replace("\\]", "]");
        let inner = rest[alt_end + 2..]
            .strip_suffix(')')
            .ok_or_else(|| "invalid Markdown image".to_string())?;
        let (src, title) = if let Some(title_start) = inner.rfind(" \"") {
            let title = inner[title_start + 2..]
                .strip_suffix('"')
                .unwrap_or(&inner[title_start + 2..]);
            (inner[..title_start].to_string(), title.to_string())
        } else {
            (inner.to_string(), String::new())
        };
        return Ok(ParsedImage {
            alt,
            src,
            title,
            width: String::new(),
            align: String::new(),
        });
    }
    if source.trim_start().starts_with("<img") {
        return Ok(ParsedImage {
            alt: html_attr(source, "alt"),
            src: html_attr(source, "src"),
            title: html_attr(source, "title"),
            width: html_attr(source, "width"),
            align: html_attr(source, "data-align"),
        });
    }
    Err("selection is not a Muya image".to_string())
}

fn html_attr(source: &str, name: &str) -> String {
    for quote in ['"', '\''] {
        let prefix = format!("{name}={quote}");
        if let Some(start) = source.find(&prefix) {
            let value_start = start + prefix.len();
            if let Some(end) = source[value_start..].find(quote) {
                return source[value_start..value_start + end].to_string();
            }
        }
    }
    String::new()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn update_link(
    state: MuyaEditorState,
    start: usize,
    end: usize,
    href: &str,
    title: &str,
    label: &str,
) -> Result<MuyaEditorTransaction, String> {
    validate_single_line(href, "link URL")?;
    validate_single_line(title, "link title")?;
    if href.trim().is_empty() {
        return Err("Muya link URL must not be empty".to_string());
    }
    let label = if label.is_empty() { href } else { label };
    let title = if title.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", escape_title(title))
    };
    replace_range(
        state,
        start,
        end,
        format!("[{}]({}{title})", escape_label(label), href.trim()),
    )
}

fn remove_footnote(state: MuyaEditorState, label: &str) -> Result<MuyaEditorTransaction, String> {
    if label.is_empty() || label.contains([']', '\r', '\n']) {
        return Err("invalid Muya footnote label".to_string());
    }
    let reference = format!("[^{label}]");
    let definition = format!("[^{label}]:");
    let mut lines = state
        .markdown
        .lines()
        .filter(|line| !line.starts_with(&definition))
        .collect::<Vec<_>>()
        .join("\n");
    lines = lines.replace(&reference, "");
    replace_range(state.clone(), 0, utf16_len(&state.markdown), lines)
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
    fn formats_and_unformats_advanced_inline_styles() {
        let marked = apply_surface_command(
            state("text", 0, 4),
            MuyaSurfaceCommand::FormatInline {
                format: "sub".to_string(),
            },
        )
        .unwrap();
        assert_eq!(marked.state.markdown, "<sub>text</sub>");
        let cleared = apply_surface_command(
            state("<sub>text</sub>", 0, 15),
            MuyaSurfaceCommand::FormatInline {
                format: "clear".to_string(),
            },
        )
        .unwrap();
        assert_eq!(cleared.state.markdown, "text");
    }

    #[test]
    fn creates_and_targets_one_table_among_multiple_tables() {
        let markdown = "| A |\n| - |\n| 1 |\n\n| B |\n| - |\n| 2 |";
        let second_start = markdown.find("| B |").unwrap();
        let transformed = apply_surface_command(
            state(markdown, 0, 0),
            MuyaSurfaceCommand::TransformTable {
                start: markdown[..second_start].encode_utf16().count(),
                end: markdown.encode_utf16().count(),
                action: "insert_row".to_string(),
                index: 1,
            },
        )
        .unwrap();
        assert_eq!(
            transformed.state.markdown,
            "| A |\n| - |\n| 1 |\n\n| B |\n| - |\n| 2 |\n|  |"
        );
        assert!(transformed
            .state
            .markdown
            .starts_with("| A |\n| - |\n| 1 |"));
    }

    #[test]
    fn inserts_and_updates_images_in_rust() {
        let inserted = apply_surface_command(
            state("", 0, 0),
            MuyaSurfaceCommand::InsertImage {
                alt: "A".to_string(),
                src: "a.png".to_string(),
                title: String::new(),
            },
        )
        .unwrap();
        assert_eq!(inserted.state.markdown, "![A](a.png)");
        let updated = apply_surface_command(
            inserted.state,
            MuyaSurfaceCommand::UpdateImage {
                start: 0,
                end: 11,
                attribute: "width".to_string(),
                value: "50%".to_string(),
            },
        )
        .unwrap();
        assert!(updated.state.markdown.contains("width=\"50%\""));
    }
}

