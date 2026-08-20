//! Production bridge for the native Explorer surface.
//!
//! Exact and Smart keyword search use Elephant's production SQLite FTS index.
//! Search rebuilds and Knowledge Graph rebuild/projection run through the
//! shared blocking-work bridge so Freya input/rendering never waits on a full
//! filesystem or SQLite operation.

use crate::{
    background, search_backend,
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
            let Some(vault) = shell.read().vault.clone() else {
                explorer
                    .write()
                    .apply_search_error(request_id, SurfaceError::NoActiveVault);
                return;
            };
            let expected_vault = vault.descriptor().path.clone();
            let worker_vault = vault.clone();
            let request_for_worker = request.clone();
            let query_for_apply = request.query.clone();
            let mut explorer_for_apply = explorer;
            let shell_for_apply = shell;
            eprintln!(
                "[freya][search] action=dispatch request_id={} query_len={} mode={}",
                request_id,
                request.query.chars().count(),
                request.mode.as_str()
            );
            background::run(
                "search-query",
                move || search(Some(&worker_vault), &request_for_worker, request_id),
                move |outcome| {
                    if !same_active_vault(shell_for_apply, &expected_vault) {
                        eprintln!(
                            "[freya][search] action=stale-vault-ignore request_id={request_id}"
                        );
                        return;
                    }
                    match outcome {
                        Ok(Ok(execution)) => {
                            explorer_for_apply
                                .write()
                                .apply_search_status(execution.status);
                            let accepted = explorer_for_apply.write().apply_search_results(
                                request_id,
                                &query_for_apply,
                                execution.results,
                                execution.concepts,
                            );
                            if !accepted {
                                eprintln!(
                                    "[freya][search] action=stale-ignore request_id={} query_len={}",
                                    request_id,
                                    query_for_apply.chars().count()
                                );
                            }
                        }
                        Ok(Err(error)) => {
                            let _ = explorer_for_apply.write().apply_search_error_for_query(
                                request_id,
                                &query_for_apply,
                                error,
                            );
                        }
                        Err(error) => {
                            let _ = explorer_for_apply.write().apply_search_error_for_query(
                                request_id,
                                &query_for_apply,
                                SurfaceError::Unknown(format!(
                                    "Search background worker failed: {error}"
                                )),
                            );
                        }
                    }
                },
            );
        }
        SearchCommand::OpenResult {
            relative_path,
            title,
        }
        | SearchCommand::OpenConceptEvidence {
            relative_path,
            title,
        } => open_search_note(shell, explorer, relative_path, title),
        SearchCommand::RefreshStatus => {
            spawn_search_status(shell, explorer, false);
        }
        SearchCommand::InspectIndex => {
            spawn_search_status(shell, explorer, true);
        }
        SearchCommand::RebuildIndex => {
            spawn_search_rebuild(shell, explorer);
        }
        SearchCommand::ClearIndex => {
            apply_search_backend_action(shell, explorer, "clear", search_backend::clear);
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

fn spawn_search_status(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    inspect: bool,
) {
    let request_id = explorer.read().search.request_id;
    let Some(vault) = shell.read().vault.clone() else {
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::NoActiveVault);
        return;
    };
    let expected_vault = vault.descriptor().path.clone();
    let worker_vault = vault.clone();
    let apply_vault = vault;
    let mut explorer_for_apply = explorer;
    let shell_for_apply = shell;
    background::run(
        "search-status",
        move || search_backend::status(&worker_vault),
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            match outcome {
                Ok(Ok(status)) => {
                    let message = if inspect {
                        format!(
                            "FTS index: {} ({} documents)",
                            status.index_path.display(),
                            status.indexed_documents
                        )
                    } else {
                        format!("{} indexed documents", status.indexed_documents)
                    };
                    explorer_for_apply
                        .write()
                        .apply_search_status(to_search_status(&apply_vault, status, message));
                }
                Ok(Err(error)) => {
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!("Search status failed: {error}")),
                    );
                }
                Err(error) => {
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!(
                            "Search status background worker failed: {error}"
                        )),
                    );
                }
            }
        },
    );
}

fn spawn_search_rebuild(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
) {
    let request_id = explorer.read().search.request_id;
    let Some(vault) = shell.read().vault.clone() else {
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::NoActiveVault);
        return;
    };
    let expected_vault = vault.descriptor().path.clone();
    let worker_vault = vault.clone();
    let apply_vault = vault;
    let mut explorer_for_apply = explorer;
    let shell_for_apply = shell;
    background::run(
        "search-rebuild",
        move || {
            let refresh = search_backend::rebuild(&worker_vault)?;
            let status = search_backend::status(&worker_vault)?;
            Ok::<_, String>((refresh, status))
        },
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            match outcome {
                Ok(Ok((refresh, status))) => {
                    explorer_for_apply.write().apply_search_status(to_search_status(
                        &apply_vault,
                        status,
                        format!(
                            "Index rebuilt: {} scanned, {} updated, {} unchanged, {} removed, {} failed",
                            refresh.scanned,
                            refresh.indexed,
                            refresh.unchanged,
                            refresh.removed,
                            refresh.failed.len()
                        ),
                    ));
                }
                Ok(Err(error)) => {
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!("Search rebuild failed: {error}")),
                    );
                }
                Err(error) => {
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!(
                            "Search rebuild background worker failed: {error}"
                        )),
                    );
                }
            }
        },
    );
}

fn apply_search_backend_action<Operation>(
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    action: &'static str,
    operation: Operation,
) where
    Operation:
        FnOnce(&VaultAdapter) -> Result<search_backend::BackendStatus, String> + Send + 'static,
{
    let request_id = explorer.read().search.request_id;
    let Some(vault) = shell.read().vault.clone() else {
        explorer
            .write()
            .apply_search_error(request_id, SurfaceError::NoActiveVault);
        return;
    };
    let expected_vault = vault.descriptor().path.clone();
    let worker_vault = vault.clone();
    let apply_vault = vault;
    let mut explorer_for_apply = explorer;
    let shell_for_apply = shell;
    background::run(
        "search-control",
        move || operation(&worker_vault),
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            match outcome {
                Ok(Ok(status)) => {
                    let message = format!(
                        "Search {action} complete: {} indexed documents",
                        status.indexed_documents
                    );
                    explorer_for_apply
                        .write()
                        .apply_search_status(to_search_status(&apply_vault, status, message));
                    eprintln!("[freya][search] action={action}:complete");
                }
                Ok(Err(error)) => {
                    eprintln!("[freya][search] action={action}:failure error={error}");
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!("Search {action} failed: {error}")),
                    );
                }
                Err(error) => {
                    let _ = explorer_for_apply.write().apply_search_error(
                        request_id,
                        SurfaceError::Unknown(format!(
                            "Search {action} background worker failed: {error}"
                        )),
                    );
                }
            }
        },
    );
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
    shell: State<ShellState>,
    mut explorer: State<explorer::ExplorerState>,
    command: crate::search_graph_contract::GraphCommand,
) {
    use crate::search_graph_contract::GraphCommand;

    match command {
        GraphCommand::Refresh | GraphCommand::RebuildIndex => {
            let action = if matches!(command, GraphCommand::Refresh) {
                "refresh"
            } else {
                "rebuild"
            };
            let Some(vault) = shell.read().vault.clone() else {
                explorer
                    .write()
                    .apply_graph_error(SurfaceError::NoActiveVault);
                return;
            };
            let expected_vault = vault.descriptor().path.clone();
            let worker_vault = vault.clone();
            let apply_vault = vault;
            let shell_for_apply = shell;
            let mut explorer_for_apply = explorer;
            eprintln!("[freya][graph] action={action}:dispatch");
            background::run(
                "graph-refresh",
                move || graph_runtime::refresh(Some(&worker_vault), false),
                move |outcome| {
                    if !same_active_vault(shell_for_apply, &expected_vault) {
                        eprintln!("[freya][graph] action={action}:stale-vault-ignore");
                        return;
                    }
                    match outcome {
                        Ok(Ok(execution)) => {
                            let mut snapshot = execution.snapshot;
                            if shell_for_apply.read().view
                                == crate::navigation_contract::WorkspaceView::Canvas
                            {
                                match crate::canvas_runtime::CanvasRuntime::open(
                                    apply_vault.root(),
                                    snapshot.clone(),
                                ) {
                                    Ok(runtime) => {
                                        snapshot = runtime.snapshot_for_render();
                                        shell_for_apply.write().canvas = Some(runtime);
                                    }
                                    Err(error) => {
                                        let message = format!("Canvas failed: {error}");
                                        eprintln!(
                                            "[freya][canvas] action=load-failure error={message}"
                                        );
                                        explorer_for_apply
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
                            explorer_for_apply.write().apply_graph_snapshot(snapshot);
                        }
                        Ok(Err(error)) => {
                            let message = format!("Graph failed: {}", error.message());
                            eprintln!(
                                "[freya][graph] action={action}:failure error={message}"
                            );
                            explorer_for_apply
                                .write()
                                .apply_graph_error(SurfaceError::Unknown(message));
                        }
                        Err(error) => {
                            explorer_for_apply.write().apply_graph_error(
                                SurfaceError::Unknown(format!(
                                    "Graph background worker failed: {error}"
                                )),
                            );
                        }
                    }
                },
            );
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
        other => eprintln!(
            "[freya][graph] action=unsupported command={other:?} error=graph command not converted"
        ),
    }
}

fn same_active_vault(shell: State<ShellState>, expected_path: &str) -> bool {
    shell
        .read()
        .vault
        .as_ref()
        .is_some_and(|vault| vault.descriptor().path == expected_path)
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
            tags: entry
                .as_ref()
                .map(|entry| entry.tags.clone())
                .unwrap_or(hit.tags),
            score,
            // Smart currently uses the production keyword index only. Keep the
            // observable match type honest until a real embedding backend is
            // connected instead of labelling lexical results as hybrid.
            match_type: SearchMatchType::Keyword,
            snippets: vec![SearchSnippet {
                text: excerpt,
                score,
            }],
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
    let message = if request.mode == SearchMode::Smart {
        "Smart search currently uses the persistent keyword FTS index; semantic ranking is not connected yet"
    } else {
        "Search executed with the persistent FTS index"
    };
    Ok(SearchExecution {
        concepts: concept_candidates(&results),
        results,
        status: to_search_status(vault, backend_status, message.to_owned()),
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
