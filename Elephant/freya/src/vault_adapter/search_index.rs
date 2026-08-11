use std::{fs, path::Path, time::UNIX_EPOCH};

use super::{production_fts, AdapterError, AdapterResult};

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

/// Rebuilds the on-disk production FTS database using only APIs committed in
/// `Elephant/backend/tauri/src/fts.rs` at the Freya branch baseline.
///
/// Scanning, excerpt generation, SQLite FTS5 storage and BM25 ranking remain
/// owned by that production module. This adapter only supplies the production
/// title and wraps clear/upsert in one SQLite transaction.
pub(super) fn rebuild(
    vault_id: &str,
    root: &Path,
    mut title_for: impl FnMut(&str) -> AdapterResult<String>,
) -> AdapterResult<IndexRefreshStatus> {
    let scanned = production_fts::scan_markdown_files(root);
    let mut documents = Vec::with_capacity(scanned.len());
    for (relative_path, full_path, body) in scanned {
        let metadata = fs::metadata(&full_path).map_err(|error| {
            AdapterError::new(format!(
                "Unable to inspect search document {relative_path}: {error}"
            ))
        })?;
        let mtime = metadata
            .modified()
            .map_err(|error| {
                AdapterError::new(format!(
                    "Unable to read search document timestamp {relative_path}: {error}"
                ))
            })?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .min(i64::MAX as u128) as i64;
        documents.push(IndexDocument {
            title: title_for(&relative_path)?,
            relative_path,
            full_path: full_path.to_string_lossy().to_string(),
            body,
            mtime,
        });
    }

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
        status: "complete".to_string(),
        scanned: indexed,
        indexed,
        unchanged: 0,
        removed: previous_count.saturating_sub(indexed),
        failed: Vec::new(),
    })
}

fn sqlite_error(operation: &str, error: rusqlite::Error) -> AdapterError {
    AdapterError::new(format!("Unable to {operation}: {error}"))
}
