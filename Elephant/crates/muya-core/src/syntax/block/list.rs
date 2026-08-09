use crate::model::ListKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListMarker<'a> {
  pub kind: ListKind,
  pub start: Option<u64>,
  pub checked: Option<bool>,
  pub content: &'a str,
  pub indent: usize,
}

pub fn parse(line: &str) -> Option<ListMarker<'_>> {
  let trimmed = line.trim_start_matches(' ');
  let indent = line.len().saturating_sub(trimmed.len());
  parse_unordered(trimmed, indent).or_else(|| parse_ordered(trimmed, indent))
}

fn parse_unordered(line: &str, indent: usize) -> Option<ListMarker<'_>> {
  let marker = line.chars().next()?;
  if !matches!(marker, '-' | '*' | '+') {
    return None;
  }

  let rest = line.get(marker.len_utf8()..)?.strip_prefix(' ')?;
  if let Some(task) = parse_task(rest, indent) {
    return Some(task);
  }

  Some(ListMarker {
    kind: ListKind::Unordered,
    start: None,
    checked: None,
    content: rest,
    indent,
  })
}

fn parse_task(rest: &str, indent: usize) -> Option<ListMarker<'_>> {
  if let Some(content) = rest.strip_prefix("[ ] ") {
    return Some(ListMarker {
      kind: ListKind::Task,
      start: None,
      checked: Some(false),
      content,
      indent,
    });
  }

  rest
    .strip_prefix("[x] ")
    .or_else(|| rest.strip_prefix("[X] "))
    .map(|content| ListMarker {
      kind: ListKind::Task,
      start: None,
      checked: Some(true),
      content,
      indent,
    })
}

fn parse_ordered(line: &str, indent: usize) -> Option<ListMarker<'_>> {
  let digits = line
    .chars()
    .take_while(|character| character.is_ascii_digit())
    .count();
  if digits == 0 || digits > 9 {
    return None;
  }

  let start = line.get(..digits)?.parse::<u64>().ok()?;
  let suffix = line.get(digits..)?;
  let rest = suffix
    .strip_prefix(". ")
    .or_else(|| suffix.strip_prefix(") "))?;

  Some(ListMarker {
    kind: ListKind::Ordered,
    start: Some(start),
    checked: None,
    content: rest,
    indent,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_unordered_ordered_and_task_items() {
    assert_eq!(
      parse("- item"),
      Some(ListMarker {
        kind: ListKind::Unordered,
        start: None,
        checked: None,
        content: "item",
        indent: 0,
      })
    );
    assert_eq!(
      parse("3. item"),
      Some(ListMarker {
        kind: ListKind::Ordered,
        start: Some(3),
        checked: None,
        content: "item",
        indent: 0,
      })
    );
    assert_eq!(
      parse("- [x] done"),
      Some(ListMarker {
        kind: ListKind::Task,
        start: None,
        checked: Some(true),
        content: "done",
        indent: 0,
      })
    );
  }

  #[test]
  fn preserves_nested_indentation() {
    assert_eq!(
      parse("    - nested"),
      Some(ListMarker {
        kind: ListKind::Unordered,
        start: None,
        checked: None,
        content: "nested",
        indent: 4,
      })
    );
  }

  #[test]
  fn rejects_incomplete_markers() {
    assert_eq!(parse("1.item"), None);
    assert_eq!(parse("-item"), None);
  }
}
