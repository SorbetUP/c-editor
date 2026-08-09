pub fn is_picture_file(filename: &str) -> bool {
  let lower = filename.to_lowercase();
  lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".webp")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_picture_files() {
    assert!(is_picture_file("a.png"));
    assert!(!is_picture_file("a.txt"));
  }
}
