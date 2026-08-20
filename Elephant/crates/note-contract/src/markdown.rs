#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineMark {
    Bold,
    Italic,
    Strike,
    Code,
}

pub fn parse_tags(markdown: &str) -> Vec<String> {
    markdown
        .split_whitespace()
        .filter_map(|word| word.strip_prefix('#'))
        .map(|tag| tag.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_'))
        .filter(|tag| !tag.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn toggle_task(markdown: &str, line_index: usize) -> Result<String, crate::NoteError> {
    let mut lines: Vec<String> = markdown.lines().map(str::to_owned).collect();
    let Some(line) = lines.get_mut(line_index) else {
        return Err(crate::NoteError::InvalidTaskLine);
    };
    if !line.contains("- [") {
        return Err(crate::NoteError::InvalidTaskLine);
    }
    if line.contains("- [ ]") {
        *line = line.replacen("- [ ]", "- [x]", 1);
    } else if line.contains("- [x]") || line.contains("- [X]") {
        *line = line
            .replacen("- [x]", "- [ ]", 1)
            .replacen("- [X]", "- [ ]", 1);
    } else {
        return Err(crate::NoteError::InvalidTaskLine);
    }
    Ok(lines.join("\n"))
}
