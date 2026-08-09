use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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

pub struct FtsIndex {
  pub conn: Connection,
}

impl FtsIndex {
  pub fn open(vault_root: &Path) -> rusqlite::Result<Self> {
    let db_dir = vault_root.join(".elephantnote").join("index");
    fs::create_dir_all(&db_dir).ok();
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

  pub fn upsert_note(&self, vault_id: &str, relative_path: &str, full_path: &str, title: &str, body: &str, mtime: i64) -> rusqlite::Result<()> {
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
    self.conn.execute("DELETE FROM notes WHERE vault_id=?1", params![vault_id])?;
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
}

fn excerpt_of(body: &str) -> String {
  let trimmed: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).take(3).collect();
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

pub fn scan_markdown_files(root: &Path) -> Vec<(String, PathBuf, String)> {
  let mut notes = Vec::new();
  scan_recursive(root, root, &mut notes);
  notes.sort_by(|a, b| a.0.cmp(&b.0));
  notes
}

fn scan_recursive(root: &Path, current: &Path, out: &mut Vec<(String, PathBuf, String)>) {
  let Ok(entries) = fs::read_dir(current) else { return };
  for entry in entries.flatten() {
    let name = entry.file_name().to_string_lossy().to_string();
    if name.starts_with('.') {
      continue;
    }
    let path = entry.path();
    let Ok(meta) = fs::symlink_metadata(&path) else { continue };
    if meta.file_type().is_symlink() {
      continue;
    }
    if meta.is_dir() {
      scan_recursive(root, &path, out);
    } else if meta.is_file() && name.to_ascii_lowercase().ends_with(".md") {
      let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
      let content = fs::read_to_string(&path).unwrap_or_default();
      out.push((relative, path, content));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn make_vault() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
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
    index.upsert_note("v", "alpha.md", "/v/alpha.md", "Alpha Note", "first document", 1).unwrap();
    index.upsert_note("v", "beta.md", "/v/beta.md", "Beta Note", "second document", 1).unwrap();
    let hits = index.search("alpha", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Alpha Note");
    assert_eq!(hits[0].path, "alpha.md");
  }

  #[test]
  fn search_ranks_relevance_by_bm25() {
    let index = FtsIndex::open_in_memory().unwrap();
    index.upsert_note("v", "a.md", "/v/a.md", "Alpha Beta", "alpha alpha alpha", 1).unwrap();
    index.upsert_note("v", "b.md", "/v/b.md", "Beta Only", "beta once", 1).unwrap();
    let hits = index.search("alpha", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "a.md");
  }

  #[test]
  fn upsert_is_idempotent() {
    let index = FtsIndex::open_in_memory().unwrap();
    index.upsert_note("v", "n.md", "/v/n.md", "Title", "body", 1).unwrap();
    index.upsert_note("v", "n.md", "/v/n.md", "Updated", "new body", 2).unwrap();
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
    index.upsert_note("v", "gone.md", "/v/gone.md", "Gone", "disappearing", 1).unwrap();
    index.remove_note("v", "gone.md").unwrap();
    let hits = index.search("disappearing", 10).unwrap();
    assert!(hits.is_empty());
  }

  #[test]
  fn clear_vault_drops_only_that_vault() {
    let index = FtsIndex::open_in_memory().unwrap();
    index.upsert_note("v1", "a.md", "/v1/a.md", "VA", "alpha", 1).unwrap();
    index.upsert_note("v2", "b.md", "/v2/b.md", "VB", "alpha", 1).unwrap();
    index.clear_vault("v1").unwrap();
    assert_eq!(index.count("v1").unwrap(), 0);
    assert_eq!(index.count("v2").unwrap(), 1);
  }

  #[test]
  fn empty_query_returns_no_hits() {
    let index = FtsIndex::open_in_memory().unwrap();
    index.upsert_note("v", "n.md", "/v/n.md", "T", "body", 1).unwrap();
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
    let paths: Vec<_> = notes.iter().map(|(p, _, _)| p.clone()).collect();
    assert!(paths.contains(&"a.md".into()));
    assert!(paths.contains(&"sub/b.md".into()));
    assert!(!paths.contains(&".hidden.md".into()));
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn open_vault_creates_db_under_elephantnote_index() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("elephantnote_fts_vault_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    let fts = FtsIndex::open(&dir).unwrap();
    fts.upsert_note("v", "x.md", "/v/x.md", "X", "ex content", 1).unwrap();
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
      index.upsert_note("v", &format!("n{i}.md"), &format!("/v/n{i}.md"), "Common", "alpha", 1).unwrap();
    }
    assert_eq!(index.search("alpha", 2).unwrap().len(), 2);
  }
}