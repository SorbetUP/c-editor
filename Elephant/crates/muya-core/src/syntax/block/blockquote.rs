pub fn strip_marker(line: &str) -> Option<&str> {
  let rest = line.strip_prefix('>')?;
  Some(rest.strip_prefix(' ').unwrap_or(rest))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn strips_quote_markers() {
    assert_eq!(strip_marker("> quote"), Some("quote"));
    assert_eq!(strip_marker(">quote"), Some("quote"));
    assert_eq!(strip_marker("quote"), None);
  }
}
