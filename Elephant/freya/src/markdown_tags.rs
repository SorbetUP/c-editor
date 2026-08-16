//! Markdown tag semantics shared by Freya's vault, editor, and metadata UI.

pub(crate) fn normalize_tag(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value.strip_prefix(['"', '\'']).unwrap_or(value);
    let value = value.strip_suffix(['"', '\'']).unwrap_or(value);
    let value = value.trim().trim_start_matches('#');
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty()).then_some(value)
}

pub(crate) fn parse_tag_list(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(value);
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            current.push(character);
            escaped = true;
        } else if matches!(character, '"' | '\'') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            current.push(character);
        } else if character == ',' && quote.is_none() {
            if let Some(tag) = normalize_tag(&current) {
                items.push(tag);
            }
            current.clear();
        } else {
            current.push(character);
        }
    }
    if let Some(tag) = normalize_tag(&current) {
        items.push(tag);
    }
    items
}

pub(crate) fn parse_markdown_tags(markdown: &str) -> Vec<String> {
    let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let mut lines = markdown.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }

    let mut header = Vec::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        header.push(line);
    }
    if !closed {
        return Vec::new();
    }
    let Some(index) = header
        .iter()
        .position(|line| field_value(line, "tags").is_some())
    else {
        return Vec::new();
    };
    let raw = field_value(header[index], "tags").unwrap_or_default();
    if !raw.is_empty() {
        return parse_tag_list(raw);
    }

    header
        .iter()
        .skip(index + 1)
        .take_while(|line| !is_field(line))
        .filter_map(|line| line.trim().strip_prefix('-'))
        .filter_map(normalize_tag)
        .collect()
}

pub(crate) fn update_markdown_tags(markdown: &str, tags: &[String], title: &str) -> String {
    let mut normalized = Vec::new();
    for tag in tags {
        if let Some(tag) = normalize_tag(tag) {
            if !normalized.contains(&tag) {
                normalized.push(tag);
            }
        }
    }
    let tags_line = format!(
        "tags: [{}]",
        normalized
            .iter()
            .map(|tag| format!("\"{}\"", tag.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut lines = markdown.lines().map(str::to_owned).collect::<Vec<_>>();
    if lines.first().is_some_and(|line| line.trim() == "---") {
        if let Some(end) = lines
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(index, line)| (line.trim() == "---").then_some(index))
        {
            if let Some(index) = lines
                .iter()
                .take(end)
                .position(|line| field_value(line, "tags").is_some())
            {
                let had_block_tags = field_value(&lines[index], "tags").is_some_and(str::is_empty);
                lines[index] = tags_line;
                if had_block_tags {
                    let delete_count = lines[index + 1..end]
                        .iter()
                        .take_while(|line| {
                            !is_field(line)
                                && (line.trim().is_empty() || line.trim().starts_with('-'))
                        })
                        .count();
                    lines.drain(index + 1..index + 1 + delete_count);
                }
            } else {
                let insert_at = lines
                    .iter()
                    .enumerate()
                    .take(end)
                    .find_map(|(index, line)| {
                        ["title", "type", "createdAt", "updatedAt"]
                            .iter()
                            .any(|key| field_value(line, key).is_some())
                            .then_some(index + 1)
                    })
                    .unwrap_or(1);
                lines.insert(insert_at, tags_line);
            }
            return lines.join("\n");
        }
    }

    let normalized_title = title.trim();
    let mut frontmatter = vec!["---".to_string()];
    if !normalized_title.is_empty() {
        frontmatter.push(format!(
            "title: \"{}\"",
            normalized_title.replace('"', "\\\"")
        ));
    }
    frontmatter.push("type: \"note\"".to_string());
    frontmatter.push(tags_line);
    frontmatter.push("---".to_string());
    let body = markdown.trim();
    if body.is_empty() {
        frontmatter.join("\n")
    } else {
        format!("{}\n\n{}", frontmatter.join("\n"), body)
    }
}

fn field_value<'a>(line: &'a str, expected: &str) -> Option<&'a str> {
    let (key, value) = line.split_once(':')?;
    (key.trim() == expected).then(|| value.trim())
}

fn is_field(line: &str) -> bool {
    line.split_once(':').is_some_and(|(key, _)| {
        let key = key.trim();
        !key.is_empty()
            && key.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
            })
    })
}
