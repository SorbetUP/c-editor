use elephantnote_knowledge_core_native::{
    KnowledgeGraph, KnowledgeNodeRef, KnowledgeRelation, KnowledgeStore, RelationStatus,
};
use std::path::Path;
use tauri::AppHandle;

fn active_store(app: &AppHandle) -> Result<KnowledgeStore, String> {
    let root = crate::vault::config::get_active_vault(app)?.path;
    KnowledgeStore::open(Path::new(&root))
}

#[tauri::command]
pub fn tauri_knowledge_graph(
    app: AppHandle,
    include_suggestions: Option<bool>,
) -> Result<KnowledgeGraph, String> {
    active_store(&app)?.graph_projection(include_suggestions.unwrap_or(false))
}

#[tauri::command]
pub fn tauri_knowledge_relation_save(
    app: AppHandle,
    relation: KnowledgeRelation,
) -> Result<(), String> {
    active_store(&app)?.save_relation(&relation)
}

#[tauri::command]
pub fn tauri_knowledge_relation_status_set(
    app: AppHandle,
    relation_id: String,
    status: RelationStatus,
) -> Result<bool, String> {
    active_store(&app)?.set_relation_status(&relation_id, status)
}

#[tauri::command]
pub fn tauri_knowledge_relation_get(
    app: AppHandle,
    relation_id: String,
) -> Result<Option<KnowledgeRelation>, String> {
    active_store(&app)?.relation_by_id(&relation_id)
}

#[tauri::command]
pub fn tauri_knowledge_relations_for_node(
    app: AppHandle,
    node: KnowledgeNodeRef,
    include_rejected: Option<bool>,
) -> Result<Vec<KnowledgeRelation>, String> {
    active_store(&app)?.relations_for_node(&node, include_rejected.unwrap_or(false))
}

#[tauri::command]
pub fn tauri_knowledge_relations_list(
    app: AppHandle,
    status: Option<RelationStatus>,
    limit: Option<usize>,
) -> Result<Vec<KnowledgeRelation>, String> {
    active_store(&app)?.list_relations(status, limit.unwrap_or(1_000))
}
