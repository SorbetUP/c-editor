use std::path::{Path, PathBuf};

pub fn clean_relative_path(path: &str) -> String {
  path.replace('\\', "/")
    .split('/')
    .filter(|part| !part.is_empty() && *part != ".")
    .collect::<Vec<_>>()
    .join("/")
}

pub fn join_path(root: impl AsRef<Path>, relative_path: &str) -> PathBuf {
  let cleaned = clean_relative_path(relative_path);
  if cleaned.is_empty() {
    root.as_ref().to_path_buf()
  } else {
    root.as_ref().join(cleaned)
  }
}

pub fn with_markdown_extension(name: &str) -> String {
  let trimmed = name.trim();
  if trimmed.to_lowercase().ends_with(".md") {
    trimmed.to_string()
  } else {
    format!("{trimmed}.md")
  }
}

#[cfg(test)]
mod generated_tauri_parity_tests {
  include!(concat!(env!("OUT_DIR"), "/generated_tauri_parity_tests.rs"));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cleans_empty_segments() {
    assert_eq!(clean_relative_path("a//b/./c.md"), "a/b/c.md");
  }

  #[test]
  fn joins_root_and_relative_path() {
    let joined = join_path("vault", "Notes/a.md");
    assert!(joined.ends_with("Notes/a.md"));
  }

  #[test]
  fn adds_markdown_extension_only_when_needed() {
    assert_eq!(with_markdown_extension("Note"), "Note.md");
    assert_eq!(with_markdown_extension("Note.md"), "Note.md");
  }
}
