use crate::model::Alignment;

pub fn split_cells(line: &str) -> Vec<String> {
  let trimmed = line.trim();
  let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
  let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);

  let mut cells = Vec::new();
  let mut current = String::new();
  let mut escaped = false;

  for character in trimmed.chars() {
    if escaped {
      if character == '|' {
        current.push('|');
      } else {
        current.push('\\');
        current.push(character);
      }
      escaped = false;
      continue;
    }

    match character {
      '\\' => escaped = true,
      '|' => {
        cells.push(current.trim().to_string());
        current.clear();
      }
      _ => current.push(character),
    }
  }

  if escaped {
    current.push('\\');
  }
  cells.push(current.trim().to_string());
  cells
}

pub fn parse_delimiter(line: &str) -> Option<Vec<Alignment>> {
  let cells = split_cells(line);
  if cells.is_empty() {
    return None;
  }

  cells
    .into_iter()
    .map(|cell| {
      let compact = cell.trim();
      let left = compact.starts_with(':');
      let right = compact.ends_with(':');
      let rule = compact.trim_matches(':');
      if rule.len() < 3 || !rule.chars().all(|character| character == '-') {
        return None;
      }
      Some(match (left, right) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        (false, false) => Alignment::Default,
      })
    })
    .collect()
}

pub fn looks_like_row(line: &str) -> bool {
  line.contains('|')
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn splits_escaped_table_cells() {
    assert_eq!(split_cells("| a | b\\|c |"), vec!["a", "b|c"]);
  }

  #[test]
  fn parses_alignment_delimiters() {
    assert_eq!(
      parse_delimiter("| :--- | ---: | :---: | --- |"),
      Some(vec![
        Alignment::Left,
        Alignment::Right,
        Alignment::Center,
        Alignment::Default
      ])
    );
    assert_eq!(parse_delimiter("| -- | nope |"), None);
  }
}
