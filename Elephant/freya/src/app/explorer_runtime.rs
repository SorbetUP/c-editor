//! Production bridge for the native Explorer surface.
//!
//! Exact and Smart keyword search use Elephant's production SQLite FTS index.
//! Full Markdown scans only happen when an index is explicitly rebuilt or when
//! a pre-index vault is queried for the first time.

use crate::{
    search_backend,
    search_graph_contract::{
        ConceptCandidate, EvidenceChunk, SearchMatchType, SearchMode, SearchRequest, SearchResult,
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
    use crate::search_graph_contract::SearchCommand;

    match command {
        SearchCommand::Open => shell.write().search_open = true,
        SearchCommand::Close => {
            shell.write().search_open = false;
            eprintln!("[freya][search] action=close");
        }
        SearchCommand::SetQuery(_) | SearchCommand::SetMode(_) => {}
        SearchCommand::ClearQuery => {
            explorer.write().finish_clear_query();
            eprintln!("[freya][search] action=clear-query");
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
        SearchCommand::RefreshStatus | SearchCommand::InspectIndex => {
            let request_id = explorer.read().search.request_id;
            let vault = shell.read().vault.clone();
            let Some(vault) = vault.as_ref() else {
                explorer
                    .write()
                    .apply_search_error(request_id, SurfaceError::NoActiveVault);
                return;
            };
            match search_backend::status(vault) {
                Ok(status) => {
                    let message = if matches!(command, SearchCommand::InspectIndex) {
                        format!(
                            "FTS index: {} ({} documents)",
                            status.index_path.display(),
                            status.indexed_documents
                        )
                    } else {
                        format!("{} indexed documents", status.indexed_documents)
                    };
                    explorer
                        .write()
                        .apply_search_status(to_search_status(vault, status, message));
                }
                Err(error) => explorer.write().apply_search_error(
                    request_id,
                    SurfaceError::Unknown(format!("Search status failed: {error}")),
                ),
            }
        }
        SearchCommand::RebuildIndex => {
            let request_id = explorer.read().search.request_id;
            let vault = shell.read().vault.clone();
            let Some(vault) = vault.as_ref() else {
                explorer
                    .write()
                    .apply_search_error(request_id, SurfaceError::NoActiveVault);
                return;
            };
            match search_backend::rebuild(vault) {
                Ok(refresh) => match search_backend::status(vault) {
                    Ok(status) => explorer.write().apply_search_status(to_search_status(
                        vault,
                        status,
                        format!(
                            "Index rebuilt: {} scanned, {} updated, {} unchanged, {} removed, {} failed",
                            refresh.scanned,
                            refresh.indexed,
                            refresh.unchanged,
                            refresh.removed,
                            refresh.failed.len()
                        ),
                    )),
                    Err(error) => explorer.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!("Search status failed after rebuild: {error}")),
                    ),
                },
                Err(error) => explorer.write().apply_search_error(
                    request_id,
                    SurfaceError::Unknown(format!("Search rebuild failed: {error}")),
                ),
            }
        }
        SearchCommand::ClearIndex => {
            apply_search_backend_action(shell, explorer, "clear", |vault| {
                search_backend::clear(vault)
            });
        }
        SearchCommand::Enable => {
            apply_search_backend_action(shell, explorer, "enable", |vault| {
                search_backend::set_enabled(vault, true)
            });
        }
        SearchCommand::Disable => {
            apply_search_backend_action(shell, explorer, "disable", |vault| {
                search_backend::set_enabled(vault, false)
            });
        }
    }
}

fn apply_search_backend_action(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    action: &str,
    operation: impl FnOnce(&VaultAdapter) -> Result<search_backend::BackendStatus, String>,
) {
    let request_id = explorer.read().search.request_id;
    let vault = shell.read().vault.clone();
    let Some(vault) = vault.as_ref() else {
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::NoActiveVault);
        return;
    };
    match operation(vault) {
        Ok(status) => {
            let message = format!(
                "Search {action} complete: {} indexed documents",
                status.indexed_documents
            );
            explorer
                .write()
                .apply_search_status(to_search_status(vault, status, message));
            eprintln!("[freya][search] action={action}:complete");
        }
        Err(error) => {
            eprintln!("[freya][search] action={action}:failure error={error}");
            explorer.write().apply_search_error(
                request_id,
                SurfaceError::Unknown(format!("Search {action} failed: {error}")),
            );
        }
    }
}

fn to_search_status(
    vault: &VaultAdapter,
    status: search_backend::BackendStatus,
    message: String,
) -> SearchStatus {
    SearchStatus {
        status: if status.enabled {
            SearchStatusKind::Ready
        } else {
            SearchStatusKind::Disabled
        },
        vault_path: vault.descriptor().path.clone(),
        indexed_documents: status.indexed_documents,
        total_documents: status.indexed_documents,
        message,
        error: String::new(),
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
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::NoActiveVault);
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
            explorer
                .write()
                .apply_search_error(request_id, SurfaceError::Unknown(message));
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
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::Unknown(error));
    }
}

fn dispatch_graph_command(
    mut shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    command: crate::search_graph_contract::GraphCommand,
) {
    use crate::search_graph_contract::GraphCommand;

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
                    let mut snapshot = execution.snapshot;
                    if shell.read().view == crate::navigation_contract::WorkspaceView::Canvas {
                        let Some(vault) = vault.as_ref() else {
                            explorer
                                .write()
                                .apply_graph_error(SurfaceError::NoActiveVault);
                            return;
                        };
                        match crate::canvas_runtime::CanvasRuntime::open(
                            vault.root(),
                            snapshot.clone(),
                        ) {
                            Ok(runtime) => {
                                snapshot = runtime.snapshot_for_render();
                                shell.write().canvas = Some(runtime);
                            }
                            Err(error) => {
                                let message = format!("Canvas failed: {error}");
                                eprintln!("[freya][canvas] action=load-failure error={message}");
                                explorer
                                    .write()
                                    .apply_graph_error(SurfaceError::Unknown(message));
                                return;
                            }
                        }
                    }
                    let nodes = snapshot.nodes.len();
                    let edges = snapshot.edges.len();
                    eprintln!(
                        "[freya][graph] action={action}:complete nodes={nodes} edges={edges}"
                    );
                    explorer.write().apply_graph_snapshot(snapshot);
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
    match request.mode {
        SearchMode::Semantic => {
            let message =
                "Semantic search unavailable in Freya: no embedding index is connected to VaultAdapter."
                    .to_string();
            eprintln!(
                "[freya][search] action=failure request_id={} mode=semantic reason=no_embedding_index",
                request_id
            );
            Err(SurfaceError::Unknown(message))
        }
        SearchMode::Exact | SearchMode::Smart => fts_search(vault, request, request_id),
    }
}

fn fts_search(
    vault: &VaultAdapter,
    request: &SearchRequest,
    request_id: u64,
) -> Result<SearchExecution, SurfaceError> {
    let hits = search_backend::query(vault, &request.query, request.limit).map_err(|error| {
        eprintln!(
            "[freya][search] action=query-failure request_id={} error={error}",
            request_id
        );
        SurfaceError::Unknown(error)
    })?;
    let mut results = Vec::with_capacity(hits.len());
    for hit in hits {
        let entry = vault.find_entry(&hit.path).ok();
        let score = (1.0 / (1.0 + hit.score.abs())) as f32;
        let excerpt = hit.excerpt.clone();
        results.push(SearchResult {
            id: format!("fts:{}", hit.path),
            uri: format!("elephantnote://vault/{}", hit.path),
            title: hit.title,
            relative_path: hit.path,
            excerpt: excerpt.clone(),
            tags: entry.as_ref().map(|entry| entry.tags.clone()).unwrap_or(hit.tags),
            score,
            match_type: if request.mode == SearchMode::Smart {
                SearchMatchType::Hybrid
            } else {
                SearchMatchType::Keyword
            },
            snippets: vec![SearchSnippet { text: excerpt, score }],
            updated_at: entry.map(|entry| entry.updated_at).unwrap_or_default(),
        });
    }
    let backend_status = search_backend::status(vault).map_err(SurfaceError::Unknown)?;
    eprintln!(
        "[freya][search] action=complete request_id={} mode={} results={} indexed={}",
        request_id,
        request.mode.as_str(),
        results.len(),
        backend_status.indexed_documents
    );
    Ok(SearchExecution {
        concepts: concept_candidates(&results),
        results,
        status: to_search_status(
            vault,
            backend_status,
            "Search executed with the persistent FTS index".to_owned(),
        ),
    })
}

fn concept_candidates(results: &[SearchResult]) -> Vec<ConceptCandidate> {
    let mut candidates = Vec::new();
    for result in results {
        let path = result.relative_path.replace('\\', "/");
        let parts = path
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
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
        candidate.score += result.score.max(0.05);
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
