pub fn matches(line: &str) -> bool {
  let compact = line
    .chars()
    .filter(|character| !character.is_whitespace())
    .collect::<String>();
  let Some(marker) = compact.chars().next() else {
    return false;
  };
  compact.len() >= 3
    && matches!(marker, '-' | '*' | '_')
    && compact.chars().all(|character| character == marker)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn recognizes_supported_breaks() {
    assert!(matches("---"));
    assert!(matches("* * *"));
    assert!(!matches("--"));
    assert!(!matches("-*-"));
  }
}
