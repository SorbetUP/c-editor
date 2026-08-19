//! Thin Freya adapter over the production Tauri trash contract.
//!
//! Trash is filesystem business logic, not renderer logic. Freya therefore
//! delegates delete/list/restore/empty directly to `vault/entries.rs` and only
//! deserializes the production JSON payload into its typed surface contract.

use std::path::Path;

use super::{
    production_entries, types::VaultDescriptor, AdapterError, AdapterResult, DeleteResult,
    EmptyTrashResult, RestoreResult, TrashEntry,
};

pub(super) fn delete(
    descriptor: &VaultDescriptor,
    _root: &Path,
    original_path: &str,
) -> AdapterResult<DeleteResult> {
    let value = production_entries::delete_entry(descriptor, original_path.to_owned())
        .map_err(AdapterError::from)?;
    serde_json::from_value(value).map_err(AdapterError::from)
}

pub(super) fn list(_root: &Path, descriptor: &VaultDescriptor) -> AdapterResult<Vec<TrashEntry>> {
    let values = production_entries::list_trash(descriptor).map_err(AdapterError::from)?;
    values
        .into_iter()
        .map(|value| serde_json::from_value(value).map_err(AdapterError::from))
        .collect()
}

pub(super) fn restore(
    descriptor: &VaultDescriptor,
    _root: &Path,
    trash_path: &str,
) -> AdapterResult<RestoreResult> {
    let value = production_entries::restore_trash(descriptor, trash_path.to_owned())
        .map_err(AdapterError::from)?;
    serde_json::from_value(value).map_err(AdapterError::from)
}

pub(super) fn empty(_root: &Path, descriptor: &VaultDescriptor) -> AdapterResult<EmptyTrashResult> {
    let value = production_entries::empty_trash(descriptor).map_err(AdapterError::from)?;
    serde_json::from_value(value).map_err(AdapterError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault_adapter::VaultAdapter;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    fn fixture() -> (std::path::PathBuf, VaultAdapter) {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-trash-shared-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Note.md"), "# Note\n").unwrap();
        let vault = VaultAdapter::open(&root).unwrap();
        (root, vault)
    }

    #[test]
    fn typed_adapter_round_trips_the_exact_production_trash() {
        let (root, vault) = fixture();
        let deleted = delete(vault.descriptor(), &root, "Note.md").unwrap();
        assert!(deleted.deleted);
        assert_eq!(deleted.original_path, "Note.md");
        let items = list(&root, vault.descriptor()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].restore_token, deleted.restore_token);
        let restored = restore(vault.descriptor(), &root, &deleted.trash_path).unwrap();
        assert!(restored.restored);
        assert!(root.join("Note.md").is_file());
        let emptied = empty(&root, vault.descriptor()).unwrap();
        assert!(emptied.emptied);
        let _ = fs::remove_dir_all(root);
    }
}
