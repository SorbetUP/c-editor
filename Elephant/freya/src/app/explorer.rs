//! Native Freya surface for the real Search and Graph workspaces.
//!
//! This module is deliberately an adapter, not a second data source.  Search
//! commands map to the channels used by `searchStore.js` and `searchIpc.js`;
//! graph commands map to the Atomic graph service used by
//! `AtomicGraphView.vue`.  No sample result, node, edge, or fallback graph is
//! created here.  Until a host injects a typed payload, the surface renders
//! its explicit idle state.

use freya::prelude::*;

use crate::{
    search_graph_contract::{
        ConceptCandidate, GraphCommand, GraphEdge, GraphFilterState, GraphLoadState, GraphNode,
        GraphNodeKind, GraphSnapshot, SearchCommand, SearchMatchType, SearchMode, SearchRequest,
        SearchResult, SearchStatus, SearchStatusKind, SurfaceError, GRAPH_RENDER_MAX_EDGES,
        SEARCH_QUERY_LIMIT_DEFAULT,
    },
    theme,
};

/// The two workspaces converted from the Vue surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExplorerSurface {
    Search,
    Graph,
}

/// Renderer-facing state.  Every visible state is explicit; the default is
/// idle with no fabricated payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExplorerPhase {
    Idle,
    Loading,
    Results,
    Empty,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExplorerSearchState {
    pub phase: ExplorerPhase,
    pub query: String,
    pub mode: SearchMode,
    pub results: Vec<SearchResult>,
    pub concepts: Vec<ConceptCandidate>,
    pub status: Option<SearchStatus>,
    pub selected_index: Option<usize>,
    pub request_id: u64,
    pub error: Option<SurfaceError>,
}

impl ExplorerSearchState {
    fn new() -> Self {
        Self {
            phase: ExplorerPhase::Idle,
            query: String::new(),
            mode: SearchMode::Exact,
            results: Vec::new(),
            concepts: Vec::new(),
            status: None,
            selected_index: None,
            request_id: 0,
            error: None,
        }
    }

    fn clear(&mut self) {
        self.phase = ExplorerPhase::Idle;
        self.query.clear();
        self.results.clear();
        self.concepts.clear();
        self.selected_index = None;
        self.error = None;
    }

    fn refresh_phase(&mut self) {
        self.phase = if self.error.is_some() {
            ExplorerPhase::Error
        } else if !self.results.is_empty() || !self.concepts.is_empty() {
            ExplorerPhase::Results
        } else if self.query.trim().is_empty() {
            ExplorerPhase::Idle
        } else {
            ExplorerPhase::Empty
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExplorerGraphState {
    pub phase: ExplorerPhase,
    pub load: GraphLoadState,
    pub snapshot: Option<GraphSnapshot>,
    pub filters: GraphFilterState,
    pub selected_node_id: Option<String>,
    pub error: Option<SurfaceError>,
}

impl ExplorerGraphState {
    fn new() -> Self {
        Self {
            phase: ExplorerPhase::Idle,
            load: GraphLoadState::Idle,
            snapshot: None,
            filters: GraphFilterState::default(),
            selected_node_id: None,
            error: None,
        }
    }

    fn refresh_phase(&mut self) {
        self.phase = if self.error.is_some() {
            ExplorerPhase::Error
        } else if matches!(
            self.load,
            GraphLoadState::InspectingIndex | GraphLoadState::RebuildingIndex
        ) {
            ExplorerPhase::Loading
        } else if self.visible_node_count() > 0 {
            ExplorerPhase::Results
        } else if self.snapshot.is_some() {
            ExplorerPhase::Empty
        } else {
            ExplorerPhase::Idle
        };
    }

    fn visible_node_count(&self) -> usize {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return 0;
        };
        snapshot
            .nodes
            .iter()
            .filter(|node| self.filters.matches_current_vue(node))
            .count()
    }
}

/// Commands emitted by this module.  The Freya host drains them and calls the
/// existing Search/Atomic bridge; this file never invents a successful reply.
#[derive(Clone, Debug, PartialEq)]
pub enum ExplorerAction {
    Search(SearchCommand),
    Graph(GraphCommand),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExplorerState {
    pub surface: ExplorerSurface,
    pub search: ExplorerSearchState,
    pub graph: ExplorerGraphState,
    pending: Vec<ExplorerAction>,
}

impl ExplorerState {
    pub fn new() -> Self {
        Self {
            surface: ExplorerSurface::Search,
            search: ExplorerSearchState::new(),
            graph: ExplorerGraphState::new(),
            pending: Vec::new(),
        }
    }

    /// Queues the exact `searchStore.search` request after trimming and
    /// clamping through the shared contract.
    pub fn submit_search(&mut self, query: impl Into<String>) {
        let query = query.into();
        self.search.query = query.clone();
        self.search.error = None;
        self.search.selected_index = None;
        let Some(request) = SearchRequest::new(query, self.search.mode, SEARCH_QUERY_LIMIT_DEFAULT)
        else {
            self.search.clear();
            self.pending
                .push(ExplorerAction::Search(SearchCommand::ClearQuery));
            return;
        };

        self.search.request_id = self.search.request_id.saturating_add(1);
        self.search.phase = ExplorerPhase::Loading;
        self.pending
            .push(ExplorerAction::Search(SearchCommand::SetQuery(
                request.query.clone(),
            )));
        self.pending
            .push(ExplorerAction::Search(SearchCommand::Submit(request)));
    }

    /// Enter in the real SearchModal searches first and opens the selected
    /// result once results already exist for the same query.
    pub fn submit_or_open_search(&mut self, query: impl Into<String>) {
        let query = query.into();
        if self.search.query.trim() == query.trim() && self.search.selected_result().is_some() {
            self.open_selected_search_result();
        } else {
            self.submit_search(query);
        }
    }

    pub fn cycle_search_mode(&mut self) {
        self.search.mode = match self.search.mode {
            SearchMode::Smart => SearchMode::Exact,
            SearchMode::Exact => SearchMode::Semantic,
            SearchMode::Semantic => SearchMode::Smart,
        };
        self.pending
            .push(ExplorerAction::Search(SearchCommand::SetMode(
                self.search.mode,
            )));
        if !self.search.query.trim().is_empty() {
            self.submit_search(self.search.query.clone());
        }
    }

    pub fn apply_search_status(&mut self, status: SearchStatus) {
        if status.status.is_loading() && self.search.query.trim().is_empty() {
            self.search.phase = ExplorerPhase::Loading;
        }
        if matches!(status.status, SearchStatusKind::Error) && !status.error.is_empty() {
            self.search.error = Some(SurfaceError::Unknown(status.error.clone()));
        }
        self.search.status = Some(status);
        self.search.refresh_phase();
    }

    /// Applies only the response belonging to the active request/query.  This
    /// is the same stale-response guard as the Vue store.
    pub fn apply_search_results(
        &mut self,
        request_id: u64,
        query: &str,
        results: Vec<SearchResult>,
        concepts: Vec<ConceptCandidate>,
    ) -> bool {
        if request_id != self.search.request_id || query.trim() != self.search.query.trim() {
            return false;
        }
        self.search.results = results;
        self.search.concepts = concepts;
        self.search.selected_index = None;
        self.search.error = None;
        self.search.refresh_phase();
        true
    }

    pub fn apply_search_error(&mut self, request_id: u64, error: SurfaceError) -> bool {
        if request_id != self.search.request_id {
            return false;
        }
        self.search.results.clear();
        self.search.concepts.clear();
        self.search.selected_index = None;
        self.search.error = Some(error);
        self.search.phase = ExplorerPhase::Error;
        true
    }

    pub fn clear_search(&mut self) {
        self.search.clear();
        self.pending
            .push(ExplorerAction::Search(SearchCommand::ClearQuery));
    }

    pub fn close_search(&mut self) {
        self.search.clear();
        self.pending
            .push(ExplorerAction::Search(SearchCommand::Close));
    }

    pub fn select_search_result(&mut self, index: usize) {
        if index < self.search.results.len() {
            self.search.selected_index = Some(index);
        }
    }

    pub fn move_search_selection(&mut self, delta: isize) {
        let count = self.search.results.len();
        if count == 0 {
            self.search.selected_index = None;
            return;
        }
        let current = self.search.selected_index.unwrap_or(0) as isize;
        self.search.selected_index = Some((current + delta).rem_euclid(count as isize) as usize);
    }

    pub fn selected_search_result(&self) -> Option<&SearchResult> {
        self.search.selected_result()
    }

    fn open_selected_search_result(&mut self) {
        let Some(result) = self.search.selected_result() else {
            return;
        };
        self.pending
            .push(ExplorerAction::Search(SearchCommand::OpenResult {
                relative_path: result.relative_path.clone(),
                title: result.title.clone(),
            }));
    }

    pub fn open_search_result(&mut self, index: usize) {
        self.select_search_result(index);
        self.open_selected_search_result();
    }

    pub fn open_concept(&mut self, concept: &ConceptCandidate) {
        let Some(evidence) = concept
            .evidence_chunks
            .iter()
            .find(|chunk| !chunk.relative_path.is_empty() || !chunk.document_path.is_empty())
        else {
            return;
        };
        self.pending
            .push(ExplorerAction::Search(SearchCommand::OpenConceptEvidence {
                relative_path: if evidence.relative_path.is_empty() {
                    evidence.document_path.clone()
                } else {
                    evidence.relative_path.clone()
                },
                title: concept.title.clone(),
            }));
    }

    pub fn request_graph_refresh(&mut self) {
        self.surface = ExplorerSurface::Graph;
        self.graph.load = GraphLoadState::InspectingIndex;
        self.graph.error = None;
        self.graph.refresh_phase();
        self.pending
            .push(ExplorerAction::Graph(GraphCommand::Refresh));
    }

    pub fn apply_graph_snapshot(&mut self, snapshot: GraphSnapshot) {
        self.graph.snapshot = Some(snapshot);
        self.graph.load = if self.graph.visible_node_count() == 0 {
            GraphLoadState::Empty
        } else {
            GraphLoadState::Ready
        };
        self.graph.error = None;
        self.graph.selected_node_id = None;
        self.graph.refresh_phase();
    }

    pub fn apply_graph_error(&mut self, error: SurfaceError) {
        self.graph.error = Some(error);
        self.graph.load = GraphLoadState::Error;
        self.graph.phase = ExplorerPhase::Error;
    }

    pub fn set_graph_filter(&mut self, query: impl Into<String>) {
        self.graph.filters.query = query.into();
        self.graph.refresh_phase();
        self.pending
            .push(ExplorerAction::Graph(GraphCommand::SetFilterQuery(
                self.graph.filters.query.clone(),
            )));
    }

    pub fn reset_graph_filter(&mut self) {
        self.graph.filters.query.clear();
        self.graph.refresh_phase();
        self.pending
            .push(ExplorerAction::Graph(GraphCommand::SetFilterQuery(
                String::new(),
            )));
    }

    pub fn visible_graph_nodes(&self) -> Vec<&GraphNode> {
        self.graph
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .nodes
                    .iter()
                    .filter(|node| self.graph.filters.matches_current_vue(node))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn visible_graph_edges(&self) -> Vec<&GraphEdge> {
        let visible = self
            .visible_graph_nodes()
            .into_iter()
            .map(|node| node.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let Some(snapshot) = self.graph.snapshot.as_ref() else {
            return Vec::new();
        };
        let mut edges = snapshot
            .edges
            .iter()
            .filter(|edge| {
                visible.contains(edge.source.as_str()) && visible.contains(edge.target.as_str())
            })
            .collect::<Vec<_>>();
        if edges.len() > GRAPH_RENDER_MAX_EDGES {
            edges.sort_by(|left, right| {
                right
                    .weight
                    .partial_cmp(&left.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            edges.truncate(GRAPH_RENDER_MAX_EDGES);
        }
        edges
    }

    pub fn select_graph_node(&mut self, id: impl Into<String>) {
        let id = id.into();
        if self.visible_graph_nodes().iter().any(|node| node.id == id) {
            self.graph.selected_node_id = Some(id.clone());
            self.pending
                .push(ExplorerAction::Graph(GraphCommand::SelectNode(id)));
        }
    }

    pub fn open_selected_graph_node(&mut self) {
        if self.graph.selected_node_id.is_some() {
            self.pending
                .push(ExplorerAction::Graph(GraphCommand::OpenSelectedNode));
        }
    }

    pub fn take_pending(&mut self) -> Vec<ExplorerAction> {
        std::mem::take(&mut self.pending)
    }
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplorerSearchState {
    fn selected_result(&self) -> Option<&SearchResult> {
        self.selected_index
            .and_then(|index| self.results.get(index))
    }
}

/// Render the native surface.  Callers should drain `ExplorerState::take_pending`
/// and execute those commands through the existing host bridge.
pub fn explorer_view(
    state: State<ExplorerState>,
    query: State<String>,
    graph_query: State<String>,
) -> Element {
    let snapshot = state.read().clone();

    let search_input = {
        let query = query.clone();
        let mut state = state;
        Input::new(query)
            .width(Size::fill())
            .placeholder("Search notes, paths, tags, or ideas…")
            .on_submit(move |value: String| {
                state.write().submit_or_open_search(value);
            })
    };

    let graph_input = {
        let mut graph_query = graph_query.clone();
        let mut state = state;
        Input::new(graph_query)
            .width(Size::fill())
            .placeholder("Filtrer les notes…")
            .on_submit(move |value: String| {
                graph_query.set(value.clone());
                state.write().set_graph_filter(value);
            })
    };

    let content = match snapshot.surface {
        ExplorerSurface::Search => search_surface(state, &snapshot, search_input, query),
        ExplorerSurface::Graph => graph_surface(state, &snapshot, graph_input),
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .color(theme::color(theme::TEXT))
        .spacing(10.)
        .child(explorer_header(state, snapshot.surface))
        .child(content)
        .into_element()
}

fn explorer_header(mut state: State<ExplorerState>, surface: ExplorerSurface) -> Element {
    let search = rect()
        .height(Size::px(34.))
        .padding(Gaps::new(0., 12., 0., 12.))
        .center()
        .background(if surface == ExplorerSurface::Search {
            theme::color(theme::SOFT)
        } else {
            theme::color(theme::SURFACE)
        })
        .with_corner_radius(8.)
        .on_mouse_up(move |_| state.write().surface = ExplorerSurface::Search)
        .a11y_alt("Search workspace")
        .child(label().text("⌕  Search"));
    let graph = rect()
        .height(Size::px(34.))
        .padding(Gaps::new(0., 12., 0., 12.))
        .center()
        .background(if surface == ExplorerSurface::Graph {
            theme::color(theme::SOFT)
        } else {
            theme::color(theme::SURFACE)
        })
        .with_corner_radius(8.)
        .on_mouse_up(move |_| state.write().surface = ExplorerSurface::Graph)
        .a11y_alt("Graph workspace")
        .child(label().text("◌  Graph"));
    rect()
        .width(Size::fill())
        .height(Size::px(42.))
        .padding(Gaps::new(4., 8., 4., 8.))
        .horizontal()
        .spacing(8.)
        .child(search)
        .child(graph)
        .into_element()
}

fn search_surface(
    mut state: State<ExplorerState>,
    snapshot: &ExplorerState,
    input: Input,
    query: State<String>,
) -> Element {
    let mut key_state = state;
    let mut query_state = query;
    let search_input = rect().a11y_alt("Search input").child(input);
    let mode = rect()
        .height(Size::px(30.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| state.write().cycle_search_mode())
        .a11y_alt(format!("Search mode: {}", snapshot.search.mode.as_str()))
        .child(label().text(format!("Mode: {}", snapshot.search.mode.as_str())));
    let clear = rect()
        .height(Size::px(30.))
        .width(Size::px(62.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| state.write().clear_search())
        .a11y_alt("Clear search")
        .child(label().text("Clear"));
    let bar = rect()
        .width(Size::fill())
        .horizontal()
        .spacing(8.)
        .child(search_input)
        .child(mode)
        .child(clear);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(0., 12., 12., 12.))
        .spacing(8.)
        .on_global_key_down(move |event: Event<KeyboardEventData>| match event.key {
            Key::Named(NamedKey::Escape) => {
                let had_query = !query_state.read().trim().is_empty();
                query_state.set(String::new());
                if had_query {
                    key_state.write().clear_search();
                } else {
                    key_state.write().close_search();
                }
            }
            Key::Named(NamedKey::ArrowDown) => key_state.write().move_search_selection(1),
            Key::Named(NamedKey::ArrowUp) => key_state.write().move_search_selection(-1),
            _ => {}
        })
        .child(bar)
        .child(search_state_content(state, snapshot))
        .into_element()
}

fn search_state_content(state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let status = snapshot.search.status.as_ref().map(|status| {
        label().color(theme::color(theme::MUTED)).text(format!(
            "Index: {} · {}/{} documents",
            status.status.as_str(),
            status.indexed_documents,
            status.total_documents
        ))
    });
    let body = match snapshot.search.phase {
        ExplorerPhase::Idle => state_message("Search notes, paths, tags, or ideas…", false),
        ExplorerPhase::Loading => state_message("Searching locally…", true),
        ExplorerPhase::Empty => state_message("No matching notes found", false),
        ExplorerPhase::Error => state_message(
            snapshot
                .search
                .error
                .as_ref()
                .map(surface_error_text)
                .unwrap_or("Search failed."),
            false,
        ),
        ExplorerPhase::Results => search_results(state, snapshot),
    };
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(12.)
        .padding(Gaps::new_all(12.))
        .spacing(8.)
        .maybe_child(status)
        .child(body)
        .into_element()
}

fn search_results(state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let mut sections = Vec::new();
    if !snapshot.search.concepts.is_empty() {
        sections.push(
            rect()
                .spacing(4.)
                .child(section_title("Wikis & concepts"))
                .children(snapshot.search.concepts.iter().map(|concept| {
                    let concept = concept.clone();
                    let concept_for_action = concept.clone();
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new_all(10.))
                        .background(theme::color(theme::SOFT))
                        .with_corner_radius(10.)
                        .on_mouse_up({
                            let mut state = state;
                            move |_| state.write().open_concept(&concept_for_action)
                        })
                        .a11y_alt(format!("Open concept {}", concept.title))
                        .child(
                            label()
                                .font_weight(FontWeight::BOLD)
                                .text(concept.title.clone()),
                        )
                        .child(
                            label()
                                .font_size(12.)
                                .color(theme::color(theme::MUTED))
                                .text(format!(
                                    "{}% · {} evidence chunks",
                                    (concept.score * 100.) as i32,
                                    concept.evidence_chunks.len()
                                )),
                        )
                        .into_element()
                }))
                .into_element(),
        );
    }
    if !snapshot.search.results.is_empty() {
        sections.push(
            rect()
                .spacing(4.)
                .child(section_title("Notes & passages"))
                .children(
                    snapshot
                        .search
                        .results
                        .iter()
                        .enumerate()
                        .map(|(index, result)| {
                            search_result_row(
                                state,
                                index,
                                result,
                                snapshot.search.selected_index == Some(index),
                            )
                        }),
                )
                .into_element(),
        );
    }
    rect().spacing(10.).children(sections).into_element()
}

fn search_result_row(
    mut state: State<ExplorerState>,
    index: usize,
    result: &SearchResult,
    selected: bool,
) -> Element {
    let result = result.clone();
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(10.))
        .background(if selected {
            theme::color(theme::SOFT)
        } else {
            theme::color(theme::BG)
        })
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().open_search_result(index))
        .a11y_alt(format!("Open note {}", result.title))
        .child(label().font_weight(FontWeight::BOLD).text(format!(
            "{}  [{}]",
            result.title,
            match_label(result.match_type)
        )))
        .child(
            label()
                .font_size(12.)
                .color(theme::color(theme::MUTED))
                .text(result.relative_path.clone()),
        )
        .child(
            label().font_size(13.).text(
                result
                    .snippets
                    .first()
                    .map(|s| s.text.clone())
                    .unwrap_or_else(|| result.excerpt.clone()),
            ),
        )
        .into_element()
}

fn graph_surface(
    mut state: State<ExplorerState>,
    snapshot: &ExplorerState,
    input: Input,
) -> Element {
    let refresh = rect()
        .height(Size::px(30.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| state.write().request_graph_refresh())
        .a11y_alt("Refresh graph")
        .child(label().text("Refresh"));
    let reset = rect()
        .height(Size::px(30.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| state.write().reset_graph_filter())
        .a11y_alt("Reset graph filter")
        .child(label().text("Reset"));
    let toolbar = rect()
        .width(Size::fill())
        .horizontal()
        .spacing(8.)
        .child(rect().a11y_alt("Graph filter input").child(input))
        .child(refresh)
        .child(reset);
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(0., 12., 12., 12.))
        .spacing(8.)
        .child(toolbar)
        .child(graph_state_content(state, snapshot))
        .into_element()
}

fn graph_state_content(state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let body = match snapshot.graph.phase {
        ExplorerPhase::Idle => state_message("Graph not loaded", false),
        ExplorerPhase::Loading => state_message("Building the semantic graph…", true),
        ExplorerPhase::Empty => state_message("No notes match the current graph filter", false),
        ExplorerPhase::Error => state_message(
            snapshot
                .graph
                .error
                .as_ref()
                .map(surface_error_text)
                .unwrap_or("Graph failed."),
            false,
        ),
        ExplorerPhase::Results => graph_results(state, snapshot),
    };
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(12.)
        .padding(Gaps::new_all(12.))
        .child(body)
        .into_element()
}

fn graph_results(mut state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let nodes = snapshot.visible_graph_nodes();
    let edges = snapshot.visible_graph_edges();
    let stats = snapshot.graph.snapshot.as_ref().map(|_| {
        label().color(theme::color(theme::MUTED)).text(format!(
            "{} nœuds · {} liens visibles",
            nodes.len(),
            edges.len()
        ))
    });
    let node_list = rect()
        .width(Size::fill())
        .spacing(4.)
        .child(section_title("Nodes"))
        .children(nodes.iter().map(|node| {
            let selected = snapshot.graph.selected_node_id.as_deref() == Some(node.id.as_str());
            graph_node_row(state, node, selected)
        }));
    let edge_list = rect()
        .width(Size::fill())
        .spacing(4.)
        .child(section_title("Edges"))
        .children(edges.iter().map(|edge| graph_edge_row(edge)));
    let selected = snapshot
        .graph
        .selected_node_id
        .as_deref()
        .and_then(|id| nodes.iter().find(|node| node.id == id));
    let selected_card = selected.map(|node| {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(10.))
            .background(theme::color(theme::SOFT))
            .with_corner_radius(10.)
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text(format!("Selected: {}", node.title)),
            )
            .child(
                label()
                    .color(theme::color(theme::MUTED))
                    .text(node.summary.clone()),
            )
            .child(
                rect()
                    .height(Size::px(30.))
                    .padding(Gaps::new(0., 10., 0., 10.))
                    .center()
                    .background(theme::color(theme::SURFACE))
                    .with_corner_radius(7.)
                    .on_mouse_up(move |_| state.write().open_selected_graph_node())
                    .a11y_alt("Open selected note")
                    .child(label().text("Open note")),
            )
            .into_element()
    });
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .spacing(10.)
        .maybe_child(stats)
        .maybe_child(selected_card)
        .child(node_list)
        .child(edge_list)
        .into_element()
}

fn graph_node_row(mut state: State<ExplorerState>, node: &GraphNode, selected: bool) -> Element {
    let id = node.id.clone();
    let tags = if node.tags.is_empty() {
        String::new()
    } else {
        format!(" · #{}", node.tags.join(" #"))
    };
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(9.))
        .background(if selected {
            theme::color(theme::SOFT)
        } else {
            theme::color(theme::BG)
        })
        .with_corner_radius(9.)
        .on_mouse_up(move |_| state.write().select_graph_node(id.clone()))
        .a11y_alt(format!("Select graph node {}", node.title))
        .child(label().font_weight(FontWeight::BOLD).text(format!(
            "{}  [{}]",
            node.title,
            node_kind_label(node.kind)
        )))
        .child(
            label()
                .font_size(12.)
                .color(theme::color(theme::MUTED))
                .text(format!("{}{}", node.relative_path, tags)),
        )
        .into_element()
}

fn graph_edge_row(edge: &GraphEdge) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(6., 9., 6., 9.))
        .background(theme::color(theme::BG))
        .with_corner_radius(7.)
        .child(
            label()
                .font_size(12.)
                .text(format!("{}  →  {}", edge.source, edge.target)),
        )
        .child(
            label()
                .font_size(11.)
                .color(theme::color(theme::MUTED))
                .text(format!(
                    "{} · {:.2} · {}",
                    edge_type_label(edge.edge_type),
                    edge.weight,
                    edge.reason
                )),
        )
        .into_element()
}

fn state_message(message: &str, loading: bool) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .center()
        .spacing(8.)
        .a11y_alt(message)
        .child(label().font_size(18.).text(if loading { "…" } else { "⌕" }))
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(message.to_string()),
        )
        .into_element()
}

fn section_title(title: &'static str) -> Element {
    label()
        .font_size(11.)
        .font_weight(FontWeight::BOLD)
        .color(theme::color(theme::MUTED))
        .text(title)
        .into_element()
}

fn match_label(kind: SearchMatchType) -> &'static str {
    match kind {
        SearchMatchType::Hybrid => "Semantic + keyword",
        SearchMatchType::Semantic => "Semantic",
        SearchMatchType::Keyword => "Keyword",
        SearchMatchType::Concept => "Concept",
        SearchMatchType::Unknown => "Local match",
    }
}

fn node_kind_label(kind: GraphNodeKind) -> &'static str {
    match kind {
        GraphNodeKind::Note => "note",
        GraphNodeKind::Folder => "folder",
        GraphNodeKind::Other => "other",
    }
}

fn edge_type_label(kind: crate::search_graph_contract::GraphEdgeType) -> &'static str {
    match kind {
        crate::search_graph_contract::GraphEdgeType::Semantic => "semantic",
        crate::search_graph_contract::GraphEdgeType::ExplicitLink => "explicit-link",
        crate::search_graph_contract::GraphEdgeType::Folder => "folder",
        crate::search_graph_contract::GraphEdgeType::Tag => "tag",
        crate::search_graph_contract::GraphEdgeType::Lexical => "lexical",
        crate::search_graph_contract::GraphEdgeType::Related => "related",
        crate::search_graph_contract::GraphEdgeType::Other => "other",
    }
}

fn surface_error_text(error: &SurfaceError) -> &str {
    match error {
        SurfaceError::NoActiveVault => "No active vault.",
        SurfaceError::InvalidQuery => "Invalid search query.",
        SurfaceError::InvalidSearchMode => "Invalid search mode.",
        SurfaceError::SearchTimedOut => "Local search timed out.",
        SurfaceError::ConceptTimedOut => "Concept routing timed out.",
        SurfaceError::SearchUnavailable => "Search service unavailable.",
        SurfaceError::IndexUnavailable => "Search index unavailable.",
        SurfaceError::GraphUnavailable => "Graph service unavailable.",
        SurfaceError::WikiProposalNotFound => "Wiki proposal not found.",
        SurfaceError::WikiWriteFailed => "Wiki write failed.",
        SurfaceError::Unknown(message) => message.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_idle_without_fabricated_search_or_graph_data() {
        let state = ExplorerState::new();
        assert_eq!(state.search.phase, ExplorerPhase::Idle);
        assert!(state.search.results.is_empty());
        assert!(state.graph.snapshot.is_none());
        assert!(state.visible_graph_nodes().is_empty());
        assert!(state.visible_graph_edges().is_empty());
        assert!(state.pending.is_empty());
    }

    #[test]
    fn empty_search_is_idle_and_emits_clear_only() {
        let mut state = ExplorerState::new();
        state.submit_search("   ");
        assert_eq!(state.search.phase, ExplorerPhase::Idle);
        assert_eq!(
            state.take_pending(),
            vec![ExplorerAction::Search(SearchCommand::ClearQuery)]
        );
    }

    #[test]
    fn search_and_graph_inputs_emit_typed_host_commands() {
        let mut state = ExplorerState::new();
        state.submit_search("  rust  ");
        assert_eq!(
            state.take_pending(),
            vec![
                ExplorerAction::Search(SearchCommand::SetQuery("rust".to_string())),
                ExplorerAction::Search(SearchCommand::Submit(SearchRequest {
                    query: "rust".to_string(),
                    mode: SearchMode::Exact,
                    limit: SEARCH_QUERY_LIMIT_DEFAULT,
                })),
            ]
        );

        state.set_graph_filter("native");
        assert_eq!(
            state.take_pending(),
            vec![ExplorerAction::Graph(GraphCommand::SetFilterQuery(
                "native".to_string()
            ))]
        );
    }

    #[test]
    fn stale_search_payload_is_rejected() {
        let mut state = ExplorerState::new();
        state.submit_search("rust");
        let request_id = state.search.request_id;
        let result = SearchResult {
            id: "note:rust".to_string(),
            uri: String::new(),
            title: "Rust".to_string(),
            relative_path: "Rust.md".to_string(),
            excerpt: "A real result".to_string(),
            tags: Vec::new(),
            score: 1.0,
            match_type: SearchMatchType::Keyword,
            snippets: Vec::new(),
            updated_at: String::new(),
        };
        assert!(!state.apply_search_results(
            request_id - 1,
            "rust",
            vec![result.clone()],
            Vec::new()
        ));
        assert!(state.search.results.is_empty());
        assert!(state.apply_search_results(request_id, "rust", vec![result], Vec::new()));
        assert_eq!(state.search.phase, ExplorerPhase::Results);
    }
}
