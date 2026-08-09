#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmphasisKind {
  Emphasis,
  Strong,
  Strike,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmphasisMatch<'a> {
  pub consumed: usize,
  pub content: &'a str,
  pub kind: EmphasisKind,
}

pub fn parse(source: &str) -> Option<EmphasisMatch<'_>> {
  if let Some(parsed) = parse_triple(source) {
    return Some(parsed);
  }

  for (delimiter, kind) in [
    ("**", EmphasisKind::Strong),
    ("__", EmphasisKind::Strong),
    ("~~", EmphasisKind::Strike),
    ("*", EmphasisKind::Emphasis),
    ("_", EmphasisKind::Emphasis),
  ] {
    let Some(rest) = source.strip_prefix(delimiter) else {
      continue;
    };
    let closing = rest.find(delimiter)?;
    if closing == 0 {
      return None;
    }
    return Some(EmphasisMatch {
      consumed: delimiter.len() + closing + delimiter.len(),
      content: &rest[..closing],
      kind,
    });
  }
  None
}

fn parse_triple(source: &str) -> Option<EmphasisMatch<'_>> {
  for delimiter in ["***", "___"] {
    let Some(rest) = source.strip_prefix(delimiter) else {
      continue;
    };
    let closing = rest.find(delimiter)?;
    if closing == 0 {
      return None;
    }
    let consumed = delimiter.len() + closing + delimiter.len();
    let inner_start = 2;
    let inner_end = consumed - 2;
    return Some(EmphasisMatch {
      consumed,
      content: &source[inner_start..inner_end],
      kind: EmphasisKind::Strong,
    });
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_basic_emphasis_forms() {
    assert_eq!(
      parse("**bold**"),
      Some(EmphasisMatch {
        consumed: 8,
        content: "bold",
        kind: EmphasisKind::Strong,
      })
    );
    assert_eq!(
      parse("*soft*"),
      Some(EmphasisMatch {
        consumed: 6,
        content: "soft",
        kind: EmphasisKind::Emphasis,
      })
    );
    assert_eq!(
      parse("~~gone~~"),
      Some(EmphasisMatch {
        consumed: 8,
        content: "gone",
        kind: EmphasisKind::Strike,
      })
    );
  }

  #[test]
  fn exposes_triple_delimiters_as_strong_with_emphasis_source_inside() {
    assert_eq!(
      parse("***alpha***"),
      Some(EmphasisMatch {
        consumed: 11,
        content: "*alpha*",
        kind: EmphasisKind::Strong,
      })
    );
    assert_eq!(
      parse("___alpha___"),
      Some(EmphasisMatch {
        consumed: 11,
        content: "_alpha_",
        kind: EmphasisKind::Strong,
      })
    );
  }

  #[test]
  fn stops_triple_parsing_at_the_first_complete_mark() {
    assert_eq!(
      parse("***one*** and ***two***"),
      Some(EmphasisMatch {
        consumed: 9,
        content: "*one*",
        kind: EmphasisKind::Strong,
      })
    );
  }
}
