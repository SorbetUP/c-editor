use crate::actions::ChatKnowledgeAction;
use crate::storage::KnowledgeStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatActionStatus {
    Proposed,
    Approved,
    Executed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatActionPreview {
    Search {
        query: String,
        limit: usize,
    },
    CreateWiki {
        title: String,
        topic: String,
        source_paths: Vec<String>,
    },
    CreateNote {
        relative_path: String,
        after_hash: String,
        excerpt: String,
    },
    ModifyNote {
        relative_path: String,
        before_hash: String,
        after_hash: String,
        before_excerpt: String,
        after_excerpt: String,
        changed_start: usize,
        changed_end: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatActionProposal {
    pub id: String,
    pub action: ChatKnowledgeAction,
    pub rationale: String,
    pub status: ChatActionStatus,
    pub preview: ChatActionPreview,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatActionExecution {
    pub proposal: ChatActionProposal,
    pub result: Value,
}

pub fn prepare_chat_action(
    vault_root: &Path,
    action: ChatKnowledgeAction,
    rationale: impl Into<String>,
) -> Result<ChatActionProposal, String> {
    let validation = action.validate();
    if !validation.valid {
        return Err(validation.errors.join(" "));
    }
    let root = canonical_vault_root(vault_root)?;
    let preview = preview_action(&root, &action)?;
    let now = unix_timestamp();
    let action_json = serde_json::to_vec(&action).map_err(|error| error.to_string())?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"chat-action");
    hasher.update(&[0]);
    hasher.update(&action_json);
    hasher.update(&[0]);
    hasher.update(now.to_string().as_bytes());
    let hex = hasher.finalize().to_hex().to_string();

    Ok(ChatActionProposal {
        id: format!("action-{}", &hex[..24]),
        action,
        rationale: rationale.into().trim().to_string(),
        status: ChatActionStatus::Proposed,
        preview,
        result: None,
        error: None,
        created_at: now,
        updated_at: now,
    })
}

pub fn execute_approved_chat_action(
    vault_root: &Path,
    store: &KnowledgeStore,
    proposal: &ChatActionProposal,
) -> Result<ChatActionExecution, String> {
    if !matches!(proposal.status, ChatActionStatus::Approved) {
        return Err("Chat action must be explicitly approved before execution.".into());
    }
    let root = canonical_vault_root(vault_root)?;
    let result = match &proposal.action {
        ChatKnowledgeAction::SearchNotes { query, limit } => {
            let hits = store.search(query, *limit)?;
            serde_json::to_value(hits).map_err(|error| error.to_string())?
        }
        ChatKnowledgeAction::CreateWiki { .. } => {
            return Err("Wiki actions require the cited model-backed Wiki generator.".into());
        }
        ChatKnowledgeAction::CreateNote {
            relative_path,
            title,
            content,
        } => {
            let target = safe_note_path(&root, relative_path, false)?;
            if target.exists() {
                return Err(format!("Note already exists: {relative_path}"));
            }
            let markdown = ensure_note_title(title, content);
            atomic_write(&root, &target, markdown.as_bytes())?;
            json!({
                "operation": "create_note",
                "relativePath": relative_path,
                "contentHash": hash_text(&markdown)
            })
        }
        ChatKnowledgeAction::AppendToNote {
            relative_path,
            content,
            expected_hash,
        } => {
            let target = safe_note_path(&root, relative_path, true)?;
            let before = read_guarded_note(&target, expected_hash)?;
            let mut after = before.clone();
            if !after.is_empty() && !after.ends_with('\n') {
                after.push('\n');
            }
            after.push_str(content);
            atomic_write(&root, &target, after.as_bytes())?;
            json!({
                "operation": "append_to_note",
                "relativePath": relative_path,
                "beforeHash": expected_hash,
                "contentHash": hash_text(&after)
            })
        }
        ChatKnowledgeAction::ReplaceNoteRange {
            relative_path,
            start_offset,
            end_offset,
            replacement,
            expected_hash,
        } => {
            let target = safe_note_path(&root, relative_path, true)?;
            let before = read_guarded_note(&target, expected_hash)?;
            validate_range(&before, *start_offset, *end_offset)?;
            let mut after = String::with_capacity(
                before.len() - (end_offset - start_offset) + replacement.len(),
            );
            after.push_str(&before[..*start_offset]);
            after.push_str(replacement);
            after.push_str(&before[*end_offset..]);
            atomic_write(&root, &target, after.as_bytes())?;
            json!({
                "operation": "replace_note_range",
                "relativePath": relative_path,
                "beforeHash": expected_hash,
                "contentHash": hash_text(&after),
                "startOffset": start_offset,
                "endOffset": end_offset
            })
        }
        ChatKnowledgeAction::ReplaceNote {
            relative_path,
            content,
            expected_hash,
        } => {
            let target = safe_note_path(&root, relative_path, true)?;
            let _before = read_guarded_note(&target, expected_hash)?;
            atomic_write(&root, &target, content.as_bytes())?;
            json!({
                "operation": "replace_note",
                "relativePath": relative_path,
                "beforeHash": expected_hash,
                "contentHash": hash_text(content)
            })
        }
    };

    let mut executed = proposal.clone();
    executed.status = ChatActionStatus::Executed;
    executed.result = Some(result.clone());
    executed.error = None;
    executed.updated_at = unix_timestamp();
    Ok(ChatActionExecution {
        proposal: executed,
        result,
    })
}

fn preview_action(root: &Path, action: &ChatKnowledgeAction) -> Result<ChatActionPreview, String> {
    match action {
        ChatKnowledgeAction::SearchNotes { query, limit } => Ok(ChatActionPreview::Search {
            query: query.clone(),
            limit: *limit,
        }),
        ChatKnowledgeAction::CreateWiki {
            title,
            topic,
            source_paths,
        } => Ok(ChatActionPreview::CreateWiki {
            title: title.clone(),
            topic: topic.clone(),
            source_paths: source_paths.clone(),
        }),
        ChatKnowledgeAction::CreateNote {
            relative_path,
            title,
            content,
        } => {
            let target = safe_note_path(root, relative_path, false)?;
            if target.exists() {
                return Err(format!("Note already exists: {relative_path}"));
            }
            let markdown = ensure_note_title(title, content);
            Ok(ChatActionPreview::CreateNote {
                relative_path: relative_path.clone(),
                after_hash: hash_text(&markdown),
                excerpt: excerpt(&markdown),
            })
        }
        ChatKnowledgeAction::AppendToNote {
            relative_path,
            content,
            expected_hash,
        } => {
            let target = safe_note_path(root, relative_path, true)?;
            let before = read_guarded_note(&target, expected_hash)?;
            let mut after = before.clone();
            if !after.is_empty() && !after.ends_with('\n') {
                after.push('\n');
            }
            let changed_start = after.len();
            after.push_str(content);
            Ok(modification_preview(
                relative_path,
                &before,
                &after,
                changed_start,
                after.len(),
            ))
        }
        ChatKnowledgeAction::ReplaceNoteRange {
            relative_path,
            start_offset,
            end_offset,
            replacement,
            expected_hash,
        } => {
            let target = safe_note_path(root, relative_path, true)?;
            let before = read_guarded_note(&target, expected_hash)?;
            validate_range(&before, *start_offset, *end_offset)?;
            let mut after = String::with_capacity(
                before.len() - (end_offset - start_offset) + replacement.len(),
            );
            after.push_str(&before[..*start_offset]);
            after.push_str(replacement);
            after.push_str(&before[*end_offset..]);
            Ok(modification_preview(
                relative_path,
                &before,
                &after,
                *start_offset,
                start_offset + replacement.len(),
            ))
        }
        ChatKnowledgeAction::ReplaceNote {
            relative_path,
            content,
            expected_hash,
        } => {
            let target = safe_note_path(root, relative_path, true)?;
            let before = read_guarded_note(&target, expected_hash)?;
            Ok(modification_preview(
                relative_path,
                &before,
                content,
                0,
                content.len(),
            ))
        }
    }
}

fn modification_preview(
    relative_path: &str,
    before: &str,
    after: &str,
    changed_start: usize,
    changed_end: usize,
) -> ChatActionPreview {
    ChatActionPreview::ModifyNote {
        relative_path: relative_path.to_string(),
        before_hash: hash_text(before),
        after_hash: hash_text(after),
        before_excerpt: excerpt(before),
        after_excerpt: excerpt(after),
        changed_start,
        changed_end,
    }
}

fn canonical_vault_root(vault_root: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(vault_root).map_err(|error| error.to_string())?;
    if !root.is_dir() {
        return Err("Vault root is not a directory.".into());
    }
    Ok(root)
}

fn safe_note_path(root: &Path, relative_path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let normalized = relative_path.replace('\\', "/");
    let relative = Path::new(&normalized);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("Unsafe note path: {relative_path}"));
    }
    if !normalized.to_ascii_lowercase().ends_with(".md") {
        return Err(format!(
            "Only Markdown notes can be modified: {relative_path}"
        ));
    }
    if normalized.split('/').any(|part| part.starts_with('.')) {
        return Err(format!("Hidden paths cannot be modified: {relative_path}"));
    }

    let target = root.join(relative);
    let mut ancestor = root.to_path_buf();
    let components = relative.components().collect::<Vec<_>>();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        if let Component::Normal(part) = component {
            ancestor.push(part);
            if ancestor.exists() {
                let metadata =
                    fs::symlink_metadata(&ancestor).map_err(|error| error.to_string())?;
                if metadata.file_type().is_symlink() {
                    return Err(format!(
                        "Refusing to traverse a symbolic link: {}",
                        ancestor.display()
                    ));
                }
                if !metadata.is_dir() {
                    return Err(format!(
                        "Path parent is not a directory: {}",
                        ancestor.display()
                    ));
                }
            }
        }
    }

    if must_exist && !target.is_file() {
        return Err(format!("Note does not exist: {relative_path}"));
    }
    if target.exists() {
        let metadata = fs::symlink_metadata(&target).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Refusing to modify a symbolic link: {relative_path}"
            ));
        }
        let canonical = fs::canonicalize(&target).map_err(|error| error.to_string())?;
        if !canonical.starts_with(root) {
            return Err(format!("Note escapes the vault: {relative_path}"));
        }
    }
    Ok(target)
}

fn read_guarded_note(path: &Path, expected_hash: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let actual_hash = hash_text(&content);
    if actual_hash != expected_hash {
        return Err(format!(
            "Note changed since the action was proposed. Expected {expected_hash}, found {actual_hash}."
        ));
    }
    Ok(content)
}

fn validate_range(content: &str, start: usize, end: usize) -> Result<(), String> {
    if start >= end || end > content.len() {
        return Err("Replacement range is outside the current note.".into());
    }
    if !content.is_char_boundary(start) || !content.is_char_boundary(end) {
        return Err("Replacement range must align with UTF-8 character boundaries.".into());
    }
    Ok(())
}

fn atomic_write(root: &Path, target: &Path, content: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "Note path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    if !canonical_parent.starts_with(root) {
        return Err("Refusing to write outside the vault.".into());
    }

    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("note.md");
    let temporary = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        unix_timestamp()
    ));
    fs::write(&temporary, content).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(())
}

fn ensure_note_title(title: &str, content: &str) -> String {
    let trimmed = content.trim_start();
    if trimmed.starts_with("# ") || title.trim().is_empty() {
        return content.to_string();
    }
    if content.is_empty() {
        format!("# {}\n", title.trim())
    } else {
        format!("# {}\n\n{}", title.trim(), content)
    }
}

fn hash_text(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

fn excerpt(content: &str) -> String {
    content.chars().take(500).collect()
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "elephant-chat-action-{name}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn stale_hash_prevents_preview_and_execution() {
        let root = temp_vault("stale");
        fs::write(root.join("A.md"), "# A\nCurrent").unwrap();
        let action = ChatKnowledgeAction::ReplaceNote {
            relative_path: "A.md".into(),
            content: "# A\nChanged".into(),
            expected_hash: "stale".into(),
        };
        assert!(prepare_chat_action(&root, action, "update").is_err());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn approved_range_edit_revalidates_hash_and_writes_atomically() {
        let root = temp_vault("range");
        let before = "# Note\nhello world";
        fs::write(root.join("Note.md"), before).unwrap();
        let action = ChatKnowledgeAction::ReplaceNoteRange {
            relative_path: "Note.md".into(),
            start_offset: 13,
            end_offset: 18,
            replacement: "Rust".into(),
            expected_hash: hash_text(before),
        };
        let mut proposal = prepare_chat_action(&root, action, "replace word").unwrap();
        proposal.status = ChatActionStatus::Approved;
        let store = KnowledgeStore::open(&root).unwrap();
        execute_approved_chat_action(&root, &store, &proposal).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("Note.md")).unwrap(),
            "# Note\nhello Rust"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn execution_fails_if_user_edits_after_approval() {
        let root = temp_vault("race");
        let before = "# Note\nOriginal";
        fs::write(root.join("Note.md"), before).unwrap();
        let action = ChatKnowledgeAction::ReplaceNote {
            relative_path: "Note.md".into(),
            content: "# Note\nModel edit".into(),
            expected_hash: hash_text(before),
        };
        let mut proposal = prepare_chat_action(&root, action, "edit").unwrap();
        proposal.status = ChatActionStatus::Approved;
        fs::write(root.join("Note.md"), "# Note\nUser edit").unwrap();
        let store = KnowledgeStore::open(&root).unwrap();
        assert!(execute_approved_chat_action(&root, &store, &proposal).is_err());
        assert_eq!(
            fs::read_to_string(root.join("Note.md")).unwrap(),
            "# Note\nUser edit"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn refuses_hidden_paths_and_symlink_traversal() {
        let root = temp_vault("paths");
        let hidden = ChatKnowledgeAction::CreateNote {
            relative_path: ".elephantnote/wiki/x.md".into(),
            title: "X".into(),
            content: "content".into(),
        };
        assert!(prepare_chat_action(&root, hidden, "hidden").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = temp_vault("outside");
            symlink(&outside, root.join("linked")).unwrap();
            let linked = ChatKnowledgeAction::CreateNote {
                relative_path: "linked/x.md".into(),
                title: "X".into(),
                content: "content".into(),
            };
            assert!(prepare_chat_action(&root, linked, "linked").is_err());
            fs::remove_dir_all(outside).ok();
        }
        fs::remove_dir_all(root).ok();
    }
}
