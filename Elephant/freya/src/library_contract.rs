//! Typed Freya contract for the existing Vue library surface.
//!
//! Provenance (source contracts, not a redesign):
//! - `Elephant/frontend/app/stores/vaultStore.js:107-110,120-138,273-328`
//!   supplies the initial sort/view values, pinned-first ordering, and vault
//!   loading state transitions.
//! - `Elephant/frontend/app/components/library/LibraryToolbar.vue:86-103`
//!   supplies the four sort values and the grid/list cycles.
//! - `Elephant/frontend/app/components/library/LibraryGrid.vue:63-65,70-123`
//!   supplies the 120-item directory page, 72-item render window, 720px
//!   prefetch threshold, generation guard, and preview request.
//! - `Elephant/frontend/app/components/library/LibraryGrid.vue:125-219`
//!   supplies folder-open, page-append, stale-response, and error transitions.
//! - `Elephant/frontend/app/components/library/LibraryGrid.vue:221-262`
//!   supplies scroll and note/folder/drawing opening behavior.
//! - `Elephant/backend/tauri/src/vault/entries.rs:373-386` and
//!   `tests/app/e2e/tauri-preload.js:105-141` supply the entry payload fields
//!   represented below (`type`, `kind`, `title`, `path`, `childrenPreview`,
//!   `excerpt`, `tags`, and `updatedAt`).

use std::cmp::Ordering;
use std::collections::HashSet;

pub const DIRECTORY_PAGE_SIZE: usize = 120;
pub const RENDER_CHUNK_SIZE: usize = 72;
pub const SCROLL_PREFETCH_PX: usize = 720;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryType {
    Note,
    Folder,
    Drawing,
    File,
    Custom(String),
}

impl EntryType {
    pub fn from_contract(value: &str) -> Self {
        match value {
            "note" => Self::Note,
            "folder" => Self::Folder,
            "drawing" => Self::Drawing,
            "file" => Self::File,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_contract(&self) -> &str {
        match self {
            Self::Note => "note",
            Self::Folder => "folder",
            Self::Drawing => "drawing",
            Self::File => "file",
            Self::Custom(value) => value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Note,
    Folder,
    Drawing,
    File,
    Custom(String),
}

impl EntryKind {
    pub fn from_contract(value: &str) -> Self {
        match value {
            "note" => Self::Note,
            "folder" => Self::Folder,
            "drawing" => Self::Drawing,
            "file" => Self::File,
            other => Self::Custom(other.to_string()),
        }
    }

    fn from_entry_type(value: &EntryType) -> Self {
        Self::from_contract(value.as_contract())
    }

    pub fn as_contract(&self) -> &str {
        match self {
            Self::Note => "note",
            Self::Folder => "folder",
            Self::Drawing => "drawing",
            Self::File => "file",
            Self::Custom(value) => value,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RelativePath(String);

impl RelativePath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().replace('\\', "/"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RelativePath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryTitle(String);

impl EntryTitle {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatedAt(String);

impl UpdatedAt {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildPreview {
    pub title: EntryTitle,
    pub entry_type: EntryType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryEntry {
    /// JSON `type`; Rust cannot use the JavaScript field name directly.
    pub entry_type: EntryType,
    /// JSON `kind`; Tauri payloads may omit it, so the typed fallback is used.
    pub kind: Option<EntryKind>,
    pub title: EntryTitle,
    pub path: RelativePath,
    pub children_preview: Vec<ChildPreview>,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub updated_at: UpdatedAt,
}

impl LibraryEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entry_type: EntryType,
        kind: Option<EntryKind>,
        title: EntryTitle,
        path: RelativePath,
        children_preview: Vec<ChildPreview>,
        excerpt: impl Into<String>,
        tags: Vec<String>,
        updated_at: UpdatedAt,
    ) -> Self {
        Self {
            entry_type,
            kind,
            title,
            path,
            children_preview,
            excerpt: excerpt.into(),
            tags,
            updated_at,
        }
    }

    pub fn effective_kind(&self) -> EntryKind {
        self.kind
            .clone()
            .unwrap_or_else(|| EntryKind::from_entry_type(&self.entry_type))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortMode {
    UpdatedNewest,
    UpdatedOldest,
    TitleAz,
    TitleZa,
}

impl SortMode {
    pub fn from_contract(value: &str) -> Option<Self> {
        match value {
            "updated-newest" => Some(Self::UpdatedNewest),
            "updated-oldest" => Some(Self::UpdatedOldest),
            "title-az" | "title" => Some(Self::TitleAz),
            "title-za" => Some(Self::TitleZa),
            _ => None,
        }
    }

    pub fn as_contract(self) -> &'static str {
        match self {
            Self::UpdatedNewest => "updated-newest",
            Self::UpdatedOldest => "updated-oldest",
            Self::TitleAz => "title-az",
            Self::TitleZa => "title-za",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::UpdatedNewest => Self::UpdatedOldest,
            Self::UpdatedOldest => Self::TitleAz,
            Self::TitleAz => Self::TitleZa,
            Self::TitleZa => Self::UpdatedNewest,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}

impl ViewMode {
    pub fn as_contract(self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::List => "list",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Grid => Self::List,
            Self::List => Self::Grid,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceView {
    Notes,
    Wiki,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VaultId(String);

impl VaultId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewMode {
    Included,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryPageRequest {
    pub relative_path: RelativePath,
    pub offset: usize,
    pub limit: usize,
    pub preview: PreviewMode,
    pub generation: u64,
    pub vault_id: Option<VaultId>,
    pub view: WorkspaceView,
}

impl DirectoryPageRequest {
    fn new(
        relative_path: RelativePath,
        offset: usize,
        generation: u64,
        vault_id: Option<VaultId>,
        view: WorkspaceView,
    ) -> Self {
        Self {
            relative_path,
            offset,
            limit: DIRECTORY_PAGE_SIZE + 1,
            preview: PreviewMode::Included,
            generation,
            vault_id,
            view,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PageAvailability {
    Exhausted,
    MayHaveMore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectoryPhase {
    Ready,
    Opening,
    LoadingMore,
    Failed(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropTarget {
    Inactive,
    Root,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateAction {
    Note,
    Drawing,
    Folder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolbarActionState {
    Idle,
    Running(CreateAction),
    Failed {
        action: CreateAction,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultAvailability {
    Available,
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionRejection {
    Busy,
    MissingVault,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionStart {
    Started,
    Rejected(ActionRejection),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryOpenTarget {
    Note(RelativePath),
    Folder(RelativePath),
    Drawing(RelativePath),
    Ignored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageApply {
    Applied,
    IgnoredStale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadMoreNoop {
    BufferedEntriesRevealed,
    Exhausted,
    AlreadyLoading,
    StaleResponse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadMoreAction {
    Fetch(DirectoryPageRequest),
    Noop(LoadMoreNoop),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAction {
    RequestMore,
    Ignore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryState {
    pub entries: Vec<LibraryEntry>,
    pub root_entries: Vec<LibraryEntry>,
    pub pinned_paths: Vec<RelativePath>,
    pub current_path: RelativePath,
    pub workspace_view: WorkspaceView,
    pub active_vault_id: Option<VaultId>,
    pub sort: SortMode,
    pub view_mode: ViewMode,
    pub visible_entry_limit: usize,
    pub page_availability: PageAvailability,
    pub generation: u64,
    pub phase: DirectoryPhase,
    pub drop_target: DropTarget,
    pub toolbar_action: ToolbarActionState,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            root_entries: Vec::new(),
            pinned_paths: Vec::new(),
            current_path: RelativePath::from(""),
            workspace_view: WorkspaceView::Notes,
            active_vault_id: None,
            sort: SortMode::UpdatedNewest,
            view_mode: ViewMode::Grid,
            visible_entry_limit: RENDER_CHUNK_SIZE,
            page_availability: PageAvailability::Exhausted,
            generation: 0,
            phase: DirectoryPhase::Ready,
            drop_target: DropTarget::Inactive,
            toolbar_action: ToolbarActionState::Idle,
        }
    }
}

impl LibraryState {
    pub fn active_entries(&self) -> Vec<&LibraryEntry> {
        let pinned: HashSet<&RelativePath> = self.pinned_paths.iter().collect();
        let mut entries: Vec<&LibraryEntry> = self.entries.iter().collect();
        entries.sort_by(|left, right| {
            let left_pinned = pinned.contains(&left.path);
            let right_pinned = pinned.contains(&right.path);
            match right_pinned.cmp(&left_pinned) {
                Ordering::Equal => match self.sort {
                    SortMode::UpdatedNewest => compare_updated(&right.updated_at, &left.updated_at),
                    SortMode::UpdatedOldest => compare_updated(&left.updated_at, &right.updated_at),
                    SortMode::TitleAz => compare_titles(left.title.as_str(), right.title.as_str()),
                    SortMode::TitleZa => compare_titles(right.title.as_str(), left.title.as_str()),
                },
                order => order,
            }
        });
        entries
    }

    pub fn filtered_entries(&self) -> Vec<&LibraryEntry> {
        self.active_entries()
            .into_iter()
            .filter(|entry| !self.is_compatibility_root_wiki(entry))
            .collect()
    }

    pub fn visible_entries(&self) -> Vec<&LibraryEntry> {
        self.filtered_entries()
            .into_iter()
            .take(self.visible_entry_limit)
            .collect()
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
    }

    pub fn cycle_view(&mut self) {
        self.view_mode = self.view_mode.next();
    }

    pub fn toggle_pinned(&mut self, path: RelativePath) {
        if let Some(index) = self.pinned_paths.iter().position(|pinned| pinned == &path) {
            self.pinned_paths.remove(index);
        } else {
            self.pinned_paths.insert(0, path);
        }
    }

    pub fn replace_entries(&mut self, entries: Vec<LibraryEntry>) {
        self.entries = entries;
        self.reset_visible_window();
    }

    pub fn reset_visible_window(&mut self) {
        self.visible_entry_limit = RENDER_CHUNK_SIZE;
        self.page_availability = if self.entries.len() >= DIRECTORY_PAGE_SIZE {
            PageAvailability::MayHaveMore
        } else {
            PageAvailability::Exhausted
        };
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn begin_open_folder(
        &mut self,
        relative_path: RelativePath,
        view: WorkspaceView,
    ) -> DirectoryPageRequest {
        self.current_path = relative_path.clone();
        self.workspace_view = view;
        self.entries.clear();
        self.phase = DirectoryPhase::Opening;
        self.reset_visible_window();
        DirectoryPageRequest::new(
            relative_path,
            0,
            self.generation,
            self.active_vault_id.clone(),
            view,
        )
    }

    pub fn apply_folder_page(
        &mut self,
        request: &DirectoryPageRequest,
        items: Vec<LibraryEntry>,
    ) -> PageApply {
        if !self.matches(request) {
            return PageApply::IgnoredStale;
        }
        self.page_availability = page_availability(items.len());
        self.entries = items.into_iter().take(DIRECTORY_PAGE_SIZE).collect();
        if self.current_path.as_str().is_empty() {
            self.root_entries = self.entries.clone();
        }
        self.phase = DirectoryPhase::Ready;
        PageApply::Applied
    }

    pub fn apply_folder_error(
        &mut self,
        request: &DirectoryPageRequest,
        message: impl Into<String>,
    ) -> PageApply {
        if !self.matches(request) {
            return PageApply::IgnoredStale;
        }
        self.entries.clear();
        self.page_availability = PageAvailability::Exhausted;
        self.phase = DirectoryPhase::Failed(message.into());
        PageApply::Applied
    }

    pub fn load_more(&mut self) -> LoadMoreAction {
        if self.visible_entry_limit < self.filtered_entries().len() {
            self.visible_entry_limit = self
                .visible_entry_limit
                .saturating_add(RENDER_CHUNK_SIZE)
                .min(self.filtered_entries().len());
            return LoadMoreAction::Noop(LoadMoreNoop::BufferedEntriesRevealed);
        }
        if self.page_availability == PageAvailability::Exhausted {
            return LoadMoreAction::Noop(LoadMoreNoop::Exhausted);
        }
        if self.phase == DirectoryPhase::LoadingMore {
            return LoadMoreAction::Noop(LoadMoreNoop::AlreadyLoading);
        }
        self.phase = DirectoryPhase::LoadingMore;
        LoadMoreAction::Fetch(DirectoryPageRequest::new(
            self.current_path.clone(),
            self.entries.len(),
            self.generation,
            self.active_vault_id.clone(),
            self.workspace_view,
        ))
    }

    pub fn apply_more_page(
        &mut self,
        request: &DirectoryPageRequest,
        items: Vec<LibraryEntry>,
    ) -> PageApply {
        if !self.matches(request) || self.phase != DirectoryPhase::LoadingMore {
            return PageApply::IgnoredStale;
        }
        self.page_availability = page_availability(items.len());
        let page: Vec<LibraryEntry> = items.into_iter().take(DIRECTORY_PAGE_SIZE).collect();
        let existing: HashSet<RelativePath> = self
            .entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect();
        let appended: Vec<LibraryEntry> = page
            .into_iter()
            .filter(|entry| !existing.contains(&entry.path))
            .collect();
        self.entries.extend(appended);
        if self.current_path.as_str().is_empty() {
            self.root_entries = self.entries.clone();
        }
        let filtered_len = self.filtered_entries().len();
        self.visible_entry_limit =
            (self.visible_entry_limit + RENDER_CHUNK_SIZE).min(filtered_len + RENDER_CHUNK_SIZE);
        self.phase = DirectoryPhase::Ready;
        PageApply::Applied
    }

    pub fn apply_more_error(
        &mut self,
        request: &DirectoryPageRequest,
        message: impl Into<String>,
    ) -> PageApply {
        if !self.matches(request) || self.phase != DirectoryPhase::LoadingMore {
            return PageApply::IgnoredStale;
        }
        self.phase = DirectoryPhase::Failed(message.into());
        PageApply::Applied
    }

    pub fn on_scroll(&self, distance_from_bottom: usize) -> ScrollAction {
        if distance_from_bottom <= SCROLL_PREFETCH_PX {
            ScrollAction::RequestMore
        } else {
            ScrollAction::Ignore
        }
    }

    pub fn set_root_drop_target(&mut self, can_drop: DropTarget) {
        self.drop_target = can_drop;
    }

    pub fn begin_action(&mut self, action: CreateAction, vault: VaultAvailability) -> ActionStart {
        if matches!(self.toolbar_action, ToolbarActionState::Running(_)) {
            return ActionStart::Rejected(ActionRejection::Busy);
        }
        if vault == VaultAvailability::Missing {
            return ActionStart::Rejected(ActionRejection::MissingVault);
        }
        self.toolbar_action = ToolbarActionState::Running(action);
        ActionStart::Started
    }

    pub fn complete_action(&mut self, action: CreateAction) {
        if self.toolbar_action == ToolbarActionState::Running(action) {
            self.toolbar_action = ToolbarActionState::Idle;
        }
    }

    pub fn fail_action(&mut self, action: CreateAction, message: impl Into<String>) {
        if self.toolbar_action == ToolbarActionState::Running(action) {
            self.toolbar_action = ToolbarActionState::Failed {
                action,
                message: message.into(),
            };
        }
    }

    pub fn open_entry(&self, entry: &LibraryEntry) -> EntryOpenTarget {
        let path = entry.path.clone();
        if is_excalidraw_path(path.as_str()) {
            return EntryOpenTarget::Drawing(path);
        }
        match entry.effective_kind() {
            EntryKind::Folder => EntryOpenTarget::Folder(path),
            EntryKind::Drawing => EntryOpenTarget::Drawing(path),
            EntryKind::Note => EntryOpenTarget::Note(path),
            _ if is_markdown_path(path.as_str()) => EntryOpenTarget::Note(path),
            _ => EntryOpenTarget::Ignored,
        }
    }

    fn is_compatibility_root_wiki(&self, entry: &LibraryEntry) -> bool {
        self.workspace_view == WorkspaceView::Notes
            && self.current_path.as_str().is_empty()
            && entry
                .path
                .as_str()
                .trim_matches('/')
                .eq_ignore_ascii_case("wiki")
    }

    fn matches(&self, request: &DirectoryPageRequest) -> bool {
        self.generation == request.generation
            && self.current_path == request.relative_path
            && self.workspace_view == request.view
            && self.active_vault_id == request.vault_id
    }
}

fn page_availability(raw_item_count: usize) -> PageAvailability {
    if raw_item_count > DIRECTORY_PAGE_SIZE {
        PageAvailability::MayHaveMore
    } else {
        PageAvailability::Exhausted
    }
}

fn is_markdown_path(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".md")
}

fn is_excalidraw_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.ends_with(".excalidraw") || path.ends_with(".excalidraw.png")
}

fn compare_titles(left: &str, right: &str) -> Ordering {
    let left_folded = left.to_lowercase();
    let right_folded = right.to_lowercase();
    left_folded
        .cmp(&right_folded)
        .then_with(|| case_weight(left).cmp(&case_weight(right)))
        .then_with(|| left.cmp(right))
}

fn case_weight(value: &str) -> Vec<u8> {
    value
        .chars()
        .map(|character| u8::from(character.is_uppercase()))
        .collect()
}

fn compare_updated(left: &UpdatedAt, right: &UpdatedAt) -> Ordering {
    match (timestamp_key(left.as_str()), timestamp_key(right.as_str())) {
        (Some(left), Some(right)) => left.cmp(&right),
        // JavaScript Date subtraction returns NaN for an invalid date; Array.sort
        // treats that comparator result as equality. Preserve stable input order.
        _ => Ordering::Equal,
    }
}

fn timestamp_key(value: &str) -> Option<i128> {
    if let Ok(seconds) = value.parse::<i128>() {
        return Some(seconds.saturating_mul(1_000));
    }
    parse_rfc3339_millis(value)
}

fn parse_rfc3339_millis(value: &str) -> Option<i128> {
    let (date, time_and_zone) = value.split_once('T')?;
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i128>().ok()?;
    let month = date_parts.next()?.parse::<i128>().ok()?;
    let day = date_parts.next()?.parse::<i128>().ok()?;
    let (time, zone) = if let Some(time) = time_and_zone.strip_suffix('Z') {
        (time, "Z")
    } else {
        let split = time_and_zone
            .char_indices()
            .rev()
            .find(|(_, character)| *character == '+' || *character == '-')
            .map(|(index, _)| index)?;
        (&time_and_zone[..split], &time_and_zone[split..])
    };
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<i128>().ok()?;
    let minute = time_parts.next()?.parse::<i128>().ok()?;
    let second_and_fraction = time_parts.next()?;
    let (second, fraction) = second_and_fraction
        .split_once('.')
        .map_or((second_and_fraction, "0"), |parts| parts);
    let second = second.parse::<i128>().ok()?;
    let millis = fraction
        .chars()
        .take(3)
        .collect::<String>()
        .parse::<i128>()
        .ok()
        .map(|value| match fraction.len() {
            1 => value * 100,
            2 => value * 10,
            _ => value,
        })?;
    let offset_minutes = match zone {
        "Z" => 0,
        value => {
            let sign = if value.starts_with('-') { -1 } else { 1 };
            let value = &value[1..];
            let (hours, minutes) = value.split_once(':')?;
            sign * (hours.parse::<i128>().ok()? * 60 + minutes.parse::<i128>().ok()?)
        }
    };
    let days = days_from_civil(year, month, day)?;
    Some((((days * 24 + hour) * 60 + minute - offset_minutes) * 60 + second) * 1_000 + millis)
}

fn days_from_civil(year: i128, month: i128, day: i128) -> Option<i128> {
    if !(1..=12).contains(&month) || day < 1 || day > 31 {
        return None;
    }
    let adjusted_year = year - i128::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year / 400
    } else {
        (adjusted_year - 399) / 400
    };
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    fn entry(
        entry_type: &str,
        kind: Option<&str>,
        title: &str,
        path: &str,
        updated_at: &str,
    ) -> LibraryEntry {
        LibraryEntry::new(
            EntryType::from_contract(entry_type),
            kind.map(EntryKind::from_contract),
            EntryTitle::new(title),
            RelativePath::from(path),
            Vec::new(),
            "excerpt",
            vec!["tag".to_string()],
            UpdatedAt::new(updated_at),
        )
    }

    #[test]
    fn toolbar_cycles_match_the_four_vue_sort_values() {
        let mut sort = SortMode::UpdatedNewest;
        let expected = ["updated-oldest", "title-az", "title-za", "updated-newest"];
        for value in expected {
            sort = sort.next();
            assert_eq!(sort.as_contract(), value);
        }
        assert_eq!(SortMode::from_contract("title"), Some(SortMode::TitleAz));
        assert_eq!(SortMode::from_contract("unknown"), None);
    }

    #[test]
    fn toolbar_cycles_grid_and_list_without_a_boolean_mode() {
        let mut view = ViewMode::Grid;
        assert_eq!(view.as_contract(), "grid");
        view = view.next();
        assert_eq!(view, ViewMode::List);
        view = view.next();
        assert_eq!(view, ViewMode::Grid);
    }

    #[test]
    fn active_entries_keep_pinned_first_and_support_all_sort_modes() {
        let mut state = LibraryState::default();
        state.entries = vec![
            entry("note", None, "Beta", "Beta.md", "2026-01-02T00:00:00.000Z"),
            entry(
                "note",
                None,
                "alpha",
                "Alpha.md",
                "2026-01-03T00:00:00.000Z",
            ),
            entry(
                "folder",
                Some("folder"),
                "Projects",
                "Projects",
                "2026-01-01T00:00:00.000Z",
            ),
        ];
        state.pinned_paths = vec![RelativePath::from("Projects")];

        state.sort = SortMode::UpdatedNewest;
        assert_eq!(state.active_entries()[0].path.as_str(), "Projects");
        assert_eq!(state.active_entries()[1].path.as_str(), "Alpha.md");
        state.sort = SortMode::UpdatedOldest;
        assert_eq!(state.active_entries()[0].path.as_str(), "Projects");
        assert_eq!(state.active_entries()[1].path.as_str(), "Beta.md");
        state.sort = SortMode::TitleAz;
        assert_eq!(state.active_entries()[1].title.as_str(), "alpha");
        state.sort = SortMode::TitleZa;
        assert_eq!(state.active_entries()[1].title.as_str(), "Beta");
    }

    #[test]
    fn toggling_pinned_entry_updates_order_and_is_reversible() {
        let mut state = LibraryState::default();
        state.entries = vec![
            entry(
                "note",
                None,
                "Alpha",
                "Alpha.md",
                "2026-01-01T00:00:00.000Z",
            ),
            entry("note", None, "Beta", "Beta.md", "2026-01-02T00:00:00.000Z"),
        ];
        state.toggle_pinned(RelativePath::from("Alpha.md"));
        assert_eq!(state.active_entries()[0].path.as_str(), "Alpha.md");
        assert_eq!(state.pinned_paths, vec![RelativePath::from("Alpha.md")]);
        state.toggle_pinned(RelativePath::from("Alpha.md"));
        assert!(state.pinned_paths.is_empty());
    }

    #[test]
    fn newly_pinned_entries_follow_the_same_most_recent_order_as_tauri() {
        let mut state = LibraryState::default();
        state.entries = vec![
            entry(
                "note",
                None,
                "Alpha",
                "Alpha.md",
                "2026-01-01T00:00:00.000Z",
            ),
            entry("note", None, "Beta", "Beta.md", "2026-01-02T00:00:00.000Z"),
        ];

        state.toggle_pinned(RelativePath::from("Beta.md"));
        state.toggle_pinned(RelativePath::from("Alpha.md"));

        assert_eq!(
            state.pinned_paths,
            vec![
                RelativePath::from("Alpha.md"),
                RelativePath::from("Beta.md")
            ]
        );
    }

    #[test]
    fn toggling_pinned_entry_preserves_the_loaded_window_and_request_generation() {
        let mut state = LibraryState::default();
        state.entries = (0..120)
            .map(|index| {
                entry(
                    "note",
                    None,
                    &format!("N{index}"),
                    &format!("N{index}.md"),
                    "1700000000",
                )
            })
            .collect();
        state.visible_entry_limit = 120;
        state.generation = 17;
        state.page_availability = PageAvailability::MayHaveMore;

        state.toggle_pinned(RelativePath::from("N119.md"));

        assert_eq!(state.visible_entry_limit, 120);
        assert_eq!(state.generation, 17);
        assert_eq!(state.page_availability, PageAvailability::MayHaveMore);
        assert_eq!(state.pinned_paths, vec![RelativePath::from("N119.md")]);
    }

    #[test]
    fn entry_contract_keeps_all_typed_payload_fields_and_kind_fallback() {
        let child = ChildPreview {
            title: EntryTitle::new("Overview"),
            entry_type: EntryType::Folder,
        };
        let value = LibraryEntry::new(
            EntryType::Folder,
            None,
            EntryTitle::new("Projects"),
            RelativePath::from("Projects"),
            vec![child],
            "",
            vec!["work".to_string()],
            UpdatedAt::new("1760000000"),
        );
        assert_eq!(value.effective_kind(), EntryKind::Folder);
        assert_eq!(value.children_preview[0].title.as_str(), "Overview");
        assert_eq!(value.tags, vec!["work"]);
        assert_eq!(value.updated_at.as_str(), "1760000000");
    }

    #[test]
    fn folder_pages_use_121_request_items_but_store_at_most_120() {
        let mut state = LibraryState::default();
        state.active_vault_id = Some(VaultId::new("vault"));
        let request = state.begin_open_folder(RelativePath::from(""), WorkspaceView::Notes);
        assert_eq!(request.limit, 121);
        assert_eq!(request.offset, 0);
        assert_eq!(request.preview, PreviewMode::Included);
        assert_eq!(state.visible_entry_limit, 72);

        let page = (0..121)
            .map(|index| {
                entry(
                    "note",
                    None,
                    &format!("N{index}"),
                    &format!("N{index}.md"),
                    "1700000000",
                )
            })
            .collect();
        assert_eq!(state.apply_folder_page(&request, page), PageApply::Applied);
        assert_eq!(state.entries.len(), 120);
        assert_eq!(state.page_availability, PageAvailability::MayHaveMore);
        assert_eq!(state.visible_entries().len(), 72);

        assert_eq!(
            state.load_more(),
            LoadMoreAction::Noop(LoadMoreNoop::BufferedEntriesRevealed)
        );
        assert_eq!(state.visible_entries().len(), 120);
        let next = match state.load_more() {
            LoadMoreAction::Fetch(request) => request,
            other => panic!("expected page fetch, got {other:?}"),
        };
        assert_eq!(next.offset, 120);
        assert_eq!(next.limit, 121);
    }

    #[test]
    fn append_page_deduplicates_paths_and_resets_loading_phase() {
        let mut state = LibraryState::default();
        let request = state.begin_open_folder(RelativePath::from(""), WorkspaceView::Notes);
        let first_page = (0..121)
            .map(|index| {
                entry(
                    "note",
                    None,
                    &format!("N{index}"),
                    &format!("N{index}.md"),
                    "1700000000",
                )
            })
            .collect();
        state.apply_folder_page(&request, first_page);
        state.visible_entry_limit = 120;
        let next = match state.load_more() {
            LoadMoreAction::Fetch(request) => request,
            _ => panic!("expected load-more request"),
        };
        let mut second_page = vec![entry("note", None, "N119", "N119.md", "1700000000")];
        second_page.extend((120..125).map(|index| {
            entry(
                "note",
                None,
                &format!("N{index}"),
                &format!("N{index}.md"),
                "1700000000",
            )
        }));
        assert_eq!(
            state.apply_more_page(&next, second_page),
            PageApply::Applied
        );
        assert_eq!(state.entries.len(), 125);
        assert_eq!(state.phase, DirectoryPhase::Ready);
    }

    #[test]
    fn stale_folder_response_is_ignored_and_errors_are_visible_in_state() {
        let mut state = LibraryState::default();
        let old = state.begin_open_folder(RelativePath::from("old"), WorkspaceView::Notes);
        let current = state.begin_open_folder(RelativePath::from("current"), WorkspaceView::Notes);
        assert_eq!(
            state.apply_folder_page(&old, Vec::new()),
            PageApply::IgnoredStale
        );
        assert_eq!(state.current_path.as_str(), "current");
        assert_eq!(
            state.apply_folder_error(&current, "directory unavailable"),
            PageApply::Applied
        );
        assert_eq!(
            state.phase,
            DirectoryPhase::Failed("directory unavailable".to_string())
        );
    }

    #[test]
    fn scroll_threshold_and_root_wiki_filter_match_library_grid() {
        let mut state = LibraryState::default();
        state.entries = vec![
            entry("note", None, "Wiki", "wiki", "1700000000"),
            entry("note", None, "Real", "Real.md", "1700000000"),
        ];
        assert_eq!(state.filtered_entries().len(), 1);
        assert_eq!(state.on_scroll(720), ScrollAction::RequestMore);
        assert_eq!(state.on_scroll(721), ScrollAction::Ignore);
    }

    #[test]
    fn open_entry_preserves_folder_note_drawing_and_unknown_routes() {
        let state = LibraryState::default();
        assert_eq!(
            state.open_entry(&entry("folder", None, "Folder", "Folder", "1700000000")),
            EntryOpenTarget::Folder(RelativePath::from("Folder"))
        );
        assert_eq!(
            state.open_entry(&entry("note", None, "Note", "Note.md", "1700000000")),
            EntryOpenTarget::Note(RelativePath::from("Note.md"))
        );
        assert_eq!(
            state.open_entry(&entry(
                "file",
                None,
                "Drawing",
                "Drawing.excalidraw",
                "1700000000"
            )),
            EntryOpenTarget::Drawing(RelativePath::from("Drawing.excalidraw"))
        );
        assert_eq!(
            state.open_entry(&entry("file", None, "Other", "Other.bin", "1700000000")),
            EntryOpenTarget::Ignored
        );
    }

    #[test]
    fn toolbar_action_transitions_are_explicit() {
        let mut state = LibraryState::default();
        assert_eq!(
            state.begin_action(CreateAction::Drawing, VaultAvailability::Missing),
            ActionStart::Rejected(ActionRejection::MissingVault)
        );
        assert_eq!(
            state.begin_action(CreateAction::Note, VaultAvailability::Available),
            ActionStart::Started
        );
        assert_eq!(
            state.begin_action(CreateAction::Folder, VaultAvailability::Available),
            ActionStart::Rejected(ActionRejection::Busy)
        );
        state.fail_action(CreateAction::Note, "create failed");
        assert_eq!(
            state.toolbar_action,
            ToolbarActionState::Failed {
                action: CreateAction::Note,
                message: "create failed".to_string()
            }
        );
        assert_eq!(
            state.begin_action(CreateAction::Folder, VaultAvailability::Available),
            ActionStart::Started
        );
        state.complete_action(CreateAction::Folder);
        assert_eq!(state.toolbar_action, ToolbarActionState::Idle);
    }
}
