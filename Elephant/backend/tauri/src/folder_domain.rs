pub fn folder_name_from_title(title: &str) -> String {
  let cleaned = title.trim().replace('/', "-");
  if cleaned.is_empty() { "New Folder".to_string() } else { cleaned }
}

pub fn child_path(parent: &str, child: &str) -> String {
  let parent = parent.trim_matches('/');
  let child = child.trim_matches('/');
  if parent.is_empty() { child.to_string() } else { format!("{}/{}", parent, child) }
}

pub fn is_wiki_folder(path: &str) -> bool {
  path.split('/').any(|part| part == "wiki")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creates_folder_names() {
    assert_eq!(folder_name_from_title("Project"), "Project");
    assert_eq!(folder_name_from_title(""), "New Folder");
  }

  #[test]
  fn builds_child_paths() {
    assert_eq!(child_path("Notes", "A"), "Notes/A");
    assert_eq!(child_path("", "A"), "A");
  }

  #[test]
  fn detects_wiki_folder() {
    assert!(is_wiki_folder("wiki/Page.md"));
    assert!(!is_wiki_folder("notes/Page.md"));
  }
}
