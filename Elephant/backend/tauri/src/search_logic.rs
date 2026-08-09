pub fn normalize_query(query: &str) -> String {
  query.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

pub fn score_text(query: &str, title: &str, body: &str) -> u32 {
  let query = normalize_query(query);
  if query.is_empty() {
    return 0;
  }
  let title = title.to_lowercase();
  let body = body.to_lowercase();
  let mut score = 0;
  if title.contains(&query) {
    score += 10;
  }
  if body.contains(&query) {
    score += 3;
  }
  score
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalizes_queries() {
    assert_eq!(normalize_query("  Hello   WORLD "), "hello world");
  }

  #[test]
  fn scores_title_above_body() {
    assert!(score_text("alpha", "alpha", "body") > score_text("alpha", "title", "alpha"));
  }

  #[test]
  fn returns_zero_for_miss() {
    assert_eq!(score_text("alpha", "beta", "gamma"), 0);
  }
}
