use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaJsonState {
    pub version: u8,
    #[serde(rename = "type")]
    pub state_type: String,
    pub blocks: Vec<MuyaJsonBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaInlineNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaJsonBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MuyaInlineNode>>,
}

impl MuyaJsonBlock {
    fn text_block(block_type: &str, text: String) -> Self {
        Self {
            block_type: block_type.to_string(),
            text: Some(text.clone()),
            children: Some(vec![MuyaInlineNode {
                node_type: "text".to_string(),
                text,
            }]),
            ..Self::default()
        }
    }
}

pub fn markdown_to_json_state(markdown: &str) -> MuyaJsonState {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let lines = normalized.split('\n').collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some((level, text)) = parse_heading(trimmed) {
            let mut block = MuyaJsonBlock::text_block("heading", text);
            block.level = Some(level);
            blocks.push(block);
            index += 1;
            continue;
        }

        if trimmed == "$$" {
            let mut body = Vec::new();
            index += 1;
            while index < lines.len() && lines[index].trim() != "$$" {
                body.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            blocks.push(MuyaJsonBlock::text_block("math_block", body.join("\n")));
            continue;
        }

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = if trimmed.starts_with("```") {
                "```"
            } else {
                "~~~"
            };
            let info = trimmed.get(3..).unwrap_or("").trim().to_string();
            let language = info.split_whitespace().next().unwrap_or("").to_string();
            let mut body = Vec::new();
            index += 1;
            while index < lines.len() && !lines[index].trim().starts_with(marker) {
                body.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            let mut block = MuyaJsonBlock::text_block("code_fence", body.join("\n"));
            block.marker = Some(marker.to_string());
            block.info = Some(info);
            block.language = Some(language);
            blocks.push(block);
            continue;
        }

        if is_table_start(&lines, index) {
            let mut table_lines = vec![lines[index], lines[index + 1]];
            index += 2;
            while index < lines.len() && is_pipe_table_row(lines[index]) {
                table_lines.push(lines[index]);
                index += 1;
            }
            blocks.push(parse_table(&table_lines));
            continue;
        }

        if let Some(block) = parse_list_line(line) {
            blocks.push(block);
            index += 1;
            continue;
        }

        if trimmed.starts_with('>') {
            let text = trimmed
                .strip_prefix('>')
                .unwrap_or("")
                .strip_prefix(' ')
                .unwrap_or_else(|| trimmed.strip_prefix('>').unwrap_or(""));
            blocks.push(MuyaJsonBlock::text_block("blockquote", text.to_string()));
            index += 1;
            continue;
        }

        blocks.push(MuyaJsonBlock::text_block("paragraph", line.to_string()));
        index += 1;
    }

    if blocks.is_empty() {
        blocks.push(MuyaJsonBlock::text_block("paragraph", String::new()));
    }

    MuyaJsonState {
        version: 1,
        state_type: "muya-json-state".to_string(),
        blocks,
    }
}

pub fn json_state_to_markdown(state: &MuyaJsonState) -> String {
    state
        .blocks
        .iter()
        .map(block_to_markdown)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn block_to_markdown(block: &MuyaJsonBlock) -> String {
    let text = block.text.as_deref().unwrap_or("");
    match block.block_type.as_str() {
        "heading" => format!("{} {text}", "#".repeat(block.level.unwrap_or(1) as usize)),
        "paragraph" => text.to_string(),
        "blockquote" => format!("> {text}"),
        "task_list_item" => format!(
            "{}- [{}] {text}",
            "  ".repeat(block.depth.unwrap_or(0)),
            if block.checked.unwrap_or(false) {
                "x"
            } else {
                " "
            }
        ),
        "list_item" => {
            let marker = if block.ordered.unwrap_or(false) {
                format!("{}.", block.index.unwrap_or(1))
            } else {
                "-".to_string()
            };
            format!("{}{marker} {text}", "  ".repeat(block.depth.unwrap_or(0)))
        }
        "code_fence" => {
            let marker = block.marker.as_deref().unwrap_or("```");
            format!(
                "{marker}{}\n{text}\n{marker}",
                block.info.as_deref().unwrap_or("")
            )
        }
        "math_block" => format!("$$\n{text}\n$$"),
        "table" => table_to_markdown(block),
        _ => text.to_string(),
    }
}

fn parse_heading(line: &str) -> Option<(u8, String)> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = line.get(level..)?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((level as u8, rest.trim_start().to_string()))
}

fn parse_list_line(line: &str) -> Option<MuyaJsonBlock> {
    let whitespace = line
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    let depth = whitespace / 2;
    let trimmed = line.trim();

    for (prefix, checked) in [
        ("- [ ] ", false),
        ("* [ ] ", false),
        ("+ [ ] ", false),
        ("- [x] ", true),
        ("- [X] ", true),
        ("* [x] ", true),
        ("* [X] ", true),
        ("+ [x] ", true),
        ("+ [X] ", true),
    ] {
        if let Some(text) = trimmed.strip_prefix(prefix) {
            let mut block = MuyaJsonBlock::text_block("task_list_item", text.to_string());
            block.depth = Some(depth);
            block.checked = Some(checked);
            return Some(block);
        }
    }

    if let Some((number, text)) = trimmed.split_once(". ") {
        if let Ok(number) = number.parse::<u64>() {
            let mut block = MuyaJsonBlock::text_block("list_item", text.to_string());
            block.depth = Some(depth);
            block.ordered = Some(true);
            block.index = Some(number);
            return Some(block);
        }
    }

    for prefix in ["- ", "* ", "+ "] {
        if let Some(text) = trimmed.strip_prefix(prefix) {
            let mut block = MuyaJsonBlock::text_block("list_item", text.to_string());
            block.depth = Some(depth);
            block.ordered = Some(false);
            return Some(block);
        }
    }
    None
}

fn is_pipe_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|')
}

fn is_table_start(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() || !is_pipe_table_row(lines[index]) {
        return false;
    }
    let separator = lines[index + 1].trim();
    separator.ends_with('|')
        && !separator.is_empty()
        && separator
            .trim_start_matches('|')
            .chars()
            .all(|character| matches!(character, ' ' | '\t' | ':' | '|' | '-'))
}

fn parse_table(lines: &[&str]) -> MuyaJsonBlock {
    MuyaJsonBlock {
        block_type: "table".to_string(),
        headers: Some(split_row(lines.first().copied().unwrap_or(""))),
        alignments: Some(
            split_row(lines.get(1).copied().unwrap_or(""))
                .into_iter()
                .map(|cell| alignment(&cell).to_string())
                .collect(),
        ),
        rows: Some(lines.iter().skip(2).map(|line| split_row(line)).collect()),
        ..MuyaJsonBlock::default()
    }
}

fn split_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn alignment(cell: &str) -> &'static str {
    if cell.starts_with(':') && cell.ends_with(':') {
        "center"
    } else if cell.starts_with(':') {
        "left"
    } else if cell.ends_with(':') {
        "right"
    } else {
        "default"
    }
}

fn table_to_markdown(block: &MuyaJsonBlock) -> String {
    let headers = block.headers.as_deref().unwrap_or(&[]);
    let alignments = block.alignments.as_deref().unwrap_or(&[]);
    let rows = block.rows.as_deref().unwrap_or(&[]);
    let header = format!("| {} |", headers.join(" | "));
    let separator = format!(
        "| {} |",
        alignments
            .iter()
            .map(|value| match value.as_str() {
                "left" => ":-",
                "center" => ":-:",
                "right" => "-:",
                _ => "-",
            })
            .collect::<Vec<_>>()
            .join(" | ")
    );
    std::iter::once(header)
        .chain(std::iter::once(separator))
        .chain(rows.iter().map(|row| format!("| {} |", row.join(" | "))))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_legacy_json_state_block_types() {
        let markdown = "## Title\n\nParagraph\n\n> Quote\n\n- [x] Task\n\n  3. Ordered\n\n```rust\nfn main() {}\n```\n\n$$\nx+y\n$$\n\n| A | B |\n| :- | -: |\n| 1 | 2 |";
        let state = markdown_to_json_state(markdown);
        assert_eq!(
            state
                .blocks
                .iter()
                .map(|block| block.block_type.as_str())
                .collect::<Vec<_>>(),
            vec![
                "heading",
                "paragraph",
                "blockquote",
                "task_list_item",
                "list_item",
                "code_fence",
                "math_block",
                "table"
            ]
        );
        assert_eq!(state.blocks[0].level, Some(2));
        assert_eq!(state.blocks[3].checked, Some(true));
        assert_eq!(state.blocks[4].index, Some(3));
        assert_eq!(
            state.blocks[7].alignments.as_ref().unwrap(),
            &vec!["left".to_string(), "right".to_string()]
        );
    }

    #[test]
    fn serializes_with_legacy_blank_line_rules() {
        let markdown = "# A\n\n- [ ] B\n\n| X |\n| :-: |\n| Y |";
        let state = markdown_to_json_state(markdown);
        assert_eq!(json_state_to_markdown(&state), markdown);
    }

    #[test]
    fn creates_a_writable_empty_paragraph() {
        let state = markdown_to_json_state("");
        assert_eq!(state.blocks.len(), 1);
        assert_eq!(state.blocks[0].block_type, "paragraph");
        assert_eq!(state.blocks[0].text.as_deref(), Some(""));
    }

    #[test]
    fn normalizes_windows_line_endings() {
        let state = markdown_to_json_state("# A\r\n\r\nB");
        assert_eq!(json_state_to_markdown(&state), "# A\n\nB");
    }
}

