pub fn join_lines(lines: &[String]) -> String {
  lines.join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn preserves_soft_line_boundaries() {
    assert_eq!(join_lines(&["a".into(), "b".into()]), "a\nb");
  }
}
