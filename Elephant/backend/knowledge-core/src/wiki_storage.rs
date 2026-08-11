use crate::storage::KnowledgeStore;
use crate::wiki_core::{WikiDraft, WikiDraftStatus};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

const WIKI_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS wiki_drafts (
  id TEXT PRIMARY KEY,
  topic TEXT NOT NULL,
  title TEXT NOT NULL,
  slug TEXT NOT NULL,
  markdown TEXT NOT NULL,
  citations_json TEXT NOT NULL,
  source_paths_json TEXT NOT NULL,
  source_hash TEXT NOT NULL,
  model_id TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(slug, status)
);
CREATE INDEX IF NOT EXISTS wiki_drafts_status_idx
  ON wiki_drafts(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS wiki_drafts_topic_idx
  ON wiki_drafts(topic, status);
"#;

impl KnowledgeStore {
    pub fn initialize_wikis(&self) -> Result<(), String> {
        open_wiki_connection(self.database_path())?
            .execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())
    }

    pub fn save_wiki_draft(&self, draft: &WikiDraft) -> Result<(), String> {
        validate_draft(draft)?;
        let conn = open_wiki_connection(self.database_path())?;
        conn.execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())?;
        let citations_json =
            serde_json::to_string(&draft.citations).map_err(|error| error.to_string())?;
        let source_paths_json =
            serde_json::to_string(&draft.source_paths).map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO wiki_drafts(
               id, topic, title, slug, markdown, citations_json, source_paths_json,
               source_hash, model_id, status, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
               topic=excluded.topic,
               title=excluded.title,
               slug=excluded.slug,
               markdown=excluded.markdown,
               citations_json=excluded.citations_json,
               source_paths_json=excluded.source_paths_json,
               source_hash=excluded.source_hash,
               model_id=excluded.model_id,
               status=excluded.status,
               updated_at=excluded.updated_at",
            params![
                draft.id,
                draft.topic,
                draft.title,
                draft.slug,
                draft.markdown,
                citations_json,
                source_paths_json,
                draft.source_hash,
                draft.model_id,
                wiki_status_name(&draft.status),
                draft.created_at,
                draft.updated_at,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn wiki_draft(&self, draft_id: &str) -> Result<Option<WikiDraft>, String> {
        let conn = open_wiki_connection(self.database_path())?;
        conn.execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())?;
        conn.query_row(
            "SELECT id, topic, title, slug, markdown, citations_json, source_paths_json,
                    source_hash, model_id, status, created_at, updated_at
             FROM wiki_drafts WHERE id=?1",
            params![draft_id],
            map_wiki_row,
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    pub fn list_wiki_drafts(
        &self,
        status: Option<WikiDraftStatus>,
        limit: usize,
    ) -> Result<Vec<WikiDraft>, String> {
        let conn = open_wiki_connection(self.database_path())?;
        conn.execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())?;
        let capped_limit = limit.clamp(1, 1_000) as i64;
        if let Some(status) = status {
            let mut statement = conn
                .prepare(
                    "SELECT id, topic, title, slug, markdown, citations_json, source_paths_json,
                            source_hash, model_id, status, created_at, updated_at
                     FROM wiki_drafts WHERE status=?1
                     ORDER BY updated_at DESC, id LIMIT ?2",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(
                    params![wiki_status_name(&status), capped_limit],
                    map_wiki_row,
                )
                .map_err(|error| error.to_string())?;
            return rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string());
        }

        let mut statement = conn
            .prepare(
                "SELECT id, topic, title, slug, markdown, citations_json, source_paths_json,
                        source_hash, model_id, status, created_at, updated_at
                 FROM wiki_drafts ORDER BY updated_at DESC, id LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![capped_limit], map_wiki_row)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn set_wiki_draft_status(
        &self,
        draft_id: &str,
        status: WikiDraftStatus,
    ) -> Result<WikiDraft, String> {
        let current = self
            .wiki_draft(draft_id)?
            .ok_or_else(|| format!("Unknown wiki draft: {draft_id}"))?;
        if !valid_status_transition(&current.status, &status) {
            return Err(format!(
                "Invalid wiki status transition: {} -> {}.",
                wiki_status_name(&current.status),
                wiki_status_name(&status)
            ));
        }
        let conn = open_wiki_connection(self.database_path())?;
        conn.execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())?;
        conn.execute(
            "UPDATE wiki_drafts SET status=?2, updated_at=unixepoch() WHERE id=?1",
            params![draft_id, wiki_status_name(&status)],
        )
        .map_err(|error| error.to_string())?;
        self.wiki_draft(draft_id)?
            .ok_or_else(|| format!("Wiki draft disappeared after update: {draft_id}"))
    }

    pub fn accept_wiki_draft(
        &self,
        vault_root: &Path,
        draft_id: &str,
    ) -> Result<(WikiDraft, PathBuf), String> {
        let mut draft = self
            .wiki_draft(draft_id)?
            .ok_or_else(|| format!("Unknown wiki draft: {draft_id}"))?;
        if !matches!(
            draft.status,
            WikiDraftStatus::Proposed | WikiDraftStatus::Outdated
        ) {
            return Err("Only proposed or outdated wiki drafts can be accepted.".into());
        }
        validate_draft(&draft)?;
        let root = fs::canonicalize(vault_root).map_err(|error| error.to_string())?;
        let wiki_dir = root.join(".elephantnote").join("wiki");
        fs::create_dir_all(&wiki_dir).map_err(|error| error.to_string())?;
        let canonical_dir = fs::canonicalize(&wiki_dir).map_err(|error| error.to_string())?;
        if !canonical_dir.starts_with(&root) {
            return Err("Wiki directory escapes the vault.".into());
        }
        let target = canonical_dir.join(format!("{}.md", draft.slug));
        atomic_write(&canonical_dir, &target, draft.markdown.as_bytes())?;
        draft.status = WikiDraftStatus::Accepted;
        draft.updated_at = unix_timestamp();
        self.save_wiki_draft(&draft)?;
        Ok((draft, target))
    }

    pub fn mark_wikis_outdated_for_source(&self, document_path: &str) -> Result<usize, String> {
        let conn = open_wiki_connection(self.database_path())?;
        conn.execute_batch(WIKI_SCHEMA)
            .map_err(|error| error.to_string())?;
        let mut statement = conn
            .prepare(
                "SELECT id, source_paths_json FROM wiki_drafts
                 WHERE status='accepted' OR status='proposed'",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?;
        let mut ids = Vec::new();
        for row in rows {
            let (id, json) = row.map_err(|error| error.to_string())?;
            let paths: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
            if paths.iter().any(|path| path == document_path) {
                ids.push(id);
            }
        }
        drop(statement);
        let mut changed = 0usize;
        for id in ids {
            changed += conn
                .execute(
                    "UPDATE wiki_drafts SET status='outdated', updated_at=unixepoch() WHERE id=?1",
                    params![id],
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(changed)
    }
}

fn open_wiki_connection(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|error| error.to_string())
}

fn map_wiki_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WikiDraft> {
    let citations_json: String = row.get(5)?;
    let source_paths_json: String = row.get(6)?;
    let citations = serde_json::from_str(&citations_json).map_err(json_read_error)?;
    let source_paths = serde_json::from_str(&source_paths_json).map_err(json_read_error)?;
    Ok(WikiDraft {
        id: row.get(0)?,
        topic: row.get(1)?,
        title: row.get(2)?,
        slug: row.get(3)?,
        markdown: row.get(4)?,
        citations,
        source_paths,
        source_hash: row.get(7)?,
        model_id: row.get(8)?,
        status: parse_wiki_status(&row.get::<_, String>(9)?),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn validate_draft(draft: &WikiDraft) -> Result<(), String> {
    if draft.id.trim().is_empty() {
        return Err("Wiki draft ID cannot be empty.".into());
    }
    if draft.title.trim().is_empty() || draft.topic.trim().is_empty() {
        return Err("Wiki title and topic are required.".into());
    }
    if draft.slug.trim().is_empty()
        || draft.slug.contains('/')
        || draft.slug.contains('\\')
        || draft.slug.contains("..")
    {
        return Err("Wiki slug is invalid.".into());
    }
    if draft.markdown.trim().is_empty() {
        return Err("Wiki Markdown cannot be empty.".into());
    }
    if draft.source_paths.is_empty() || draft.citations.is_empty() {
        return Err("A generated wiki requires cited sources.".into());
    }
    if draft.model_id.trim().is_empty() {
        return Err("Wiki draft must record the model ID.".into());
    }
    Ok(())
}

fn valid_status_transition(current: &WikiDraftStatus, next: &WikiDraftStatus) -> bool {
    matches!(
        (current, next),
        (WikiDraftStatus::Proposed, WikiDraftStatus::Accepted)
            | (WikiDraftStatus::Proposed, WikiDraftStatus::Rejected)
            | (WikiDraftStatus::Accepted, WikiDraftStatus::Outdated)
            | (WikiDraftStatus::Outdated, WikiDraftStatus::Accepted)
            | (WikiDraftStatus::Outdated, WikiDraftStatus::Rejected)
    )
}

fn wiki_status_name(status: &WikiDraftStatus) -> &'static str {
    match status {
        WikiDraftStatus::Proposed => "proposed",
        WikiDraftStatus::Accepted => "accepted",
        WikiDraftStatus::Rejected => "rejected",
        WikiDraftStatus::Outdated => "outdated",
    }
}

fn parse_wiki_status(value: &str) -> WikiDraftStatus {
    match value {
        "accepted" => WikiDraftStatus::Accepted,
        "rejected" => WikiDraftStatus::Rejected,
        "outdated" => WikiDraftStatus::Outdated,
        _ => WikiDraftStatus::Proposed,
    }
}

fn json_read_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn atomic_write(root: &Path, target: &Path, bytes: &[u8]) -> Result<(), String> {
    if !target.starts_with(root) {
        return Err("Refusing to write wiki outside the generated wiki directory.".into());
    }
    let temporary = target.with_extension(format!("md.{}.tmp", std::process::id()));
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(())
}

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;
    use crate::wiki_core::{
        collect_wiki_sources, render_wiki, wiki_draft_from_rendered, WikiClaim, WikiSynthesis,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "elephant-wiki-{name}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn draft() -> WikiDraft {
        let document = analyze_markdown("Notes/Iroh.md", "# Iroh\nIroh is P2P.", 1);
        let sources = collect_wiki_sources(&[document]);
        let synthesis = WikiSynthesis {
            title: "Iroh".into(),
            summary: vec![WikiClaim {
                text: "Iroh is peer-to-peer technology.".into(),
                citation_chunk_ids: vec![sources[0].chunk_id.clone()],
            }],
            sections: Vec::new(),
            related_wikis: Vec::new(),
        };
        let rendered = render_wiki(&synthesis, "Iroh", &sources).unwrap();
        wiki_draft_from_rendered("Iroh", rendered, "model", 10)
    }

    #[test]
    fn persists_and_accepts_generated_wiki_in_hidden_folder() {
        let root = temp_vault("accept");
        let store = KnowledgeStore::open(&root).unwrap();
        let draft = draft();
        store.save_wiki_draft(&draft).unwrap();
        let (accepted, path) = store.accept_wiki_draft(&root, &draft.id).unwrap();
        assert_eq!(accepted.status, WikiDraftStatus::Accepted);
        assert!(path.starts_with(root.join(".elephantnote/wiki")));
        assert!(path.is_file());
        assert!(fs::read_to_string(path)
            .unwrap()
            .contains("generated: true"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn source_changes_mark_existing_wiki_outdated() {
        let root = temp_vault("outdated");
        let store = KnowledgeStore::open(&root).unwrap();
        let draft = draft();
        store.save_wiki_draft(&draft).unwrap();
        store.accept_wiki_draft(&root, &draft.id).unwrap();
        assert_eq!(
            store
                .mark_wikis_outdated_for_source("Notes/Iroh.md")
                .unwrap(),
            1
        );
        assert_eq!(
            store.wiki_draft(&draft.id).unwrap().unwrap().status,
            WikiDraftStatus::Outdated
        );
        fs::remove_dir_all(root).ok();
    }
}
