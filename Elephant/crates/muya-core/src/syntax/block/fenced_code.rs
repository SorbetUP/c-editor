#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FenceOpen {
  pub marker: char,
  pub length: usize,
  pub info: Option<String>,
}

pub fn parse_opening(line: &str) -> Option<FenceOpen> {
  let trimmed = line.trim_start();
  let marker = trimmed.chars().next()?;
  if !matches!(marker, '`' | '~') {
    return None;
  }
  let length = trimmed
    .chars()
    .take_while(|character| *character == marker)
    .count();
  if length < 3 {
    return None;
  }
  let info = trimmed
    .get(length..)
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToOwned::to_owned);
  Some(FenceOpen {
    marker,
    length,
    info,
  })
}

pub fn is_closing(line: &str, opening: &FenceOpen) -> bool {
  let trimmed = line.trim();
  let count = trimmed
    .chars()
    .take_while(|character| *character == opening.marker)
    .count();
  count >= opening.length && trimmed.chars().skip(count).all(char::is_whitespace)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_and_closes_matching_fences() {
    let opening = parse_opening("```rust").unwrap();
    assert_eq!(opening.info.as_deref(), Some("rust"));
    assert!(is_closing("```", &opening));
    assert!(!is_closing("~~~", &opening));
  }
}
