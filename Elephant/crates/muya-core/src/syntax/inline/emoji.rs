use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct EmojiRecord {
  emoji: String,
  aliases: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmojiMatch {
  pub shortcode: String,
  pub value: String,
  pub consumed: usize,
  pub valid: bool,
}

pub fn parse(source: &str) -> Option<EmojiMatch> {
  let rest = source.strip_prefix(':')?;
  let close = rest.find(':')?;
  let shortcode = &rest[..close];
  if shortcode.is_empty()
    || shortcode
      .chars()
      .any(|character| character == ':' || character.is_whitespace())
  {
    return None;
  }

  let value = lookup(shortcode);
  Some(EmojiMatch {
    shortcode: shortcode.to_string(),
    value: value
      .map(str::to_owned)
      .unwrap_or_else(|| format!(":{shortcode}:")),
    consumed: close + 2,
    valid: value.is_some(),
  })
}

pub fn lookup(shortcode: &str) -> Option<&'static str> {
  catalog()
    .iter()
    .find(|record| record.aliases.iter().any(|alias| alias == shortcode))
    .map(|record| record.emoji.as_str())
}

fn catalog() -> &'static [EmojiRecord] {
  static CATALOG: OnceLock<Vec<EmojiRecord>> = OnceLock::new();
  CATALOG.get_or_init(|| {
    serde_json::from_str(include_str!("../../../data/muya-emojis.json"))
      .expect("vendored Muya emoji catalog must remain valid JSON")
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn uses_the_same_aliases_as_tauri_muyas_catalog() {
    let parsed = parse(":grinning: tail").expect("known Muya emoji");
    assert_eq!(parsed.shortcode, "grinning");
    assert_eq!(parsed.value, "😀");
    assert_eq!(parsed.consumed, ":grinning:".len());
    assert!(parsed.valid);
  }

  #[test]
  fn preserves_unknown_aliases_as_editable_emoji_tokens() {
    let parsed = parse(":definitely_not_a_real_emoji:").expect("Muya token syntax");
    assert_eq!(parsed.shortcode, "definitely_not_a_real_emoji");
    assert_eq!(parsed.value, ":definitely_not_a_real_emoji:");
    assert!(!parsed.valid);
  }

  #[test]
  fn rejects_spaces_and_empty_aliases() {
    assert!(parse("::").is_none());
    assert!(parse(":not valid:").is_none());
  }
}
