use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::vault_runtime as runtime;

#[derive(Clone, Debug, PartialEq)]
pub struct VaultEntry {
    pub relative_path: String,
    pub title: String,
    pub is_folder: bool,
    pub preview: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VaultSnapshot {
    pub root: PathBuf,
    pub entries: Vec<VaultEntry>,
}

#[derive(Clone, Debug)]
pub struct VaultService {
    descriptor: runtime::types::VaultDescriptor,
}

impl VaultService {
    pub fn open(root: &Path) -> Result<Self, String> {
        let root = fs::canonicalize(root)
            .map_err(|error| format!("Unable to open vault {}: {error}", root.display()))?;
        if !root.is_dir() {
            return Err(format!("Vault path is not a directory: {}", root.display()));
        }

        let name = runtime::config::basename(&root);
        Ok(Self {
            descriptor: runtime::types::VaultDescriptor {
                id: runtime::types::slug_id(&name),
                name,
                path: root.to_string_lossy().replace('\\', "/"),
                icon: String::new(),
                last_opened_at: runtime::config::now_string(),
                enabled: true,
            },
        })
    }

    pub fn root(&self) -> &Path {
        Path::new(&self.descriptor.path)
    }

    pub fn snapshot(&self) -> Result<VaultSnapshot, String> {
        eprintln!(
            "[freya][vault] action:start name=list-visible-entries root={}",
            self.descriptor.name
        );
        let entries = self.list("")?;
        eprintln!(
            "[freya][vault] action:done name=list-visible-entries entries={}",
            entries.len()
        );
        Ok(VaultSnapshot {
            root: self.root().to_path_buf(),
            entries,
        })
    }

    pub fn list(&self, relative_path: &str) -> Result<Vec<VaultEntry>, String> {
        let relative_path = self.validate_visible_path(relative_path, true)?;
        Ok(
            runtime::entries::list_directory_page(&self.descriptor, &relative_path, 0, None, true)?
                .into_iter()
                .filter_map(entry_from_value)
                .collect(),
        )
    }

    pub fn create_note(
        &self,
        relative_path: Option<String>,
        filename: Option<String>,
        title: Option<String>,
    ) -> Result<VaultEntry, String> {
        let relative_path = relative_path
            .map(|path| self.validate_visible_path(&path, true))
            .transpose()?;
        self.ensure_internal_layout_safe()?;
        let value =
            runtime::entries::create_note(&self.descriptor, relative_path, filename, title)?;
        entry_from_value(value)
            .ok_or_else(|| "Created note did not return a visible entry.".to_string())
    }

    pub fn create_folder(&self, relative_path: Option<String>) -> Result<VaultEntry, String> {
        let relative_path = relative_path
            .map(|path| self.validate_visible_path(&path, false))
            .transpose()?;
        self.ensure_internal_layout_safe()?;
        let value = runtime::entries::create_folder(&self.descriptor, relative_path)?;
        entry_from_value(value)
            .ok_or_else(|| "Created folder did not return a visible entry.".to_string())
    }

    pub fn rename(&self, relative_path: String, title: String) -> Result<(), String> {
        let relative_path = self.validate_visible_path(&relative_path, false)?;
        self.ensure_internal_layout_safe()?;
        runtime::entries::rename_entry(&self.descriptor, relative_path, title)
    }

    pub fn move_entry(
        &self,
        relative_path: String,
        target_directory_path: Option<String>,
    ) -> Result<(), String> {
        let relative_path = self.validate_visible_path(&relative_path, false)?;
        let target_directory_path = target_directory_path
            .map(|path| self.validate_visible_path(&path, true))
            .transpose()?;
        self.ensure_internal_layout_safe()?;
        runtime::entries::move_entry(&self.descriptor, relative_path, target_directory_path)
    }

    pub fn delete(&self, relative_path: String) -> Result<Value, String> {
        let relative_path = self.validate_visible_path(&relative_path, false)?;
        self.ensure_internal_layout_safe()?;
        runtime::entries::delete_entry(&self.descriptor, relative_path)
    }

    fn validate_visible_path(&self, path: &str, allow_empty: bool) -> Result<String, String> {
        let normalized = runtime::entries::validate_relative_path(path)?;
        if !allow_empty && normalized.is_empty() {
            return Err("A visible vault entry path is required.".to_string());
        }
        if normalized
            .split('/')
            .any(runtime::entries::is_ignored_entry)
        {
            return Err(format!(
                "Refusing hidden or internal vault path: {normalized}"
            ));
        }
        self.ensure_no_symlink_components(&normalized)?;
        Ok(normalized)
    }

    fn ensure_no_symlink_components(&self, relative_path: &str) -> Result<(), String> {
        let mut current = self.root().to_path_buf();
        for component in relative_path.split('/').filter(|part| !part.is_empty()) {
            current.push(component);
            let metadata = match fs::symlink_metadata(&current) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(error.to_string()),
            };
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "Refusing to follow a symlinked vault path: {}",
                    current.display()
                ));
            }
        }
        Ok(())
    }

    fn ensure_internal_layout_safe(&self) -> Result<(), String> {
        for relative_path in [".elephantnote", ".elephantnote/trash"] {
            let path = self.root().join(relative_path);
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                if metadata.file_type().is_symlink() {
                    return Err(format!(
                        "Refusing to use a symlinked internal vault path: {}",
                        path.display()
                    ));
                }
            }
        }
        Ok(())
    }
}

fn entry_from_value(value: Value) -> Option<VaultEntry> {
    let relative_path = value.get("path")?.as_str()?.to_string();
    let title = value.get("title")?.as_str()?.to_string();
    let is_folder = value.get("isDirectory")?.as_bool()?;
    let entry_type = value.get("type").and_then(Value::as_str).unwrap_or("");
    if is_folder || entry_type == "note" {
        let preview = value
            .get("childrenPreview")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("title").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        Some(VaultEntry {
            relative_path,
            title,
            is_folder,
            preview,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("elephant-freya-{name}"));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale fixture");
        }
        fs::create_dir_all(&root).expect("create fixture");
        root
    }

    #[test]
    fn reuses_existing_vault_crud_and_trash_contract() {
        let root = temp_root("vault-crud");
        fs::create_dir_all(root.join(".assets")).expect("create hidden directory");
        fs::write(root.join(".assets/hidden.md"), "hidden").expect("write hidden note");
        let service = VaultService::open(&root).expect("open fixture vault");

        let folder = service
            .create_folder(Some("Projects".to_string()))
            .expect("create folder");
        assert!(folder.is_folder);
        let note = service
            .create_note(
                Some("Projects".to_string()),
                Some("Plan.md".to_string()),
                Some("Plan".to_string()),
            )
            .expect("create note");
        assert_eq!(note.relative_path, "Projects/Plan.md");
        assert_eq!(
            fs::read_to_string(root.join(&note.relative_path)).unwrap(),
            "# Plan\n"
        );

        service
            .rename("Projects/Plan.md".to_string(), "Renamed".to_string())
            .expect("rename note");
        service
            .move_entry("Projects/Renamed.md".to_string(), None)
            .expect("move note");
        assert!(root.join("Renamed.md").is_file());
        assert!(!root.join("Projects/Renamed.md").exists());

        let deleted = service
            .delete("Renamed.md".to_string())
            .expect("trash note");
        assert_eq!(deleted.get("deleted").and_then(Value::as_bool), Some(true));
        assert!(!root.join("Renamed.md").exists());
        assert!(root.join(".elephantnote/trash").is_dir());

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn rejects_unsafe_paths_before_touching_disk() {
        let root = temp_root("vault-path-safety");
        let service = VaultService::open(&root).expect("open fixture vault");
        let error = service
            .create_folder(Some("../outside".to_string()))
            .expect_err("parent traversal must be rejected");
        assert!(error.contains("Parent path components are not allowed"));
        assert!(!root.parent().unwrap().join("outside").exists());
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn rejects_hidden_paths_for_every_mutating_entry_operation() {
        let root = temp_root("vault-hidden-paths");
        fs::create_dir_all(root.join(".assets")).expect("create hidden directory");
        fs::write(root.join("Visible.md"), "visible").expect("write visible note");
        let service = VaultService::open(&root).expect("open fixture vault");

        for path in [".assets", ".git", ".tmp", "Nested/.config", "draft~"] {
            assert!(
                service.create_folder(Some(path.to_string())).is_err(),
                "{path}"
            );
            assert!(service.list(path).is_err(), "{path}");
            assert!(service.delete(path.to_string()).is_err(), "{path}");
            assert!(
                service
                    .move_entry("Visible.md".to_string(), Some(path.to_string()))
                    .is_err(),
                "{path}"
            );
        }
        assert!(root.join("Visible.md").is_file());
        assert!(!root.join(".assets/Visible.md").exists());
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_parent_before_filesystem_mutation() {
        use std::os::unix::fs::symlink;

        let root = temp_root("vault-symlink-paths");
        let outside = temp_root("vault-symlink-outside");
        symlink(&outside, root.join("linked")).expect("create symlink");
        let service = VaultService::open(&root).expect("open fixture vault");

        let error = service
            .create_note(
                Some("linked".to_string()),
                Some("Escape.md".to_string()),
                Some("Escape".to_string()),
            )
            .expect_err("symlinked parent must be rejected");
        assert!(error.contains("symlinked vault path"));
        assert!(!outside.join("Escape.md").exists());
        fs::remove_dir_all(root).expect("remove fixture");
        fs::remove_dir_all(outside).expect("remove outside fixture");
    }
}
