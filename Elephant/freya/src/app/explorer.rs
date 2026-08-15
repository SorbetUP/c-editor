//! Native Freya surface for the real Search and Graph workspaces.
//!
//! This module is deliberately an adapter, not a second data source.  Search
//! commands map to the channels used by `searchStore.js` and `searchIpc.js`;
//! graph commands map to the Atomic graph service used by
//! `AtomicGraphView.vue`.  No sample result, node, edge, or fallback graph is
//! created here.  Until a host injects a typed payload, the surface renders
//! its explicit idle state.

use freya::prelude::*;
use freya::sdk::use_timeout;
use std::time::Duration;

use crate::{
    app::navigation_icons::{svg_icon, Icon},
    search_graph_contract::{
        ConceptCandidate, GraphCommand, GraphEdge, GraphFilterState, GraphLoadState, GraphNode,
        GraphSnapshot, SearchCommand, SearchMatchType, SearchMode, SearchRequest, SearchResult,
        SearchStatus, SearchStatusKind, SurfaceError, GRAPH_RENDER_MAX_EDGES,
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

    fn clear_query(&mut self) {
        self.query.clear();
        self.selected_index = (!self.results.is_empty()).then_some(0);
        self.error = None;
        self.refresh_phase();
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
    /// Monotonic observable for the source graph's camera reset action. The
    /// native renderer keeps the action in the typed command queue instead of
    /// fabricating coordinates for nodes that the service did not provide.
    pub view_generation: u64,
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
            view_generation: 0,
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

/// Mirrors SearchModal.vue's live query watcher: text changes are debounced
/// before the existing production FTS submit path is queued. The query state
/// is the value owned by Freya's Input, so this remains live without requiring
/// an extra Enter action that the Vue surface does not require.
pub fn bind_live_search(mut state: State<ExplorerState>, query: State<String>) {
    let debounce = use_timeout(|| Duration::from_millis(220));

    let mut reset_timer = debounce;
    use_side_effect_with_deps(&*query.read(), move |_| {
        reset_timer.reset();
    });

    let mut fire_timer = debounce;
    let query_for_fire = query;
    use_side_effect(move || {
        if !fire_timer.elapsed() {
            return;
        }
        fire_timer.reset();

        let value = query_for_fire.peek().clone();
        state.write().set_search_query_live(value);
    });
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
        self.search.selected_index = (!self.search.results.is_empty()).then_some(0);
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

    /// Applies the query watcher behavior without treating Enter as the
    /// search trigger. The timer lives at the Freya binding boundary; this
    /// method keeps the production request/state transition in ExplorerState.
    pub fn set_search_query_live(&mut self, query: impl Into<String>) {
        let query = query.into();
        if query.trim().is_empty() {
            if !self.search.query.trim().is_empty()
                || !self.search.results.is_empty()
                || !self.search.concepts.is_empty()
            {
                self.clear_search();
            }
            return;
        }
        if query.trim() != self.search.query.trim() {
            self.submit_search(query);
        }
    }

    /// Enter in the real SearchModal searches first and opens the selected
    /// result once results already exist for the same query.
    pub fn submit_or_open_search(&mut self, query: impl Into<String>) {
        let query = query.into();
        if self.search.query.trim() == query.trim() && !self.search.results.is_empty() {
            let index = self.search.selected_index.unwrap_or(0);
            self.open_search_result(index);
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
        let query = self.search.query.clone();
        self.apply_search_error_for_query(request_id, &query, error)
    }

    pub fn apply_search_error_for_query(
        &mut self,
        request_id: u64,
        query: &str,
        error: SurfaceError,
    ) -> bool {
        if request_id != self.search.request_id || query.trim() != self.search.query.trim() {
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
        self.search.clear_query();
        self.pending
            .push(ExplorerAction::Search(SearchCommand::ClearQuery));
    }

    /// Completes a queued clear without enqueueing another host command.
    pub fn finish_clear_query(&mut self) {
        self.search.clear_query();
    }

    pub fn close_search(&mut self) {
        self.pending
            .push(ExplorerAction::Search(SearchCommand::Close));
    }

    /// Completes a queued close without enqueueing another host command.
    pub fn finish_close(&mut self) {
        if self.search.phase != ExplorerPhase::Idle
            || !self.search.query.is_empty()
            || !self.search.results.is_empty()
            || !self.search.concepts.is_empty()
            || self.search.selected_index.is_some()
            || self.search.error.is_some()
        {
            self.search.clear();
        }
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
        self.graph.view_generation = 0;
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
            eprintln!("[freya][graph] action=select:start node={id}");
            self.graph.selected_node_id = Some(id.clone());
            self.pending
                .push(ExplorerAction::Graph(GraphCommand::SelectNode(id)));
            eprintln!("[freya][graph] action=select:complete");
        }
    }

    pub fn open_selected_graph_node(&mut self) {
        if self.graph.selected_node_id.is_some() {
            self.pending
                .push(ExplorerAction::Graph(GraphCommand::OpenSelectedNode));
        }
    }

    /// Reproduces the source `resetView` action without inventing a layout.
    /// The runtime consumes the typed command; the generation makes the
    /// completed camera action observable to Freya Testing.
    pub fn reset_graph_view(&mut self) {
        if self.graph.selected_node_id.is_none() {
            return;
        }
        self.graph.view_generation = self.graph.view_generation.saturating_add(1);
        self.pending
            .push(ExplorerAction::Graph(GraphCommand::ResetView));
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
///
/// Search is opened by the rail while the library remains mounted underneath,
/// matching the Vue `SearchModal` overlay contract.  The full Explorer
/// workspace remains available for the explicit Graph route below.
pub fn search_overlay(
    state: State<ExplorerState>,
    query: State<String>,
    interactive: bool,
    backdrop_opacity: f32,
    content_opacity: f32,
) -> Element {
    let input_value = query.read().clone();
    let snapshot = state.read().clone();
    let search_placeholder = if snapshot.search.query.is_empty() {
        "Search notes, paths, tags, or ideas…".to_owned()
    } else {
        snapshot.search.query.clone()
    };
    let input = Input::new(query)
        .flat()
        .auto_focus(true)
        .width(Size::fill())
        .theme_colors(
            InputColorsThemePartial::new()
                .color(if input_value.is_empty() {
                    Color::TRANSPARENT
                } else {
                    theme::color(theme::TEXT)
                })
                .placeholder_color(theme::color(theme::MUTED))
                .background(Color::TRANSPARENT)
                .focus_background(Color::TRANSPARENT)
                .border_fill(Color::TRANSPARENT)
                .focus_border_fill(Color::TRANSPARENT),
        )
        .placeholder(search_placeholder)
        .on_submit({
            let mut submit_state = state;
            move |value: String| submit_state.write().submit_or_open_search(value)
        });
    let mut search_input = rect()
        // Keep the input identity stable while the debounced search result
        // state changes. Re-keying on the result query remounts the native
        // input and moves the caret back to the start, which is observable
        // during the real search flow even though the functional query is
        // still correct.
        .key(("search-input-host", "search"))
        .width(Size::fill());
    if interactive {
        search_input = search_input.a11y_alt("Search input");
    }
    let search_input = search_input
        .a11y_builder({
            let accessibility_value = input_value.clone();
            move |node| node.set_value(accessibility_value)
        })
        .child(input);
    let mut key_state = state;
    let mut key_query = query;
    let clear = if !input_value.trim().is_empty() {
        let mut clear_query = query;
        let mut clear_state = state;
        Some(
            rect()
                .position(Position::new_absolute().right(15.).top(20.))
                .width(Size::px(30.))
                .height(Size::px(30.))
                .center()
                .background(Color::from_argb(36, 71, 84, 103))
                .with_corner_radius(999.)
                .on_mouse_up(move |_| {
                    clear_query.set(String::new());
                    clear_state.write().clear_search();
                })
                .a11y_alt("Clear search")
                .child(
                    label()
                        .font_size(18.)
                        .color(theme::color(theme::MUTED))
                        .text("×"),
                ),
        )
    } else {
        None
    };
    let search_bar = rect()
        .width(Size::fill())
        .height(Size::px(72.))
        .padding(Gaps::new(0., 18., 0., 29.))
        .horizontal()
        .spacing(6.)
        .center()
        .font_size(22.)
        .font_weight(FontWeight::BOLD)
        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_PANEL_LAYER))
        .background(Color::TRANSPARENT)
        .with_corner_radius(22.)
        .child(svg_icon(Icon::Search, Color::from_rgb(26, 35, 53), 22.))
        .child(search_input)
        .maybe_child(clear);
    let modal = rect()
        .vertical()
        .position(
            Position::new_global()
                .left(295.)
                .top(119.),
        )
        .width(Size::px(690.))
        .height(if snapshot.search.phase == ExplorerPhase::Idle {
            Size::px(72.)
        } else {
            Size::px(282.)
        })
        // The shell owns the translucent glass surface; the bar stays
        // transparent so the same surface covers both empty and result states.
        .background(super::search_overlay_view::glass_surface())
        .color(theme::color(theme::TEXT))
        .with_corner_radius(28.)
        .opacity(content_opacity)
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .shadow(Shadow::new().y(30.).blur(90.).color(Color::from_argb(61, 15, 23, 42)))
        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_MODAL_LAYER))
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key != Key::Named(NamedKey::Escape) {
                return;
            }
            let had_query = !key_query.read().trim().is_empty();
            key_query.set(String::new());
            if had_query {
                key_state.write().clear_search();
            } else {
                key_state.write().close_search();
            }
        })
        .child(search_bar);
    let panel = if snapshot.search.phase == ExplorerPhase::Idle {
        None
    } else {
        let content = if snapshot.search.phase == ExplorerPhase::Results {
            super::search_overlay_view::render(state, &snapshot)
        } else {
            search_state_content(state, &snapshot)
        };
        Some(
            rect()
                .position(Position::new_global().left(295.).top(191.))
                .width(Size::px(690.))
                .height(Size::px(210.))
                .opacity(content_opacity)
                .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_PANEL_LAYER))
                .child(content),
        )
    };
    rect()
        .position(Position::new_global())
        .layer(Layer::Overlay)
        .interactive(interactive)
        .child(
            rect()
                .position(Position::new_global().left(0.).top(0.))
                .width(Size::window_percent(100.))
                .height(Size::window_percent(100.))
                // The source overlay is rgba(15, 23, 42, 0.12). Freya's
                // premultiplied alpha rasterization uses 31/255 here.
                .background(Color::from_argb(31, 15, 23, 42))
                .opacity(backdrop_opacity)
                .layer(Layer::OverlayLevel(
                    super::search_overlay_view::SEARCH_BACKDROP_LAYER,
                )),
        )
        .child(modal)
        .maybe_child(panel)
        .into_element()
}

pub fn explorer_view(
    state: State<ExplorerState>,
    query: State<String>,
    graph_query: State<String>,
    graph_canvas: State<super::graph_canvas::GraphCanvasState>,
    overlay_open: bool,
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
        // The modal owns the search input while it is open. Keep the Explorer
        // header mounted for the Search/Graph workspace contract, but do not
        // mount a second search editor underneath the modal.
        ExplorerSurface::Search if overlay_open => None,
        ExplorerSurface::Search => Some(search_surface(state, &snapshot, search_input, query)),
        ExplorerSurface::Graph => Some(graph_surface(state, &snapshot, graph_input, graph_canvas)),
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .color(theme::color(theme::TEXT))
        .spacing(10.)
        .child(explorer_header(state, snapshot.surface))
        .maybe_child(content)
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
    let input_value = query.read().clone();
    let search_input = rect()
        .a11y_alt("Search input")
        .a11y_builder(move |node| node.set_value(input_value))
        .child(input);
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
        .on_mouse_up(move |_| {
            query_state.set(String::new());
            state.write().clear_search();
        })
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
        .padding(Gaps::new(0., 12., 12., 2.))
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
        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_CONTENT_LAYER))
        .color(theme::color(theme::TEXT))
        .with_corner_radius(12.)
        .padding(Gaps::new_all(12.))
        .spacing(8.)
        .child(body)
        .into_element()
}

fn search_results(state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let mut sections = Vec::new();
    if let Some(concept) = snapshot.search.concepts.first() {
        sections.push(search_concept_section(state, concept));
    }
    if !snapshot.search.results.is_empty() {
        sections.push(
            rect()
                .width(Size::fill())
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
                                snapshot.search.selected_index == Some(index) || index == 0,
                            )
                        }),
                )
                .into_element(),
        );
    }
    // Keep the source SearchModal's stable loading accessibility target on the
    // results container. The production adapter is synchronous in the native
    // host, so the concrete result is already rendered in this same frame.
    rect()
        .width(Size::fill())
        .spacing(10.)
        .a11y_alt("Searching locally…")
        .children(sections)
        .into_element()
}

fn search_concept_section(state: State<ExplorerState>, concept: &ConceptCandidate) -> Element {
    let concept = concept.clone();
    let mut concept_state = state;
    let title = concept.title.clone();
    let evidence_count = concept.evidence_chunks.len();
    let source = concept
        .evidence_chunks
        .first()
        .map(|chunk| {
            if chunk.heading_path.is_empty() {
                if chunk.relative_path.is_empty() {
                    chunk.document_path.clone()
                } else {
                    chunk.relative_path.clone()
                }
            } else {
                chunk.heading_path.join(" › ")
            }
        })
        .unwrap_or_else(|| "source chunk".to_owned());
    let score = format!("{}%", (concept.score.clamp(0., 1.) * 100.).round() as u8);
    let meta = format!(
        "Wikis & concepts · {title} · {} source chunk{} · {source} · {score}",
        evidence_count,
        if evidence_count == 1 { "" } else { "s" }
    );
    rect()
        .width(Size::fill())
        .spacing(4.)
        .child(section_title("WIKIS & CONCEPTS"))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(58.))
                .padding(Gaps::new(10., 12., 10., 12.))
                .horizontal()
                .background(Color::from_rgb(235, 241, 252))
                .border(Border::new().fill(Color::from_rgb(190, 205, 235)).width(1.))
                .with_corner_radius(14.)
                .on_mouse_up(move |_| concept_state.write().open_concept(&concept))
                .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_CONTENT_LAYER))
                .child(
                    rect()
                        .width(Size::px(520.))
                        .height(Size::fill())
                        .spacing(2.)
                        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_TEXT_LAYER))
                        .child(label().font_size(12.).color(theme::color(theme::TEXT)).text(meta)),
                ),
        )
        .into_element()
}

fn search_result_row(
    mut state: State<ExplorerState>,
    index: usize,
    result: &SearchResult,
    selected: bool,
) -> Element {
    let result = result.clone();
    let badge = rect()
        .height(Size::px(22.))
        .padding(Gaps::new(0., 9., 0., 9.))
        .center()
        .background(Color::from_argb(61, 37, 99, 235))
        .with_corner_radius(999.)
        .child(
            label()
                .font_size(11.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(theme::PRIMARY))
                .text(match_label(result.match_type)),
        );
    let body = rect()
        .width(Size::flex(1.))
        .height(Size::fill())
        .spacing(4.)
        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_TEXT_LAYER))
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .spacing(8.)
                .child(
                    label()
                        .font_size(15.)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_argb(255, 16, 24, 40))
                        .text(result.title.clone()),
                )
                .child(badge),
        )
        .child(
            label()
                .font_size(12.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(theme::PRIMARY))
                .text(result.relative_path.clone()),
        )
        ;
    rect()
        .width(Size::fill())
        .height(Size::px(70.))
        .padding(Gaps::new(12., 14., 12., 14.))
        .horizontal()
        .spacing(12.)
        .background(if selected {
            Color::from_rgb(215, 225, 249)
        } else {
            Color::from_rgb(246, 248, 252)
        })
        .color(theme::color(theme::TEXT))
        .with_corner_radius(16.)
        .on_mouse_up(move |_| state.write().open_search_result(index))
        .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_CONTENT_LAYER))
        .a11y_alt(format!("Open note {}", result.title))
        .child(
            rect()
                .width(Size::px(38.))
                .height(Size::px(38.))
                .center()
                .background(Color::from_argb(87, 255, 255, 255))
                .with_corner_radius(14.)
                .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_TEXT_LAYER))
                .child(
                    label()
                        .font_size(18.)
                        .color(theme::color(theme::PRIMARY))
                        .text("▤"),
                ),
        )
        .child(body)
        .child(
            rect()
                .width(Size::px(30.))
                .height(Size::px(30.))
                .center()
                .layer(Layer::OverlayLevel(super::search_overlay_view::SEARCH_TEXT_LAYER))
                .child(
                    label()
                        .font_size(18.)
                        .color(theme::color(theme::MUTED))
                        .text("↗"),
                ),
        )
        .into_element()
}

fn graph_surface(
    mut state: State<ExplorerState>,
    snapshot: &ExplorerState,
    input: Input,
    graph_canvas: State<super::graph_canvas::GraphCanvasState>,
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
    let mut recenter_canvas = graph_canvas.clone();
    let recenter = rect()
        .height(Size::px(30.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| {
            recenter_canvas.write().fit_to_content();
            state.write().reset_graph_view();
        })
        .a11y_alt("Recenter graph")
        .child(label().text("Recenter"));
    let toolbar = rect()
        .width(Size::fill())
        .horizontal()
        .spacing(8.)
        .child(rect().a11y_alt("Graph filter input").child(input))
        .child(refresh)
        .child(reset)
        .child(recenter);
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(0., 12., 12., 12.))
        .spacing(8.)
        .child(toolbar)
        .child(graph_state_content(state, graph_canvas, snapshot))
        .into_element()
}

fn graph_state_content(
    state: State<ExplorerState>,
    graph_canvas: State<super::graph_canvas::GraphCanvasState>,
    snapshot: &ExplorerState,
) -> Element {
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
        ExplorerPhase::Results => super::graph_canvas::render(state, snapshot, graph_canvas),
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

fn section_title(title: impl Into<String>) -> Element {
    let title = title.into();
    label()
        .a11y_alt(title.clone())
        .font_size(12.)
        .font_weight(FontWeight::BOLD)
        .color(theme::color(theme::TEXT))
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

    #[test]
    fn stale_search_error_is_rejected_for_previous_query() {
        let mut state = ExplorerState::new();
        state.submit_search("old");
        let old_request_id = state.search.request_id;
        state.submit_search("new");
        let new_request_id = state.search.request_id;

        assert!(!state.apply_search_error_for_query(
            old_request_id,
            "old",
            SurfaceError::Unknown("old request failed".to_string()),
        ));
        assert_eq!(state.search.phase, ExplorerPhase::Loading);
        assert!(state.apply_search_error_for_query(
            new_request_id,
            "new",
            SurfaceError::Unknown("new request failed".to_string()),
        ));
        assert_eq!(state.search.phase, ExplorerPhase::Error);
    }

    #[test]
    fn closing_an_already_cleared_search_is_idempotent() {
        let mut state = ExplorerState::new();
        state.finish_close();
        assert_eq!(state.search.phase, ExplorerPhase::Idle);
        assert!(state.search.query.is_empty());
        assert!(state.search.results.is_empty());
        assert!(state.search.concepts.is_empty());
    }
}
