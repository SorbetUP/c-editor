#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeSpanMatch<'a> {
  pub consumed: usize,
  pub code: &'a str,
}

pub fn parse(source: &str) -> Option<CodeSpanMatch<'_>> {
  let ticks = source
    .chars()
    .take_while(|character| *character == '`')
    .count();
  if ticks == 0 {
    return None;
  }

  let delimiter = "`".repeat(ticks);
  let rest = source.get(ticks..)?;
  let closing = rest.find(&delimiter)?;
  let code = rest.get(..closing)?;
  Some(CodeSpanMatch {
    consumed: ticks + closing + ticks,
    code,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_matching_backtick_runs() {
    assert_eq!(
      parse("`code` tail"),
      Some(CodeSpanMatch {
        consumed: 6,
        code: "code",
      })
    );
    assert_eq!(
      parse("``a`b``"),
      Some(CodeSpanMatch {
        consumed: 7,
        code: "a`b"
      })
    );
    assert_eq!(parse("`open"), None);
  }
}
