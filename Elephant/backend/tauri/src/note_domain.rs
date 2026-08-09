use crate::markdown_engine::{parse_markdown, render_note};
use crate::path_utils::with_markdown_extension;

pub fn note_filename_from_title(title: &str) -> String {
  let cleaned = title.trim().replace('/', "-");
  with_markdown_extension(if cleaned.is_empty() { "Untitled" } else { &cleaned })
}

pub fn create_note_markdown(title: &str, tags: &[String]) -> String {
  render_note(title, "note", tags, &format!("# {}", title.trim()))
}

pub fn rename_note_markdown(markdown: &str, new_title: &str, fallback: &str) -> String {
  let mut parsed = parse_markdown(markdown, fallback);
  parsed.title = new_title.trim().to_string();
  render_note(&parsed.title, &parsed.note_type, &parsed.tags, &parsed.body)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creates_note_filenames() {
    assert_eq!(note_filename_from_title("Hello World"), "Hello World.md");
    assert_eq!(note_filename_from_title(""), "Untitled.md");
  }

  #[test]
  fn creates_markdown_note() {
    let note = create_note_markdown("Alpha", &["tag".into()]);
    assert!(note.contains("title: \"Alpha\""));
    assert!(note.contains("# Alpha"));
  }

  #[test]
  fn renames_markdown_note() {
    let renamed = rename_note_markdown("---\ntitle: \"Old\"\ntags: []\n---\n\n# Old", "New", "old.md");
    assert!(renamed.contains("title: \"New\""));
  }
}
