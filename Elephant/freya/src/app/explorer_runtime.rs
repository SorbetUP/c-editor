//! Production bridge for the native Explorer surface.
//!
//! Search is executed through the production Tauri FTS index. This module only
//! maps `FtsIndex::Hit` into the renderer contract; it owns no second index or
//! result store.

use crate::{
    search_graph_contract::{
        ConceptCandidate, EvidenceChunk, SearchMatchType, SearchMode, SearchRequest, SearchResult,
        SearchSnippet, SearchStatus, SearchStatusKind, SurfaceError,
    },
    vault_adapter::{PageRequest, VaultAdapter, VaultEntry, MAX_PAGE_SIZE},
};

use super::{explorer, ShellState};
use freya::prelude::State;
use std::{collections::VecDeque, fs};

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
    mut shell: State<ShellState>,
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
        SearchMode::Exact | SearchMode::Smart => {
            let execution = exact_search(vault, request, request_id)?;
            eprintln!(
                "[freya][search] action=complete request_id={} mode={} results={}",
                request_id,
                request.mode.as_str(),
                execution.results.len()
            );
            Ok(execution)
        }
    }
}

fn exact_search(
    vault: &VaultAdapter,
    request: &SearchRequest,
    request_id: u64,
) -> Result<SearchExecution, SurfaceError> {
    let entries = visible_markdown_entries(vault, request_id)?;
    let query = request.query.to_lowercase();
    let mut results = Vec::new();

    for entry in &entries {
        let markdown = fs::read_to_string(&entry.full_path).map_err(|error| {
            eprintln!(
                "[freya][search] action=read:failure request_id={} path={} error={error}",
                request_id, entry.path
            );
            SurfaceError::Unknown(format!(
                "Search document could not be read {}: {error}",
                entry.path
            ))
        })?;
        let haystack = format!("{}\n{}", entry.path, markdown).to_lowercase();
        let Some(index) = haystack.find(&query) else {
            continue;
        };
        let excerpt = exact_excerpt(&markdown, &query, &entry.path, &entry.excerpt);
        let score = if index == 0 { 1.0 } else { 0.75 };
        results.push(SearchResult {
            id: format!("exact:{}", entry.path),
            uri: format!("elephantnote://vault/{}", entry.path),
            title: entry.title.clone(),
            relative_path: entry.path.clone(),
            excerpt: excerpt.clone(),
            tags: entry.tags.clone(),
            score,
            match_type: SearchMatchType::Keyword,
            snippets: vec![SearchSnippet {
                text: excerpt,
                score: 1.0,
            }],
            updated_at: entry.updated_at.clone(),
        });
        if results.len() >= request.limit {
            break;
        }
    }

    let scanned = entries.len();
    let mode_message = if request.mode == SearchMode::Smart {
        "Smart search used the exact fallback"
    } else {
        "Exact search scanned"
    };
    Ok(SearchExecution {
        concepts: concept_candidates(&results),
        results,
        status: SearchStatus {
            status: SearchStatusKind::Ready,
            vault_path: vault.descriptor().path.clone(),
            indexed_documents: scanned,
            total_documents: scanned,
            message: format!("{mode_message} {scanned} documents"),
            error: String::new(),
        },
    })
}

fn visible_markdown_entries(
    vault: &VaultAdapter,
    request_id: u64,
) -> Result<Vec<VaultEntry>, SurfaceError> {
    let mut directories = VecDeque::from([String::new()]);
    let mut entries = Vec::new();
    while let Some(directory) = directories.pop_front() {
        let mut offset = 0;
        loop {
            let page = vault
                .list(
                    PageRequest::new(directory.clone())
                        .with_window(offset, MAX_PAGE_SIZE)
                        .without_preview(),
                )
                .map_err(|error| {
                    eprintln!(
                        "[freya][search] action=list:failure request_id={} directory={} error={error}",
                        request_id, directory
                    );
                    SurfaceError::Unknown(format!("Search could not list {directory}: {error}"))
                })?;
            for entry in page.entries {
                if entry.is_directory {
                    directories.push_back(entry.path);
                } else if entry.path.to_ascii_lowercase().ends_with(".md") {
                    entries.push(entry);
                }
            }
            let Some(next_offset) = page.next_offset else {
                break;
            };
            offset = next_offset;
        }
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn exact_excerpt(markdown: &str, query: &str, path: &str, fallback: &str) -> String {
    let compact = markdown.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact_lower = compact.to_lowercase();
    if let Some(index) = compact_lower.find(query) {
        let start = index.saturating_sub(70);
        let end = (index + query.len() + 90).min(compact.len());
        if let Some(snippet) = compact.get(start..end) {
            return snippet.to_string();
        }
    }
    if path.to_lowercase().contains(query) {
        return path.to_string();
    }
    fallback.to_string()
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
