const SPECIAL: &[char] = &['\\', '`', '!', '[', '*', '_', '~', '\n'];

pub fn take(source: &str) -> &str {
  let mut end = source.len();
  for (index, character) in source.char_indices() {
    if index > 0 && SPECIAL.contains(&character) {
      end = index;
      break;
    }
    if index == 0 && SPECIAL.contains(&character) {
      end = character.len_utf8();
      break;
    }
  }
  &source[..end]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn consumes_plain_text_until_the_next_inline_marker() {
    assert_eq!(take("hello **world**"), "hello ");
    assert_eq!(take("😀text"), "😀text");
    assert_eq!(take("*"), "*");
  }
}
