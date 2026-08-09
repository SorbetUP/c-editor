use crate::model::InlineMarkKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CrossingMark {
  pub mark: InlineMarkKind,
  pub start_utf16: u32,
  pub end_utf16: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CrossingMarks {
  pub base: String,
  pub marks: Vec<CrossingMark>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Token {
  start: usize,
  delimiter: &'static str,
  mark: InlineMarkKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Interval {
  open: usize,
  close: usize,
  delimiter: &'static str,
  mark: InlineMarkKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RemovedDelimiter {
  start: usize,
  end: usize,
}

pub(super) fn detect(source: &str) -> Option<CrossingMarks> {
  if source.contains("***") || source.contains("___") {
    return None;
  }

  let mut active = intervals(&tokens(source));
  let mut selected = Vec::new();
  loop {
    let Some((index, _)) = active
      .iter()
      .enumerate()
      .filter_map(|(index, candidate)| {
        let crossings = active
          .iter()
          .filter(|other| candidate != *other && crosses(*candidate, **other))
          .count();
        (crossings > 0).then_some((index, crossings))
      })
      .max_by(|(left_index, left_count), (right_index, right_count)| {
        left_count
          .cmp(right_count)
          .then_with(|| active[*right_index].open.cmp(&active[*left_index].open))
      })
    else {
      break;
    };
    selected.push(active.remove(index));
  }
  if selected.is_empty() {
    return None;
  }

  let mut removals = selected
    .iter()
    .flat_map(|interval| {
      let length = interval.delimiter.len();
      [
        RemovedDelimiter {
          start: interval.open,
          end: interval.open + length,
        },
        RemovedDelimiter {
          start: interval.close,
          end: interval.close + length,
        },
      ]
    })
    .collect::<Vec<_>>();
  removals.sort_by_key(|removed| removed.start);

  let base = remove_delimiters(source, &removals);
  let marks = selected
    .into_iter()
    .map(|interval| CrossingMark {
      mark: interval.mark,
      start_utf16: mapped_utf16_offset(source, &removals, interval.open),
      end_utf16: mapped_utf16_offset(source, &removals, interval.close),
    })
    .collect();

  Some(CrossingMarks { base, marks })
}

fn remove_delimiters(source: &str, removals: &[RemovedDelimiter]) -> String {
  let removed_bytes = removals
    .iter()
    .map(|removed| removed.end - removed.start)
    .sum::<usize>();
  let mut output = String::with_capacity(source.len().saturating_sub(removed_bytes));
  let mut cursor = 0usize;
  for removed in removals {
    output.push_str(&source[cursor..removed.start]);
    cursor = removed.end;
  }
  output.push_str(&source[cursor..]);
  output
}

fn mapped_utf16_offset(source: &str, removals: &[RemovedDelimiter], original_offset: usize) -> u32 {
  let removed_utf16 = removals
    .iter()
    .filter(|removed| removed.start < original_offset)
    .map(|removed| utf16_len(&source[removed.start..removed.end]))
    .sum::<u32>();
  utf16_len(&source[..original_offset]).saturating_sub(removed_utf16)
}

fn tokens(source: &str) -> Vec<Token> {
  let mut output = Vec::new();
  let mut offset = 0usize;

  while offset < source.len() {
    let remaining = &source[offset..];
    if remaining.starts_with('\\') {
      offset += remaining.chars().take(2).map(char::len_utf8).sum::<usize>();
      continue;
    }
    if remaining.starts_with('`') {
      let run = remaining
        .chars()
        .take_while(|character| *character == '`')
        .count();
      let delimiter = "`".repeat(run);
      if let Some(closing) = remaining[run..].find(&delimiter) {
        offset += run + closing + delimiter.len();
      } else {
        offset += run;
      }
      continue;
    }

    let matched = [
      ("~~", InlineMarkKind::Strike),
      ("**", InlineMarkKind::Strong),
      ("__", InlineMarkKind::Strong),
      ("*", InlineMarkKind::Emphasis),
      ("_", InlineMarkKind::Emphasis),
    ]
    .into_iter()
    .find(|(delimiter, _)| remaining.starts_with(delimiter));

    if let Some((delimiter, mark)) = matched {
      output.push(Token {
        start: offset,
        delimiter,
        mark,
      });
      offset += delimiter.len();
    } else {
      offset += remaining.chars().next().map(char::len_utf8).unwrap_or(1);
    }
  }

  output
}

fn intervals(tokens: &[Token]) -> Vec<Interval> {
  let mut output = Vec::new();
  for delimiter in ["~~", "**", "__", "*", "_"] {
    let matching = tokens
      .iter()
      .filter(|token| token.delimiter == delimiter)
      .copied()
      .collect::<Vec<_>>();
    for pair in matching.chunks_exact(2) {
      output.push(Interval {
        open: pair[0].start,
        close: pair[1].start,
        delimiter,
        mark: pair[0].mark,
      });
    }
  }
  output
}

fn crosses(left: Interval, right: Interval) -> bool {
  (left.open < right.open && right.open < left.close && left.close < right.close)
    || (right.open < left.open && left.open < right.close && right.close < left.close)
}

fn utf16_len(value: &str) -> u32 {
  value.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn chooses_the_mark_crossing_the_most_other_intervals() {
    assert_eq!(
      detect("**al~~pha** beta *gam~~ma*"),
      Some(CrossingMarks {
        base: "**alpha** beta *gamma*".to_string(),
        marks: vec![CrossingMark {
          mark: InlineMarkKind::Strike,
          start_utf16: 4,
          end_utf16: 19,
        }],
      })
    );
  }

  #[test]
  fn prefers_the_earlier_opening_interval_when_cross_counts_match() {
    assert_eq!(
      detect("al*pha **be*ta** gamma"),
      Some(CrossingMarks {
        base: "alpha **beta** gamma".to_string(),
        marks: vec![CrossingMark {
          mark: InlineMarkKind::Emphasis,
          start_utf16: 2,
          end_utf16: 10,
        }],
      })
    );
  }

  #[test]
  fn extracts_every_interval_needed_to_remove_all_crossings() {
    assert_eq!(
      detect("al*p~~ha **be*t~~a** gamma"),
      Some(CrossingMarks {
        base: "alpha **beta** gamma".to_string(),
        marks: vec![
          CrossingMark {
            mark: InlineMarkKind::Emphasis,
            start_utf16: 2,
            end_utf16: 10,
          },
          CrossingMark {
            mark: InlineMarkKind::Strike,
            start_utf16: 3,
            end_utf16: 11,
          },
        ],
      })
    );
  }

  #[test]
  fn ignores_nested_and_code_delimiters() {
    assert_eq!(detect("**bold *soft***"), None);
    assert_eq!(detect("`**not a mark**`"), None);
  }
}
