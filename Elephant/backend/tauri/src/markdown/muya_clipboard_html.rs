use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HtmlNode {
    tag: String,
    attrs: BTreeMap<String, String>,
    children: Vec<HtmlNode>,
    text: String,
}

impl HtmlNode {
    fn root() -> Self {
        Self {
            tag: "root".to_string(),
            ..Self::default()
        }
    }

    fn text(value: String) -> Self {
        Self {
            tag: "#text".to_string(),
            text: value,
            ..Self::default()
        }
    }
}

pub fn clipboard_payload_to_markdown(html: &str, text: &str) -> String {
    if html.trim().is_empty() {
        return text.to_string();
    }
    let root = parse_html_fragment(html);
    normalize_markdown(&render_node(&root))
}

fn parse_html_fragment(input: &str) -> HtmlNode {
    let mut stack = vec![HtmlNode::root()];
    let mut cursor = 0usize;
    let bytes = input.as_bytes();

    while cursor < bytes.len() {
        if input[cursor..].starts_with("<!--") {
            cursor = input[cursor + 4..]
                .find("-->")
                .map_or(bytes.len(), |offset| cursor + 4 + offset + 3);
            continue;
        }

        if bytes[cursor] != b'<' {
            let next = input[cursor..]
                .find('<')
                .map_or(bytes.len(), |offset| cursor + offset);
            let value = decode_entities(&input[cursor..next]);
            if !value.is_empty() {
                stack
                    .last_mut()
                    .expect("root exists")
                    .children
                    .push(HtmlNode::text(value));
            }
            cursor = next;
            continue;
        }

        let Some(relative_end) = input[cursor..].find('>') else {
            let value = decode_entities(&input[cursor..]);
            stack
                .last_mut()
                .expect("root exists")
                .children
                .push(HtmlNode::text(value));
            break;
        };
        let end = cursor + relative_end;
        let raw_tag = input[cursor + 1..end].trim();
        cursor = end + 1;

        if raw_tag.starts_with('!') || raw_tag.starts_with('?') {
            continue;
        }

        if let Some(name) = raw_tag.strip_prefix('/') {
            close_element(&mut stack, normalize_tag_name(name));
            continue;
        }

        let self_closing = raw_tag.ends_with('/');
        let (tag, attrs) = parse_start_tag(raw_tag.trim_end_matches('/').trim());
        if tag.is_empty() {
            continue;
        }

        if matches!(tag.as_str(), "script" | "style") {
            if let Some(close) = find_case_insensitive(&input[cursor..], &format!("</{tag}>")) {
                cursor += close + tag.len() + 3;
            } else {
                cursor = bytes.len();
            }
            continue;
        }

        if matches!(
            tag.as_str(),
            "meta" | "link" | "input" | "img" | "br" | "hr"
        ) || self_closing
        {
            stack
                .last_mut()
                .expect("root exists")
                .children
                .push(HtmlNode {
                    tag,
                    attrs,
                    ..HtmlNode::default()
                });
            continue;
        }

        stack.push(HtmlNode {
            tag,
            attrs,
            ..HtmlNode::default()
        });
    }

    while stack.len() > 1 {
        close_top(&mut stack);
    }
    stack.pop().unwrap_or_else(HtmlNode::root)
}

fn close_element(stack: &mut Vec<HtmlNode>, tag: String) {
    if tag.is_empty() || stack.len() <= 1 {
        return;
    }
    if let Some(position) = stack.iter().rposition(|node| node.tag == tag) {
        while stack.len() > position {
            close_top(stack);
        }
    }
}

fn close_top(stack: &mut Vec<HtmlNode>) {
    if stack.len() <= 1 {
        return;
    }
    let node = stack.pop().expect("checked length");
    stack.last_mut().expect("parent exists").children.push(node);
}

fn parse_start_tag(raw: &str) -> (String, BTreeMap<String, String>) {
    let mut chars = raw.char_indices().peekable();
    let mut name_end = raw.len();
    while let Some((index, character)) = chars.next() {
        if character.is_whitespace() {
            name_end = index;
            break;
        }
    }
    let tag = normalize_tag_name(&raw[..name_end]);
    let attrs = parse_attributes(raw.get(name_end..).unwrap_or(""));
    (tag, attrs)
}

fn parse_attributes(raw: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    let mut cursor = 0usize;
    let bytes = raw.as_bytes();

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let start = cursor;
        while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'='
        {
            cursor += 1;
        }
        let name = raw[start..cursor].trim().to_ascii_lowercase();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let mut value = String::new();
        if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && matches!(bytes[cursor], b'\'' | b'"') {
                let quote = bytes[cursor];
                cursor += 1;
                let value_start = cursor;
                while cursor < bytes.len() && bytes[cursor] != quote {
                    cursor += 1;
                }
                value = decode_entities(&raw[value_start..cursor]);
                cursor = (cursor + 1).min(bytes.len());
            } else {
                let value_start = cursor;
                while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                value = decode_entities(&raw[value_start..cursor]);
            }
        }

        if !name.is_empty() && !name.starts_with("on") && !name.starts_with("data-") {
            attrs.insert(name, value);
        }
    }
    attrs
}

fn render_node(node: &HtmlNode) -> String {
    match node.tag.as_str() {
        "#text" => node.text.clone(),
        "root" | "body" | "html" | "span" | "section" | "article" => render_children(node),
        "script" | "style" | "meta" | "link" | "o:p" => String::new(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = node.tag[1..].parse::<usize>().unwrap_or(1);
            format!("{} {}\n\n", "#".repeat(level), render_children(node).trim())
        }
        "p" | "div" => format!("{}\n\n", render_children(node).trim()),
        "br" => "\n".to_string(),
        "hr" => "\n\n---\n\n".to_string(),
        "strong" | "b" => format!("**{}**", render_children(node).trim()),
        "em" | "i" => format!("*{}*", render_children(node).trim()),
        "del" | "s" | "strike" => format!("~~{}~~", render_children(node).trim()),
        "code" => format!("`{}`", render_children(node).trim()),
        "pre" => format!("```\n{}\n```\n\n", render_children(node).trim_matches('\n')),
        "blockquote" => {
            render_children(node)
                .lines()
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n\n"
        }
        "a" => {
            let label = render_children(node).trim().to_string();
            let href = safe_url(node.attrs.get("href").map(String::as_str).unwrap_or(""));
            format!("[{label}]({href})")
        }
        "img" => {
            let alt = node.attrs.get("alt").map(String::as_str).unwrap_or("");
            let src = safe_url(node.attrs.get("src").map(String::as_str).unwrap_or(""));
            format!("![{alt}]({src})")
        }
        "ul" | "ol" => render_children(node),
        "li" => render_list_item(node),
        "input" => String::new(),
        "table" => render_table(node),
        "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => render_children(node),
        _ => render_children(node),
    }
}

fn render_children(node: &HtmlNode) -> String {
    node.children.iter().map(render_node).collect::<String>()
}

fn render_list_item(node: &HtmlNode) -> String {
    let checked = descendant_input_state(node);
    let content = node
        .children
        .iter()
        .filter(|child| child.tag != "input")
        .map(render_node)
        .collect::<String>()
        .trim()
        .to_string();
    match checked {
        Some(true) => format!("- [x] {content}\n"),
        Some(false) => format!("- [ ] {content}\n"),
        None => format!("- {content}\n"),
    }
}

fn descendant_input_state(node: &HtmlNode) -> Option<bool> {
    for child in &node.children {
        if child.tag == "input" {
            let input_type = child
                .attrs
                .get("type")
                .map(|value| value.to_ascii_lowercase());
            if input_type
                .as_deref()
                .is_none_or(|value| value == "checkbox")
            {
                return Some(child.attrs.contains_key("checked"));
            }
        }
        if let Some(value) = descendant_input_state(child) {
            return Some(value);
        }
    }
    None
}

fn render_table(node: &HtmlNode) -> String {
    let mut rows = Vec::<Vec<String>>::new();
    collect_table_rows(node, &mut rows);
    if rows.is_empty() {
        return String::new();
    }
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return String::new();
    }
    for row in &mut rows {
        row.resize(columns, String::new());
    }
    let header = rows.remove(0);
    let mut lines = vec![
        format!("| {} |", header.join(" | ")),
        format!("| {} |", vec!["-"; columns].join(" | ")),
    ];
    lines.extend(
        rows.into_iter()
            .map(|row| format!("| {} |", row.join(" | "))),
    );
    format!("{}\n\n", lines.join("\n"))
}

fn collect_table_rows(node: &HtmlNode, rows: &mut Vec<Vec<String>>) {
    if node.tag == "tr" {
        let cells = node
            .children
            .iter()
            .filter(|child| matches!(child.tag.as_str(), "th" | "td"))
            .map(|child| collapse_whitespace(&render_children(child)))
            .collect::<Vec<_>>();
        if !cells.is_empty() {
            rows.push(cells);
        }
        return;
    }
    for child in &node.children {
        collect_table_rows(child, rows);
    }
}

fn safe_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.to_ascii_lowercase().starts_with("javascript:") {
        format!("blocked-javascript:{}", &trimmed["javascript:".len()..])
    } else {
        trimmed.to_string()
    }
}

fn normalize_markdown(value: &str) -> String {
    let mut output = String::new();
    let mut newline_run = 0usize;
    for character in value.chars() {
        if character == '\n' {
            newline_run += 1;
            if newline_run <= 2 {
                output.push(character);
            }
        } else {
            newline_run = 0;
            output.push(character);
        }
    }
    output.trim().to_string()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_tag_name(value: &str) -> String {
    value
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_plain_text() {
        assert_eq!(clipboard_payload_to_markdown("", "plain"), "plain");
    }

    #[test]
    fn converts_rich_inline_html_and_blocks() {
        let html = "<h2>Title</h2><p>Hello <strong>bold</strong> and <em>italic</em>.</p>";
        assert_eq!(
            clipboard_payload_to_markdown(html, ""),
            "## Title\n\nHello **bold** and *italic*."
        );
    }

    #[test]
    fn converts_links_images_tasks_and_tables() {
        let html = "<ul><li><input type=\"checkbox\" checked>Done</li><li>Item</li></ul><p><a href=\"https://example.com\">Site</a> <img src=\"a.png\" alt=\"A\"></p><table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>";
        let markdown = clipboard_payload_to_markdown(html, "");
        assert!(markdown.contains("- [x] Done"));
        assert!(markdown.contains("- Item"));
        assert!(markdown.contains("[Site](https://example.com) ![A](a.png)"));
        assert!(markdown.contains("| A | B |\n| - | - |\n| 1 | 2 |"));
    }

    #[test]
    fn removes_executable_content_and_blocks_javascript_urls() {
        let html =
            "<script>alert(1)</script><a href=\"javascript:alert(2)\" onclick=\"x()\">bad</a>";
        let markdown = clipboard_payload_to_markdown(html, "");
        assert!(!markdown.contains("alert(1)"));
        assert!(!markdown.contains("onclick"));
        assert!(markdown.contains("blocked-javascript:alert(2)"));
    }

    #[test]
    fn accepts_uppercase_tags_and_single_quoted_attributes() {
        let html = "<P><A HREF='https://example.com'>Link</A></P>";
        assert_eq!(
            clipboard_payload_to_markdown(html, ""),
            "[Link](https://example.com)"
        );
    }
}

