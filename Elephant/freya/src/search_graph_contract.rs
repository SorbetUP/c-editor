//! Typed contracts for the existing Vue search, graph, and wiki surfaces.
//!
//! This file is intentionally renderer-free.  It records the data shapes,
//! state transitions, commands, and limits consumed by a future Freya
//! conversion; it does not search a vault, build a graph, or fabricate wiki
//! results.
//!
//! Provenance was read from the working-tree implementation:
//! - `Elephant/frontend/app/search/SearchModal.vue` and `SearchResultItem.vue`
//!   define the modal states, keyboard behavior, result fields, and labels.
//! - `Elephant/frontend/app/stores/searchStore.js` defines query normalization,
//!   debounce/polling behavior, backend-first search, local fallback, stale
//!   request protection, and status/index lifecycle.
//! - `Elephant/backend/js/search/searchTypes.js`, `searchIpc.js`,
//!   `searchLibrary.js`, and `graphLibrary.js` define the real search modes,
//!   IPC limits, inspection payload, graph node/edge/cluster shapes, and
//!   deterministic index fallback.
//! - `Elephant/frontend/app/components/views/AtomicGraphView.vue` defines the
//!   graph controls, filtering, selection, timelapse, and rendering limits.
//! - `Elephant/backend/js/atomic/AtomicFeatureService.js` and `atomicIpc.js`
//!   define the graph/wiki service payloads and `en:atomic:*` channels.
//! - `Elephant/frontend/app/stores/vaultStore.js`, `shared/wiki.js`,
//!   `backend/js/wiki/wikiLibrary.js`, and `backend/js/vaults.js` define wiki
//!   persistence, proposal status, citations, source inspection, and page
//!   creation.
//! - `Elephant/frontend/app/stores/navigationStore.js` and `vaultStore.js`
//!   define the workspace IDs used to open `wiki` and `graph`.

pub const SEARCH_SOURCE_ID: &str = "search";
pub const GRAPH_SOURCE_ID: &str = "graph";
pub const WIKI_SOURCE_ID: &str = "wiki";
pub const SEARCH_MODAL_COMPONENT_ID: &str = "SearchModal.vue";
pub const GRAPH_COMPONENT_ID: &str = "AtomicGraphView.vue";

pub const WORKSPACE_SOURCE_IDS: &[&str] = &[
    "notes",
    "wiki",
    "chat",
    "dashboard",
    "canvas",
    "graph",
    "calendar",
    "models",
];

pub const SEARCH_DEBOUNCE_MS: u64 = 220;
pub const SEARCH_STATUS_POLL_MS: u64 = 1_500;
pub const SEARCH_REQUEST_TIMEOUT_MS: u64 = 15_000;
pub const CONCEPT_REQUEST_TIMEOUT_MS: u64 = 8_000;
pub const SEARCH_QUERY_LIMIT_DEFAULT: usize = 20;
pub const SEARCH_QUERY_LIMIT_MIN: usize = 1;
pub const SEARCH_QUERY_LIMIT_MAX: usize = 50;
pub const CONCEPT_LIMIT_DEFAULT: usize = 5;
pub const CONCEPT_LIMIT_MAX: usize = 12;
pub const CONCEPT_EVIDENCE_LIMIT_DEFAULT: usize = 5;
pub const CONCEPT_EVIDENCE_LIMIT_MAX: usize = 8;

pub const GRAPH_SERVICE_MAX_NOTES: usize = 5_000;
pub const GRAPH_SERVICE_MAX_LINKS_PER_NOTE: usize = 8;
pub const GRAPH_SERVICE_MAX_EDGES: usize = 12_000;
pub const GRAPH_SERVICE_MAX_INVERTED_BUCKET: usize = 120;
pub const GRAPH_RENDER_MAX_EDGES: usize = 3_000;
pub const GRAPH_SEMANTIC_THRESHOLD: f32 = 0.28;
pub const GRAPH_LEXICAL_THRESHOLD: f32 = 0.24;
pub const WIKI_MAX_RECORDS: usize = 40;
pub const WIKI_MAX_CITATIONS: usize = 8;
pub const WIKI_RELATED_NODE_LIMIT: usize = 12;

pub const SEARCH_IPC_CHANNELS: &[&str] = &[
    "en:search:init-vault",
    "en:search:query",
    "en:search:concepts",
    "en:search:status",
    "en:search:inspect",
    "en:search:rebuild",
    "en:search:clear",
    "en:search:disable",
    "en:search:enable",
];

pub const ATOMIC_IPC_CHANNELS: &[&str] = &[
    "en:atomic:api:describe",
    "en:atomic:api:call",
    "en:atomic:providers",
    "en:atomic:overview",
    "en:atomic:graph",
    "en:atomic:wiki",
    "en:atomic:wiki:create-page",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceId {
    Search,
    Graph,
    Wiki,
}

impl SurfaceId {
    pub const fn source_id(self) -> &'static str {
        match self {
            Self::Search => SEARCH_SOURCE_ID,
            Self::Graph => GRAPH_SOURCE_ID,
            Self::Wiki => WIKI_SOURCE_ID,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Provenance {
    pub surface: SurfaceId,
    pub source_id: &'static str,
    pub source_path: &'static str,
    pub template_anchor: &'static str,
    pub script_anchor: &'static str,
    pub backend_anchors: &'static [&'static str],
}

pub const PROVENANCE: &[Provenance] = &[
    Provenance {
        surface: SurfaceId::Search,
        source_id: SEARCH_SOURCE_ID,
        source_path: "Elephant/frontend/app/search/SearchModal.vue",
        template_anchor: ".en-search-bar input; .en-search-results; SearchResultItem",
        script_anchor: "handleKeyDown; watch(store.query); watch(store.isOpen)",
        backend_anchors: &[
            "Elephant/frontend/app/stores/searchStore.js",
            "Elephant/backend/js/search/searchIpc.js",
            "Elephant/backend/js/search/searchLibrary.js",
        ],
    },
    Provenance {
        surface: SurfaceId::Graph,
        source_id: GRAPH_SOURCE_ID,
        source_path: "Elephant/frontend/app/components/views/AtomicGraphView.vue",
        template_anchor: ".en-graph-stage; .en-graph-settings-panel; .en-timeline-panel",
        script_anchor: "rawGraph; filteredNodes; filteredEdges; ensureGraphData",
        backend_anchors: &[
            "Elephant/backend/js/search/graphLibrary.js",
            "Elephant/backend/js/atomic/AtomicFeatureService.js",
            "Elephant/backend/js/atomic/atomicIpc.js",
        ],
    },
    Provenance {
        surface: SurfaceId::Wiki,
        source_id: WIKI_SOURCE_ID,
        source_path: "Elephant/frontend/app/stores/vaultStore.js",
        template_anchor: "loadWiki; acceptWikiProposal; dismissWikiProposal",
        script_anchor: "activeWorkspaceView === 'wiki'; wikiRecords; wikiLoading",
        backend_anchors: &[
            "Elephant/shared/wiki.js",
            "Elephant/backend/js/wiki/wikiLibrary.js",
            "Elephant/backend/js/vaults.js",
        ],
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMode {
    Smart,
    Exact,
    Semantic,
}

impl SearchMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Smart => "smart",
            Self::Exact => "exact",
            Self::Semantic => "semantic",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "smart" => Some(Self::Smart),
            "exact" => Some(Self::Exact),
            "semantic" => Some(Self::Semantic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchRequest {
    pub query: String,
    pub mode: SearchMode,
    pub limit: usize,
}

impl SearchRequest {
    /// Mirrors `searchStore.search`: trim before deciding whether to call the
    /// backend. An empty query produces no request and clears visible results.
    pub fn new(query: impl Into<String>, mode: SearchMode, limit: usize) -> Option<Self> {
        let query = query.into().trim().to_owned();
        if query.is_empty() {
            return None;
        }
        Some(Self {
            query,
            mode,
            limit: limit.clamp(SEARCH_QUERY_LIMIT_MIN, SEARCH_QUERY_LIMIT_MAX),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchStatusKind {
    Disabled,
    NotInitialized,
    ModelMissing,
    ModelLoading,
    Indexing,
    Ready,
    Error,
    Unknown,
}

impl SearchStatusKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::NotInitialized => "not_initialized",
            Self::ModelMissing => "model_missing",
            Self::ModelLoading => "model_loading",
            Self::Indexing => "indexing",
            Self::Ready => "ready",
            Self::Error => "error",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "disabled" => Self::Disabled,
            "not_initialized" => Self::NotInitialized,
            "model_missing" => Self::ModelMissing,
            "model_loading" => Self::ModelLoading,
            "indexing" => Self::Indexing,
            "ready" => Self::Ready,
            "error" => Self::Error,
            _ => Self::Unknown,
        }
    }

    pub const fn is_loading(self) -> bool {
        matches!(self, Self::ModelLoading | Self::Indexing)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchStatus {
    pub status: SearchStatusKind,
    pub vault_path: String,
    pub indexed_documents: usize,
    pub total_documents: usize,
    pub message: String,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchSnippet {
    pub text: String,
    pub score: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMatchType {
    Semantic,
    Keyword,
    Hybrid,
    Concept,
    Unknown,
}

impl SearchMatchType {
    pub fn parse(value: &str) -> Self {
        match value {
            "semantic" => Self::Semantic,
            "keyword" => Self::Keyword,
            "hybrid" => Self::Hybrid,
            "concept" => Self::Concept,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub id: String,
    pub uri: String,
    pub title: String,
    pub relative_path: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub score: f32,
    pub match_type: SearchMatchType,
    pub snippets: Vec<SearchSnippet>,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceChunk {
    pub id: String,
    pub document_path: String,
    pub relative_path: String,
    pub chunk_index: usize,
    pub heading_path: Vec<String>,
    pub score: f32,
    pub preview: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConceptCandidate {
    pub id: String,
    pub title: String,
    pub aliases: Vec<String>,
    pub score: f32,
    pub confidence: f32,
    pub match_type: SearchMatchType,
    pub evidence_chunks: Vec<EvidenceChunk>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConceptQueryProfile {
    pub raw: String,
    pub normalized: String,
    pub terms: Vec<String>,
    pub quoted: Vec<String>,
    pub is_short: bool,
    pub is_question: bool,
    pub is_ambiguous_candidate: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConceptRoute {
    pub query: ConceptQueryProfile,
    pub ambiguous: bool,
    pub candidates: Vec<ConceptCandidate>,
    pub kept_candidate_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchState {
    pub is_open: bool,
    pub query: String,
    pub mode: SearchMode,
    pub results: Vec<SearchResult>,
    pub concept_results: Vec<ConceptCandidate>,
    pub concept_route: Option<ConceptRoute>,
    pub status: SearchStatus,
    pub busy: bool,
    pub error: String,
    pub selected_index: usize,
    pub last_request_id: u64,
}

impl SearchState {
    pub fn has_query(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub fn has_search_content(&self) -> bool {
        !self.results.is_empty() || !self.concept_results.is_empty()
    }

    /// Mirrors `SearchModal.showPanel`, including status visibility rules.
    pub fn show_panel(&self) -> bool {
        !self.error.is_empty()
            || self.busy
            || self.has_search_content()
            || self.has_query()
            || !matches!(
                self.status.status,
                SearchStatusKind::Ready | SearchStatusKind::NotInitialized
            )
    }

    pub fn move_selection(&mut self, delta: isize) {
        let count = self.results.len();
        if count == 0 {
            return;
        }
        self.selected_index =
            (self.selected_index as isize + delta).rem_euclid(count as isize) as usize;
    }

    pub fn accepts_result(&self, request_id: u64, query: &str) -> bool {
        request_id == self.last_request_id && query.trim() == self.query.trim()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchCommand {
    Open,
    Close,
    SetQuery(String),
    ClearQuery,
    SetMode(SearchMode),
    Submit(SearchRequest),
    RefreshStatus,
    InspectIndex,
    RebuildIndex,
    ClearIndex,
    Enable,
    Disable,
    OpenResult {
        relative_path: String,
        title: String,
    },
    OpenConceptEvidence {
        relative_path: String,
        title: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceError {
    NoActiveVault,
    InvalidQuery,
    InvalidSearchMode,
    SearchTimedOut,
    ConceptTimedOut,
    SearchUnavailable,
    IndexUnavailable,
    GraphUnavailable,
    WikiProposalNotFound,
    WikiWriteFailed,
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphHeading {
    pub level: u8,
    pub title: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphNodeKind {
    Note,
    Folder,
    Other,
}

impl GraphNodeKind {
    pub fn parse(value: &str) -> Self {
        match value {
            "note" => Self::Note,
            "folder" | "directory" => Self::Folder,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub relative_path: String,
    pub title: String,
    pub kind: GraphNodeKind,
    pub folder: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub headings: Vec<GraphHeading>,
    pub key_terms: Vec<String>,
    pub updated_at: String,
    pub weak_title: bool,
    pub source_count: usize,
    pub chunk_count: usize,
    pub position: Option<GraphPosition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphEdgeType {
    Semantic,
    ExplicitLink,
    Folder,
    Tag,
    Lexical,
    Related,
    Other,
}

impl GraphEdgeType {
    pub fn parse(value: &str) -> Self {
        match value {
            "semantic" => Self::Semantic,
            "explicit-link" => Self::ExplicitLink,
            "folder" => Self::Folder,
            "tag" => Self::Tag,
            "lexical" => Self::Lexical,
            "related" => Self::Related,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: GraphEdgeType,
    pub reason: String,
    pub weight: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphClusterKind {
    Semantic,
    Folder,
    Other,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCluster {
    pub id: String,
    pub kind: GraphClusterKind,
    pub label: String,
    pub node_count: usize,
    pub paths: Vec<String>,
    pub tags: Vec<String>,
    pub cohesion: f32,
    pub edge_count: usize,
    pub key_terms: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphStats {
    pub total_notes: usize,
    pub total_candidate_edges: usize,
    pub rendered_edges: usize,
    pub max_links_per_note: usize,
    pub max_edges: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphDataSource {
    SearchInspection,
    AtomicGraphService,
    VaultEntriesFallback,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphSnapshot {
    pub generated_at: String,
    pub vault_root: String,
    pub source: GraphDataSource,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub clusters: Vec<GraphCluster>,
    pub stats: GraphStats,
    pub index_path: String,
    pub search_status: Option<SearchStatus>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphFilterState {
    pub query: String,
    pub filter_tags: bool,
    pub filter_existing: bool,
    pub filter_orphans: bool,
}

impl Default for GraphFilterState {
    fn default() -> Self {
        Self {
            query: String::new(),
            filter_tags: true,
            filter_existing: true,
            filter_orphans: false,
        }
    }
}

impl GraphFilterState {
    /// Exact current Vue predicate: `filteredNodes` checks title or joined
    /// tags whenever `filterQuery` is non-empty. The three switches are
    /// rendered controls, but are not read by that computed property today.
    pub fn matches_current_vue(&self, node: &GraphNode) -> bool {
        let query = self.query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }
        node.title.to_lowercase().contains(&query)
            || node.tags.join(" ").to_lowercase().contains(&query)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphDisplayOptions {
    pub show_labels: bool,
    pub show_stats: bool,
    pub label_threshold: f32,
    pub node_size_scale: f32,
    pub link_thickness: f32,
    pub force_center: f32,
    pub force_repulsion: f32,
    pub force_link: f32,
    pub force_link_distance: f32,
    pub zoom: f32,
}

impl Default for GraphDisplayOptions {
    fn default() -> Self {
        Self {
            show_labels: true,
            show_stats: true,
            label_threshold: 7.0,
            node_size_scale: 1.0,
            link_thickness: 1.0,
            force_center: 0.32,
            force_repulsion: 420.0,
            force_link: 0.55,
            force_link_distance: 95.0,
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphLoadState {
    Idle,
    InspectingIndex,
    RebuildingIndex,
    Ready,
    Empty,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelapseState {
    pub active: bool,
    pub playing: bool,
    pub progress: u8,
    pub speed: u8,
}

impl Default for TimelapseState {
    fn default() -> Self {
        Self {
            active: false,
            playing: false,
            progress: 0,
            speed: 1,
        }
    }
}

impl TimelapseState {
    pub fn visible_node_count(&self, node_count: usize) -> usize {
        if node_count == 0 {
            return 0;
        }
        ((f32::from(self.progress) / 100.0) * node_count as f32)
            .floor()
            .max(1.0) as usize
    }

    pub fn cycle_speed(&mut self) {
        self.speed = match self.speed {
            1 => 2,
            2 => 4,
            _ => 1,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphState {
    pub load: GraphLoadState,
    pub snapshot: Option<GraphSnapshot>,
    pub filters: GraphFilterState,
    pub display: GraphDisplayOptions,
    pub timelapse: TimelapseState,
    pub panel_open: bool,
    pub selected_node_id: Option<String>,
    pub card_collapsed: bool,
    pub error: Option<SurfaceError>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphCommand {
    Refresh,
    RebuildIndex,
    TogglePanel,
    ResetOptions,
    SetFilterQuery(String),
    SetFilterTags(bool),
    SetFilterExisting(bool),
    SetFilterOrphans(bool),
    SetDisplay(GraphDisplayOptions),
    SetZoom(f32),
    ResetView,
    Animate,
    SelectNode(String),
    DeselectNode,
    ToggleCardCollapsed,
    OpenSelectedNode,
    AddGroup { name: String, color: String },
    ToggleTimelapse,
    ToggleTimelapsePlaying,
    SeekTimelapse(u8),
    CycleTimelapseSpeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WikiRecordStatus {
    Proposed,
    Accepted,
    Dismissed,
}

impl WikiRecordStatus {
    pub fn parse(value: &str) -> Self {
        match value {
            "accepted" => Self::Accepted,
            "dismissed" => Self::Dismissed,
            _ => Self::Proposed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiCitation {
    pub path: String,
    pub title: String,
    pub excerpt: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiRecord {
    pub id: String,
    pub topic: String,
    pub title: String,
    pub summary: String,
    pub citations: Vec<WikiCitation>,
    pub status: WikiRecordStatus,
    pub created_at: String,
    pub updated_at: String,
    pub note_path: String,
}

impl WikiRecord {
    pub fn is_visible_proposal(&self) -> bool {
        !matches!(self.status, WikiRecordStatus::Dismissed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiRelatedNode {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub kind: String,
    pub link_type: String,
    pub weight: f32,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiSourceInsight {
    pub path: String,
    pub title: String,
    pub summary: String,
    pub kind: String,
    pub tags: Vec<String>,
    pub source_count: usize,
    pub chunk_count: usize,
    pub updated_at: String,
    pub citations: Vec<WikiCitation>,
    pub related_nodes: Vec<WikiRelatedNode>,
    pub cluster: Option<GraphCluster>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikiGraphSummary {
    pub nodes: usize,
    pub semantic_links: usize,
    pub clusters: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WikiLoadState {
    Idle,
    Loading,
    Ready,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WikiState {
    pub load: WikiLoadState,
    pub records: Vec<WikiRecord>,
    pub selected_record_id: Option<String>,
    pub selected_citation_path: Option<String>,
    pub source_insight: Option<WikiSourceInsight>,
    pub error: Option<SurfaceError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WikiCommand {
    Load {
        regenerate: bool,
    },
    AcceptProposal {
        id: String,
    },
    DismissProposal {
        id: String,
    },
    CreatePage {
        record_id: String,
    },
    InspectSource {
        path: String,
        record_id: Option<String>,
    },
    InspectContext {
        path: String,
        record_id: Option<String>,
        limit: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceCommand {
    Search(SearchCommand),
    Graph(GraphCommand),
    Wiki(WikiCommand),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(kind: SearchStatusKind) -> SearchStatus {
        SearchStatus {
            status: kind,
            vault_path: String::new(),
            indexed_documents: 0,
            total_documents: 0,
            message: String::new(),
            error: String::new(),
        }
    }

    fn node(title: &str, tags: &[&str]) -> GraphNode {
        GraphNode {
            id: title.to_owned(),
            relative_path: format!("{title}.md"),
            title: title.to_owned(),
            kind: GraphNodeKind::Note,
            folder: String::new(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            summary: String::new(),
            headings: Vec::new(),
            key_terms: Vec::new(),
            updated_at: String::new(),
            weak_title: false,
            source_count: 0,
            chunk_count: 0,
            position: None,
        }
    }

    #[test]
    fn search_request_matches_trim_and_backend_limit_contract() {
        assert_eq!(
            SearchRequest::new("  graph  ", SearchMode::Exact, 999),
            Some(SearchRequest {
                query: "graph".to_owned(),
                mode: SearchMode::Exact,
                limit: SEARCH_QUERY_LIMIT_MAX,
            })
        );
        assert_eq!(SearchRequest::new("   ", SearchMode::Exact, 20), None);
        assert_eq!(SearchMode::parse("semantic"), Some(SearchMode::Semantic));
        assert_eq!(SearchStatusKind::parse("indexing").is_loading(), true);
    }

    #[test]
    fn search_state_matches_panel_and_keyboard_selection_rules() {
        let mut state = SearchState {
            is_open: true,
            query: "alpha".to_owned(),
            mode: SearchMode::Exact,
            results: Vec::new(),
            concept_results: Vec::new(),
            concept_route: None,
            status: status(SearchStatusKind::Ready),
            busy: false,
            error: String::new(),
            selected_index: 0,
            last_request_id: 7,
        };
        assert!(state.show_panel());
        state.results = vec![
            SearchResult {
                id: "a".to_owned(),
                uri: String::new(),
                title: "A".to_owned(),
                relative_path: "A.md".to_owned(),
                excerpt: String::new(),
                tags: Vec::new(),
                score: 1.0,
                match_type: SearchMatchType::Keyword,
                snippets: Vec::new(),
                updated_at: String::new(),
            },
            SearchResult {
                id: "b".to_owned(),
                uri: String::new(),
                title: "B".to_owned(),
                relative_path: "B.md".to_owned(),
                excerpt: String::new(),
                tags: Vec::new(),
                score: 0.5,
                match_type: SearchMatchType::Keyword,
                snippets: Vec::new(),
                updated_at: String::new(),
            },
        ];
        state.move_selection(-1);
        assert_eq!(state.selected_index, 1);
        assert!(state.accepts_result(7, " alpha "));
        assert!(!state.accepts_result(6, "alpha"));
    }

    #[test]
    fn graph_filter_is_title_or_tags_and_preserves_current_toggle_semantics() {
        let filter = GraphFilterState {
            query: "rust".to_owned(),
            filter_tags: false,
            filter_existing: false,
            filter_orphans: true,
        };
        assert!(filter.matches_current_vue(&node("Native Rust", &[])));
        assert!(
            filter.matches_current_vue(&node("Note", &["rust"])),
            "Vue still checks tags even when its switch is off"
        );
        assert!(!filter.matches_current_vue(&node("JavaScript", &["web"])),);
    }

    #[test]
    fn graph_and_timelapse_limits_match_services_and_view() {
        assert_eq!(GRAPH_SERVICE_MAX_NOTES, 5_000);
        assert_eq!(GRAPH_SERVICE_MAX_EDGES, 12_000);
        assert_eq!(GRAPH_RENDER_MAX_EDGES, 3_000);
        let mut timelapse = TimelapseState::default();
        assert_eq!(timelapse.visible_node_count(10), 1);
        timelapse.progress = 50;
        assert_eq!(timelapse.visible_node_count(10), 5);
        timelapse.cycle_speed();
        timelapse.cycle_speed();
        timelapse.cycle_speed();
        assert_eq!(timelapse.speed, 1);
    }

    #[test]
    fn wiki_visibility_and_source_ids_match_navigation_contract() {
        assert_eq!(SurfaceId::Graph.source_id(), "graph");
        assert_eq!(SurfaceId::Wiki.source_id(), "wiki");
        assert!(WORKSPACE_SOURCE_IDS.contains(&"wiki"));
        assert!(WORKSPACE_SOURCE_IDS.contains(&"graph"));
        let record = WikiRecord {
            id: "wiki-rust".to_owned(),
            topic: "rust".to_owned(),
            title: "Rust".to_owned(),
            summary: String::new(),
            citations: Vec::new(),
            status: WikiRecordStatus::Proposed,
            created_at: String::new(),
            updated_at: String::new(),
            note_path: String::new(),
        };
        assert!(record.is_visible_proposal());
        assert_eq!(
            WikiRecordStatus::parse("dismissed"),
            WikiRecordStatus::Dismissed
        );
        assert_eq!(PROVENANCE.len(), 3);
        assert!(SEARCH_IPC_CHANNELS.contains(&"en:search:inspect"));
        assert!(ATOMIC_IPC_CHANNELS.contains(&"en:atomic:wiki:create-page"));
    }
}
