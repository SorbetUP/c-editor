use crate::storage::KnowledgeStore;
use crate::taxonomy::{normalize_alias, CanonicalTag, TagAlias, TagStatus};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

const TAXONOMY_SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS tags (
  id TEXT PRIMARY KEY,
  canonical_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  parent_id TEXT,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'active',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE(canonical_name, parent_id),
  FOREIGN KEY(parent_id) REFERENCES tags(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS tags_parent_idx ON tags(parent_id);
CREATE INDEX IF NOT EXISTS tags_status_idx ON tags(status);

CREATE TABLE IF NOT EXISTS tag_aliases (
  normalized_alias TEXT NOT NULL,
  language TEXT NOT NULL DEFAULT '',
  tag_id TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY(normalized_alias, language),
  FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS tag_aliases_tag_idx ON tag_aliases(tag_id);

CREATE TABLE IF NOT EXISTS document_tags (
  document_path TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  origin TEXT NOT NULL,
  confidence REAL,
  evidence_chunk_ids_json TEXT NOT NULL DEFAULT '[]',
  status TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY(document_path, tag_id, origin),
  FOREIGN KEY(document_path) REFERENCES documents(relative_path) ON DELETE CASCADE,
  FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS document_tags_document_idx ON document_tags(document_path, status);
CREATE INDEX IF NOT EXISTS document_tags_tag_idx ON document_tags(tag_id, status);

CREATE TABLE IF NOT EXISTS rejected_tag_suggestions (
  document_path TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  PRIMARY KEY(document_path, normalized_name),
  FOREIGN KEY(document_path) REFERENCES documents(relative_path) ON DELETE CASCADE
);
"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TagAssignmentOrigin {
    User,
    Model,
    Import,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TagAssignmentStatus {
    Suggested,
    Accepted,
    Rejected,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentTagAssignment {
    pub document_path: String,
    pub tag_id: String,
    pub origin: TagAssignmentOrigin,
    pub confidence: Option<f32>,
    pub evidence_chunk_ids: Vec<String>,
    pub status: TagAssignmentStatus,
    pub reason: String,
}

impl KnowledgeStore {
    pub fn initialize_taxonomy(&self) -> Result<(), String> {
        open_taxonomy_connection(self.database_path())?
            .execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())
    }

    pub fn upsert_canonical_tag(&self, tag: &CanonicalTag) -> Result<(), String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO tags(id, canonical_name, display_name, parent_id, description, status, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, unixepoch())
             ON CONFLICT(id) DO UPDATE SET
               canonical_name=excluded.canonical_name,
               display_name=excluded.display_name,
               parent_id=excluded.parent_id,
               description=excluded.description,
               status=excluded.status,
               updated_at=unixepoch()",
            params![
                tag.id,
                tag.canonical_name,
                tag.display_name,
                tag.parent_id,
                tag.description,
                tag_status_name(&tag.status),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn add_tag_alias(&self, alias: &TagAlias) -> Result<(), String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let language = alias.language.as_deref().unwrap_or("");
        conn.execute(
            "INSERT INTO tag_aliases(normalized_alias, language, tag_id)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(normalized_alias, language) DO UPDATE SET tag_id=excluded.tag_id",
            params![alias.normalized_alias, language, alias.tag_id],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn resolve_tag(
        &self,
        name_or_alias: &str,
        language: Option<&str>,
    ) -> Result<Option<CanonicalTag>, String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let normalized = normalize_alias(name_or_alias)?;
        let language = language.unwrap_or("").trim().to_ascii_lowercase();

        let tag_id = conn
            .query_row(
                "SELECT tag_id FROM tag_aliases
                 WHERE normalized_alias=?1 AND language IN (?2, '')
                 ORDER BY CASE WHEN language=?2 THEN 0 ELSE 1 END
                 LIMIT 1",
                params![normalized, language],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;

        if let Some(tag_id) = tag_id {
            return load_tag(&conn, &tag_id);
        }

        conn.query_row(
            "SELECT id, canonical_name, display_name, parent_id, description, status
             FROM tags WHERE canonical_name=?1 AND status='active' LIMIT 1",
            params![normalized],
            map_tag_row,
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    pub fn list_canonical_tags(&self) -> Result<Vec<CanonicalTag>, String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let mut statement = conn
            .prepare(
                "SELECT id, canonical_name, display_name, parent_id, description, status
                 FROM tags WHERE status='active' ORDER BY canonical_name",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], map_tag_row)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn save_document_tag_assignment(
        &self,
        assignment: &DocumentTagAssignment,
    ) -> Result<(), String> {
        validate_assignment(assignment)?;
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let evidence = serde_json::to_string(&assignment.evidence_chunk_ids)
            .map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO document_tags(
               document_path, tag_id, origin, confidence, evidence_chunk_ids_json, status, reason, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, unixepoch())
             ON CONFLICT(document_path, tag_id, origin) DO UPDATE SET
               confidence=excluded.confidence,
               evidence_chunk_ids_json=excluded.evidence_chunk_ids_json,
               status=excluded.status,
               reason=excluded.reason,
               updated_at=unixepoch()",
            params![
                assignment.document_path,
                assignment.tag_id,
                assignment_origin_name(&assignment.origin),
                assignment.confidence.map(f64::from),
                evidence,
                assignment_status_name(&assignment.status),
                assignment.reason,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn reject_tag_name(
        &self,
        document_path: &str,
        proposed_name: &str,
        reason: &str,
    ) -> Result<(), String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let normalized = normalize_alias(proposed_name)?;
        conn.execute(
            "INSERT INTO rejected_tag_suggestions(document_path, normalized_name, reason)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(document_path, normalized_name) DO UPDATE SET reason=excluded.reason",
            params![document_path, normalized, reason.trim()],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn is_tag_name_rejected(
        &self,
        document_path: &str,
        proposed_name: &str,
    ) -> Result<bool, String> {
        let conn = open_taxonomy_connection(self.database_path())?;
        conn.execute_batch(TAXONOMY_SCHEMA)
            .map_err(|error| error.to_string())?;
        let normalized = normalize_alias(proposed_name)?;
        let found = conn
            .query_row(
                "SELECT 1 FROM rejected_tag_suggestions
                 WHERE document_path=?1 AND normalized_name=?2",
                params![document_path, normalized],
                |_row| Ok(()),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        Ok(found.is_some())
    }
}

fn open_taxonomy_connection(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|error| error.to_string())
}

fn load_tag(conn: &Connection, tag_id: &str) -> Result<Option<CanonicalTag>, String> {
    conn.query_row(
        "SELECT id, canonical_name, display_name, parent_id, description, status
         FROM tags WHERE id=?1 LIMIT 1",
        params![tag_id],
        map_tag_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn map_tag_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CanonicalTag> {
    let status: String = row.get(5)?;
    Ok(CanonicalTag {
        id: row.get(0)?,
        canonical_name: row.get(1)?,
        display_name: row.get(2)?,
        parent_id: row.get(3)?,
        description: row.get(4)?,
        status: parse_tag_status(&status),
    })
}

fn validate_assignment(assignment: &DocumentTagAssignment) -> Result<(), String> {
    if assignment.document_path.trim().is_empty() {
        return Err("Document path cannot be empty.".into());
    }
    if assignment.tag_id.trim().is_empty() {
        return Err("Tag ID cannot be empty.".into());
    }
    if let Some(confidence) = assignment.confidence {
        if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
            return Err("Tag confidence must be between 0 and 1.".into());
        }
    }
    if matches!(assignment.origin, TagAssignmentOrigin::Model)
        && assignment.evidence_chunk_ids.is_empty()
    {
        return Err("Model tag assignments require evidence chunks.".into());
    }
    Ok(())
}

fn tag_status_name(status: &TagStatus) -> &'static str {
    match status {
        TagStatus::Active => "active",
        TagStatus::Merged => "merged",
        TagStatus::Hidden => "hidden",
    }
}

fn parse_tag_status(value: &str) -> TagStatus {
    match value {
        "merged" => TagStatus::Merged,
        "hidden" => TagStatus::Hidden,
        _ => TagStatus::Active,
    }
}

fn assignment_origin_name(origin: &TagAssignmentOrigin) -> &'static str {
    match origin {
        TagAssignmentOrigin::User => "user",
        TagAssignmentOrigin::Model => "model",
        TagAssignmentOrigin::Import => "import",
        TagAssignmentOrigin::System => "system",
    }
}

fn assignment_status_name(status: &TagAssignmentStatus) -> &'static str {
    match status {
        TagAssignmentStatus::Suggested => "suggested",
        TagAssignmentStatus::Accepted => "accepted",
        TagAssignmentStatus::Rejected => "rejected",
        TagAssignmentStatus::Manual => "manual",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;
    use crate::taxonomy::CanonicalTag;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "elephant-taxonomy-{name}-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn resolves_explicit_multilingual_aliases() {
        let root = temp_vault("aliases");
        fs::create_dir_all(&root).unwrap();
        let store = KnowledgeStore::open(&root).unwrap();
        let tag = CanonicalTag::new("Intelligence artificielle", None, "AI topic").unwrap();
        store.upsert_canonical_tag(&tag).unwrap();
        store
            .add_tag_alias(
                &TagAlias::new("Artificial Intelligence", tag.id.clone(), Some("en".into()))
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(
            store
                .resolve_tag("Artificial Intelligence", Some("en"))
                .unwrap()
                .unwrap()
                .id,
            tag.id
        );
        assert!(store.resolve_tag("AI", Some("en")).unwrap().is_none());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn persists_evidence_backed_model_assignment_and_rejection_memory() {
        let root = temp_vault("assignment");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let document = analyze_markdown("Notes/Iroh.md", "# Iroh\nP2P networking", 1);
        store.upsert_document(&document).unwrap();
        let tag = CanonicalTag::new("Iroh", None, "Technology").unwrap();
        store.upsert_canonical_tag(&tag).unwrap();
        store
            .save_document_tag_assignment(&DocumentTagAssignment {
                document_path: document.relative_path.clone(),
                tag_id: tag.id,
                origin: TagAssignmentOrigin::Model,
                confidence: Some(0.97),
                evidence_chunk_ids: vec![document.chunks[0].id.clone()],
                status: TagAssignmentStatus::Suggested,
                reason: "Direct subject".into(),
            })
            .unwrap();
        store
            .reject_tag_name(&document.relative_path, "Networking", "Too broad")
            .unwrap();
        assert!(store
            .is_tag_name_rejected(&document.relative_path, "networking")
            .unwrap());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn model_assignments_without_evidence_are_rejected() {
        let assignment = DocumentTagAssignment {
            document_path: "a.md".into(),
            tag_id: "tag-a".into(),
            origin: TagAssignmentOrigin::Model,
            confidence: Some(0.7),
            evidence_chunk_ids: Vec::new(),
            status: TagAssignmentStatus::Suggested,
            reason: String::new(),
        };
        assert!(validate_assignment(&assignment).is_err());
    }
}
