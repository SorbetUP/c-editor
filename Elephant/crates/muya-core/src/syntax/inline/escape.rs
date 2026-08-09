#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EscapeMatch {
  pub consumed: usize,
  pub value: char,
}

pub fn parse(source: &str) -> Option<EscapeMatch> {
  let mut chars = source.chars();
  if chars.next()? != '\\' {
    return None;
  }
  let value = chars.next()?;
  if !value.is_ascii_punctuation() {
    return None;
  }
  Some(EscapeMatch {
    consumed: 1 + value.len_utf8(),
    value,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_escaped_punctuation() {
    assert_eq!(
      parse("\\*x"),
      Some(EscapeMatch {
        consumed: 2,
        value: '*'
      })
    );
    assert_eq!(parse("\\a"), None);
  }
}
