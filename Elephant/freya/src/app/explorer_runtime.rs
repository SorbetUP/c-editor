//! Production bridge for the native Explorer surface.
//!
//! Search is executed through the production Tauri FTS index. This module only
//! maps `FtsIndex::Hit` into the renderer contract; it owns no second index or
//! result store.

use crate::{
    search_graph_contract::{
        ConceptCandidate, EvidenceChunk, SearchMatchType, SearchRequest, SearchResult,
        SearchSnippet, SearchStatus, SearchStatusKind, SurfaceError,
    },
    vault_adapter::VaultAdapter,
};

use super::{explorer, ShellState};
use freya::prelude::State;

#[path = "graph_runtime.rs"]
mod graph_runtime;

pub(super) fn drain_explorer_actions(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
) {
    let actions = explorer.write().take_pending();
    for action in actions {
        match action {
            explorer::ExplorerAction::Search(command) => {
                dispatch_search_command(shell, explorer, command)
            }
            explorer::ExplorerAction::Graph(command) => {
                dispatch_graph_command(shell, explorer, command)
            }
        }
    }
}

fn dispatch_search_command(
    mut shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    command: crate::search_graph_contract::SearchCommand,
) {
    use crate::search_graph_contract::{SearchCommand, SurfaceError};

    match command {
        SearchCommand::Open => shell.write().search_open = true,
        SearchCommand::Close => {
            shell.write().search_open = false;
            explorer.write().finish_close();
            eprintln!("[freya][search] action=close");
        }
        SearchCommand::SetQuery(_) | SearchCommand::SetMode(_) => {}
        SearchCommand::ClearQuery => {
            explorer.write().finish_clear_query();
            eprintln!("[freya][search] action=clear");
        }
        SearchCommand::Submit(request) => {
            let request_id = explorer.read().search.request_id;
            let vault = shell.read().vault.clone();
            match search(vault.as_ref(), &request, request_id) {
                Ok(execution) => {
                    explorer.write().apply_search_status(execution.status);
                    let accepted = explorer.write().apply_search_results(
                        request_id,
                        &request.query,
                        execution.results,
                        execution.concepts,
                    );
                    if !accepted {
                        eprintln!(
                            "[freya][search] action=stale-ignore request_id={} query_len={}",
                            request_id,
                            request.query.chars().count()
                        );
                    }
                }
                Err(error) => {
                    let accepted = explorer.write().apply_search_error_for_query(
                        request_id,
                        &request.query,
                        error,
                    );
                    if !accepted {
                        eprintln!(
                            "[freya][search] action=stale-error-ignore request_id={} query_len={}",
                            request_id,
                            request.query.chars().count()
                        );
                    }
                }
            }
        }
        SearchCommand::OpenResult {
            relative_path,
            title,
        }
        | SearchCommand::OpenConceptEvidence {
            relative_path,
            title,
        } => open_search_note(shell, explorer, relative_path, title),
        SearchCommand::RefreshStatus
        | SearchCommand::InspectIndex
        | SearchCommand::RebuildIndex
        | SearchCommand::ClearIndex
        | SearchCommand::Enable
        | SearchCommand::Disable => {
            let request_id = explorer.read().search.request_id;
            eprintln!(
                "[freya][search] action=unsupported request_id={} error=search host command unavailable",
                request_id
            );
            explorer
                .write()
                .apply_search_error(request_id, SurfaceError::SearchUnavailable);
        }
    }
}

fn open_search_note(
    mut shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    relative_path: String,
    title: String,
) {
    let request_id = explorer.read().search.request_id;
    eprintln!(
        "[freya][search] action=open_result:start request_id={} path={} title_len={}",
        request_id,
        relative_path,
        title.chars().count()
    );
    let vault = shell.read().vault.clone();
    let Some(vault) = vault else {
        explorer.write().apply_search_error(
            request_id,
            crate::search_graph_contract::SurfaceError::NoActiveVault,
        );
        return;
    };
    let entry = match vault.find_entry(&relative_path) {
        Ok(entry) => entry,
        Err(error) => {
            let message = format!("Search failed for {relative_path}: {error}");
            eprintln!(
                "[freya][search] action=open_result:failure request_id={} path={} error={error}",
                request_id, relative_path
            );
            explorer.write().apply_search_error(
                request_id,
                crate::search_graph_contract::SurfaceError::Unknown(message),
            );
            return;
        }
    };
    let mut shell_state = shell.write();
    shell_state.view = crate::navigation_contract::WorkspaceView::Notes;
    shell_state.open_note(&entry);
    if shell_state.editor.is_some() {
        shell_state.search_open = false;
        drop(shell_state);
        explorer.write().finish_close();
        eprintln!(
            "[freya][search] action=open_result:complete request_id={} path={}",
            request_id, relative_path
        );
    } else {
        let error = shell_state
            .error
            .clone()
            .unwrap_or_else(|| "Unable to open search result.".to_string());
        drop(shell_state);
        eprintln!(
            "[freya][search] action=open_result:failure request_id={} path={} error={error}",
            request_id, relative_path
        );
        explorer.write().apply_search_error(
            request_id,
            crate::search_graph_contract::SurfaceError::Unknown(error),
        );
    }
}

fn dispatch_graph_command(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    command: crate::search_graph_contract::GraphCommand,
) {
    use crate::search_graph_contract::{GraphCommand, SurfaceError};

    match command {
        GraphCommand::Refresh | GraphCommand::RebuildIndex => {
            let action = match command {
                GraphCommand::Refresh => "refresh",
                GraphCommand::RebuildIndex => "rebuild",
                _ => unreachable!("graph action already matched"),
            };
            let vault = shell.read().vault.clone();
            match graph_runtime::refresh(vault.as_ref(), false) {
                Ok(execution) => {
                    let nodes = execution.snapshot.nodes.len();
                    let edges = execution.snapshot.edges.len();
                    eprintln!(
                        "[freya][graph] action={action}:complete nodes={nodes} edges={edges}"
                    );
                    explorer.write().apply_graph_snapshot(execution.snapshot);
                }
                Err(error) => {
                    let message = format!("Graph failed: {}", error.message());
                    eprintln!("[freya][graph] action={action}:failure error={message}");
                    explorer
                        .write()
                        .apply_graph_error(SurfaceError::Unknown(message));
                }
            }
        }
        GraphCommand::SetFilterQuery(query) => eprintln!(
            "[freya][graph] action=filter:complete query_len={}",
            query.chars().count()
        ),
        GraphCommand::SelectNode(id) => {
            eprintln!("[freya][graph] action=select:complete id={id}");
        }
        GraphCommand::ResetView => {
            eprintln!("[freya][graph] action=reset-view:complete");
        }
        GraphCommand::OpenSelectedNode => {
            let selection = {
                let state = explorer.read();
                state
                    .graph
                    .selected_node_id
                    .as_ref()
                    .and_then(|selected_id| {
                        state
                            .graph
                            .snapshot
                            .as_ref()?
                            .nodes
                            .iter()
                            .find(|node| &node.id == selected_id)
                            .map(|node| (node.relative_path.clone(), node.title.clone()))
                    })
            };
            if let Some((relative_path, title)) = selection {
                open_search_note(shell, explorer, relative_path, title);
            } else {
                eprintln!("[freya][graph] action=open-selected:failure reason=no_selection");
                explorer.write().apply_graph_error(SurfaceError::Unknown(
                    "Graph failed: no selected node.".to_string(),
                ));
            }
        }
        _ => eprintln!("[freya][graph] action=unsupported error=graph command not converted"),
    }
}

pub(super) struct SearchExecution {
    pub results: Vec<SearchResult>,
    pub concepts: Vec<ConceptCandidate>,
    pub status: SearchStatus,
}

pub(super) fn search(
    vault: Option<&VaultAdapter>,
    request: &SearchRequest,
    request_id: u64,
) -> Result<SearchExecution, SurfaceError> {
    let Some(vault) = vault else {
        eprintln!(
            "[freya][search] action=failure request_id={} reason=no_active_vault",
            request_id
        );
        return Err(SurfaceError::NoActiveVault);
    };

    eprintln!(
        "[freya][search] action=start request_id={} mode={} query_len={} limit={} vault={}",
        request_id,
        request.mode.as_str(),
        request.query.chars().count(),
        request.limit,
        vault.descriptor().path
    );
    let refresh = vault.rebuild_search_index().map_err(|error| {
        eprintln!(
            "[freya][search] action=index:failure request_id={} error={error}",
            request_id
        );
        SurfaceError::Unknown(format!("Search index failed: {error}"))
    })?;
    eprintln!(
        "[freya][search] action=index:status request_id={} status={} scanned={} indexed={} unchanged={} removed={} failed={}",
        request_id,
        refresh.status,
        refresh.scanned,
        refresh.indexed,
        refresh.unchanged,
        refresh.removed,
        refresh.failed.len()
    );
    if let Some(failure) = refresh.failed.first() {
        eprintln!(
            "[freya][search] action=index:failure request_id={} path={} error={}",
            request_id, failure.path, failure.error
        );
        return Err(SurfaceError::Unknown(format!(
            "Search index could not read {}: {}",
            failure.path, failure.error
        )));
    }
    let hits = vault
        .search_index(&request.query, request.limit)
        .map_err(|error| {
            eprintln!(
                "[freya][search] action=failure request_id={} error={error}",
                request_id
            );
            SurfaceError::Unknown(format!("Search failed: {error}"))
        })?;
    let results = hits
        .into_iter()
        .map(|hit| SearchResult {
            id: format!("note:{}", hit.path),
            uri: hit.path.clone(),
            title: hit.title,
            relative_path: hit.path,
            excerpt: hit.excerpt.clone(),
            tags: hit.tags,
            score: hit.score as f32,
            match_type: SearchMatchType::Unknown,
            snippets: vec![SearchSnippet {
                text: hit.excerpt,
                score: hit.score as f32,
            }],
            updated_at: String::new(),
        })
        .collect::<Vec<_>>();
    let result_count = results.len();
    let concepts = concept_candidates(&results);
    eprintln!(
        "[freya][search] action=complete request_id={} results={}",
        request_id, result_count
    );
    Ok(SearchExecution {
        results,
        concepts,
        status: SearchStatus {
            status: SearchStatusKind::Ready,
            vault_path: vault.descriptor().path.clone(),
            indexed_documents: refresh.scanned,
            total_documents: refresh.scanned,
            message: format!("Indexed {} documents", refresh.scanned),
            error: String::new(),
        },
    })
}

fn concept_candidates(results: &[SearchResult]) -> Vec<ConceptCandidate> {
    let mut candidates = Vec::new();
    for result in results {
        let path = result.relative_path.replace('\\', "/");
        let parts = path.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
        let id = if parts.len() > 1 {
            parts[0].to_owned()
        } else {
            parts
                .last()
                .copied()
                .unwrap_or(result.title.as_str())
                .trim_end_matches(".md")
                .to_owned()
        };
        let index = candidates
            .iter()
            .position(|candidate: &ConceptCandidate| candidate.id == id)
            .unwrap_or_else(|| {
                candidates.push(ConceptCandidate {
                    id: id.clone(),
                    title: id,
                    aliases: Vec::new(),
                    score: 0.,
                    confidence: 0.,
                    match_type: SearchMatchType::Concept,
                    evidence_chunks: Vec::new(),
                });
                candidates.len() - 1
            });
        let candidate = &mut candidates[index];
        candidate.score += result.score.max(1.0);
        candidate.confidence = candidate.score.min(1.);
        if candidate.evidence_chunks.len() < 4 {
            candidate.evidence_chunks.push(EvidenceChunk {
                id: format!("{}:0", result.relative_path),
                document_path: result.relative_path.clone(),
                relative_path: result.relative_path.clone(),
                chunk_index: 0,
                heading_path: Vec::new(),
                score: result.score,
                preview: result.excerpt.clone(),
            });
        }
    }
    for candidate in &mut candidates {
        candidate.score = candidate.score.min(1.);
        candidate.confidence = candidate.score;
    }
    candidates
}
