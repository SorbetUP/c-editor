use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ChatKnowledgeAction {
    SearchNotes {
        query: String,
        #[serde(default = "default_search_limit")]
        limit: usize,
    },
    CreateWiki {
        title: String,
        topic: String,
        #[serde(default)]
        source_paths: Vec<String>,
    },
    CreateNote {
        relative_path: String,
        title: String,
        content: String,
    },
    AppendToNote {
        relative_path: String,
        content: String,
        expected_hash: String,
    },
    ReplaceNoteRange {
        relative_path: String,
        start_offset: usize,
        end_offset: usize,
        replacement: String,
        expected_hash: String,
    },
    ReplaceNote {
        relative_path: String,
        content: String,
        expected_hash: String,
    },
}

fn default_search_limit() -> usize {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionValidation {
    pub valid: bool,
    pub mutates_user_content: bool,
    pub requires_approval: bool,
    pub errors: Vec<String>,
}

impl ChatKnowledgeAction {
    pub fn mutates_user_content(&self) -> bool {
        matches!(
            self,
            Self::CreateNote { .. }
                | Self::AppendToNote { .. }
                | Self::ReplaceNoteRange { .. }
                | Self::ReplaceNote { .. }
        )
    }

    pub fn requires_approval(&self) -> bool {
        self.mutates_user_content() || matches!(self, Self::CreateWiki { .. })
    }

    pub fn validate(&self) -> ActionValidation {
        let mut errors = Vec::new();
        match self {
            Self::SearchNotes { query, limit } => {
                if query.trim().is_empty() {
                    errors.push("Search query cannot be empty.".into());
                }
                if *limit == 0 || *limit > 100 {
                    errors.push("Search limit must be between 1 and 100.".into());
                }
            }
            Self::CreateWiki {
                title,
                topic,
                source_paths,
            } => {
                if title.trim().is_empty() {
                    errors.push("Wiki title cannot be empty.".into());
                }
                if topic.trim().is_empty() {
                    errors.push("Wiki topic cannot be empty.".into());
                }
                for path in source_paths {
                    validate_note_path(path, &mut errors);
                }
            }
            Self::CreateNote {
                relative_path,
                title,
                ..
            } => {
                validate_note_path(relative_path, &mut errors);
                if title.trim().is_empty() {
                    errors.push("Note title cannot be empty.".into());
                }
            }
            Self::AppendToNote {
                relative_path,
                content,
                expected_hash,
            } => {
                validate_note_path(relative_path, &mut errors);
                validate_write_guard(content, expected_hash, &mut errors);
            }
            Self::ReplaceNoteRange {
                relative_path,
                start_offset,
                end_offset,
                replacement,
                expected_hash,
            } => {
                validate_note_path(relative_path, &mut errors);
                validate_write_guard(replacement, expected_hash, &mut errors);
                if start_offset >= end_offset {
                    errors.push("Replacement range must have start_offset < end_offset.".into());
                }
            }
            Self::ReplaceNote {
                relative_path,
                content,
                expected_hash,
            } => {
                validate_note_path(relative_path, &mut errors);
                validate_write_guard(content, expected_hash, &mut errors);
            }
        }

        ActionValidation {
            valid: errors.is_empty(),
            mutates_user_content: self.mutates_user_content(),
            requires_approval: self.requires_approval(),
            errors,
        }
    }
}

fn validate_write_guard(content: &str, expected_hash: &str, errors: &mut Vec<String>) {
    if content.is_empty() {
        errors.push("Write content cannot be empty.".into());
    }
    if expected_hash.trim().is_empty() {
        errors.push("A content hash is required to prevent overwriting newer edits.".into());
    }
}

fn validate_note_path(value: &str, errors: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        errors.push("A relative note path is required.".into());
        return;
    }

    let normalized = trimmed.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute() || has_windows_drive_prefix(&normalized) {
        errors.push(format!("Absolute paths are forbidden: {trimmed}"));
    }
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir | Component::RootDir))
    {
        errors.push(format!("Path traversal is forbidden: {trimmed}"));
    }
    if normalized.split('/').any(|part| part.starts_with('.')) {
        errors.push(format!(
            "Hidden paths cannot be modified by chat actions: {trimmed}"
        ));
    }
    if !normalized.to_ascii_lowercase().ends_with(".md") {
        errors.push(format!(
            "Knowledge actions only support Markdown notes: {trimmed}"
        ));
    }
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_read_only_and_needs_no_approval() {
        let action = ChatKnowledgeAction::SearchNotes {
            query: "iroh".into(),
            limit: 10,
        };
        let validation = action.validate();
        assert!(validation.valid);
        assert!(!validation.mutates_user_content);
        assert!(!validation.requires_approval);
    }

    #[test]
    fn writes_require_hash_and_approval() {
        let action = ChatKnowledgeAction::ReplaceNote {
            relative_path: "Notes/Iroh.md".into(),
            content: "# Iroh".into(),
            expected_hash: String::new(),
        };
        let validation = action.validate();
        assert!(!validation.valid);
        assert!(validation.requires_approval);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("content hash")));
    }

    #[test]
    fn hidden_and_traversal_paths_are_rejected() {
        for path in [
            "../outside.md",
            ".elephantnote/wiki/x.md",
            "/tmp/x.md",
            "C:/tmp/x.md",
        ] {
            let action = ChatKnowledgeAction::CreateNote {
                relative_path: path.into(),
                title: "X".into(),
                content: "# X".into(),
            };
            assert!(!action.validate().valid, "path should be rejected: {path}");
        }
    }

    #[test]
    fn wiki_generation_requires_approval_but_does_not_mutate_user_notes() {
        let action = ChatKnowledgeAction::CreateWiki {
            title: "Iroh".into(),
            topic: "Iroh networking".into(),
            source_paths: vec!["Notes/Iroh.md".into()],
        };
        let validation = action.validate();
        assert!(validation.valid);
        assert!(!validation.mutates_user_content);
        assert!(validation.requires_approval);
    }
}
