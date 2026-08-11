use crate::vault::VaultService;
pub use crate::vault::{VaultEntry, VaultSnapshot};
use std::path::Path;

pub fn load_from_environment() -> Result<VaultSnapshot, String> {
    let raw_root = std::env::var("ELEPHANT_FREYA_VAULT").map_err(|_| {
        "No vault selected. Set ELEPHANT_FREYA_VAULT to an existing vault.".to_string()
    })?;
    load_vault(Path::new(raw_root.trim()))
}

pub fn load_vault(root: &Path) -> Result<VaultSnapshot, String> {
    VaultService::open(root)?.snapshot()
}

pub fn create_note(
    root: &Path,
    relative_path: Option<String>,
    filename: Option<String>,
    title: Option<String>,
) -> Result<VaultEntry, String> {
    VaultService::open(root)?.create_note(relative_path, filename, title)
}

pub fn create_folder(root: &Path, relative_path: Option<String>) -> Result<VaultEntry, String> {
    VaultService::open(root)?.create_folder(relative_path)
}

pub fn rename_entry(root: &Path, relative_path: String, title: String) -> Result<(), String> {
    VaultService::open(root)?.rename(relative_path, title)
}

pub fn move_entry(
    root: &Path,
    relative_path: String,
    target_directory_path: Option<String>,
) -> Result<(), String> {
    VaultService::open(root)?.move_entry(relative_path, target_directory_path)
}

pub fn delete_entry(root: &Path, relative_path: String) -> Result<serde_json::Value, String> {
    VaultService::open(root)?.delete(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("elephant-freya-{name}"));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale fixture");
        }
        fs::create_dir_all(&root).expect("create fixture");
        root
    }

    #[test]
    fn lists_visible_markdown_and_hides_internal_paths() {
        let root = temp_root("visible-entries");
        fs::create_dir_all(root.join(".assets")).expect("create hidden directory");
        fs::create_dir_all(root.join("Projects")).expect("create folder");
        fs::write(root.join("Welcome.md"), "# Welcome").expect("write note");
        fs::write(root.join(".assets/hidden.md"), "secret").expect("write hidden note");

        let snapshot = load_vault(&root).expect("load fixture");
        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.title == "Projects"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.title == "Welcome"));
        assert!(!snapshot.entries.iter().any(|entry| entry.title == "hidden"));

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn reports_missing_vault_as_a_visible_loader_error() {
        let missing = PathBuf::from("/tmp/elephant-freya-vault-does-not-exist");
        let error = load_vault(&missing).expect_err("missing vault must not be treated as empty");
        assert!(error.contains("Unable to open vault"));
    }
}
