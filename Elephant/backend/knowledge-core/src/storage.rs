use crate::model::{
    DocumentSnapshot, ExplicitLink, KnowledgeChunk, KnowledgeSearchHit, KnowledgeSection,
    KnowledgeStatus,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
INSERT INTO schema_meta(key, value) VALUES ('schema_version', '1')
ON CONFLICT(key) DO UPDATE SET value=excluded.value;

CREATE TABLE IF NOT EXISTS documents (
  relative_path TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  modified_at INTEGER NOT NULL,
  indexed_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS sections (
  id TEXT PRIMARY KEY,
  document_path TEXT NOT NULL,
  heading TEXT NOT NULL,
  level INTEGER NOT NULL,
  ordinal INTEGER NOT NULL,
  start_offset INTEGER NOT NULL,
  end_offset INTEGER NOT NULL,
  FOREIGN KEY(document_path) REFERENCES documents(relative_path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS sections_document_idx ON sections(document_path, ordinal);

CREATE TABLE IF NOT EXISTS chunks (
  id TEXT PRIMARY KEY,
  document_path TEXT NOT NULL,
  section_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  start_offset INTEGER NOT NULL,
  end_offset INTEGER NOT NULL,
  token_estimate INTEGER NOT NULL,
  content_hash TEXT NOT NULL,
  text TEXT NOT NULL,
  FOREIGN KEY(document_path) REFERENCES documents(relative_path) ON DELETE CASCADE,
  FOREIGN KEY(section_id) REFERENCES sections(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS chunks_document_idx ON chunks(document_path, ordinal);
CREATE INDEX IF NOT EXISTS chunks_section_idx ON chunks(section_id);

CREATE TABLE IF NOT EXISTS wikilinks (
  document_path TEXT NOT NULL,
  target TEXT NOT NULL,
  label TEXT NOT NULL,
  start_offset INTEGER NOT NULL,
  end_offset INTEGER NOT NULL,
  FOREIGN KEY(document_path) REFERENCES documents(relative_path) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS wikilinks_document_idx ON wikilinks(document_path);
CREATE INDEX IF NOT EXISTS wikilinks_target_idx ON wikilinks(target);

CREATE VIRTUAL TABLE IF NOT EXISTS chunk_search USING fts5(
  chunk_id UNINDEXED,
  relative_path UNINDEXED,
  title,
  heading,
  body,
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE IF NOT EXISTS action_proposals (
  id TEXT PRIMARY KEY,
  action_json TEXT NOT NULL,
  rationale TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'proposed',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
"#;

pub struct KnowledgeStore {
    conn: Connection,
    database_path: PathBuf,
}

impl KnowledgeStore {
    pub fn open(vault_root: &Path) -> Result<Self, String> {
        let directory = vault_root.join(".elephantnote").join("knowledge");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let database_path = directory.join("knowledge.sqlite");
        let conn = Connection::open(&database_path).map_err(|error| error.to_string())?;
        conn.execute_batch(SCHEMA)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            conn,
            database_path,
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|error| error.to_string())?;
        conn.execute_batch(SCHEMA)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            conn,
            database_path: PathBuf::from(":memory:"),
        })
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn existing_hash(&self, relative_path: &str) -> Result<Option<String>, String> {
        self.conn
            .query_row(
                "SELECT content_hash FROM documents WHERE relative_path=?1",
                params![relative_path],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn upsert_document(&mut self, document: &DocumentSnapshot) -> Result<(), String> {
        let transaction = self.conn.transaction().map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO documents(relative_path, title, content_hash, modified_at, indexed_at)
                 VALUES (?1, ?2, ?3, ?4, unixepoch())
                 ON CONFLICT(relative_path) DO UPDATE SET
                   title=excluded.title,
                   content_hash=excluded.content_hash,
                   modified_at=excluded.modified_at,
                   indexed_at=unixepoch()",
                params![
                    document.relative_path,
                    document.title,
                    document.content_hash,
                    document.modified_at
                ],
            )
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "DELETE FROM chunk_search WHERE relative_path=?1",
                params![document.relative_path],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM wikilinks WHERE document_path=?1",
                params![document.relative_path],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM chunks WHERE document_path=?1",
                params![document.relative_path],
            )
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM sections WHERE document_path=?1",
                params![document.relative_path],
            )
            .map_err(|error| error.to_string())?;

        for section in &document.sections {
            transaction
                .execute(
                    "INSERT INTO sections(id, document_path, heading, level, ordinal, start_offset, end_offset)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        section.id,
                        document.relative_path,
                        section.heading,
                        section.level as i64,
                        section.ordinal as i64,
                        section.start_offset as i64,
                        section.end_offset as i64,
                    ],
                )
                .map_err(|error| error.to_string())?;
        }

        for chunk in &document.chunks {
            let heading = document
                .sections
                .iter()
                .find(|section| section.id == chunk.section_id)
                .map(|section| section.heading.as_str())
                .unwrap_or(document.title.as_str());
            transaction
                .execute(
                    "INSERT INTO chunks(id, document_path, section_id, ordinal, start_offset, end_offset, token_estimate, content_hash, text)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        chunk.id,
                        document.relative_path,
                        chunk.section_id,
                        chunk.ordinal as i64,
                        chunk.start_offset as i64,
                        chunk.end_offset as i64,
                        chunk.token_estimate as i64,
                        chunk.content_hash,
                        chunk.text,
                    ],
                )
                .map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "INSERT INTO chunk_search(chunk_id, relative_path, title, heading, body) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![chunk.id, document.relative_path, document.title, heading, chunk.text],
                )
                .map_err(|error| error.to_string())?;
        }

        for link in &document.explicit_links {
            transaction
                .execute(
                    "INSERT INTO wikilinks(document_path, target, label, start_offset, end_offset) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        document.relative_path,
                        link.target,
                        link.label,
                        link.start_offset as i64,
                        link.end_offset as i64,
                    ],
                )
                .map_err(|error| error.to_string())?;
        }

        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn prune_documents(&mut self, present_paths: &HashSet<String>) -> Result<usize, String> {
        let paths = {
            let mut statement = self
                .conn
                .prepare("SELECT relative_path FROM documents")
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| error.to_string())?;
            let collected = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?;
            collected
        };
        let stale = paths
            .into_iter()
            .filter(|path| !present_paths.contains(path))
            .collect::<Vec<_>>();
        let transaction = self.conn.transaction().map_err(|error| error.to_string())?;
        for path in &stale {
            transaction
                .execute(
                    "DELETE FROM chunk_search WHERE relative_path=?1",
                    params![path],
                )
                .map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "DELETE FROM documents WHERE relative_path=?1",
                    params![path],
                )
                .map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(stale.len())
    }

    pub fn status(&self) -> Result<KnowledgeStatus, String> {
        Ok(KnowledgeStatus {
            documents: count(&self.conn, "documents")?,
            sections: count(&self.conn, "sections")?,
            chunks: count(&self.conn, "chunks")?,
            explicit_links: count(&self.conn, "wikilinks")?,
            pending_actions: self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM action_proposals WHERE status='proposed'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?,
            database_path: self.database_path.to_string_lossy().to_string(),
        })
    }

    pub fn inspect_document(
        &self,
        relative_path: &str,
    ) -> Result<Option<DocumentSnapshot>, String> {
        let document = self
            .conn
            .query_row(
                "SELECT title, content_hash, modified_at FROM documents WHERE relative_path=?1",
                params![relative_path],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let Some((title, content_hash, modified_at)) = document else {
            return Ok(None);
        };

        let sections = query_sections(&self.conn, relative_path)?;
        let chunks = query_chunks(&self.conn, relative_path)?;
        let explicit_links = query_links(&self.conn, relative_path)?;
        Ok(Some(DocumentSnapshot {
            relative_path: relative_path.to_string(),
            title,
            content_hash,
            modified_at,
            sections,
            chunks,
            explicit_links,
        }))
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeSearchHit>, String> {
        let fts_query = to_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT relative_path, title, heading, chunk_id,
                        snippet(chunk_search, 4, '', '', ' … ', 32),
                        bm25(chunk_search),
                        c.start_offset, c.end_offset
                 FROM chunk_search
                 JOIN chunks c ON c.id=chunk_search.chunk_id
                 WHERE chunk_search MATCH ?1
                 ORDER BY bm25(chunk_search)
                 LIMIT ?2",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![fts_query, limit.clamp(1, 100) as i64], |row| {
                let raw_score: f64 = row.get(5)?;
                Ok(KnowledgeSearchHit {
                    relative_path: row.get(0)?,
                    title: row.get(1)?,
                    heading: row.get(2)?,
                    chunk_id: row.get(3)?,
                    excerpt: row.get(4)?,
                    score: -raw_score,
                    start_offset: row.get::<_, i64>(6)? as usize,
                    end_offset: row.get::<_, i64>(7)? as usize,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }
}

fn count(conn: &Connection, table: &str) -> Result<i64, String> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .map_err(|error| error.to_string())
}

fn query_sections(conn: &Connection, path: &str) -> Result<Vec<KnowledgeSection>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, heading, level, ordinal, start_offset, end_offset
             FROM sections WHERE document_path=?1 ORDER BY ordinal",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![path], |row| {
            Ok(KnowledgeSection {
                id: row.get(0)?,
                heading: row.get(1)?,
                level: row.get::<_, i64>(2)? as u8,
                ordinal: row.get::<_, i64>(3)? as usize,
                start_offset: row.get::<_, i64>(4)? as usize,
                end_offset: row.get::<_, i64>(5)? as usize,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn query_chunks(conn: &Connection, path: &str) -> Result<Vec<KnowledgeChunk>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, section_id, ordinal, start_offset, end_offset, token_estimate, content_hash, text
             FROM chunks WHERE document_path=?1 ORDER BY ordinal",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![path], |row| {
            Ok(KnowledgeChunk {
                id: row.get(0)?,
                section_id: row.get(1)?,
                ordinal: row.get::<_, i64>(2)? as usize,
                start_offset: row.get::<_, i64>(3)? as usize,
                end_offset: row.get::<_, i64>(4)? as usize,
                token_estimate: row.get::<_, i64>(5)? as usize,
                content_hash: row.get(6)?,
                text: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn query_links(conn: &Connection, path: &str) -> Result<Vec<ExplicitLink>, String> {
    let mut statement = conn
        .prepare(
            "SELECT target, label, start_offset, end_offset
             FROM wikilinks WHERE document_path=?1 ORDER BY start_offset",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![path], |row| {
            Ok(ExplicitLink {
                target: row.get(0)?,
                label: row.get(1)?,
                start_offset: row.get::<_, i64>(2)? as usize,
                end_offset: row.get::<_, i64>(3)? as usize,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn to_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;

    #[test]
    fn upsert_replaces_all_derived_rows() {
        let mut store = KnowledgeStore::open_in_memory().unwrap();
        let first = analyze_markdown("a.md", "# Alpha\n\nold body [[B]]", 1);
        store.upsert_document(&first).unwrap();
        let second = analyze_markdown("a.md", "# Alpha\n\nnew body", 2);
        store.upsert_document(&second).unwrap();
        let status = store.status().unwrap();
        assert_eq!(status.documents, 1);
        assert_eq!(status.explicit_links, 0);
        assert!(store.search("new", 10).unwrap().len() == 1);
        assert!(store.search("old", 10).unwrap().is_empty());
    }

    #[test]
    fn inspect_roundtrips_offsets_and_links() {
        let mut store = KnowledgeStore::open_in_memory().unwrap();
        let snapshot = analyze_markdown("Notes/a.md", "# Alpha\n\nSee [[Beta]].", 7);
        store.upsert_document(&snapshot).unwrap();
        let loaded = store.inspect_document("Notes/a.md").unwrap().unwrap();
        assert_eq!(loaded.title, "Alpha");
        assert_eq!(loaded.content_hash, snapshot.content_hash);
        assert_eq!(loaded.sections, snapshot.sections);
        assert_eq!(loaded.chunks, snapshot.chunks);
        assert_eq!(loaded.explicit_links, snapshot.explicit_links);
    }

    #[test]
    fn prune_removes_missing_documents_and_search_rows() {
        let mut store = KnowledgeStore::open_in_memory().unwrap();
        store
            .upsert_document(&analyze_markdown("a.md", "# A\nalpha", 1))
            .unwrap();
        store
            .upsert_document(&analyze_markdown("b.md", "# B\nbeta", 1))
            .unwrap();
        let present = HashSet::from(["a.md".to_string()]);
        assert_eq!(store.prune_documents(&present).unwrap(), 1);
        assert_eq!(store.status().unwrap().documents, 1);
        assert!(store.search("beta", 10).unwrap().is_empty());
    }
}
