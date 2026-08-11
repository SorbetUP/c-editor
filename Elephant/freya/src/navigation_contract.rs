//! Navigation contracts translated from the active Vue shell.
//!
//! This module contains state and commands only. It deliberately has no
//! renderer, fallback view, or invented navigation item.
//!
//! Provenance (working-tree sources, inspected before this file was written):
//! - `Elephant/frontend/app/components/shell/AppShell.vue:103-107,190-204,261-291,480-526,599-603`
//!   supplies the shell-owned views, sidebar bounds, persistence key, and
//!   open/search/settings/toggle/resize actions.
//! - `Elephant/frontend/app/components/navigation/iconRailLayout.js:1-8,38-40,42-57`
//!   supplies the core rail IDs, labels, separator prefix, add-on ID format,
//!   and rail ordering/move rules.
//! - `Elephant/frontend/app/components/navigation/IconRail.vue:2-5,18-32,287-340,380-478`
//!   supplies rendered titles/accessibility names, rail actions, dynamic
//!   add-on items, visibility, and drag/drop move behavior.
//! - `Elephant/frontend/app/components/navigation/SidebarNav.vue:5-45,79-117,129-139`
//!   supplies the `All notes`/`Notes`/`Search notes` labels, entry filtering,
//!   root open, lazy-directory loader, and root move action.
//! - `Elephant/frontend/app/components/navigation/SidebarTreeEntry.vue:4-105,132-224`
//!   supplies recursive depth, active-path rules, lazy child state, accessible
//!   expand/collapse/remove labels, open actions, and folder move behavior.

pub const SIDEBAR_WIDTH_MIN: u16 = 184;
pub const SIDEBAR_WIDTH_MAX: u16 = 320;
pub const SIDEBAR_WIDTH_DEFAULT: u16 = 232;
pub const SIDEBAR_WIDTH_STORAGE_KEY: &str = "elephantnote:sidebarWidth";

pub const WORKSPACE_NAVIGATION_LABEL: &str = "Workspace navigation";
pub const ALL_NOTES_LABEL: &str = "All notes";
pub const NOTES_LABEL: &str = "Notes";
pub const SEARCH_NOTES_LABEL: &str = "Search notes";
pub const SEARCH_LABEL: &str = "Search";
pub const SETTINGS_LABEL: &str = "Settings";
pub const VAULTS_LABEL: &str = "Vaults";
pub const CHANGE_VAULT_ICON_LABEL: &str = "Change vault icon";
pub const DEFAULT_VAULT_ICON_LABEL: &str = "Use default vault icon";
pub const ADD_VAULT_LABEL: &str = "Add another vault";
pub const MANAGE_VAULTS_LABEL: &str = "Manage vaults";
pub const REMOVE_FROM_SIDEBAR_LABEL: &str = "Remove from sidebar";
pub const ICON_RAIL_SEPARATOR_PREFIX: &str = "separator:";
pub const VAULT_RAIL_DESCRIPTION: &str = "Open the active vault switcher.";
pub const SIDEBAR_RAIL_DESCRIPTION: &str = "Show or hide the navigation sidebar.";
pub const SEARCH_RAIL_DESCRIPTION: &str = "Open global search.";

/// The workspace IDs that the vault store accepts in `WORKSPACE_VIEWS`.
pub const VAULT_STORE_WORKSPACE_VIEWS: &[&str] = &[
    "notes", "wiki", "chat", "canvas", "graph", "calendar", "models",
];

/// The core set AppShell keeps when an add-on view is closed.
pub const APP_SHELL_CORE_WORKSPACE_VIEWS: &[&str] = &["notes", "dashboard", "canvas"];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SidebarWidth(u16);

impl SidebarWidth {
    pub const fn default() -> Self {
        Self(SIDEBAR_WIDTH_DEFAULT)
    }

    /// Mirrors `normalizeSidebarWidth`: `Number(value) || 232`, then clamp.
    pub fn from_number(value: f64) -> Self {
        let value = if value.is_finite() && value != 0.0 {
            value
        } else {
            f64::from(SIDEBAR_WIDTH_DEFAULT)
        };
        Self(value.clamp(f64::from(SIDEBAR_WIDTH_MIN), f64::from(SIDEBAR_WIDTH_MAX)) as u16)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceView {
    Notes,
    Wiki,
    Chat,
    Dashboard,
    Canvas,
    Graph,
    Calendar,
    Models,
    Addon(String),
}

impl WorkspaceView {
    pub fn from_source_id(id: &str) -> Self {
        match id {
            "notes" => Self::Notes,
            "wiki" => Self::Wiki,
            "chat" => Self::Chat,
            "dashboard" => Self::Dashboard,
            "canvas" => Self::Canvas,
            "graph" => Self::Graph,
            "calendar" => Self::Calendar,
            "models" => Self::Models,
            id => Self::Addon(id.to_owned()),
        }
    }

    pub fn source_id(&self) -> &str {
        match self {
            Self::Notes => "notes",
            Self::Wiki => "wiki",
            Self::Chat => "chat",
            Self::Dashboard => "dashboard",
            Self::Canvas => "canvas",
            Self::Graph => "graph",
            Self::Calendar => "calendar",
            Self::Models => "models",
            Self::Addon(id) => id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenTarget {
    VaultSwitcher,
    Workspace(WorkspaceView),
    AddonView(String),
    Directory(String),
    Note(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NavigationCommand {
    Open(OpenTarget),
    CloseAddonView,
    Search,
    Settings {
        section: String,
    },
    ToggleSidebar,
    LoadChildren {
        directory: String,
    },
    MoveEntry {
        source: String,
        target_directory: String,
    },
    DetachSidebarEntry {
        path: String,
    },
    RunAddonAction {
        action_id: String,
    },
    MoveRailItem {
        source_id: String,
        target_index: usize,
    },
}

impl NavigationCommand {
    pub fn settings() -> Self {
        Self::Settings {
            section: "appearance".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RailItemKind {
    Action(NavigationCommand),
    Separator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RailItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub title: String,
    pub accessibility_label: String,
    pub active: bool,
    pub draggable: bool,
    pub kind: RailItemKind,
}

impl RailItem {
    pub fn vault(active_vault_name: Option<&str>) -> Self {
        let name = active_vault_name
            .filter(|name| !name.is_empty())
            .unwrap_or("No vault");
        let title = format!("{name} - open vault switcher");
        Self {
            id: "vault".to_owned(),
            label: "Vault".to_owned(),
            description: Some(VAULT_RAIL_DESCRIPTION.to_owned()),
            title: title.clone(),
            accessibility_label: title,
            active: false,
            draggable: false,
            kind: RailItemKind::Action(NavigationCommand::Open(OpenTarget::VaultSwitcher)),
        }
    }

    pub fn sidebar_toggle(visible: bool) -> Self {
        let title = if visible {
            "Hide sidebar"
        } else {
            "Show sidebar"
        };
        Self {
            id: "sidebar-toggle".to_owned(),
            label: "Sidebar".to_owned(),
            description: Some(SIDEBAR_RAIL_DESCRIPTION.to_owned()),
            title: title.to_owned(),
            accessibility_label: title.to_owned(),
            active: false,
            draggable: false,
            kind: RailItemKind::Action(NavigationCommand::ToggleSidebar),
        }
    }

    pub fn search() -> Self {
        Self {
            id: "search".to_owned(),
            label: SEARCH_LABEL.to_owned(),
            description: Some(SEARCH_RAIL_DESCRIPTION.to_owned()),
            title: SEARCH_LABEL.to_owned(),
            accessibility_label: SEARCH_LABEL.to_owned(),
            active: false,
            draggable: true,
            kind: RailItemKind::Action(NavigationCommand::Search),
        }
    }

    pub fn settings() -> Self {
        Self {
            id: "settings".to_owned(),
            label: SETTINGS_LABEL.to_owned(),
            description: None,
            title: SETTINGS_LABEL.to_owned(),
            accessibility_label: SETTINGS_LABEL.to_owned(),
            active: false,
            draggable: false,
            kind: RailItemKind::Action(NavigationCommand::settings()),
        }
    }

    pub fn addon_view(view_id: &str, title: &str, active: bool) -> Self {
        let view_id = view_id.trim();
        Self {
            id: format!("addon-view:{view_id}"),
            label: title.to_owned(),
            description: None,
            title: title.to_owned(),
            accessibility_label: title.to_owned(),
            active,
            draggable: true,
            kind: RailItemKind::Action(NavigationCommand::Open(OpenTarget::AddonView(
                view_id.to_owned(),
            ))),
        }
    }

    pub fn legacy_addon_view(
        addon_id: &str,
        item_id: &str,
        view_id: &str,
        title: &str,
        active: bool,
    ) -> Self {
        Self {
            id: format!("addon-item:{addon_id}:{item_id}"),
            label: title.to_owned(),
            description: None,
            title: title.to_owned(),
            accessibility_label: title.to_owned(),
            active,
            draggable: true,
            kind: RailItemKind::Action(NavigationCommand::Open(OpenTarget::Workspace(
                WorkspaceView::from_source_id(view_id),
            ))),
        }
    }

    pub fn legacy_addon_action(
        addon_id: &str,
        item_id: &str,
        title: &str,
        action_id: &str,
    ) -> Self {
        Self {
            id: format!("addon-item:{addon_id}:{item_id}"),
            label: title.to_owned(),
            description: None,
            title: title.to_owned(),
            accessibility_label: title.to_owned(),
            active: false,
            draggable: true,
            kind: RailItemKind::Action(NavigationCommand::RunAddonAction {
                action_id: action_id.to_owned(),
            }),
        }
    }

    pub fn separator(id: &str) -> Option<Self> {
        if !id.starts_with(ICON_RAIL_SEPARATOR_PREFIX) {
            return None;
        }
        Some(Self {
            id: id.to_owned(),
            label: String::new(),
            description: None,
            title: String::new(),
            accessibility_label: String::new(),
            active: false,
            draggable: false,
            kind: RailItemKind::Separator,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RailLayout {
    pub available: Vec<RailItem>,
}

impl RailLayout {
    pub fn new(available: Vec<RailItem>) -> Self {
        Self { available }
    }

    pub fn visible(&self, configured_order: &[String], hidden: &[String]) -> Vec<RailItem> {
        let available_ids: Vec<String> =
            self.available.iter().map(|item| item.id.clone()).collect();
        let hidden = normalize_ids(hidden)
            .into_iter()
            .filter(|id| available_ids.iter().any(|available| available == id))
            .collect::<Vec<_>>();
        let hidden = hidden.into_iter().collect::<std::collections::HashSet<_>>();
        let order = normalize_order(configured_order, &available_ids);
        let by_id = self
            .available
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect::<std::collections::HashMap<_, _>>();

        order
            .into_iter()
            .filter(|id| !hidden.contains(id))
            .filter_map(|id| {
                by_id
                    .get(id.as_str())
                    .map(|item| (*item).clone())
                    .or_else(|| RailItem::separator(&id))
            })
            .collect()
    }

    pub fn move_item(order: &[String], id: &str, target_index: usize) -> Vec<String> {
        let mut order = normalize_ids(order);
        let Some(current_index) = order.iter().position(|item| item == id) else {
            return order;
        };
        let bounded_index = target_index.min(order.len().saturating_sub(1));
        if current_index == bounded_index {
            return order;
        }
        let item = order.remove(current_index);
        order.insert(bounded_index, item);
        order
    }
}

fn normalize_ids(values: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || normalized.iter().any(|item| item == value) {
            continue;
        }
        normalized.push(value.to_owned());
    }
    normalized
}

fn normalize_order(configured: &[String], available: &[String]) -> Vec<String> {
    let allowed = available.iter().collect::<std::collections::HashSet<_>>();
    let mut order = normalize_ids(configured)
        .into_iter()
        .filter(|id| allowed.contains(id) || id.starts_with(ICON_RAIL_SEPARATOR_PREFIX))
        .collect::<Vec<_>>();

    for id in ["sidebar-toggle", "vault"] {
        if available.iter().any(|available_id| available_id == id)
            && !order.iter().any(|item| item == id)
        {
            order.insert(0, id.to_owned());
        }
    }
    for id in available {
        if !order.iter().any(|item| item == id) {
            order.push(id.clone());
        }
    }
    order
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidebarEntryKind {
    Folder,
    Note,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarEntry {
    pub path: String,
    pub title: String,
    pub kind: SidebarEntryKind,
    pub count: Option<u64>,
    pub detachable: bool,
}

impl SidebarEntry {
    pub fn is_visible(&self) -> bool {
        if self.path.is_empty() || is_hidden_path(&self.path) {
            return false;
        }
        self.kind == SidebarEntryKind::Folder || self.path.to_ascii_lowercase().ends_with(".md")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarTreeNode {
    pub entry: SidebarEntry,
    pub depth: usize,
    pub expanded: bool,
    pub loaded: bool,
    pub loading: bool,
    pub children: Vec<Self>,
}

impl SidebarTreeNode {
    pub fn new(entry: SidebarEntry, depth: usize) -> Self {
        Self {
            entry,
            depth,
            expanded: false,
            loaded: false,
            loading: false,
            children: Vec::new(),
        }
    }

    pub fn is_folder(&self) -> bool {
        self.entry.kind == SidebarEntryKind::Folder
    }

    pub fn is_active(&self, active_path: &str, active_note_path: &str) -> bool {
        if self.is_folder() {
            !self.entry.path.is_empty()
                && (active_path == self.entry.path
                    || active_path.starts_with(&format!("{}/", self.entry.path)))
        } else {
            active_note_path == self.entry.path
        }
    }

    pub fn toggle_command(&mut self) -> Option<NavigationCommand> {
        if !self.is_folder() {
            return None;
        }
        self.expanded = !self.expanded;
        if self.expanded && !self.loaded && !self.loading {
            self.loading = true;
            Some(NavigationCommand::LoadChildren {
                directory: self.entry.path.clone(),
            })
        } else {
            None
        }
    }

    pub fn open_commands(&mut self) -> Vec<NavigationCommand> {
        let mut commands = vec![NavigationCommand::CloseAddonView];
        if self.is_folder() {
            commands.push(NavigationCommand::Open(OpenTarget::Directory(
                self.entry.path.clone(),
            )));
            self.expanded = true;
            self.loaded = false;
            self.loading = true;
            commands.push(NavigationCommand::LoadChildren {
                directory: self.entry.path.clone(),
            });
        } else {
            commands.push(NavigationCommand::Open(OpenTarget::Note(
                self.entry.path.clone(),
            )));
        }
        commands
    }

    pub fn move_command(&self, target_directory: &str) -> NavigationCommand {
        NavigationCommand::MoveEntry {
            source: self.entry.path.clone(),
            target_directory: target_directory.to_owned(),
        }
    }

    pub fn detach_command(&self) -> Option<NavigationCommand> {
        self.entry
            .detachable
            .then(|| NavigationCommand::DetachSidebarEntry {
                path: self.entry.path.clone(),
            })
    }

    pub fn set_children(&mut self, entries: impl IntoIterator<Item = SidebarEntry>) {
        self.children = entries
            .into_iter()
            .filter(SidebarEntry::is_visible)
            .map(|entry| Self::new(entry, self.depth + 1))
            .collect();
        self.loaded = true;
        self.loading = false;
    }

    pub fn fail_loading(&mut self) {
        self.children.clear();
        self.loading = false;
        // The Vue source leaves `loaded` false after a failed request, so a
        // subsequent expansion retries the same directory.
    }

    pub fn toggle_accessibility_label(&self) -> Option<String> {
        self.is_folder().then(|| {
            format!(
                "{} {}",
                if self.expanded { "Collapse" } else { "Expand" },
                self.entry.title
            )
        })
    }

    pub fn remove_accessibility_label(&self) -> Option<String> {
        self.entry
            .detachable
            .then(|| format!("Remove {} from sidebar", self.entry.title))
    }
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn is_hidden_path(path: &str) -> bool {
    normalize_path(path)
        .split('/')
        .filter(|part| !part.is_empty())
        .any(|part| part.starts_with('.'))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarNavigation {
    pub entries: Vec<SidebarTreeNode>,
}

impl SidebarNavigation {
    pub fn new(entries: impl IntoIterator<Item = SidebarEntry>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .filter(SidebarEntry::is_visible)
                .map(|entry| SidebarTreeNode::new(entry, 0))
                .collect(),
        }
    }

    pub fn all_notes_command(&self) -> Vec<NavigationCommand> {
        vec![
            NavigationCommand::CloseAddonView,
            NavigationCommand::Open(OpenTarget::Directory(String::new())),
        ]
    }

    pub fn search_command(&self) -> NavigationCommand {
        NavigationCommand::Search
    }

    pub fn root_move_command(&self, source: &SidebarEntry) -> NavigationCommand {
        NavigationCommand::MoveEntry {
            source: source.path.clone(),
            target_directory: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(path: &str, title: &str) -> SidebarEntry {
        SidebarEntry {
            path: path.to_owned(),
            title: title.to_owned(),
            kind: SidebarEntryKind::Folder,
            count: None,
            detachable: true,
        }
    }

    fn note(path: &str, title: &str) -> SidebarEntry {
        SidebarEntry {
            path: path.to_owned(),
            title: title.to_owned(),
            kind: SidebarEntryKind::Note,
            count: None,
            detachable: false,
        }
    }

    #[test]
    fn source_bounds_and_labels_are_explicit() {
        assert_eq!(SidebarWidth::default().get(), 232);
        assert_eq!(SidebarWidth::from_number(100.0).get(), 184);
        assert_eq!(SidebarWidth::from_number(400.0).get(), 320);
        assert_eq!(SidebarWidth::from_number(f64::NAN).get(), 232);
        assert_eq!(SIDEBAR_WIDTH_STORAGE_KEY, "elephantnote:sidebarWidth");
        assert_eq!(WORKSPACE_NAVIGATION_LABEL, "Workspace navigation");
        assert_eq!(SEARCH_NOTES_LABEL, "Search notes");
        assert_eq!(VAULT_STORE_WORKSPACE_VIEWS.len(), 7);
        assert_eq!(
            APP_SHELL_CORE_WORKSPACE_VIEWS,
            &["notes", "dashboard", "canvas"]
        );
    }

    #[test]
    fn rail_preserves_source_titles_visibility_and_move_rules() {
        let available = vec![
            RailItem::vault(Some("Work")),
            RailItem::sidebar_toggle(true),
            RailItem::search(),
            RailItem::settings(),
        ];
        let layout = RailLayout::new(available);
        let visible = layout.visible(
            &["search".to_owned(), "separator:one".to_owned()],
            &["search".to_owned(), "unknown".to_owned()],
        );
        assert_eq!(
            visible
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["vault", "sidebar-toggle", "separator:one", "settings"]
        );
        assert_eq!(visible[0].accessibility_label, "Work - open vault switcher");
        assert!(!visible[1].draggable);
        assert_eq!(visible[2].kind, RailItemKind::Separator);
        assert_eq!(
            RailItem::sidebar_toggle(false).accessibility_label,
            "Show sidebar"
        );
        assert_eq!(
            RailLayout::move_item(
                &["vault".into(), "search".into(), "settings".into()],
                "settings",
                0
            ),
            vec!["settings", "vault", "search"]
        );
    }

    #[test]
    fn sidebar_filters_source_entries_and_preserves_recursive_lazy_contract() {
        let nav = SidebarNavigation::new([
            folder("docs", "Docs"),
            note("readme.md", "Readme"),
            note("image.png", "Image"),
            note(".hidden.md", "Hidden"),
        ]);
        assert_eq!(nav.entries.len(), 2);
        assert_eq!(nav.entries[0].depth, 0);
        let mut docs = nav.entries[0].clone();
        assert!(docs.is_active("docs/plan.md", ""));
        assert_eq!(
            docs.toggle_accessibility_label().as_deref(),
            Some("Expand Docs")
        );
        assert_eq!(
            docs.toggle_command(),
            Some(NavigationCommand::LoadChildren {
                directory: "docs".into()
            })
        );
        assert!(docs.loading);
        docs.set_children([
            folder("docs/nested", "Nested"),
            note("docs/plan.md", "Plan"),
        ]);
        assert!(docs.loaded);
        assert_eq!(
            docs.children
                .iter()
                .map(|node| node.depth)
                .collect::<Vec<_>>(),
            vec![1, 1]
        );
        assert_eq!(
            docs.children[0].toggle_accessibility_label().as_deref(),
            Some("Expand Nested")
        );
        assert_eq!(
            docs.move_command("archive"),
            NavigationCommand::MoveEntry {
                source: "docs".into(),
                target_directory: "archive".into()
            }
        );
        assert_eq!(
            docs.remove_accessibility_label().as_deref(),
            Some("Remove Docs from sidebar")
        );
    }

    #[test]
    fn source_actions_are_explicit() {
        let mut note_node = SidebarTreeNode::new(note("plan.md", "Plan"), 0);
        assert_eq!(
            note_node.open_commands(),
            vec![
                NavigationCommand::CloseAddonView,
                NavigationCommand::Open(OpenTarget::Note("plan.md".into()))
            ]
        );
        assert_eq!(
            SidebarNavigation::new([]).all_notes_command(),
            vec![
                NavigationCommand::CloseAddonView,
                NavigationCommand::Open(OpenTarget::Directory(String::new()))
            ]
        );
        assert_eq!(
            NavigationCommand::settings(),
            NavigationCommand::Settings {
                section: "appearance".into()
            }
        );
        assert_eq!(
            SidebarNavigation::new([]).search_command(),
            NavigationCommand::Search
        );
    }
}
