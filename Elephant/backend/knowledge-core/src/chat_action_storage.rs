use crate::chat_actions::{ChatActionProposal, ChatActionStatus};
use crate::storage::KnowledgeStore;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const CHAT_ACTION_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS chat_action_proposals (
  id TEXT PRIMARY KEY,
  action_json TEXT NOT NULL,
  rationale TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL,
  preview_json TEXT NOT NULL,
  result_json TEXT,
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS chat_action_proposals_status_idx
  ON chat_action_proposals(status, updated_at DESC);
"#;

impl KnowledgeStore {
    pub fn initialize_chat_actions(&self) -> Result<(), String> {
        open_chat_action_connection(self.database_path())?
            .execute_batch(CHAT_ACTION_SCHEMA)
            .map_err(|error| error.to_string())
    }

    pub fn save_chat_action_proposal(&self, proposal: &ChatActionProposal) -> Result<(), String> {
        let conn = open_chat_action_connection(self.database_path())?;
        conn.execute_batch(CHAT_ACTION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let action_json =
            serde_json::to_string(&proposal.action).map_err(|error| error.to_string())?;
        let preview_json =
            serde_json::to_string(&proposal.preview).map_err(|error| error.to_string())?;
        let result_json = proposal
            .result
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        conn.execute(
            "INSERT INTO chat_action_proposals(
               id, action_json, rationale, status, preview_json, result_json, error,
               created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               action_json=excluded.action_json,
               rationale=excluded.rationale,
               status=excluded.status,
               preview_json=excluded.preview_json,
               result_json=excluded.result_json,
               error=excluded.error,
               updated_at=excluded.updated_at",
            params![
                proposal.id,
                action_json,
                proposal.rationale,
                chat_action_status_name(&proposal.status),
                preview_json,
                result_json,
                proposal.error,
                proposal.created_at,
                proposal.updated_at,
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn chat_action_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<Option<ChatActionProposal>, String> {
        let conn = open_chat_action_connection(self.database_path())?;
        conn.execute_batch(CHAT_ACTION_SCHEMA)
            .map_err(|error| error.to_string())?;
        conn.query_row(
            "SELECT id, action_json, rationale, status, preview_json, result_json,
                    error, created_at, updated_at
             FROM chat_action_proposals WHERE id=?1",
            params![proposal_id],
            map_proposal_row,
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    pub fn list_chat_action_proposals(
        &self,
        status: Option<ChatActionStatus>,
        limit: usize,
    ) -> Result<Vec<ChatActionProposal>, String> {
        let conn = open_chat_action_connection(self.database_path())?;
        conn.execute_batch(CHAT_ACTION_SCHEMA)
            .map_err(|error| error.to_string())?;
        let capped_limit = limit.clamp(1, 1_000) as i64;
        if let Some(status) = status {
            let mut statement = conn
                .prepare(
                    "SELECT id, action_json, rationale, status, preview_json, result_json,
                            error, created_at, updated_at
                     FROM chat_action_proposals WHERE status=?1
                     ORDER BY updated_at DESC, id LIMIT ?2",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(
                    params![chat_action_status_name(&status), capped_limit],
                    map_proposal_row,
                )
                .map_err(|error| error.to_string())?;
            return rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string());
        }

        let mut statement = conn
            .prepare(
                "SELECT id, action_json, rationale, status, preview_json, result_json,
                        error, created_at, updated_at
                 FROM chat_action_proposals ORDER BY updated_at DESC, id LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![capped_limit], map_proposal_row)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn transition_chat_action(
        &self,
        proposal_id: &str,
        next_status: ChatActionStatus,
    ) -> Result<ChatActionProposal, String> {
        let mut proposal = self
            .chat_action_proposal(proposal_id)?
            .ok_or_else(|| format!("Unknown chat action proposal: {proposal_id}"))?;
        if !valid_transition(&proposal.status, &next_status) {
            return Err(format!(
                "Invalid chat action transition: {} -> {}.",
                chat_action_status_name(&proposal.status),
                chat_action_status_name(&next_status)
            ));
        }
        proposal.status = next_status;
        proposal.updated_at = unix_timestamp();
        self.save_chat_action_proposal(&proposal)?;
        Ok(proposal)
    }
}

fn open_chat_action_connection(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|error| error.to_string())
}

fn map_proposal_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatActionProposal> {
    let action_json: String = row.get(1)?;
    let preview_json: String = row.get(4)?;
    let result_json: Option<String> = row.get(5)?;
    let action = serde_json::from_str(&action_json).map_err(json_read_error)?;
    let preview = serde_json::from_str(&preview_json).map_err(json_read_error)?;
    let result = result_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(json_read_error)?;
    Ok(ChatActionProposal {
        id: row.get(0)?,
        action,
        rationale: row.get(2)?,
        status: parse_chat_action_status(&row.get::<_, String>(3)?),
        preview,
        result,
        error: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn json_read_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn valid_transition(current: &ChatActionStatus, next: &ChatActionStatus) -> bool {
    matches!(
        (current, next),
        (ChatActionStatus::Proposed, ChatActionStatus::Approved)
            | (ChatActionStatus::Proposed, ChatActionStatus::Rejected)
            | (ChatActionStatus::Approved, ChatActionStatus::Executed)
            | (ChatActionStatus::Approved, ChatActionStatus::Failed)
            | (ChatActionStatus::Approved, ChatActionStatus::Rejected)
    )
}

pub(crate) fn chat_action_status_name(status: &ChatActionStatus) -> &'static str {
    match status {
        ChatActionStatus::Proposed => "proposed",
        ChatActionStatus::Approved => "approved",
        ChatActionStatus::Executed => "executed",
        ChatActionStatus::Rejected => "rejected",
        ChatActionStatus::Failed => "failed",
    }
}

fn parse_chat_action_status(value: &str) -> ChatActionStatus {
    match value {
        "approved" => ChatActionStatus::Approved,
        "executed" => ChatActionStatus::Executed,
        "rejected" => ChatActionStatus::Rejected,
        "failed" => ChatActionStatus::Failed,
        _ => ChatActionStatus::Proposed,
    }
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
    use crate::actions::ChatKnowledgeAction;
    use crate::chat_actions::prepare_chat_action;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "elephant-chat-storage-{name}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn persists_proposal_and_enforces_transitions() {
        let root = temp_vault("transition");
        let store = KnowledgeStore::open(&root).unwrap();
        let proposal = prepare_chat_action(
            &root,
            ChatKnowledgeAction::SearchNotes {
                query: "iroh".into(),
                limit: 10,
            },
            "Search notes",
        )
        .unwrap();
        store.save_chat_action_proposal(&proposal).unwrap();
        assert_eq!(
            store.chat_action_proposal(&proposal.id).unwrap(),
            Some(proposal.clone())
        );
        let approved = store
            .transition_chat_action(&proposal.id, ChatActionStatus::Approved)
            .unwrap();
        assert_eq!(approved.status, ChatActionStatus::Approved);
        assert!(store
            .transition_chat_action(&proposal.id, ChatActionStatus::Proposed)
            .is_err());
        fs::remove_dir_all(root).ok();
    }
}
