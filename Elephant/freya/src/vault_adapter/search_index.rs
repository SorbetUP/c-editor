use std::{collections::VecDeque, fs, time::UNIX_EPOCH};

use super::{
    production_fts, AdapterError, AdapterResult, PageRequest, VaultAdapter, MAX_PAGE_SIZE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexFailure {
    pub path: String,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexRefreshStatus {
    pub status: String,
    pub scanned: usize,
    pub indexed: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub failed: Vec<IndexFailure>,
}

struct IndexDocument {
    relative_path: String,
    full_path: String,
    title: String,
    body: String,
    mtime: i64,
}

/// Rebuilds the on-disk production FTS database using the stable API shared by
/// the committed backend and the in-flight backend worktree.
///
/// Vault discovery and title semantics come from the production entries
/// boundary. Excerpt generation, SQLite FTS5 storage and BM25 ranking remain
/// exclusively owned by `production_fts::FtsIndex`.
pub(super) fn rebuild(adapter: &VaultAdapter) -> AdapterResult<IndexRefreshStatus> {
    let (documents, failed, scanned) = collect_documents(adapter)?;
    let vault_id = &adapter.descriptor().id;
    let root = adapter.root();

    let index =
        production_fts::FtsIndex::open(root).map_err(|error| sqlite_error("open", error))?;
    let previous_count = index
        .count(vault_id)
        .map_err(|error| sqlite_error("count existing documents", error))?
        .max(0) as usize;

    index
        .conn
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|error| sqlite_error("begin rebuild transaction", error))?;
    let rebuild_result = (|| {
        index
            .clear_vault(vault_id)
            .map_err(|error| sqlite_error("clear prior vault index", error))?;
        for document in &documents {
            index
                .upsert_note(
                    vault_id,
                    &document.relative_path,
                    &document.full_path,
                    &document.title,
                    &document.body,
                    document.mtime,
                )
                .map_err(|error| {
                    sqlite_error(&format!("index document {}", document.relative_path), error)
                })?;
        }
        Ok(())
    })();

    if let Err(error) = rebuild_result {
        let rollback = index.conn.execute_batch("ROLLBACK");
        return match rollback {
            Ok(()) => Err(error),
            Err(rollback_error) => Err(AdapterError::new(format!(
                "{error}; rollback failed: {rollback_error}"
            ))),
        };
    }
    index
        .conn
        .execute_batch("COMMIT")
        .map_err(|error| sqlite_error("commit rebuilt index", error))?;

    let indexed = documents.len();
    Ok(IndexRefreshStatus {
        status: if failed.is_empty() {
            "complete".to_string()
        } else {
            "partial".to_string()
        },
        scanned,
        indexed,
        unchanged: 0,
        removed: previous_count.saturating_sub(indexed),
        failed,
    })
}

fn collect_documents(
    adapter: &VaultAdapter,
) -> AdapterResult<(Vec<IndexDocument>, Vec<IndexFailure>, usize)> {
    let mut directories = VecDeque::from([String::new()]);
    let mut documents = Vec::new();
    let mut failed = Vec::new();
    let mut scanned = 0;

    while let Some(directory) = directories.pop_front() {
        let mut offset = 0;
        loop {
            let page = adapter.list(
                PageRequest::new(directory.clone())
                    .with_window(offset, MAX_PAGE_SIZE)
                    .without_preview(),
            )?;
            for entry in page.entries {
                if entry.is_directory {
                    directories.push_back(entry.path);
                    continue;
                }
                if !entry.path.to_ascii_lowercase().ends_with(".md") {
                    continue;
                }
                scanned += 1;
                match document_from_entry(&entry) {
                    Ok(document) => documents.push(document),
                    Err(error) => failed.push(IndexFailure {
                        path: entry.path,
                        error: error.to_string(),
                    }),
                }
            }
            let Some(next_offset) = page.next_offset else {
                break;
            };
            offset = next_offset;
        }
    }

    documents.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    failed.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((documents, failed, scanned))
}

fn document_from_entry(entry: &super::VaultEntry) -> AdapterResult<IndexDocument> {
    let body = fs::read_to_string(&entry.full_path).map_err(|error| {
        AdapterError::new(format!(
            "Unable to read search document {}: {error}",
            entry.path
        ))
    })?;
    let metadata = fs::metadata(&entry.full_path).map_err(|error| {
        AdapterError::new(format!(
            "Unable to inspect search document {}: {error}",
            entry.path
        ))
    })?;
    let mtime = metadata
        .modified()
        .map_err(|error| {
            AdapterError::new(format!(
                "Unable to read search document timestamp {}: {error}",
                entry.path
            ))
        })?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .min(i64::MAX as u128) as i64;
    Ok(IndexDocument {
        relative_path: entry.path.clone(),
        full_path: entry.full_path.clone(),
        title: entry.title.clone(),
        body,
        mtime,
    })
}

fn sqlite_error(operation: &str, error: rusqlite::Error) -> AdapterError {
    AdapterError::new(format!("Unable to {operation}: {error}"))
}
