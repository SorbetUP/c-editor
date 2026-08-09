mod crossing_marks;
pub mod inline;

use crate::model::{
  Alignment, BlockKind, Document, FrontMatterStyle, ListKind, NodeId, NodeKind, SourceRange,
};
use crate::syntax::block::{
  blockquote, fenced_code, front_matter, heading, list, paragraph, table, thematic_break,
};

#[derive(Clone, Debug)]
struct SourceLine {
  text: String,
  start: u32,
  end: u32,
}

pub fn parse_markdown(markdown: &str) -> Document {
  let lines = source_lines(markdown);
  let rejected_front_matter = rejected_front_matter_style(&lines);
  let mut document = Document::new();
  let mut index = 0usize;

  while index < lines.len() {
    let line = &lines[index];
    if line.text.trim().is_empty() {
      index += 1;
      continue;
    }

    if index == 0 {
      if let Some(next) = try_parse_front_matter(&mut document, &lines) {
        index = next;
        continue;
      }
    }

    if let Some(opening) = fenced_code::parse_opening(&line.text) {
      index = parse_fenced_code(&mut document, &lines, index, opening);
      continue;
    }

    if let Some(level) = lines
      .get(index + 1)
      .and_then(|underline| heading::parse_setext_underline(&underline.text))
    {
      let underline = &lines[index + 1];
      let reserved_front_matter_delimiter =
        rejected_front_matter.is_some_and(|style| front_matter::is_closing(&underline.text, style));
      if !reserved_front_matter_delimiter {
        append_text_block(
          &mut document,
          BlockKind::Heading { level },
          line.text.trim(),
          SourceRange::new(line.start, underline.end),
        );
        index += 2;
        continue;
      }
    }

    if table::looks_like_row(&line.text) {
      if let Some(alignments) = lines
        .get(index + 1)
        .and_then(|delimiter| table::parse_delimiter(&delimiter.text))
      {
        index = parse_table(&mut document, &lines, index, alignments);
        continue;
      }
    }

    if let Some(parsed) = heading::parse_atx(&line.text) {
      append_text_block(
        &mut document,
        BlockKind::Heading {
          level: parsed.level,
        },
        parsed.text,
        SourceRange::new(line.start, line.end),
      );
      index += 1;
      continue;
    }

    if thematic_break::matches(&line.text) {
      let block = document.allocate(
        NodeKind::Block(BlockKind::ThematicBreak),
        Some(SourceRange::new(line.start, line.end)),
      );
      document.append_child(document.root, block);
      index += 1;
      continue;
    }

    if blockquote::strip_marker(&line.text).is_some() {
      index = parse_blockquote(&mut document, &lines, index);
      continue;
    }

    if list::parse(&line.text).is_some() {
      index = parse_list(&mut document, &lines, index);
      continue;
    }

    index = parse_paragraph(&mut document, &lines, index);
  }

  document
}

fn rejected_front_matter_style(lines: &[SourceLine]) -> Option<FrontMatterStyle> {
  let style = front_matter::parse_opening(&lines.first()?.text)?;
  let closing_index = lines
    .iter()
    .enumerate()
    .skip(1)
    .find_map(|(index, line)| front_matter::is_closing(&line.text, style).then_some(index));
  match closing_index {
    None => Some(style),
    Some(index)
      if index == 1
        || lines
          .get(index + 1)
          .is_some_and(|line| !line.text.is_empty()) =>
    {
      Some(style)
    }
    Some(_) => None,
  }
}

fn try_parse_front_matter(document: &mut Document, lines: &[SourceLine]) -> Option<usize> {
  let opening = lines.first()?;
  let style = front_matter::parse_opening(&opening.text)?;
  let closing_index = lines
    .iter()
    .enumerate()
    .skip(1)
    .find_map(|(index, line)| front_matter::is_closing(&line.text, style).then_some(index))?;
  if closing_index == 1
    || lines
      .get(closing_index + 1)
      .is_some_and(|line| !line.text.is_empty())
  {
    return None;
  }

  let body = lines[1..closing_index]
    .iter()
    .map(|line| line.text.as_str())
    .collect::<Vec<_>>()
    .join("\n");
  append_literal_block(
    document,
    BlockKind::FrontMatter { style },
    &body,
    SourceRange::new(opening.start, lines[closing_index].end),
  );
  Some(closing_index + 1)
}

fn source_lines(markdown: &str) -> Vec<SourceLine> {
  let mut offset = 0u32;
  markdown
    .split_inclusive('\n')
    .map(|segment| {
      let text = segment
        .strip_suffix('\n')
        .unwrap_or(segment)
        .trim_end_matches('\r')
        .to_string();
      let end = offset + text.encode_utf16().count() as u32;
      let next = offset + segment.encode_utf16().count() as u32;
      let line = SourceLine {
        text,
        start: offset,
        end,
      };
      offset = next;
      line
    })
    .collect()
}

fn parse_fenced_code(
  document: &mut Document,
  lines: &[SourceLine],
  start_index: usize,
  opening: fenced_code::FenceOpen,
) -> usize {
  let mut index = start_index + 1;
  let mut body = Vec::new();
  let mut end = lines[start_index].end;

  while index < lines.len() {
    let line = &lines[index];
    end = line.end;
    if fenced_code::is_closing(&line.text, &opening) {
      index += 1;
      break;
    }
    body.push(line.text.clone());
    index += 1;
  }

  append_literal_block(
    document,
    BlockKind::CodeBlock {
      language: opening.info,
      fenced: true,
    },
    &body.join("\n"),
    SourceRange::new(lines[start_index].start, end),
  );
  index
}

fn parse_blockquote(document: &mut Document, lines: &[SourceLine], start_index: usize) -> usize {
  let mut index = start_index;
  let mut body = Vec::new();
  let mut end = lines[start_index].end;

  while let Some(line) = lines.get(index) {
    let Some(text) = blockquote::strip_marker(&line.text) else {
      break;
    };
    body.push(text.to_string());
    end = line.end;
    index += 1;
  }

  append_text_block(
    document,
    BlockKind::BlockQuote,
    &body.join("\n"),
    SourceRange::new(lines[start_index].start, end),
  );
  index
}

fn parse_list(document: &mut Document, lines: &[SourceLine], start_index: usize) -> usize {
  let first = list::parse(&lines[start_index].text).expect("list marker must exist");
  let (list_node, index, _) =
    parse_list_level(document, lines, start_index, first.indent, first.kind);
  document.append_child(document.root, list_node);
  index
}

fn parse_list_level(
  document: &mut Document,
  lines: &[SourceLine],
  start_index: usize,
  indent: usize,
  list_kind: ListKind,
) -> (NodeId, usize, u32) {
  let first = list::parse(&lines[start_index].text).expect("nested list marker must exist");
  let list_node = document.allocate(
    NodeKind::Block(BlockKind::List {
      kind: list_kind,
      start: first.start,
    }),
    None,
  );
  let mut index = start_index;
  let mut end = lines[start_index].end;

  while let Some(line) = lines.get(index) {
    let Some(marker) = list::parse(&line.text) else {
      break;
    };
    if marker.indent != indent || marker.kind != list_kind {
      break;
    }

    let range = SourceRange::new(line.start, line.end);
    let item = document.allocate(
      NodeKind::Block(BlockKind::ListItem {
        checked: marker.checked,
      }),
      Some(range),
    );
    let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), Some(range));
    inline::append_inlines(document, paragraph, marker.content, line.start);
    document.append_child(item, paragraph);
    document.append_child(list_node, item);
    end = line.end;
    index += 1;

    while let Some(next_line) = lines.get(index) {
      let Some(next_marker) = list::parse(&next_line.text) else {
        break;
      };
      if next_marker.indent <= indent {
        break;
      }
      let (nested, next_index, nested_end) =
        parse_list_level(document, lines, index, next_marker.indent, next_marker.kind);
      document.append_child(item, nested);
      index = next_index;
      end = nested_end;
    }
  }

  if let Some(node) = document.node_mut(list_node) {
    node.source = Some(SourceRange::new(lines[start_index].start, end));
  }
  (list_node, index, end)
}

fn parse_table(
  document: &mut Document,
  lines: &[SourceLine],
  start_index: usize,
  alignments: Vec<Alignment>,
) -> usize {
  let header = table::split_cells(&lines[start_index].text);
  let mut rows = Vec::new();
  let mut index = start_index + 2;
  let mut end = lines[start_index + 1].end;

  while let Some(line) = lines.get(index) {
    if line.text.trim().is_empty() || !table::looks_like_row(&line.text) {
      break;
    }
    rows.push((table::split_cells(&line.text), line.start));
    end = line.end;
    index += 1;
  }

  let table_node = document.allocate(
    NodeKind::Block(BlockKind::Table),
    Some(SourceRange::new(lines[start_index].start, end)),
  );
  append_table_row(
    document,
    table_node,
    &header,
    &alignments,
    true,
    lines[start_index].start,
  );
  for (row, start) in rows {
    append_table_row(document, table_node, &row, &alignments, false, start);
  }
  document.append_child(document.root, table_node);
  index
}

fn append_table_row(
  document: &mut Document,
  table_node: NodeId,
  values: &[String],
  alignments: &[Alignment],
  header: bool,
  source_start: u32,
) {
  let row = document.allocate(NodeKind::Block(BlockKind::TableRow), None);
  for (column, alignment) in alignments.iter().copied().enumerate() {
    let cell = document.allocate(
      NodeKind::Block(BlockKind::TableCell { alignment, header }),
      None,
    );
    inline::append_inlines(
      document,
      cell,
      values.get(column).map(String::as_str).unwrap_or_default(),
      source_start,
    );
    document.append_child(row, cell);
  }
  document.append_child(table_node, row);
}

fn parse_paragraph(document: &mut Document, lines: &[SourceLine], start_index: usize) -> usize {
  let mut index = start_index;
  let mut values = Vec::new();
  let mut end = lines[start_index].end;

  while let Some(line) = lines.get(index) {
    if line.text.trim().is_empty() || (index > start_index && starts_block(lines, index)) {
      break;
    }
    values.push(line.text.clone());
    end = line.end;
    index += 1;
  }

  let text = paragraph::join_lines(&values);
  append_text_block(
    document,
    BlockKind::Paragraph,
    &text,
    SourceRange::new(lines[start_index].start, end),
  );
  index
}

fn starts_block(lines: &[SourceLine], index: usize) -> bool {
  let line = &lines[index].text;
  fenced_code::parse_opening(line).is_some()
    || heading::parse_atx(line).is_some()
    || thematic_break::matches(line)
    || blockquote::strip_marker(line).is_some()
    || list::parse(line).is_some()
    || lines
      .get(index + 1)
      .is_some_and(|next| heading::parse_setext_underline(&next.text).is_some())
    || (table::looks_like_row(line)
      && lines
        .get(index + 1)
        .is_some_and(|next| table::parse_delimiter(&next.text).is_some()))
}

fn append_text_block(document: &mut Document, kind: BlockKind, content: &str, source: SourceRange) {
  let block = document.allocate(NodeKind::Block(kind), Some(source));
  inline::append_inlines(document, block, content, source.start);
  document.append_child(document.root, block);
}

fn append_literal_block(
  document: &mut Document,
  kind: BlockKind,
  content: &str,
  source: SourceRange,
) {
  let block = document.allocate(NodeKind::Block(kind), Some(source));
  inline::append_literal(document, block, content, source.start);
  document.append_child(document.root, block);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::serializer::to_markdown;

  #[test]
  fn parses_yaml_and_toml_front_matter_before_other_blocks() {
    let yaml = parse_markdown("---\ntitle: Alpha\ntags:\n  - one\n---\n\nbody");
    let first = yaml.children(yaml.root).next().unwrap();
    assert!(matches!(
      first.kind,
      NodeKind::Block(BlockKind::FrontMatter {
        style: FrontMatterStyle::Yaml
      })
    ));
    assert_eq!(
      to_markdown(&yaml),
      "---\ntitle: Alpha\ntags:\n  - one\n---\n\nbody"
    );

    let toml = parse_markdown("+++\ntitle = \"Alpha\"\n+++\n\nbody");
    let first = toml.children(toml.root).next().unwrap();
    assert!(matches!(
      first.kind,
      NodeKind::Block(BlockKind::FrontMatter {
        style: FrontMatterStyle::Toml
      })
    ));
    assert_eq!(to_markdown(&toml), "+++\ntitle = \"Alpha\"\n+++\n\nbody");
  }

  #[test]
  fn leaves_invalid_front_matter_delimiters_as_markdown() {
    let unclosed = parse_markdown("---\ntitle: Alpha");
    assert_eq!(to_markdown(&unclosed), "---\n\ntitle: Alpha");

    let no_blank_line = parse_markdown("---\ntitle: Alpha\n---\nbody");
    assert_eq!(
      to_markdown(&no_blank_line),
      "---\n\ntitle: Alpha\n\n---\n\nbody"
    );
  }

  #[test]
  fn parses_nested_lists_structurally() {
    let document = parse_markdown("- parent\n  - child\n    3. grandchild\n- sibling");
    let list = document.children(document.root).next().unwrap().id;
    let first_item = document.children(list).next().unwrap().id;
    let nested = document.children(first_item).nth(1).unwrap();
    assert!(matches!(
      nested.kind,
      NodeKind::Block(BlockKind::List {
        kind: ListKind::Unordered,
        ..
      })
    ));
    let nested_item = document.children(nested.id).next().unwrap().id;
    let ordered = document.children(nested_item).nth(1).unwrap();
    assert!(matches!(
      ordered.kind,
      NodeKind::Block(BlockKind::List {
        kind: ListKind::Ordered,
        start: Some(3)
      })
    ));
    assert_eq!(
      to_markdown(&document),
      "- parent\n  - child\n    3. grandchild\n- sibling"
    );
  }
}
