use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSection {
    pub id: String,
    pub heading: String,
    pub level: u8,
    pub ordinal: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeChunk {
    pub id: String,
    pub section_id: String,
    pub ordinal: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub token_estimate: usize,
    pub content_hash: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplicitLink {
    pub target: String,
    pub label: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentSnapshot {
    pub relative_path: String,
    pub title: String,
    pub content_hash: String,
    pub modified_at: i64,
    pub sections: Vec<KnowledgeSection>,
    pub chunks: Vec<KnowledgeChunk>,
    pub explicit_links: Vec<ExplicitLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct KnowledgeStatus {
    pub documents: i64,
    pub sections: i64,
    pub chunks: i64,
    pub explicit_links: i64,
    pub pending_actions: i64,
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeSearchHit {
    pub relative_path: String,
    pub title: String,
    pub heading: String,
    pub chunk_id: String,
    pub excerpt: String,
    pub score: f64,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RebuildReport {
    pub scanned: usize,
    pub indexed: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub failed: Vec<RebuildFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RebuildFailure {
    pub relative_path: String,
    pub error: String,
}
