use crate::model::{
  Alignment, BlockKind, Document, InlineKind, InlineMarkKind, ListKind, MarkFragmentEdge, Node,
  NodeKind,
};

pub fn to_markdown(document: &Document) -> String {
  let mut blocks = Vec::new();
  for node in document.children(document.root) {
    blocks.push(serialize_block(document, node));
  }
  blocks.join("\n\n")
}

fn serialize_block(document: &Document, node: &Node) -> String {
  match &node.kind {
    NodeKind::Block(BlockKind::Paragraph) => serialize_inlines(document, node),
    NodeKind::Block(BlockKind::Heading { level }) => {
      format!(
        "{} {}",
        "#".repeat((*level).into()),
        serialize_inlines(document, node)
      )
    }
    NodeKind::Block(BlockKind::ThematicBreak) => "---".to_string(),
    NodeKind::Block(BlockKind::BlockQuote) => serialize_inlines(document, node)
      .lines()
      .map(|line| format!("> {line}"))
      .collect::<Vec<_>>()
      .join("\n"),
    NodeKind::Block(BlockKind::CodeBlock { language, .. }) => {
      format!(
        "```{}\n{}\n```",
        language.as_deref().unwrap_or(""),
        serialize_inlines(document, node)
      )
    }
    NodeKind::Block(BlockKind::FrontMatter { style }) => {
      let (opening, closing) = style.delimiters();
      format!(
        "{opening}\n{}\n{closing}",
        serialize_inlines(document, node)
      )
    }
    NodeKind::Block(BlockKind::List { kind, start }) => {
      serialize_list(document, node, *kind, *start)
    }
    NodeKind::Block(BlockKind::Table) => serialize_table(document, node),
    _ => serialize_inlines(document, node),
  }
}

fn serialize_list(document: &Document, list: &Node, kind: ListKind, start: Option<u64>) -> String {
  document
    .children(list.id)
    .enumerate()
    .map(|(index, item)| serialize_list_item(document, item, kind, start, index))
    .collect::<Vec<_>>()
    .join("\n")
}

fn serialize_list_item(
  document: &Document,
  item: &Node,
  kind: ListKind,
  start: Option<u64>,
  index: usize,
) -> String {
  let children = document.children(item.id).collect::<Vec<_>>();
  let content = children
    .first()
    .filter(|child| matches!(child.kind, NodeKind::Block(BlockKind::Paragraph)))
    .map(|paragraph| serialize_inlines(document, paragraph))
    .unwrap_or_default();
  let marker = match kind {
    ListKind::Unordered => "-".to_string(),
    ListKind::Ordered => format!("{}.", start.unwrap_or(1) + index as u64),
    ListKind::Task => {
      let checked = match &item.kind {
        NodeKind::Block(BlockKind::ListItem {
          checked: Some(true),
        }) => "x",
        _ => " ",
      };
      format!("- [{checked}]")
    }
  };

  let mut lines = vec![format!("{marker} {content}")];
  for child in children.into_iter().skip(1) {
    if matches!(child.kind, NodeKind::Block(BlockKind::CodeBlock { .. })) {
      lines.push("  ".to_string());
    }
    let nested = serialize_block(document, child);
    lines.extend(nested.lines().map(|line| format!("  {line}")));
  }
  lines.join("\n")
}

fn serialize_table(document: &Document, table: &Node) -> String {
  let rows = document.children(table.id).collect::<Vec<_>>();
  let Some(header) = rows.first() else {
    return String::new();
  };

  let alignments = document
    .children(header.id)
    .map(|cell| match &cell.kind {
      NodeKind::Block(BlockKind::TableCell { alignment, .. }) => *alignment,
      _ => Alignment::Default,
    })
    .collect::<Vec<_>>();
  let table_data = rows
    .iter()
    .map(|row| table_cells(document, row))
    .collect::<Vec<_>>();
  let mut column_widths = vec![5; alignments.len()];

  for row in &table_data {
    for (column, cell) in row.iter().take(column_widths.len()).enumerate() {
      column_widths[column] = column_widths[column].max(utf16_len(cell) + 2);
    }
  }

  let mut lines = vec![format_padded_table_row(&table_data[0], &column_widths)];
  lines.push(format_table_separator(&alignments, &column_widths));
  for row in table_data.into_iter().skip(1) {
    lines.push(format_padded_table_row(&row, &column_widths));
  }
  lines.join("\n")
}

fn table_cells(document: &Document, row: &Node) -> Vec<String> {
  document
    .children(row.id)
    .map(|cell| {
      escape_unescaped_pipes(&serialize_inlines(document, cell))
        .trim()
        .to_string()
    })
    .collect()
}

fn escape_unescaped_pipes(value: &str) -> String {
  let mut result = String::with_capacity(value.len());
  let mut escaped = false;
  for character in value.chars() {
    if character == '|' && !escaped {
      result.push('\\');
    }
    result.push(character);
    escaped = character == '\\' && !escaped;
    if character != '\\' {
      escaped = false;
    }
  }
  result
}

fn utf16_len(value: &str) -> usize {
  value.encode_utf16().count()
}

fn format_padded_table_row(cells: &[String], column_widths: &[usize]) -> String {
  let cells = cells
    .iter()
    .take(column_widths.len())
    .zip(column_widths)
    .map(|(cell, width)| {
      let padding = width.saturating_sub(utf16_len(cell) + 1);
      format!(" {cell}{}", " ".repeat(padding))
    })
    .collect::<Vec<_>>()
    .join("|");
  format!("|{cells}|")
}

fn format_table_separator(alignments: &[Alignment], column_widths: &[usize]) -> String {
  let cells = alignments
    .iter()
    .zip(column_widths)
    .map(|(alignment, width)| {
      let dashes = "-".repeat(width.saturating_sub(2));
      match alignment {
        Alignment::Default => format!(" {dashes} "),
        Alignment::Left => format!(":{dashes} "),
        Alignment::Center => format!(":{dashes}:"),
        Alignment::Right => format!(" {dashes}:"),
      }
    })
    .collect::<Vec<_>>()
    .join("|");
  format!("|{cells}|")
}

fn serialize_inlines(document: &Document, node: &Node) -> String {
  node
    .children
    .iter()
    .filter_map(|child| document.node(*child))
    .map(|child| serialize_inline(document, child))
    .collect::<Vec<_>>()
    .join("")
}

fn serialize_inline(document: &Document, node: &Node) -> String {
  match &node.kind {
    NodeKind::Inline(InlineKind::Text { value }) => value.clone(),
    NodeKind::Inline(InlineKind::Escaped { value }) => format!("\\{value}"),
    NodeKind::Inline(InlineKind::Emphasis) => {
      format!("*{}*", serialize_inlines(document, node))
    }
    NodeKind::Inline(InlineKind::Strong) => {
      format!("**{}**", serialize_inlines(document, node))
    }
    NodeKind::Inline(InlineKind::Strike) => {
      format!("~~{}~~", serialize_inlines(document, node))
    }
    NodeKind::Inline(InlineKind::MarkFragment { mark, edge, .. }) => {
      serialize_mark_fragment(document, node, *mark, *edge)
    }
    NodeKind::Inline(InlineKind::CodeSpan { code }) => {
      let delimiter = if code.contains('`') { "``" } else { "`" };
      format!("{delimiter}{code}{delimiter}")
    }
    NodeKind::Inline(InlineKind::Link { destination, title }) => {
      format!(
        "[{}]({}{})",
        serialize_inlines(document, node),
        destination,
        serialize_title(title)
      )
    }
    NodeKind::Inline(InlineKind::Image { source, title, alt }) => {
      format!("![{alt}]({source}{})", serialize_title(title))
    }
    NodeKind::Inline(InlineKind::AutoLink { destination }) => {
      format!("<{destination}>")
    }
    NodeKind::Inline(InlineKind::InlineHtml { raw }) => raw.clone(),
    NodeKind::Inline(InlineKind::InlineMath { source }) => format!("${source}$"),
    NodeKind::Inline(InlineKind::Emoji { shortcode, .. }) => format!(":{shortcode}:"),
    NodeKind::Inline(InlineKind::Superscript) => {
      format!("^{}^", serialize_inlines(document, node))
    }
    NodeKind::Inline(InlineKind::Subscript) => {
      format!("~{}~", serialize_inlines(document, node))
    }
    NodeKind::Inline(InlineKind::FootnoteReference { label }) => {
      format!("[^{label}]")
    }
    NodeKind::Inline(InlineKind::SoftBreak) => "\n".to_string(),
    NodeKind::Inline(InlineKind::HardBreak) => "  \n".to_string(),
    _ => serialize_inlines(document, node),
  }
}

fn serialize_mark_fragment(
  document: &Document,
  node: &Node,
  mark: InlineMarkKind,
  edge: MarkFragmentEdge,
) -> String {
  let delimiter = match mark {
    InlineMarkKind::Emphasis => "*",
    InlineMarkKind::Strong => "**",
    InlineMarkKind::Strike => "~~",
  };
  let content = serialize_inlines(document, node);
  match edge {
    MarkFragmentEdge::Start => format!("{delimiter}{content}"),
    MarkFragmentEdge::Middle => content,
    MarkFragmentEdge::End => format!("{content}{delimiter}"),
  }
}

fn serialize_title(title: &Option<String>) -> String {
  title
    .as_ref()
    .map(|value| format!(" \"{}\"", value.replace('"', "\\\"")))
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;
  use crate::parse_markdown;

  #[test]
  fn serializes_the_initial_supported_slice() {
    let document = parse_markdown("# Title\n\nBody\n\n---\n");
    assert_eq!(to_markdown(&document), "# Title\n\nBody\n\n---");
  }

  #[test]
  fn serializes_blockquotes_and_fenced_code() {
    let document = parse_markdown("> Quote\n> continued\n\n```rust\nfn main() {}\n```\n");
    assert_eq!(
      to_markdown(&document),
      "> Quote\n> continued\n\n```rust\nfn main() {}\n```"
    );
  }

  #[test]
  fn canonicalizes_setext_lists_and_tables() {
    let document = parse_markdown(
      "Title\n=====\n\n3. three\n4. four\n\n- [x] done\n- [ ] todo\n\n| Name | Score |\n| :--- | ---: |\n| Ada | 10 |\n",
    );
    assert_eq!(
      to_markdown(&document),
      "# Title\n\n3. three\n4. four\n\n- [x] done\n- [ ] todo\n\n| Name | Score |\n|:---- | -----:|\n| Ada  | 10    |"
    );
  }

  #[test]
  fn uses_utf16_widths_when_padding_table_cells() {
    let document = parse_markdown("| A | B |\n| --- | --- |\n| 😀 | x |");
    assert_eq!(
      to_markdown(&document),
      "| A   | B   |\n| --- | --- |\n| 😀  | x   |"
    );
  }

  #[test]
  fn round_trips_nested_lists() {
    let markdown = "- parent\n  - child\n    3. grandchild\n- sibling";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
  }

  #[test]
  fn round_trips_the_executable_inline_slice() {
    let markdown = "A **bold** *soft* ~~gone~~ [link](https://example.com \"Title\") ![alt](image.png) `code` \\*.";
    let document = parse_markdown(markdown);
    assert_eq!(to_markdown(&document), markdown);
  }
}
