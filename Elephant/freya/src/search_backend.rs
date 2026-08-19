//! Freya search lifecycle backed by Elephant's production SQLite FTS index.
//!
//! Querying never rescans Markdown files. A full scan is performed only when
//! explicitly rebuilding or when an older vault has no index yet. Search
//! enable/disable is persisted beside the index so the native surface can
//! implement the same lifecycle commands as the Tauri search store.

use crate::vault_adapter::{production_fts, VaultAdapter};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchState {
    version: u32,
    enabled: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self { version: 1, enabled: true }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BackendStatus {
    pub enabled: bool,
    pub indexed_documents: usize,
    pub index_path: PathBuf,
}

pub(crate) fn status(vault: &VaultAdapter) -> Result<BackendStatus> {
    let state = read_state(vault.root())?;
    let index = production_fts::FtsIndex::open(vault.root())
        .map_err(|error| format!("Unable to open search index: {error}"))?;
    let count = index
        .count(&vault.descriptor().id)
        .map_err(|error| format!("Unable to inspect search index: {error}"))?;
    Ok(BackendStatus {
        enabled: state.enabled,
        indexed_documents: usize::try_from(count.max(0)).unwrap_or(usize::MAX),
        index_path: index_path(vault.root()),
    })
}

pub(crate) fn set_enabled(vault: &VaultAdapter, enabled: bool) -> Result<BackendStatus> {
    let state = SearchState { version: 1, enabled };
    write_state(vault.root(), &state)?;
    if enabled && status(vault)?.indexed_documents == 0 {
        rebuild(vault)?;
    }
    status(vault)
}

pub(crate) fn rebuild(vault: &VaultAdapter) -> Result<production_fts::IndexRefreshStatus> {
    let index = production_fts::FtsIndex::open(vault.root())
        .map_err(|error| format!("Unable to open search index: {error}"))?;
    index
        .rebuild_from_files(&vault.descriptor().id, vault.root())
        .map_err(|error| format!("Unable to rebuild search index: {error}"))
}

pub(crate) fn clear(vault: &VaultAdapter) -> Result<BackendStatus> {
    let index = production_fts::FtsIndex::open(vault.root())
        .map_err(|error| format!("Unable to open search index: {error}"))?;
    index
        .clear_vault(&vault.descriptor().id)
        .map_err(|error| format!("Unable to clear search index: {error}"))?;
    status(vault)
}

pub(crate) fn query(
    vault: &VaultAdapter,
    query: &str,
    limit: usize,
) -> Result<Vec<production_fts::Hit>> {
    let state = read_state(vault.root())?;
    if !state.enabled {
        return Err("Search is disabled for this vault".to_owned());
    }
    let mut current = status(vault)?;
    if current.indexed_documents == 0 {
        let refresh = rebuild(vault)?;
        if !refresh.failed.is_empty() && refresh.indexed == 0 {
            return Err(format!(
                "Search index rebuild failed for {} file(s)",
                refresh.failed.len()
            ));
        }
        current = status(vault)?;
    }
    eprintln!(
        "[freya][search-index] action=query indexed={} query_len={} limit={}",
        current.indexed_documents,
        query.chars().count(),
        limit
    );
    let index = production_fts::FtsIndex::open(vault.root())
        .map_err(|error| format!("Unable to open search index: {error}"))?;
    index
        .search(query, limit)
        .map_err(|error| format!("Unable to query search index: {error}"))
}

fn index_dir(root: &Path) -> PathBuf {
    root.join(".elephantnote").join("index")
}

fn index_path(root: &Path) -> PathBuf {
    index_dir(root).join("notes.sqlite")
}

fn state_path(root: &Path) -> PathBuf {
    index_dir(root).join("search-state.json")
}

fn read_state(root: &Path) -> Result<SearchState> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(SearchState::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Unable to read search state {}: {error}", path.display()))?;
    let state: SearchState = serde_json::from_str(&raw)
        .map_err(|error| format!("Unable to parse search state {}: {error}", path.display()))?;
    if state.version != 1 {
        return Err(format!("Unsupported search state version {}", state.version));
    }
    Ok(state)
}

fn write_state(root: &Path, state: &SearchState) -> Result<()> {
    let path = state_path(root);
    let parent = path.parent().ok_or_else(|| "Search state has no parent".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, &path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> (PathBuf, VaultAdapter) {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-search-backend-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Alpha.md"), "# Alpha\nneedle elephant search").unwrap();
        fs::write(root.join("Beta.md"), "# Beta\nother content").unwrap();
        let vault = VaultAdapter::open(&root).unwrap();
        (root, vault)
    }

    #[test]
    fn query_builds_index_once_and_uses_fts() {
        let (root, vault) = fixture();
        assert_eq!(status(&vault).unwrap().indexed_documents, 0);
        let hits = query(&vault, "needle", 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "Alpha.md");
        assert_eq!(status(&vault).unwrap().indexed_documents, 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn disable_is_persistent_and_deny_by_default_for_queries() {
        let (root, vault) = fixture();
        set_enabled(&vault, false).unwrap();
        assert!(!status(&vault).unwrap().enabled);
        assert!(query(&vault, "needle", 20).is_err());
        set_enabled(&vault, true).unwrap();
        assert!(status(&vault).unwrap().enabled);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clear_removes_only_indexed_documents() {
        let (root, vault) = fixture();
        rebuild(&vault).unwrap();
        assert_eq!(status(&vault).unwrap().indexed_documents, 2);
        clear(&vault).unwrap();
        assert_eq!(status(&vault).unwrap().indexed_documents, 0);
        assert!(root.join("Alpha.md").is_file());
        let _ = fs::remove_dir_all(root);
    }
}