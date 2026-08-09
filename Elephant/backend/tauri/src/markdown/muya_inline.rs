use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaRenderInlineNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MuyaRenderInlineNode>>,
}

impl MuyaRenderInlineNode {
    fn text(text: String) -> Self {
        Self {
            node_type: "text".to_string(),
            text,
            ..Self::default()
        }
    }
}

pub fn parse_inlines(source: &str) -> Vec<MuyaRenderInlineNode> {
    let mut nodes = Vec::new();
    let mut offset = 0usize;

    while offset < source.len() {
        let rest = &source[offset..];

        if let Some((node, consumed)) = parse_image(rest) {
            nodes.push(node);
            offset += consumed;
            continue;
        }

        if let Some((node, consumed)) = parse_link(rest) {
            nodes.push(node);
            offset += consumed;
            continue;
        }

        if let Some((node, consumed)) = parse_paired(rest, "**", "strong", true)
            .or_else(|| parse_paired(rest, "__", "strong", true))
            .or_else(|| parse_paired(rest, "~~", "strike", true))
            .or_else(|| parse_paired(rest, "`", "code", false))
            .or_else(|| parse_paired(rest, "*", "emphasis", true))
            .or_else(|| parse_paired(rest, "_", "emphasis", true))
        {
            nodes.push(node);
            offset += consumed;
            continue;
        }

        if rest.starts_with("\\\n") {
            nodes.push(MuyaRenderInlineNode {
                node_type: "hard_break".to_string(),
                text: String::new(),
                marker: Some("\\".to_string()),
                ..MuyaRenderInlineNode::default()
            });
            offset += 2;
            continue;
        }

        let length = next_special_offset(rest).unwrap_or(rest.len()).max(1);
        push_text(&mut nodes, &rest[..length]);
        offset += length;
    }

    if nodes.is_empty() {
        nodes.push(MuyaRenderInlineNode::text(String::new()));
    }

    nodes
}

fn parse_image(source: &str) -> Option<(MuyaRenderInlineNode, usize)> {
    if !source.starts_with("![") {
        return None;
    }
    let label_end = source.get(2..)?.find("](")? + 2;
    let destination_start = label_end + 2;
    let destination_end = source.get(destination_start..)?.find(')')? + destination_start;
    let alt = source.get(2..label_end)?.to_string();
    let destination = source.get(destination_start..destination_end)?;
    let (href, title) = parse_destination(destination);
    if href.is_empty() {
        return None;
    }

    Some((
        MuyaRenderInlineNode {
            node_type: "image".to_string(),
            text: source.get(..=destination_end)?.to_string(),
            href: Some(href),
            title,
            alt: Some(alt),
            ..MuyaRenderInlineNode::default()
        },
        destination_end + 1,
    ))
}

fn parse_link(source: &str) -> Option<(MuyaRenderInlineNode, usize)> {
    if !source.starts_with('[') || source.starts_with("![") {
        return None;
    }
    let label_end = source.get(1..)?.find("](")? + 1;
    let destination_start = label_end + 2;
    let destination_end = source.get(destination_start..)?.find(')')? + destination_start;
    let label = source.get(1..label_end)?.to_string();
    let destination = source.get(destination_start..destination_end)?;
    let (href, title) = parse_destination(destination);
    if href.is_empty() {
        return None;
    }

    Some((
        MuyaRenderInlineNode {
            node_type: "link".to_string(),
            text: label.clone(),
            marker: Some("[]()".to_string()),
            href: Some(href),
            title,
            children: Some(parse_inlines(&label)),
            ..MuyaRenderInlineNode::default()
        },
        destination_end + 1,
    ))
}

fn parse_paired(
    source: &str,
    marker: &str,
    node_type: &str,
    recursive: bool,
) -> Option<(MuyaRenderInlineNode, usize)> {
    if !source.starts_with(marker) {
        return None;
    }
    let body_start = marker.len();
    let body_end = closing_marker_offset(source, marker, body_start)?;
    if body_end == body_start {
        return None;
    }
    let body = source.get(body_start..body_end)?.to_string();
    let consumed = body_end + marker.len();

    Some((
        MuyaRenderInlineNode {
            node_type: node_type.to_string(),
            text: body.clone(),
            marker: Some(marker.to_string()),
            children: recursive.then(|| parse_inlines(&body)),
            ..MuyaRenderInlineNode::default()
        },
        consumed,
    ))
}

fn closing_marker_offset(source: &str, marker: &str, from: usize) -> Option<usize> {
    let delimiter = marker.chars().next()?;
    let marker_width = marker.len();
    let mut search = from;

    while search < source.len() {
        let relative = source.get(search..)?.find(delimiter)?;
        let run_start = search + relative;
        let run_length = source
            .get(run_start..)?
            .chars()
            .take_while(|character| *character == delimiter)
            .map(char::len_utf8)
            .sum::<usize>();

        if run_length >= marker_width {
            // Use the rightmost part of a delimiter run. For `***`, this allows
            // the first star to close nested emphasis and the final two to close
            // the surrounding strong marker, matching Muya/CommonMark behavior.
            return Some(run_start + run_length - marker_width);
        }
        search = run_start + run_length.max(delimiter.len_utf8());
    }

    None
}

fn parse_destination(value: &str) -> (String, Option<String>) {
    let trimmed = value.trim();
    for quote in ['"', '\''] {
        if !trimmed.ends_with(quote) {
            continue;
        }
        let prefix = &trimmed[..trimmed.len().saturating_sub(1)];
        if let Some(index) = prefix.rfind(quote) {
            let has_separator = index > 0
                && prefix
                    .as_bytes()
                    .get(index.saturating_sub(1))
                    .is_some_and(|byte| byte.is_ascii_whitespace());
            if has_separator {
                let href = prefix[..index].trim().to_string();
                let title = prefix[index + quote.len_utf8()..].to_string();
                return (href, (!title.is_empty()).then_some(title));
            }
        }
    }
    (trimmed.to_string(), None)
}

fn next_special_offset(value: &str) -> Option<usize> {
    value.char_indices().skip(1).find_map(|(index, character)| {
        matches!(character, '!' | '[' | '*' | '_' | '~' | '`' | '\\').then_some(index)
    })
}

fn push_text(nodes: &mut Vec<MuyaRenderInlineNode>, value: &str) {
    if value.is_empty() {
        return;
    }
    if let Some(previous) = nodes.last_mut() {
        if previous.node_type == "text" {
            previous.text.push_str(value);
            return;
        }
    }
    nodes.push(MuyaRenderInlineNode::text(value.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_inline_marks_and_links() {
        let nodes = parse_inlines("**bold and *italic*** [docs](https://example.com \"Docs\")");
        assert_eq!(nodes[0].node_type, "strong");
        assert!(nodes[0]
            .children
            .as_ref()
            .is_some_and(|children| children.iter().any(|node| node.node_type == "emphasis")));
        assert_eq!(nodes[2].node_type, "link");
        assert_eq!(nodes[2].href.as_deref(), Some("https://example.com"));
        assert_eq!(nodes[2].title.as_deref(), Some("Docs"));
    }

    #[test]
    fn closes_separate_strong_ranges_independently() {
        let nodes = parse_inlines("**one** and **two**");
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.node_type == "strong")
                .count(),
            2
        );
        assert_eq!(nodes[0].text, "one");
        assert_eq!(nodes[2].text, "two");
    }

    #[test]
    fn parses_local_markdown_image_as_typed_node() {
        let nodes = parse_inlines("before ![drawing](../../.assets/drawing.png) after");
        let image = nodes
            .iter()
            .find(|node| node.node_type == "image")
            .expect("image node");
        assert_eq!(image.alt.as_deref(), Some("drawing"));
        assert_eq!(image.href.as_deref(), Some("../../.assets/drawing.png"));
    }

    #[test]
    fn leaves_unclosed_markers_as_text() {
        let nodes = parse_inlines("text ** unfinished");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type, "text");
        assert_eq!(nodes[0].text, "text ** unfinished");
    }
}

