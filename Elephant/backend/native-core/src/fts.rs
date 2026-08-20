use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS notes (
  rowid INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  full_path TEXT NOT NULL,
  title TEXT NOT NULL,
  excerpt TEXT,
  mtime INTEGER NOT NULL DEFAULT 0,
  UNIQUE(vault_id, relative_path)
);
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
  title,
  body,
  content='notes',
  content_rowid='rowid'
);
CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
  INSERT INTO notes_fts(rowid, title, body) VALUES (new.rowid, new.title, new.excerpt);
END;
CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.excerpt);
END;
CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
  INSERT INTO notes_fts(notes_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.excerpt);
  INSERT INTO notes_fts(rowid, title, body) VALUES (new.rowid, new.title, new.excerpt);
END;
";

#[derive(Serialize, Debug, Clone)]
pub struct Hit {
    pub path: String,
    pub full_path: String,
    pub title: String,
    pub excerpt: String,
    pub score: f64,
    pub tags: Vec<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct IndexFailure {
    pub path: String,
    pub error: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct IndexRefreshStatus {
    pub status: String,
    pub scanned: usize,
    pub indexed: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub failed: Vec<IndexFailure>,
}

impl IndexRefreshStatus {
    fn complete(
        scanned: usize,
        indexed: usize,
        unchanged: usize,
        removed: usize,
        failed: Vec<IndexFailure>,
    ) -> Self {
        let status = if failed.is_empty() {
            "complete"
        } else {
            "partial"
        };
        IndexRefreshStatus {
            status: status.to_string(),
            scanned,
            indexed,
            unchanged,
            removed,
            failed,
        }
    }
}

#[derive(Debug)]
pub struct ScannedMarkdown {
    pub relative_path: String,
    pub full_path: PathBuf,
    pub body: String,
    pub mtime: i64,
}

#[derive(Debug)]
pub struct MarkdownScan {
    pub files: Vec<ScannedMarkdown>,
    pub failed: Vec<IndexFailure>,
}

#[derive(Debug)]
pub enum IndexError {
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::Io { path, error } => write!(formatter, "{}: {error}", path.display()),
            IndexError::Sqlite(error) => write!(formatter, "sqlite: {error}"),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<rusqlite::Error> for IndexError {
    fn from(error: rusqlite::Error) -> Self {
        IndexError::Sqlite(error)
    }
}

#[derive(Debug)]
struct IndexedNote {
    full_path: String,
    title: String,
    excerpt: String,
    mtime: i64,
}

pub struct FtsIndex {
    pub conn: Connection,
}

impl FtsIndex {
    pub fn open(vault_root: &Path) -> rusqlite::Result<Self> {
        let db_dir = vault_root.join(".elephantnote").join("index");
        fs::create_dir_all(&db_dir)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let db_path = db_dir.join("notes.sqlite");
        let conn = Connection::open(db_path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(FtsIndex { conn })
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(FtsIndex { conn })
    }

    pub fn upsert_note(
        &self,
        vault_id: &str,
        relative_path: &str,
        full_path: &str,
        title: &str,
        body: &str,
        mtime: i64,
    ) -> rusqlite::Result<()> {
        let excerpt = excerpt_of(body);
        self.conn.execute(
      "INSERT INTO notes (vault_id, relative_path, full_path, title, excerpt, mtime) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
       ON CONFLICT(vault_id, relative_path) DO UPDATE SET full_path=excluded.full_path, title=excluded.title, excerpt=excluded.excerpt, mtime=excluded.mtime",
      params![vault_id, relative_path, full_path, title, excerpt, mtime],
    )?;
        Ok(())
    }

    pub fn remove_note(&self, vault_id: &str, relative_path: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM notes WHERE vault_id=?1 AND relative_path=?2",
            params![vault_id, relative_path],
        )?;
        Ok(())
    }

    pub fn clear_vault(&self, vault_id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM notes WHERE vault_id=?1", params![vault_id])?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<Hit>> {
        let fts_query = to_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self.conn.prepare(
            "SELECT n.relative_path, n.full_path, n.title, n.excerpt, bm25(notes_fts) as score
       FROM notes_fts
       JOIN notes n ON n.rowid = notes_fts.rowid
       WHERE notes_fts MATCH ?1
       ORDER BY score
       LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
            Ok(Hit {
                path: row.get::<_, String>(0)?,
                full_path: row.get::<_, String>(1)?,
                title: row.get::<_, String>(2)?,
                excerpt: row.get::<_, String>(3)?,
                score: row.get::<_, f64>(4)?,
                tags: Vec::new(),
            })
        })?;
        let mut hits = Vec::new();
        for hit in rows {
            hits.push(hit?);
        }
        Ok(hits)
    }

    pub fn count(&self, vault_id: &str) -> rusqlite::Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE vault_id=?1",
            params![vault_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn rebuild_from_files(
        &self,
        vault_id: &str,
        root: &Path,
    ) -> Result<IndexRefreshStatus, IndexError> {
        let scan = scan_markdown_files(root)?;
        let existing = self.indexed_notes(vault_id)?;
        let mut seen = HashMap::new();
        let mut indexed = 0;
        let mut unchanged = 0;

        for note in &scan.files {
            let title = title_of(&note.relative_path, &note.body);
            let excerpt = excerpt_of(&note.body);
            let full_path = note.full_path.to_string_lossy().to_string();
            seen.insert(
                note.relative_path.clone(),
                (
                    full_path.clone(),
                    title.clone(),
                    excerpt.clone(),
                    note.mtime,
                ),
            );

            if existing.get(&note.relative_path).is_some_and(|current| {
                current.full_path == full_path
                    && current.title == title
                    && current.excerpt == excerpt
                    && current.mtime == note.mtime
            }) {
                unchanged += 1;
            } else {
                self.upsert_note(
                    vault_id,
                    &note.relative_path,
                    &full_path,
                    &title,
                    &note.body,
                    note.mtime,
                )?;
                indexed += 1;
            }
        }

        let mut removed = 0;
        for relative_path in existing.keys() {
            if seen.contains_key(relative_path)
                || scan
                    .failed
                    .iter()
                    .any(|failure| protects_path(&failure.path, relative_path))
            {
                continue;
            }
            self.remove_note(vault_id, relative_path)?;
            removed += 1;
        }

        Ok(IndexRefreshStatus::complete(
            scan.files.len(),
            indexed,
            unchanged,
            removed,
            scan.failed,
        ))
    }

    fn indexed_notes(&self, vault_id: &str) -> rusqlite::Result<HashMap<String, IndexedNote>> {
        let mut statement = self.conn.prepare(
            "SELECT relative_path, full_path, title, excerpt, mtime FROM notes WHERE vault_id=?1",
        )?;
        let rows = statement.query_map(params![vault_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                IndexedNote {
                    full_path: row.get(1)?,
                    title: row.get(2)?,
                    excerpt: row.get(3)?,
                    mtime: row.get(4)?,
                },
            ))
        })?;
        rows.collect()
    }
}

fn excerpt_of(body: &str) -> String {
    let trimmed: Vec<&str> = body
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(3)
        .collect();
    let joined = trimmed.join(" ");
    let chars: Vec<char> = joined.chars().collect();
    if chars.len() <= 200 {
        joined
    } else {
        chars[..200].iter().collect()
    }
}

fn to_fts_query(query: &str) -> String {
    let normalized: String = query.trim().to_lowercase();
    if normalized.is_empty() {
        return String::new();
    }
    normalized
        .split_whitespace()
        .map(|token| {
            if token.contains('"') || token.contains('*') {
                token.to_string()
            } else {
                format!("\"{}\"*", token.replace('"', ""))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn scan_markdown_files(root: &Path) -> Result<MarkdownScan, IndexError> {
    let root_metadata = fs::symlink_metadata(root).map_err(|error| IndexError::Io {
        path: root.to_path_buf(),
        error,
    })?;
    if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
        return Err(IndexError::Io {
            path: root.to_path_buf(),
            error: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "vault root must be a real directory",
            ),
        });
    }

    let mut scan = MarkdownScan {
        files: Vec::new(),
        failed: Vec::new(),
    };
    scan_recursive(root, root, &mut scan);
    scan.files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    scan.failed
        .sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.error.cmp(&b.error)));
    Ok(scan)
}

fn scan_recursive(root: &Path, current: &Path, scan: &mut MarkdownScan) {
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(error) => {
            scan.failed.push(IndexFailure {
                path: relative_path(root, current),
                error: error.to_string(),
            });
            return;
        }
    };
    let mut entries = entries.collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        a.as_ref()
            .ok()
            .map(|e| e.file_name())
            .cmp(&b.as_ref().ok().map(|e| e.file_name()))
    });
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                scan.failed.push(IndexFailure {
                    path: relative_path(root, current),
                    error: error.to_string(),
                });
                continue;
            }
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(error) => {
                scan.failed.push(IndexFailure {
                    path: relative_path(root, &path),
                    error: error.to_string(),
                });
                continue;
            }
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            scan_recursive(root, &path, scan);
        } else if meta.is_file() && name.to_ascii_lowercase().ends_with(".md") {
            let relative = relative_path(root, &path);
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(error) => {
                    scan.failed.push(IndexFailure {
                        path: relative,
                        error: error.to_string(),
                    });
                    continue;
                }
            };
            let mtime = match meta.modified() {
                Ok(modified) => modified
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
                    .min(i64::MAX as u128) as i64,
                Err(error) => {
                    scan.failed.push(IndexFailure {
                        path: relative,
                        error: error.to_string(),
                    });
                    continue;
                }
            };
            scan.files.push(ScannedMarkdown {
                relative_path: relative,
                full_path: path,
                body: content,
                mtime,
            });
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn protects_path(failure_path: &str, candidate: &str) -> bool {
    failure_path.is_empty()
        || failure_path == candidate
        || candidate.starts_with(&format!("{failure_path}/"))
}

fn title_of(relative_path: &str, body: &str) -> String {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.trim_start_matches('#').trim().to_string())
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| {
            Path::new(relative_path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(relative_path)
                .to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_vault() -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("elephantnote_fts_test_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn to_fts_query_quotes_tokens() {
        assert_eq!(to_fts_query("hello world"), "\"hello\"* \"world\"*");
        assert_eq!(to_fts_query(""), "");
    }

    #[test]
    fn upsert_and_search_basic() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note(
                "v",
                "alpha.md",
                "/v/alpha.md",
                "Alpha Note",
                "first document",
                1,
            )
            .unwrap();
        index
            .upsert_note(
                "v",
                "beta.md",
                "/v/beta.md",
                "Beta Note",
                "second document",
                1,
            )
            .unwrap();
        let hits = index.search("alpha", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Alpha Note");
        assert_eq!(hits[0].path, "alpha.md");
    }

    #[test]
    fn search_ranks_relevance_by_bm25() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note("v", "a.md", "/v/a.md", "Alpha Beta", "alpha alpha alpha", 1)
            .unwrap();
        index
            .upsert_note("v", "b.md", "/v/b.md", "Beta Only", "beta once", 1)
            .unwrap();
        let hits = index.search("alpha", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "a.md");
    }

    #[test]
    fn upsert_is_idempotent() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note("v", "n.md", "/v/n.md", "Title", "body", 1)
            .unwrap();
        index
            .upsert_note("v", "n.md", "/v/n.md", "Updated", "new body", 2)
            .unwrap();
        let hits = index.search("updated", 10).unwrap();
        assert_eq!(hits.len(), 1);
        let hits = index.search("body", 10).unwrap();
        assert_eq!(hits.len(), 1);
        let hits = index.search("body", 10).unwrap();
        assert_eq!(hits[0].excerpt, "new body");
    }

    #[test]
    fn remove_note_clears_from_index() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note("v", "gone.md", "/v/gone.md", "Gone", "disappearing", 1)
            .unwrap();
        index.remove_note("v", "gone.md").unwrap();
        let hits = index.search("disappearing", 10).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn clear_vault_drops_only_that_vault() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note("v1", "a.md", "/v1/a.md", "VA", "alpha", 1)
            .unwrap();
        index
            .upsert_note("v2", "b.md", "/v2/b.md", "VB", "alpha", 1)
            .unwrap();
        index.clear_vault("v1").unwrap();
        assert_eq!(index.count("v1").unwrap(), 0);
        assert_eq!(index.count("v2").unwrap(), 1);
    }

    #[test]
    fn empty_query_returns_no_hits() {
        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note("v", "n.md", "/v/n.md", "T", "body", 1)
            .unwrap();
        assert!(index.search("", 10).unwrap().is_empty());
    }

    #[test]
    fn scan_markdown_files_walks_recursive() {
        let dir = make_vault();
        fs::write(dir.join("a.md"), "# A").unwrap();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/b.md"), "# B").unwrap();
        fs::write(dir.join(".hidden.md"), "# Hidden").unwrap();
        let notes = scan_markdown_files(&dir);
        let paths: Vec<_> = notes
            .unwrap()
            .files
            .iter()
            .map(|note| note.relative_path.clone())
            .collect();
        assert!(paths.contains(&"a.md".into()));
        assert!(paths.contains(&"sub/b.md".into()));
        assert!(!paths.contains(&".hidden.md".into()));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rebuild_indexes_existing_markdown_and_removes_disappeared_notes() {
        let dir = make_vault();
        fs::write(dir.join("keep.md"), "# Keep\nalpha body").unwrap();
        fs::create_dir_all(dir.join("nested").join(".private")).unwrap();
        fs::write(dir.join("nested").join("new.MD"), "# New\nbeta body").unwrap();
        fs::write(
            dir.join("nested").join(".private").join("hidden.md"),
            "# Hidden\ngamma body",
        )
        .unwrap();

        let index = FtsIndex::open_in_memory().unwrap();
        index
            .upsert_note(
                "vault",
                "gone.md",
                "/vault/gone.md",
                "Gone",
                "stale body",
                1,
            )
            .unwrap();
        let status = index.rebuild_from_files("vault", &dir).unwrap();

        assert_eq!(status.status, "complete");
        assert_eq!(status.scanned, 2);
        assert_eq!(status.indexed, 2);
        assert_eq!(status.unchanged, 0);
        assert_eq!(status.removed, 1);
        assert!(status.failed.is_empty());
        assert_eq!(index.count("vault").unwrap(), 2);
        assert!(index.search("stale", 10).unwrap().is_empty());
        assert_eq!(index.search("beta", 10).unwrap()[0].path, "nested/new.MD");
        assert!(index.search("gamma", 10).unwrap().is_empty());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rebuild_is_deterministic_and_reports_unchanged_files() {
        let dir = make_vault();
        fs::write(dir.join("z.md"), "# Zed\nzulu").unwrap();
        fs::write(dir.join("a.md"), "# Alpha\nalpha").unwrap();
        let index = FtsIndex::open_in_memory().unwrap();

        let first = index.rebuild_from_files("vault", &dir).unwrap();
        let second = index.rebuild_from_files("vault", &dir).unwrap();

        assert_eq!(first.status, "complete");
        assert_eq!(second.status, "complete");
        assert_eq!(first.scanned, 2);
        assert_eq!(second.scanned, 2);
        assert_eq!(second.indexed, 0);
        assert_eq!(second.unchanged, 2);
        assert_eq!(second.removed, 0);
        assert_eq!(second.failed, Vec::<IndexFailure>::new());
        let paths: Vec<_> = index
            .search("alpha", 10)
            .unwrap()
            .into_iter()
            .map(|hit| hit.path)
            .collect();
        assert_eq!(paths, vec!["a.md"]);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn open_vault_creates_db_under_elephantnote_index() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("elephantnote_fts_vault_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let fts = FtsIndex::open(&dir).unwrap();
        fts.upsert_note("v", "x.md", "/v/x.md", "X", "ex content", 1)
            .unwrap();
        let hits = fts.search("ex", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "x.md");
        let db_path = dir.join(".elephantnote").join("index").join("notes.sqlite");
        assert!(db_path.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn excerpt_truncates_long_bodies() {
        let long = "a ".repeat(500);
        let excerpt = excerpt_of(&long);
        assert!(excerpt.chars().count() <= 200);
    }

    #[test]
    fn search_limits_to_query_limit() {
        let index = FtsIndex::open_in_memory().unwrap();
        for i in 0..5 {
            index
                .upsert_note(
                    "v",
                    &format!("n{i}.md"),
                    &format!("/v/n{i}.md"),
                    "Common",
                    "alpha",
                    1,
                )
                .unwrap();
        }
        assert_eq!(index.search("alpha", 2).unwrap().len(), 2);
    }
}
