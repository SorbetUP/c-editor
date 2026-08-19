//! Freya search lifecycle backed by Elephant's production SQLite FTS index.
//!
//! Querying never rescans Markdown files. A full scan is performed only when
//! explicitly rebuilding, when an older vault has no index yet, or when a
//! directory-level filesystem event makes an exact per-note delta impossible.
//! Normal note create/modify/remove/rename events are applied incrementally.

use crate::vault_adapter::{production_fts, VaultAdapter};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchState {
    version: u32,
    enabled: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BackendStatus {
    pub enabled: bool,
    pub indexed_documents: usize,
    pub index_path: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IncrementalRefresh {
    pub upserted: usize,
    pub removed: usize,
    pub rebuilt: bool,
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
    let state = SearchState {
        version: 1,
        enabled,
    };
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

/// Apply real filesystem events to the production FTS index.
///
/// Note files can be updated exactly. Directory create/remove/rename events are
/// intentionally collapsed into one production rebuild because the watcher may
/// only provide the directory path while an arbitrary number of indexed notes
/// changed beneath it.
pub(crate) fn refresh_paths(
    vault: &VaultAdapter,
    changed_paths: &[PathBuf],
) -> Result<IncrementalRefresh> {
    if changed_paths.is_empty() || !read_state(vault.root())?.enabled {
        return Ok(IncrementalRefresh::default());
    }

    let root = fs::canonicalize(vault.root())
        .map_err(|error| format!("Unable to resolve active vault for search refresh: {error}"))?;
    let mut unique = BTreeSet::new();
    for path in changed_paths {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        if let Ok(relative) = path.strip_prefix(&root) {
            if !is_hidden_relative_path(relative) {
                unique.insert(path);
            }
        }
    }
    if unique.is_empty() {
        return Ok(IncrementalRefresh::default());
    }

    let index = production_fts::FtsIndex::open(&root)
        .map_err(|error| format!("Unable to open search index: {error}"))?;
    let mut result = IncrementalRefresh::default();
    let mut needs_rebuild = false;

    for path in unique {
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| format!("Search refresh path escaped the vault: {}", path.display()))?;
        let relative_path = relative.to_string_lossy().replace('\\', "/");
        if relative_path.is_empty() {
            needs_rebuild = true;
            continue;
        }

        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                // A symlink can hide an arbitrary subtree. The production scan
                // deliberately ignores it, so rebuild to remove any stale rows
                // that may have existed before the replacement.
                needs_rebuild = true;
            }
            Ok(metadata) if metadata.is_dir() => {
                needs_rebuild = true;
            }
            Ok(metadata) if metadata.is_file() && is_markdown_path(&path) => {
                let canonical = fs::canonicalize(&path)
                    .map_err(|error| format!("Resolve changed note {relative_path}: {error}"))?;
                if !canonical.starts_with(&root) {
                    return Err(format!("Changed note escaped the active vault: {relative_path}"));
                }
                let markdown = fs::read_to_string(&canonical)
                    .map_err(|error| format!("Read changed note {relative_path}: {error}"))?;
                let title = vault
                    .find_entry(&relative_path)
                    .map(|entry| entry.title)
                    .unwrap_or_else(|_| title_from_markdown(&relative_path, &markdown));
                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
                    .unwrap_or_default();
                index
                    .upsert_note(
                        &vault.descriptor().id,
                        &relative_path,
                        &canonical.to_string_lossy(),
                        &title,
                        &markdown,
                        mtime,
                    )
                    .map_err(|error| format!("Update search index for {relative_path}: {error}"))?;
                result.upserted = result.upserted.saturating_add(1);
            }
            Ok(_) => {
                // Non-Markdown files have no FTS row.
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if is_markdown_path(&path) {
                    index
                        .remove_note(&vault.descriptor().id, &relative_path)
                        .map_err(|error| {
                            format!("Remove deleted note {relative_path} from search index: {error}")
                        })?;
                    result.removed = result.removed.saturating_add(1);
                } else {
                    // A removed/renamed directory no longer has metadata and
                    // can contain many stale note rows.
                    needs_rebuild = true;
                }
            }
            Err(error) => {
                return Err(format!(
                    "Inspect changed search path {}: {error}",
                    path.display()
                ));
            }
        }
    }

    if needs_rebuild {
        let refresh = index
            .rebuild_from_files(&vault.descriptor().id, &root)
            .map_err(|error| format!("Rebuild search index after directory change: {error}"))?;
        if !refresh.failed.is_empty() && refresh.indexed == 0 && refresh.unchanged == 0 {
            return Err(format!(
                "Search refresh rebuild failed for {} path(s)",
                refresh.failed.len()
            ));
        }
        result.rebuilt = true;
    }

    eprintln!(
        "[freya][search-index] action=filesystem-refresh changed={} upserted={} removed={} rebuilt={}",
        changed_paths.len(),
        result.upserted,
        result.removed,
        result.rebuilt
    );
    Ok(result)
}

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn is_hidden_relative_path(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(value) => value.to_string_lossy().starts_with('.'),
        _ => false,
    })
}

fn title_from_markdown(relative_path: &str, markdown: &str) -> String {
    markdown
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            Path::new(relative_path)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled")
                .to_owned()
        })
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
    let parent = path
        .parent()
        .ok_or_else(|| "Search state has no parent".to_owned())?;
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
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
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

    #[test]
    fn filesystem_refresh_updates_and_removes_single_note_without_full_scan() {
        let (root, vault) = fixture();
        rebuild(&vault).unwrap();
        let alpha = root.join("Alpha.md");
        fs::write(&alpha, "# Alpha\nnew incremental phrase").unwrap();
        let refreshed = refresh_paths(&vault, std::slice::from_ref(&alpha)).unwrap();
        assert_eq!(refreshed.upserted, 1);
        assert!(!refreshed.rebuilt);
        assert_eq!(query(&vault, "incremental", 10).unwrap().len(), 1);

        fs::remove_file(&alpha).unwrap();
        let refreshed = refresh_paths(&vault, std::slice::from_ref(&alpha)).unwrap();
        assert_eq!(refreshed.removed, 1);
        assert!(!refreshed.rebuilt);
        assert!(query(&vault, "incremental", 10).unwrap().is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn directory_event_rebuilds_once_for_subtree_consistency() {
        let (root, vault) = fixture();
        rebuild(&vault).unwrap();
        let folder = root.join("Folder");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("Gamma.md"), "# Gamma\nsubtree phrase").unwrap();
        let refreshed = refresh_paths(&vault, std::slice::from_ref(&folder)).unwrap();
        assert!(refreshed.rebuilt);
        assert_eq!(query(&vault, "subtree", 10).unwrap().len(), 1);
        let _ = fs::remove_dir_all(root);
    }
}
