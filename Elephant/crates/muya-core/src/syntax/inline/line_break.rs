#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakKind {
  Soft,
  Hard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakMatch {
  pub consumed: usize,
  pub kind: BreakKind,
}

pub fn parse(source: &str) -> Option<BreakMatch> {
  if source.starts_with("  \n") {
    return Some(BreakMatch {
      consumed: 3,
      kind: BreakKind::Hard,
    });
  }
  if source.starts_with("\\\n") {
    return Some(BreakMatch {
      consumed: 2,
      kind: BreakKind::Hard,
    });
  }
  source.starts_with('\n').then_some(BreakMatch {
    consumed: 1,
    kind: BreakKind::Soft,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn distinguishes_soft_and_hard_breaks() {
    assert_eq!(
      parse("  \nnext"),
      Some(BreakMatch {
        consumed: 3,
        kind: BreakKind::Hard,
      })
    );
    assert_eq!(
      parse("\nnext"),
      Some(BreakMatch {
        consumed: 1,
        kind: BreakKind::Soft,
      })
    );
  }
}
