use crate::model::DocumentSnapshot;
use crate::relations::{
    node_kind_name, parse_node_kind, parse_relation_origin, parse_relation_status,
    parse_relation_type, relation_origin_name, relation_status_name, relation_type_name,
    KnowledgeNodeKind, KnowledgeNodeRef, KnowledgeRelation, RelationOrigin, RelationStatus,
    RelationType,
};
use crate::storage::KnowledgeStore;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::Path;

const RELATION_SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS knowledge_relations (
  id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL,
  target_kind TEXT NOT NULL,
  target_id TEXT NOT NULL,
  relation_type TEXT NOT NULL,
  origin TEXT NOT NULL,
  status TEXT NOT NULL,
  confidence REAL,
  evidence_chunk_ids_json TEXT NOT NULL DEFAULT '[]',
  reason TEXT NOT NULL DEFAULT '',
  model_id TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX IF NOT EXISTS knowledge_relations_source_idx
  ON knowledge_relations(source_kind, source_id, status);
CREATE INDEX IF NOT EXISTS knowledge_relations_target_idx
  ON knowledge_relations(target_kind, target_id, status);
CREATE INDEX IF NOT EXISTS knowledge_relations_type_idx
  ON knowledge_relations(relation_type, status);
CREATE INDEX IF NOT EXISTS knowledge_relations_origin_idx
  ON knowledge_relations(origin, status);
"#;

impl KnowledgeStore {
    pub fn initialize_relations(&self) -> Result<(), String> {
        open_relation_connection(self.database_path())?
            .execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())
    }

    pub fn save_relation(&self, relation: &KnowledgeRelation) -> Result<(), String> {
        let source_document = if matches!(relation.source.kind, KnowledgeNodeKind::Document) {
            self.inspect_document(&relation.source.id)?
        } else {
            None
        };
        let validation = relation.validate(source_document.as_ref());
        if !validation.valid {
            return Err(validation.errors.join(" "));
        }

        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let evidence = serde_json::to_string(&relation.evidence_chunk_ids)
            .map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO knowledge_relations(
               id, source_kind, source_id, target_kind, target_id, relation_type,
               origin, status, confidence, evidence_chunk_ids_json, reason, model_id, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, unixepoch())
             ON CONFLICT(id) DO UPDATE SET
               status=excluded.status,
               confidence=excluded.confidence,
               evidence_chunk_ids_json=excluded.evidence_chunk_ids_json,
               reason=excluded.reason,
               model_id=excluded.model_id,
               updated_at=unixepoch()",
            params![
                relation.id,
                node_kind_name(&relation.source.kind),
                relation.source.id,
                node_kind_name(&relation.target.kind),
                relation.target.id,
                relation_type_name(&relation.relation_type),
                relation_origin_name(&relation.origin),
                relation_status_name(&relation.status),
                relation.confidence.map(f64::from),
                evidence,
                relation.reason,
                relation.model_id,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_relation_status(
        &self,
        relation_id: &str,
        status: RelationStatus,
    ) -> Result<bool, String> {
        if relation_id.trim().is_empty() {
            return Err("Relation ID cannot be empty.".into());
        }
        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let changed = conn
            .execute(
                "UPDATE knowledge_relations SET status=?2, updated_at=unixepoch() WHERE id=?1",
                params![relation_id, relation_status_name(&status)],
            )
            .map_err(|error| error.to_string())?;
        Ok(changed > 0)
    }

    pub fn relation_by_id(&self, relation_id: &str) -> Result<Option<KnowledgeRelation>, String> {
        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        conn.query_row(
            "SELECT id, source_kind, source_id, target_kind, target_id, relation_type,
                    origin, status, confidence, evidence_chunk_ids_json, reason, model_id
             FROM knowledge_relations WHERE id=?1",
            params![relation_id],
            map_relation_row,
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    pub fn relations_for_node(
        &self,
        node: &KnowledgeNodeRef,
        include_rejected: bool,
    ) -> Result<Vec<KnowledgeRelation>, String> {
        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let sql = if include_rejected {
            "SELECT id, source_kind, source_id, target_kind, target_id, relation_type,
                    origin, status, confidence, evidence_chunk_ids_json, reason, model_id
             FROM knowledge_relations
             WHERE (source_kind=?1 AND source_id=?2) OR (target_kind=?1 AND target_id=?2)
             ORDER BY updated_at DESC, id"
        } else {
            "SELECT id, source_kind, source_id, target_kind, target_id, relation_type,
                    origin, status, confidence, evidence_chunk_ids_json, reason, model_id
             FROM knowledge_relations
             WHERE ((source_kind=?1 AND source_id=?2) OR (target_kind=?1 AND target_id=?2))
               AND status!='rejected'
             ORDER BY updated_at DESC, id"
        };
        let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(
                params![node_kind_name(&node.kind), node.id],
                map_relation_row,
            )
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn list_relations(
        &self,
        status: Option<RelationStatus>,
        limit: usize,
    ) -> Result<Vec<KnowledgeRelation>, String> {
        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let capped_limit = limit.clamp(1, 5_000) as i64;
        if let Some(status) = status {
            let mut statement = conn
                .prepare(
                    "SELECT id, source_kind, source_id, target_kind, target_id, relation_type,
                            origin, status, confidence, evidence_chunk_ids_json, reason, model_id
                     FROM knowledge_relations WHERE status=?1
                     ORDER BY updated_at DESC, id LIMIT ?2",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(
                    params![relation_status_name(&status), capped_limit],
                    map_relation_row,
                )
                .map_err(|error| error.to_string())?;
            return rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string());
        }

        let mut statement = conn
            .prepare(
                "SELECT id, source_kind, source_id, target_kind, target_id, relation_type,
                        origin, status, confidence, evidence_chunk_ids_json, reason, model_id
                 FROM knowledge_relations ORDER BY updated_at DESC, id LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![capped_limit], map_relation_row)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn sync_markdown_relations(&self, document: &DocumentSnapshot) -> Result<usize, String> {
        let conn = open_relation_connection(self.database_path())?;
        conn.execute_batch(RELATION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM knowledge_relations
                 WHERE source_kind='document' AND source_id=?1 AND origin='markdown'",
                params![document.relative_path],
            )
            .map_err(|error| error.to_string())?;

        let mut unique_relations = HashSet::new();
        let mut inserted = 0usize;
        for link in &document.explicit_links {
            let relation = KnowledgeRelation::new(
                KnowledgeNodeRef {
                    kind: KnowledgeNodeKind::Document,
                    id: document.relative_path.clone(),
                },
                KnowledgeNodeRef {
                    kind: KnowledgeNodeKind::Document,
                    id: link.target.clone(),
                },
                RelationType::ExplicitLink,
                RelationOrigin::Markdown,
                RelationStatus::Explicit,
                None,
                Vec::new(),
                format!("Explicit wikilink: {}", link.label),
                None,
            );
            if !unique_relations.insert(relation.id.clone()) {
                continue;
            }
            let evidence = "[]";
            transaction
                .execute(
                    "INSERT INTO knowledge_relations(
                       id, source_kind, source_id, target_kind, target_id, relation_type,
                       origin, status, confidence, evidence_chunk_ids_json, reason, model_id, updated_at
                     ) VALUES (?1, 'document', ?2, 'document', ?3, 'explicit_link',
                               'markdown', 'explicit', NULL, ?4, ?5, NULL, unixepoch())
                     ON CONFLICT(id) DO UPDATE SET
                       status=excluded.status,
                       evidence_chunk_ids_json=excluded.evidence_chunk_ids_json,
                       reason=excluded.reason,
                       updated_at=unixepoch()",
                    params![
                        relation.id,
                        document.relative_path,
                        link.target,
                        evidence,
                        relation.reason,
                    ],
                )
                .map_err(|error| error.to_string())?;
            inserted += 1;
        }
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(inserted)
    }
}

fn open_relation_connection(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|error| error.to_string())
}

fn map_relation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeRelation> {
    let evidence_json: String = row.get(9)?;
    let evidence_chunk_ids = serde_json::from_str(&evidence_json).unwrap_or_default();
    Ok(KnowledgeRelation {
        id: row.get(0)?,
        source: KnowledgeNodeRef {
            kind: parse_node_kind(&row.get::<_, String>(1)?),
            id: row.get(2)?,
        },
        target: KnowledgeNodeRef {
            kind: parse_node_kind(&row.get::<_, String>(3)?),
            id: row.get(4)?,
        },
        relation_type: parse_relation_type(&row.get::<_, String>(5)?),
        origin: parse_relation_origin(&row.get::<_, String>(6)?),
        status: parse_relation_status(&row.get::<_, String>(7)?),
        confidence: row.get::<_, Option<f64>>(8)?.map(|value| value as f32),
        evidence_chunk_ids,
        reason: row.get(10)?,
        model_id: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "elephant-relations-{name}-{}-{stamp}",
            std::process::id()
        ))
    }

    fn document_node(path: &str) -> KnowledgeNodeRef {
        KnowledgeNodeRef {
            kind: KnowledgeNodeKind::Document,
            id: path.into(),
        }
    }

    #[test]
    fn saves_and_updates_evidence_backed_relation() {
        let root = temp_vault("save");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let source = analyze_markdown("A.md", "# A\nA supports B.", 1);
        store.upsert_document(&source).unwrap();
        let relation = KnowledgeRelation::new(
            document_node("A.md"),
            document_node("B.md"),
            RelationType::Supports,
            RelationOrigin::Model,
            RelationStatus::Suggested,
            Some(0.91),
            vec![source.chunks[0].id.clone()],
            "Explicit support statement.",
            Some("small-model".into()),
        );
        store.save_relation(&relation).unwrap();
        assert_eq!(
            store.relation_by_id(&relation.id).unwrap(),
            Some(relation.clone())
        );
        assert!(store
            .set_relation_status(&relation.id, RelationStatus::Accepted)
            .unwrap());
        assert_eq!(
            store.relation_by_id(&relation.id).unwrap().unwrap().status,
            RelationStatus::Accepted
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn syncs_only_real_markdown_wikilinks() {
        let root = temp_vault("markdown");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let document = analyze_markdown(
            "A.md",
            "# A\nSee [[B]] and ![[Embedded.png]] and [external](https://example.com).",
            1,
        );
        store.upsert_document(&document).unwrap();
        let count = store.sync_markdown_relations(&document).unwrap();
        assert_eq!(count, 1);
        let relations = store
            .relations_for_node(
                &KnowledgeNodeRef {
                    kind: KnowledgeNodeKind::Document,
                    id: "A.md".into(),
                },
                false,
            )
            .unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].target.id, "B");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn duplicate_markdown_wikilinks_are_idempotent() {
        let root = temp_vault("duplicate-markdown");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let document = analyze_markdown("A.md", "# A\nSee [[B]] then [[B|same target]] again.", 1);
        store.upsert_document(&document).unwrap();
        let count = store.sync_markdown_relations(&document).unwrap();
        assert_eq!(count, 1);
        let count = store.sync_markdown_relations(&document).unwrap();
        assert_eq!(count, 1);
        let relations = store
            .relations_for_node(
                &KnowledgeNodeRef {
                    kind: KnowledgeNodeKind::Document,
                    id: "A.md".into(),
                },
                false,
            )
            .unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].target.id, "B");
        fs::remove_dir_all(root).ok();
    }
}
