//! Traceability contracts for the active Vue/Tauri shell.
//!
//! This module deliberately contains no Freya widgets and no replacement
//! implementation.  It is the source-of-truth index that a Freya assembly
//! must consume while converting the existing web surface component by
//! component.  Values are the resolved desktop CSS values after the active
//! `runtime-layout-fixes.css` stylesheet has been loaded.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentId {
    AppShell,
    TopVaultBar,
    IconRail,
    SidebarNav,
    MainContent,
    LibraryToolbar,
    CreateEntryMenu,
    LibraryGrid,
    NoteCard,
    NoteEditorHost,
}

impl ComponentId {
    pub const ALL: [Self; 10] = [
        Self::AppShell,
        Self::TopVaultBar,
        Self::IconRail,
        Self::SidebarNav,
        Self::MainContent,
        Self::LibraryToolbar,
        Self::CreateEntryMenu,
        Self::LibraryGrid,
        Self::NoteCard,
        Self::NoteEditorHost,
    ];

    pub const fn source_name(self) -> &'static str {
        match self {
            Self::AppShell => "AppShell.vue",
            Self::TopVaultBar => "TopVaultBar.vue",
            Self::IconRail => "IconRail.vue",
            Self::SidebarNav => "SidebarNav.vue",
            Self::MainContent => "MainContent.vue",
            Self::LibraryToolbar => "LibraryToolbar.vue",
            Self::CreateEntryMenu => "CreateEntryMenu.vue",
            Self::LibraryGrid => "LibraryGrid.vue",
            Self::NoteCard => "NoteCard.vue",
            Self::NoteEditorHost => "NoteEditorHost.vue",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticSurface {
    Class,
    TextLabel,
    AriaLabel,
    DataAttribute,
    DataTestId,
    Role,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticContract {
    pub selector: &'static str,
    pub surface: SemanticSurface,
    pub value: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DimensionContract {
    pub selector: &'static str,
    pub property: &'static str,
    pub resolved_value: &'static str,
    pub css_source: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateContract {
    pub name: &'static str,
    pub source_signal: &'static str,
    pub observable_effect: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventContract {
    pub trigger: &'static str,
    pub handler_or_effect: &'static str,
    pub source_signal: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Provenance {
    pub component: ComponentId,
    pub source_path: &'static str,
    pub template_anchor: &'static str,
    pub script_anchor: &'static str,
    pub style_sources: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentContract {
    pub id: ComponentId,
    pub provenance: Provenance,
    pub semantics: &'static [SemanticContract],
    pub dimensions: &'static [DimensionContract],
    pub states: &'static [StateContract],
    pub events: &'static [EventContract],
}

const APP_SHELL_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/shell/AppShell.vue<style scoped>",
    "Elephant/frontend/app/styles/app-shell-runtime-fixes.css",
    "Elephant/frontend/app/styles/runtime-layout-fixes.css",
];
const TOP_VAULT_BAR_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/shell/TopVaultBar.vue<style scoped>",
    "Elephant/frontend/app/styles/app-shell-runtime-fixes.css",
];
const ICON_RAIL_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/navigation/IconRail.vue<style scoped>",
    "Elephant/frontend/app/styles/runtime-layout-fixes.css",
];
const SIDEBAR_NAV_STYLES: &[&str] =
    &["Elephant/frontend/app/components/navigation/SidebarNav.vue<style scoped>"];
const MAIN_CONTENT_STYLES: &[&str] =
    &["Elephant/frontend/app/components/shell/MainContent.vue<style scoped>"];
const LIBRARY_TOOLBAR_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/library/LibraryToolbar.vue<style scoped>",
    "Elephant/frontend/app/styles/app-shell.css",
];
const CREATE_ENTRY_MENU_STYLES: &[&str] =
    &["Elephant/frontend/app/components/library/CreateEntryMenu.vue<style scoped>"];
const LIBRARY_GRID_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/library/LibraryGrid.vue<style scoped>",
    "Elephant/frontend/app/styles/runtime-layout-fixes.css",
];
const NOTE_CARD_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/library/NoteCard.vue<style scoped>",
    "Elephant/frontend/app/styles/runtime-layout-fixes.css",
];
const NOTE_EDITOR_HOST_STYLES: &[&str] = &[
    "Elephant/frontend/app/components/editor/NoteEditorHost.vue",
    "Elephant/frontend/app/styles/app-shell.css",
    "Elephant/frontend/app/styles/runtime-layout-fixes.css",
];

const APP_SHELL_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-shell",
        surface: SemanticSurface::Class,
        value: "en-shell",
    },
    SemanticContract {
        selector: ".en-shell",
        surface: SemanticSurface::Class,
        value: "en-theme-{themeMode}",
    },
    SemanticContract {
        selector: ".en-shell",
        surface: SemanticSurface::Class,
        value: "en-theme-{themeClassId}",
    },
    SemanticContract {
        selector: ".en-mobile-icon-button:first",
        surface: SemanticSurface::AriaLabel,
        value: "Open navigation | Close navigation",
    },
    SemanticContract {
        selector: ".en-mobile-search",
        surface: SemanticSurface::TextLabel,
        value: "Search notes",
    },
    SemanticContract {
        selector: ".en-mobile-icon-button:last",
        surface: SemanticSurface::AriaLabel,
        value: "Settings",
    },
    SemanticContract {
        selector: ".en-mobile-scrim",
        surface: SemanticSurface::AriaLabel,
        value: "Close navigation",
    },
    SemanticContract {
        selector: "[data-sidebar-resizer]",
        surface: SemanticSurface::DataAttribute,
        value: "data-sidebar-resizer",
    },
    SemanticContract {
        selector: "[data-sidebar-resizer]",
        surface: SemanticSurface::Role,
        value: "separator",
    },
    SemanticContract {
        selector: "[data-sidebar-resizer]",
        surface: SemanticSurface::AriaLabel,
        value: "Resize sidebar",
    },
    SemanticContract {
        selector: ".en-mobile-fab",
        surface: SemanticSurface::AriaLabel,
        value: "Create",
    },
    SemanticContract {
        selector: ".en-mobile-fab",
        surface: SemanticSurface::Role,
        value: "aria-haspopup=menu; aria-expanded={open}",
    },
];
const APP_SHELL_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-shell",
        property: "height",
        resolved_value: "100vh / 100dvh",
        css_source: "AppShell.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-body",
        property: "grid-template-columns",
        resolved_value: "var(--en-sidebar-width) 0 minmax(0, 1fr)",
        css_source: "AppShell.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-shell",
        property: "--en-sidebar-width",
        resolved_value: "232px default; 184px..320px clamped and persisted",
        css_source: "AppShell.vue<script setup> normalizeSidebarWidth + shellStyle",
    },
    DimensionContract {
        selector: ".en-sidebar-resizer::before",
        property: "width",
        resolved_value: "12px hit area",
        css_source: "AppShell.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-sidebar-resizer::after",
        property: "width / height",
        resolved_value: "3px / 64px",
        css_source: "AppShell.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-mobile-scrim",
        property: "drawer",
        resolved_value: "--en-mobile-drawer-progress 0..1",
        css_source: "AppShell.vue<script setup>",
    },
];
const APP_SHELL_STATES: &[StateContract] = &[
    StateContract {
        name: "vault-empty",
        source_signal: "!store.hasVault",
        observable_effect: "renders EmptyVaultPicker instead of en-shell",
    },
    StateContract {
        name: "theme",
        source_signal: "themeMode/themeClassId + localStorage",
        observable_effect: "theme classes and CSS tokens change",
    },
    StateContract {
        name: "sidebar-visible",
        source_signal: "sidebarVisible",
        observable_effect: "sidebar grid column is present or zero",
    },
    StateContract {
        name: "mobile-drawer",
        source_signal: "isMobileShell/drawerProgress/drawerDragging",
        observable_effect: "mobile topbar, scrim and drawer transitions",
    },
    StateContract {
        name: "settings-open",
        source_signal: "isSettingsOpen",
        observable_effect: "SettingsPanel is mounted",
    },
    StateContract {
        name: "addon-zones",
        source_signal: "shellRightZones",
        observable_effect: "registered shell.right contributions render",
    },
];
const APP_SHELL_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "pointerdown/move/up/cancel on .en-shell",
        handler_or_effect: "track mobile drawer gesture",
        source_signal: "handleDrawerPointer*",
    },
    EventContract {
        trigger: "click mobile navigation/search/settings/scrim/create",
        handler_or_effect: "toggle drawer or open search/settings/create",
        source_signal: "toggleMobileSidebar/openSearch/openSettings/handleMobileCreate",
    },
    EventContract {
        trigger: "pointerdown + ArrowLeft/ArrowRight on resizer",
        handler_or_effect: "resize sidebar by 16px and persist",
        source_signal: "startResize/handleResizeKeydown",
    },
    EventContract {
        trigger: "keydown Cmd/Ctrl+F, Cmd/Ctrl+K, Cmd/Ctrl+R, Alt+ArrowLeft/Right",
        handler_or_effect: "find/search/sync/history navigation",
        source_signal: "handleShortcut",
    },
    EventContract {
        trigger: "resize/orientation/vault-files-changed/mt::tab-saved",
        handler_or_effect: "refresh responsive shell and visible vault state",
        source_signal: "onMounted listeners",
    },
];

const TOP_VAULT_BAR_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-topstrip",
        surface: SemanticSurface::Class,
        value: "en-topstrip",
    },
    SemanticContract {
        selector: "[data-tauri-drag-region]",
        surface: SemanticSurface::DataAttribute,
        value: "data-tauri-drag-region",
    },
    SemanticContract {
        selector: ".en-window-controls",
        surface: SemanticSurface::AriaLabel,
        value: "Window controls",
    },
    SemanticContract {
        selector: ".en-window-control:first",
        surface: SemanticSurface::AriaLabel,
        value: "Minimize window",
    },
    SemanticContract {
        selector: ".en-window-control:nth-child(2)",
        surface: SemanticSurface::AriaLabel,
        value: "Maximize window | Restore window",
    },
    SemanticContract {
        selector: ".en-window-control-close",
        surface: SemanticSurface::AriaLabel,
        value: "Close window",
    },
];
const TOP_VAULT_BAR_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-topstrip",
        property: "height",
        resolved_value: "28px",
        css_source: "app-shell-runtime-fixes.css (active override over 32px base)",
    },
    DimensionContract {
        selector: ".en-topstrip-nav",
        property: "left / top / width / height",
        resolved_value: "56px / 4px / 76px / 24px",
        css_source: "TopVaultBar.vue<style scoped> + app-shell-runtime-fixes.css",
    },
    DimensionContract {
        selector: ".en-topstrip-nav-macos",
        property: "left / top",
        resolved_value: "84px / 4px",
        css_source: "TopVaultBar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-window-control",
        property: "width / min-width / height",
        resolved_value: "46px / 46px / 100% (28px)",
        css_source: "TopVaultBar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-window-control-icon",
        property: "width / height",
        resolved_value: "15px / 15px",
        css_source: "TopVaultBar.vue<style scoped>",
    },
];
const TOP_VAULT_BAR_STATES: &[StateContract] = &[
    StateContract {
        name: "sidebar-visible",
        source_signal: "sidebarVisible prop",
        observable_effect: "topstrip receives en-topstrip-sidebar-hidden class when false",
    },
    StateContract {
        name: "macos",
        source_signal: "navigator platform/userAgent",
        observable_effect: "native window controls are hidden",
    },
    StateContract {
        name: "maximized",
        source_signal: "isMaximized",
        observable_effect: "maximize control label/icon changes",
    },
];
const TOP_VAULT_BAR_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "dblclick on .en-topstrip-drag",
        handler_or_effect: "toggle maximize",
        source_signal: "handleMaximizeClick",
    },
    EventContract {
        trigger: "click minimize/maximize/close",
        handler_or_effect: "invoke Tauri window API",
        source_signal: "getCurrentWindow().minimize/maximize/unmaximize/close",
    },
    EventContract {
        trigger: "window resized",
        handler_or_effect: "synchronize maximized state",
        source_signal: "onResized(syncWindowState)",
    },
];

const ICON_RAIL_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-rail",
        surface: SemanticSurface::AriaLabel,
        value: "Workspace navigation",
    },
    SemanticContract {
        selector: ".en-rail-icon",
        surface: SemanticSurface::AriaLabel,
        value: "{item.title}",
    },
    SemanticContract {
        selector: ".en-rail-vault",
        surface: SemanticSurface::AriaLabel,
        value: "{vaultTooltip}",
    },
    SemanticContract {
        selector: ".en-rail-icon:last",
        surface: SemanticSurface::AriaLabel,
        value: "Settings",
    },
    SemanticContract {
        selector: ".en-vault-menu",
        surface: SemanticSurface::TextLabel,
        value: "Vaults",
    },
    SemanticContract {
        selector: ".en-vault-menu-edit",
        surface: SemanticSurface::TextLabel,
        value: "Change vault icon",
    },
    SemanticContract {
        selector: ".en-vault-menu-add",
        surface: SemanticSurface::TextLabel,
        value: "Add another vault | Manage vaults",
    },
];
const ICON_RAIL_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-rail",
        property: "width / flex-basis",
        resolved_value: "56px / 56px",
        css_source: "runtime-layout-fixes.css (active override over 48px rules)",
    },
    DimensionContract {
        selector: ".en-rail",
        property: "padding-top",
        resolved_value: "8px desktop; 36px macOS",
        css_source: "IconRail.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-rail-icon",
        property: "width / height",
        resolved_value: "34px / 34px",
        css_source: "IconRail.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-rail-vault",
        property: "width / height",
        resolved_value: "34px / 34px",
        css_source: "IconRail.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-rail-icon-svg",
        property: "width / height",
        resolved_value: "18px / 18px",
        css_source: "IconRail.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-vault-menu",
        property: "left / min-width",
        resolved_value: "52px / 250px",
        css_source: "IconRail.vue<style scoped>",
    },
];
const ICON_RAIL_STATES: &[StateContract] = &[
    StateContract {
        name: "rail-item-active",
        source_signal: "item.active",
        observable_effect: "active class and soft background",
    },
    StateContract {
        name: "sidebar-toggle",
        source_signal: "item.id == sidebar-toggle",
        observable_effect: "neutral icon swaps to direction icon on hover",
    },
    StateContract {
        name: "vault-menu",
        source_signal: "showVaultMenu",
        observable_effect: "vault switcher is visible on hover",
    },
    StateContract {
        name: "addon-contributions",
        source_signal: "visibleRailItems",
        observable_effect: "registered rail items/separators and order render",
    },
];
const ICON_RAIL_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "click rail item/settings",
        handler_or_effect: "emit search/settings/sidebar/addon navigation",
        source_signal: "runRailItem/openSettings",
    },
    EventContract {
        trigger: "dragstart/dragover/drop/dragend rail item",
        handler_or_effect: "reorder rail contribution",
        source_signal: "startRailDrag/allowRailDrop/dropRailItem/finishRailDrag",
    },
    EventContract {
        trigger: "click vault/select/edit/add/manage",
        handler_or_effect: "switch vault or update vault configuration",
        source_signal: "switchVault/toggleIconPicker/addVault/openVaultSettings",
    },
];

const SIDEBAR_NAV_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-sidebar",
        surface: SemanticSurface::Class,
        value: "en-sidebar",
    },
    SemanticContract {
        selector: ".en-all-notes",
        surface: SemanticSurface::TextLabel,
        value: "All notes",
    },
    SemanticContract {
        selector: ".en-tags-label",
        surface: SemanticSurface::TextLabel,
        value: "Notes",
    },
    SemanticContract {
        selector: ".en-tags-search-btn",
        surface: SemanticSurface::AriaLabel,
        value: "Search notes",
    },
    SemanticContract {
        selector: ".en-sidebar-main",
        surface: SemanticSurface::Class,
        value: "en-sidebar-main; recursive SidebarTreeEntry mount point",
    },
];
const SIDEBAR_NAV_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-sidebar",
        property: "min-width",
        resolved_value: "0px; parent controls 184px..320px",
        css_source: "SidebarNav.vue<style scoped> + AppShell shellStyle",
    },
    DimensionContract {
        selector: ".en-sidebar-scroll",
        property: "padding / overflow",
        resolved_value: "8px 0 0 / auto",
        css_source: "SidebarNav.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-all-notes",
        property: "min-height / margin / horizontal padding",
        resolved_value: "38px / 0 8px 8px / 12px",
        css_source: "SidebarNav.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-tags-search-btn",
        property: "width / height",
        resolved_value: "24px / 24px",
        css_source: "SidebarNav.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-sidebar-main",
        property: "gap / horizontal padding",
        resolved_value: "2px / 6px",
        css_source: "SidebarNav.vue<style scoped>",
    },
];
const SIDEBAR_NAV_STATES: &[StateContract] = &[
    StateContract {
        name: "all-notes-active",
        source_signal: "activeWorkspaceView == notes && currentPath == '' && !openedNotePath",
        observable_effect: "active class on All notes",
    },
    StateContract {
        name: "root-drop-target",
        source_signal: "isRootDropTarget/isRootDropDisabled",
        observable_effect: "root outline and dropEffect change",
    },
    StateContract {
        name: "visible-tree",
        source_signal: "filterSidebarEntries(rootSidebarEntries)",
        observable_effect: "hidden paths and non-folder/non-markdown entries are omitted",
    },
    StateContract {
        name: "addon-after-tree",
        source_signal: "sidebarAfterTreeZones",
        observable_effect: "registered sidebar.after-tree contributions render",
    },
];
const SIDEBAR_NAV_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "click All notes",
        handler_or_effect: "close addon view and open root directory",
        source_signal: "openAllNotes",
    },
    EventContract {
        trigger: "click Search notes",
        handler_or_effect: "emit search",
        source_signal: "emit('search')",
    },
    EventContract {
        trigger: "dragover/dragleave/drop root",
        handler_or_effect: "validate and move dragged entry to root",
        source_signal: "handleRootDragOver/Leave/Drop",
    },
    EventContract {
        trigger: "click recursive SidebarTreeEntry",
        handler_or_effect: "open directory or note",
        source_signal: "openDirectory/openNote",
    },
];

const MAIN_CONTENT_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-main",
        surface: SemanticSurface::Class,
        value: "en-main",
    },
    SemanticContract {
        selector: ".en-main",
        surface: SemanticSurface::Class,
        value: "has-editor-open",
    },
    SemanticContract {
        selector: ".en-library",
        surface: SemanticSurface::Class,
        value: "en-library",
    },
    SemanticContract {
        selector: ".en-main-editor",
        surface: SemanticSurface::Class,
        value: "en-main-editor",
    },
];
const MAIN_CONTENT_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-main",
        property: "min-width / min-height",
        resolved_value: "0px / 0px",
        css_source: "MainContent.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-main",
        property: "display / flex-direction",
        resolved_value: "flex / column",
        css_source: "MainContent.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library",
        property: "flex / overflow",
        resolved_value: "1 / hidden",
        css_source: "MainContent.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-main-editor",
        property: "min-height / flex",
        resolved_value: "0px / 1",
        css_source: "MainContent.vue<style scoped>",
    },
];
const MAIN_CONTENT_STATES: &[StateContract] = &[
    StateContract {
        name: "addon-workspace",
        source_signal: "!hasOpenNote && activeAddonViewId",
        observable_effect: "AddonWorkspaceRouter is rendered",
    },
    StateContract {
        name: "library",
        source_signal: "!hasOpenNote && showLibrary",
        observable_effect: "LibraryToolbar and LibraryGrid are rendered",
    },
    StateContract {
        name: "workspace-panels",
        source_signal: "!hasOpenNote && !activeAddonViewId && activeWorkspaceView == notes",
        observable_effect: "visible workspace.notes contributions render",
    },
    StateContract {
        name: "editor",
        source_signal: "hasOpenNote",
        observable_effect: "NoteEditorHost replaces the library",
    },
];
const MAIN_CONTENT_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "addon-workspace close",
        handler_or_effect: "clear active addon view",
        source_signal: "emit('close-addon-view')",
    },
    EventContract {
        trigger: "opened note/editor identity watcher",
        handler_or_effect: "log note visibility and tab state",
        source_signal: "watch(hasOpenNote, openedNoteAbsolutePath, tabs)",
    },
];

const LIBRARY_TOOLBAR_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-library-toolbar",
        surface: SemanticSurface::Class,
        value: "en-library-toolbar",
    },
    SemanticContract {
        selector: ".en-create-button",
        surface: SemanticSurface::AriaLabel,
        value: "Create",
    },
    SemanticContract {
        selector: ".en-create-button",
        surface: SemanticSurface::Role,
        value: "aria-haspopup=menu; aria-expanded={open}; aria-busy={isBusy}",
    },
    SemanticContract {
        selector: ".en-sort-cycle",
        surface: SemanticSurface::AriaLabel,
        value: "Sort: Updated newest | Sort: Updated oldest | Sort: Title A-Z | Sort: Title Z-A",
    },
    SemanticContract {
        selector: ".en-sort-cycle",
        surface: SemanticSurface::DataAttribute,
        value: "data-sort={updated-newest|updated-oldest|title-az|title-za}",
    },
    SemanticContract {
        selector: ".en-view-cycle",
        surface: SemanticSurface::AriaLabel,
        value: "Show notes as list | Show notes as grid",
    },
    SemanticContract {
        selector: ".en-view-cycle",
        surface: SemanticSurface::DataAttribute,
        value: "data-view-mode={grid|list}",
    },
    SemanticContract {
        selector: ".en-library-action-error",
        surface: SemanticSurface::Role,
        value: "alert",
    },
];
const LIBRARY_TOOLBAR_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-library-toolbar",
        property: "padding / gap",
        resolved_value: "10px 12px / 18px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-button",
        property: "width / height",
        resolved_value: "56px / 56px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-icon",
        property: "width / height",
        resolved_value: "27px / 27px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library-actions",
        property: "gap",
        resolved_value: "14px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-sort-cycle, .en-view-cycle",
        property: "width / height",
        resolved_value: "52px / 52px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-icon",
        property: "width / height",
        resolved_value: "22px / 22px",
        css_source: "LibraryToolbar.vue<style scoped>",
    },
];
const LIBRARY_TOOLBAR_STATES: &[StateContract] = &[
    StateContract {
        name: "busy",
        source_signal: "busyAction/isBusy",
        observable_effect: "create button disabled and aria-busy true",
    },
    StateContract {
        name: "action-error",
        source_signal: "actionError",
        observable_effect: "alert text appears beside Create",
    },
    StateContract {
        name: "sort-cycle",
        source_signal: "store.sort",
        observable_effect: "one of four sort labels/data-sort values",
    },
    StateContract {
        name: "view-cycle",
        source_signal: "store.viewMode",
        observable_effect: "grid/list icon, label and data-view-mode change",
    },
];
const LIBRARY_TOOLBAR_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "click Create and select menu item",
        handler_or_effect: "create note/folder or open drawing",
        source_signal: "toggle/handleCreateSelection",
    },
    EventContract {
        trigger: "click sort",
        handler_or_effect: "cycle updated/title ordering",
        source_signal: "cycleSort",
    },
    EventContract {
        trigger: "click view",
        handler_or_effect: "toggle grid/list",
        source_signal: "cycleView",
    },
];

const CREATE_ENTRY_MENU_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-create-menu",
        surface: SemanticSurface::Class,
        value: "en-create-menu",
    },
    SemanticContract {
        selector: ".en-create-menu-popover",
        surface: SemanticSurface::Role,
        value: "menu",
    },
    SemanticContract {
        selector: ".en-create-menu-popover",
        surface: SemanticSurface::AriaLabel,
        value: "Create",
    },
    SemanticContract {
        selector: ".en-create-menu-heading",
        surface: SemanticSurface::TextLabel,
        value: "Create",
    },
    SemanticContract {
        selector: ".en-create-menu-option",
        surface: SemanticSurface::Role,
        value: "menuitem",
    },
    SemanticContract {
        selector: ".en-create-menu-option:nth-child(1)",
        surface: SemanticSurface::TextLabel,
        value: "Note — Create a new note",
    },
    SemanticContract {
        selector: ".en-create-menu-option:nth-child(2)",
        surface: SemanticSurface::TextLabel,
        value: "Drawing — Open a new Excalidraw canvas",
    },
    SemanticContract {
        selector: ".en-create-menu-option:nth-child(3)",
        surface: SemanticSurface::TextLabel,
        value: "Folder — Organize notes in a folder",
    },
    SemanticContract {
        selector: "[data-testid=excalidraw-logo]",
        surface: SemanticSurface::DataTestId,
        value: "excalidraw-logo",
    },
    SemanticContract {
        selector: "[data-excalidraw-asset]",
        surface: SemanticSurface::DataAttribute,
        value: "shared-muya-icon",
    },
];
const CREATE_ENTRY_MENU_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-create-menu-popover",
        property: "width / padding / border-radius",
        resolved_value: "280px / 8px / 14px",
        css_source: "CreateEntryMenu.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-menu-option",
        property: "width / gap / padding",
        resolved_value: "100% / 12px / 10px",
        css_source: "CreateEntryMenu.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-menu-icon",
        property: "width / height",
        resolved_value: "20px / 20px",
        css_source: "CreateEntryMenu.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-menu-mobile",
        property: "right / bottom",
        resolved_value: "max(20px, safe-area-inset) / max(20px, safe-area-inset)",
        css_source: "CreateEntryMenu.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-create-menu-mobile .en-create-menu-popover",
        property: "bottom",
        resolved_value: "calc(100% + 10px)",
        css_source: "CreateEntryMenu.vue<style scoped>",
    },
];
const CREATE_ENTRY_MENU_STATES: &[StateContract] = &[
    StateContract {
        name: "closed",
        source_signal: "isOpen == false",
        observable_effect: "only trigger slot is mounted",
    },
    StateContract {
        name: "open",
        source_signal: "isOpen == true",
        observable_effect: "role=menu popover and three menuitems mount",
    },
    StateContract {
        name: "disabled",
        source_signal: "props.disabled",
        observable_effect: "toggle and trigger selection are inert",
    },
    StateContract {
        name: "mobile",
        source_signal: "props.mobile",
        observable_effect: "fixed FAB anchoring and upward popover",
    },
];
const CREATE_ENTRY_MENU_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "trigger click",
        handler_or_effect: "toggle isOpen unless disabled",
        source_signal: "toggle",
    },
    EventContract {
        trigger: "menuitem click",
        handler_or_effect: "close menu and emit selected key",
        source_signal: "select",
    },
    EventContract {
        trigger: "document pointerdown outside",
        handler_or_effect: "close menu",
        source_signal: "closeOnOutsidePointer",
    },
    EventContract {
        trigger: "document Escape",
        handler_or_effect: "close menu",
        source_signal: "closeOnEscape",
    },
];

const LIBRARY_GRID_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-library-grid",
        surface: SemanticSurface::Class,
        value: "en-library-grid",
    },
    SemanticContract {
        selector: ".en-library-grid",
        surface: SemanticSurface::DataAttribute,
        value: "data-view-mode={grid|list}",
    },
    SemanticContract {
        selector: ".en-library-grid-surface--grid",
        surface: SemanticSurface::DataAttribute,
        value: "data-layout=grid",
    },
    SemanticContract {
        selector: ".en-library-grid-surface--list",
        surface: SemanticSurface::DataAttribute,
        value: "data-layout=list",
    },
];
const LIBRARY_GRID_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-library-grid",
        property: "desktop padding-top",
        resolved_value: "72px (viewport min-width 761px)",
        css_source: "LibraryGrid.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library-grid-surface",
        property: "padding",
        resolved_value: "0 10px 10px",
        css_source: "LibraryGrid.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library-grid-surface--grid",
        property: "columns / gap",
        resolved_value: "repeat(auto-fill, minmax(min(240px, 100%), 1fr)) / 10px",
        css_source: "LibraryGrid.vue<style scoped> + runtime-layout-fixes.css",
    },
    DimensionContract {
        selector: ".en-library-grid-surface--list",
        property: "gap",
        resolved_value: "6px",
        css_source: "LibraryGrid.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library-grid",
        property: "overflow-x",
        resolved_value: "hidden",
        css_source: "runtime-layout-fixes.css",
    },
    DimensionContract {
        selector: ".en-library-grid",
        property: "directory page / render chunk / prefetch",
        resolved_value: "120 entries / 72 entries / 720px",
        css_source: "LibraryGrid.vue<script setup>",
    },
];
const LIBRARY_GRID_STATES: &[StateContract] = &[
    StateContract {
        name: "grid",
        source_signal: "store.viewMode == grid",
        observable_effect: "grid surface and featured first card render",
    },
    StateContract {
        name: "list",
        source_signal: "store.viewMode == list",
        observable_effect: "list surface and compact cards render",
    },
    StateContract {
        name: "empty",
        source_signal: "!visibleEntries.length",
        observable_effect: "is-empty class and empty surface",
    },
    StateContract {
        name: "root-drop",
        source_signal: "isRootDropTarget/isRootDropDisabled",
        observable_effect: "outline and move acceptance change",
    },
    StateContract {
        name: "paged",
        source_signal: "visibleEntryLimit/directoryMayHaveMore/loadingMoreEntries",
        observable_effect: "72-item render chunks and 120-item directory pages",
    },
];
const LIBRARY_GRID_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "scroll within grid",
        handler_or_effect: "prefetch/load next page at 720px from bottom",
        source_signal: "handleGridScroll/loadMoreVisibleEntries",
    },
    EventContract {
        trigger: "dragover/dragleave/drop grid root",
        handler_or_effect: "validate and move entry to current directory",
        source_signal: "handleRootDragOver/Leave/Drop",
    },
    EventContract {
        trigger: "NoteCard open/rename/delete",
        handler_or_effect: "open folder/note/drawing or mutate entry",
        source_signal: "openEntry/renameEntry/deleteEntry",
    },
];

const NOTE_CARD_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-card.en-note-card",
        surface: SemanticSurface::Class,
        value: "en-card en-note-card",
    },
    SemanticContract {
        selector: ".en-card-pin-button",
        surface: SemanticSurface::AriaLabel,
        value: "Pin entry | Unpin entry",
    },
    SemanticContract {
        selector: ".en-card-menu",
        surface: SemanticSurface::AriaLabel,
        value: "Note actions | Folder actions",
    },
    SemanticContract {
        selector: "[data-entry-action=rename]",
        surface: SemanticSurface::AriaLabel,
        value: "Rename",
    },
    SemanticContract {
        selector: "[data-entry-action=sidebar]",
        surface: SemanticSurface::AriaLabel,
        value: "Hide from sidebar | Show in sidebar",
    },
    SemanticContract {
        selector: "[data-entry-action=delete]",
        surface: SemanticSurface::AriaLabel,
        value: "Delete",
    },
    SemanticContract {
        selector: "[data-entry-rename-input]",
        surface: SemanticSurface::AriaLabel,
        value: "Rename {title}",
    },
    SemanticContract {
        selector: ".en-folder-preview",
        surface: SemanticSurface::AriaLabel,
        value: "Folder contents preview | Empty folder",
    },
    SemanticContract {
        selector: ".en-note-card-drawing-preview img",
        surface: SemanticSurface::DataAttribute,
        value: "data-elephant-excalidraw-preview=true",
    },
    SemanticContract {
        selector: ".en-note-card-drawing-preview img",
        surface: SemanticSurface::TextLabel,
        value: "{title} preview (alt)",
    },
    SemanticContract {
        selector: "[data-entry-action=rename]",
        surface: SemanticSurface::DataAttribute,
        value: "data-entry-action=rename",
    },
    SemanticContract {
        selector: "[data-entry-action=delete]",
        surface: SemanticSurface::DataAttribute,
        value: "data-entry-action=delete",
    },
];
const NOTE_CARD_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-card",
        property: "min-height / padding / border-radius",
        resolved_value: "176px / 10px / 10px",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-card.is-featured",
        property: "min-height",
        resolved_value: "220px (folders remain 176px)",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-library-grid.list .en-note-card",
        property: "min-height",
        resolved_value: "58px",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-card-pin-button, .en-card-menu",
        property: "width / height",
        resolved_value: "30px / 30px",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-card-popover button",
        property: "width / height",
        resolved_value: "32px / 32px",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-note-document-icon",
        property: "width / height",
        resolved_value: "22px / 22px (20px / 20px list)",
        css_source: "NoteCard.vue<style scoped>",
    },
    DimensionContract {
        selector: ".en-note-card-drawing-preview",
        property: "height",
        resolved_value: "112px (156px featured)",
        css_source: "NoteCard.vue<style scoped>",
    },
];
const NOTE_CARD_STATES: &[StateContract] = &[
    StateContract {
        name: "featured",
        source_signal: "featured prop",
        observable_effect: "220px card unless folder",
    },
    StateContract {
        name: "pinned",
        source_signal: "isPinned",
        observable_effect: "yellow filled pin and is-pinned class",
    },
    StateContract {
        name: "folder/drawing/note",
        source_signal: "entry kind",
        observable_effect: "folder preview, drawing image or excerpt",
    },
    StateContract {
        name: "menu",
        source_signal: "isMenuOpen",
        observable_effect: "rename/sidebar/delete popover actions mount",
    },
    StateContract {
        name: "renaming",
        source_signal: "isRenaming",
        observable_effect: "input replaces title and Enter/Escape are active",
    },
    StateContract {
        name: "drag",
        source_signal: "isDragging/isDropTarget/isDropDisabled",
        observable_effect: "opacity/border/drop acceptance change",
    },
];
const NOTE_CARD_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "mouseenter/mouseleave/click/dblclick/contextmenu",
        handler_or_effect: "hover, open menu, open entry or begin rename",
        source_signal: "handleCardClick/toggleMenu/openContextMenu/beginRename",
    },
    EventContract {
        trigger: "click pin/sidebar/delete/rename",
        handler_or_effect: "toggle or emit entry mutation",
        source_signal: "togglePin/toggleSidebarVisibility/deleteEntry/commitRename",
    },
    EventContract {
        trigger: "dragstart/dragover/dragleave/drop/dragend",
        handler_or_effect: "write, validate and move dragged entry",
        source_signal: "handleDragStart/Over/Leave/Drop/End",
    },
    EventContract {
        trigger: "keydown Enter/Escape in rename input",
        handler_or_effect: "commit or cancel rename",
        source_signal: "commitRename/cancelRename",
    },
];

const NOTE_EDITOR_HOST_SEMANTICS: &[SemanticContract] = &[
    SemanticContract {
        selector: ".en-editor-layer",
        surface: SemanticSurface::Class,
        value: "en-editor-layer",
    },
    SemanticContract {
        selector: ".en-editor-panel",
        surface: SemanticSurface::Class,
        value: "en-editor-panel",
    },
    SemanticContract {
        selector: ".en-note-editor-shell",
        surface: SemanticSurface::DataAttribute,
        value: "data-entry-drop-target=note-editor",
    },
    SemanticContract {
        selector: ".en-editor-host",
        surface: SemanticSurface::Class,
        value: "en-editor-host; EditorWithTabs/Muya mount point",
    },
    SemanticContract {
        selector: ".en-note-editor-shell",
        surface: SemanticSurface::TextLabel,
        value: "No direct label; NoteEditorTopBar/Footer own child labels",
    },
];
const NOTE_EDITOR_HOST_DIMENSIONS: &[DimensionContract] = &[
    DimensionContract {
        selector: ".en-editor-layer",
        property: "width / height",
        resolved_value: "100% / 100%",
        css_source: "app-shell.css",
    },
    DimensionContract {
        selector: ".en-editor-panel",
        property: "height / display",
        resolved_value: "100% / flex column",
        css_source: "app-shell.css",
    },
    DimensionContract {
        selector: ".en-note-editor-shell",
        property: "min-height / flex",
        resolved_value: "0px / 1",
        css_source: "app-shell.css",
    },
    DimensionContract {
        selector: ".en-editor-host",
        property: "min-height / flex / overflow",
        resolved_value: "0px / 1 / hidden",
        css_source: "app-shell.css + runtime-layout-fixes.css",
    },
    DimensionContract {
        selector: ".en-note-toolbar",
        property: "min-height / horizontal padding",
        resolved_value: "56px / 24px",
        css_source: "app-shell.css (child editor toolbar contract)",
    },
];
const NOTE_EDITOR_HOST_STATES: &[StateContract] = &[
    StateContract {
        name: "editor-document",
        source_signal: "markdown/cursor/muyaIndexCursor",
        observable_effect: "EditorWithTabs receives the active document contract",
    },
    StateContract {
        name: "scrolled",
        source_signal: "isEditorScrolled",
        observable_effect: "NoteEditorTopBar receives compact=true after 24px",
    },
    StateContract {
        name: "tag-editing",
        source_signal: "isAddingTag/isEditingTag/tagDraft",
        observable_effect: "top bar tag controls change",
    },
    StateContract {
        name: "footer",
        source_signal: "showEditorFooter/isTypographyOpen",
        observable_effect: "footer and typography controls change",
    },
    StateContract {
        name: "autosave",
        source_signal: "markdown watcher + 500ms poll",
        observable_effect: "note write is scheduled/flushed with logs",
    },
];
const NOTE_EDITOR_HOST_EVENTS: &[EventContract] = &[
    EventContract {
        trigger: "dragover/drop .en-note-editor-shell",
        handler_or_effect: "insert dropped note/folder/drawing link into markdown",
        source_signal: "handleEditorDragOver/handleEditorDrop",
    },
    EventContract {
        trigger: "top bar update-title/toggle-pin/close/tag events",
        handler_or_effect: "mutate markdown, pin state, tags or close after flush",
        source_signal: "updateTitle/togglePin/closeOpenedNote/tag handlers",
    },
    EventContract {
        trigger: "editor markdown changes",
        handler_or_effect: "schedule or flush real vault save",
        source_signal: "updateCurrentFileMarkdown/scheduleNoteSave/persistNoteMarkdown",
    },
    EventContract {
        trigger: "editor footer typography/theme",
        handler_or_effect: "change text scale or shell theme",
        source_signal: "setTextScale/toggleTheme",
    },
];

pub const APP_SHELL: ComponentContract = ComponentContract {
    id: ComponentId::AppShell,
    provenance: Provenance {
        component: ComponentId::AppShell,
        source_path: "Elephant/frontend/app/components/shell/AppShell.vue",
        template_anchor: "<div class=\"en-shell\">",
        script_anchor: "shellStyle/updateMobileShell/handleShortcut",
        style_sources: APP_SHELL_STYLES,
    },
    semantics: APP_SHELL_SEMANTICS,
    dimensions: APP_SHELL_DIMENSIONS,
    states: APP_SHELL_STATES,
    events: APP_SHELL_EVENTS,
};
pub const TOP_VAULT_BAR: ComponentContract = ComponentContract {
    id: ComponentId::TopVaultBar,
    provenance: Provenance {
        component: ComponentId::TopVaultBar,
        source_path: "Elephant/frontend/app/components/shell/TopVaultBar.vue",
        template_anchor: "<header class=\"en-topstrip\">",
        script_anchor: "syncWindowState/handleMinimizeClick/handleMaximizeClick/handleCloseClick",
        style_sources: TOP_VAULT_BAR_STYLES,
    },
    semantics: TOP_VAULT_BAR_SEMANTICS,
    dimensions: TOP_VAULT_BAR_DIMENSIONS,
    states: TOP_VAULT_BAR_STATES,
    events: TOP_VAULT_BAR_EVENTS,
};
pub const ICON_RAIL: ComponentContract = ComponentContract {
    id: ComponentId::IconRail,
    provenance: Provenance {
        component: ComponentId::IconRail,
        source_path: "Elephant/frontend/app/components/navigation/IconRail.vue",
        template_anchor: "<nav class=\"en-rail\">",
        script_anchor: "visibleRailItems/runRailItem/startRailDrag/dropRailItem",
        style_sources: ICON_RAIL_STYLES,
    },
    semantics: ICON_RAIL_SEMANTICS,
    dimensions: ICON_RAIL_DIMENSIONS,
    states: ICON_RAIL_STATES,
    events: ICON_RAIL_EVENTS,
};
pub const SIDEBAR_NAV: ComponentContract = ComponentContract {
    id: ComponentId::SidebarNav,
    provenance: Provenance {
        component: ComponentId::SidebarNav,
        source_path: "Elephant/frontend/app/components/navigation/SidebarNav.vue",
        template_anchor: "<aside class=\"en-sidebar\">",
        script_anchor: "sidebarEntries/openAllNotes/handleRootDrop",
        style_sources: SIDEBAR_NAV_STYLES,
    },
    semantics: SIDEBAR_NAV_SEMANTICS,
    dimensions: SIDEBAR_NAV_DIMENSIONS,
    states: SIDEBAR_NAV_STATES,
    events: SIDEBAR_NAV_EVENTS,
};
pub const MAIN_CONTENT: ComponentContract = ComponentContract {
    id: ComponentId::MainContent,
    provenance: Provenance {
        component: ComponentId::MainContent,
        source_path: "Elephant/frontend/app/components/shell/MainContent.vue",
        template_anchor: "<main class=\"en-main\">",
        script_anchor: "hasOpenNote/showLibrary/workspacePanels",
        style_sources: MAIN_CONTENT_STYLES,
    },
    semantics: MAIN_CONTENT_SEMANTICS,
    dimensions: MAIN_CONTENT_DIMENSIONS,
    states: MAIN_CONTENT_STATES,
    events: MAIN_CONTENT_EVENTS,
};
pub const LIBRARY_TOOLBAR: ComponentContract = ComponentContract {
    id: ComponentId::LibraryToolbar,
    provenance: Provenance {
        component: ComponentId::LibraryToolbar,
        source_path: "Elephant/frontend/app/components/library/LibraryToolbar.vue",
        template_anchor: "<div class=\"en-library-toolbar\">",
        script_anchor: "sortOptions/cycleSort/cycleView/handleCreateSelection",
        style_sources: LIBRARY_TOOLBAR_STYLES,
    },
    semantics: LIBRARY_TOOLBAR_SEMANTICS,
    dimensions: LIBRARY_TOOLBAR_DIMENSIONS,
    states: LIBRARY_TOOLBAR_STATES,
    events: LIBRARY_TOOLBAR_EVENTS,
};
pub const CREATE_ENTRY_MENU: ComponentContract = ComponentContract {
    id: ComponentId::CreateEntryMenu,
    provenance: Provenance {
        component: ComponentId::CreateEntryMenu,
        source_path: "Elephant/frontend/app/components/library/CreateEntryMenu.vue",
        template_anchor: "<div class=\"en-create-menu-popover\" role=\"menu\">",
        script_anchor: "entries/toggle/select/closeOnOutsidePointer/closeOnEscape",
        style_sources: CREATE_ENTRY_MENU_STYLES,
    },
    semantics: CREATE_ENTRY_MENU_SEMANTICS,
    dimensions: CREATE_ENTRY_MENU_DIMENSIONS,
    states: CREATE_ENTRY_MENU_STATES,
    events: CREATE_ENTRY_MENU_EVENTS,
};
pub const LIBRARY_GRID: ComponentContract = ComponentContract {
    id: ComponentId::LibraryGrid,
    provenance: Provenance {
        component: ComponentId::LibraryGrid,
        source_path: "Elephant/frontend/app/components/library/LibraryGrid.vue",
        template_anchor: "<div class=\"en-library-grid\">",
        script_anchor: "fetchDirectoryPage/loadMoreVisibleEntries/openEntry",
        style_sources: LIBRARY_GRID_STYLES,
    },
    semantics: LIBRARY_GRID_SEMANTICS,
    dimensions: LIBRARY_GRID_DIMENSIONS,
    states: LIBRARY_GRID_STATES,
    events: LIBRARY_GRID_EVENTS,
};
pub const NOTE_CARD: ComponentContract = ComponentContract {
    id: ComponentId::NoteCard,
    provenance: Provenance {
        component: ComponentId::NoteCard,
        source_path: "Elephant/frontend/app/components/library/NoteCard.vue",
        template_anchor: "<article class=\"en-card en-note-card\">",
        script_anchor: "handleCardClick/toggleMenu/beginRename/handleDragStart",
        style_sources: NOTE_CARD_STYLES,
    },
    semantics: NOTE_CARD_SEMANTICS,
    dimensions: NOTE_CARD_DIMENSIONS,
    states: NOTE_CARD_STATES,
    events: NOTE_CARD_EVENTS,
};
pub const NOTE_EDITOR_HOST: ComponentContract = ComponentContract {
    id: ComponentId::NoteEditorHost,
    provenance: Provenance {
        component: ComponentId::NoteEditorHost,
        source_path: "Elephant/frontend/app/components/editor/NoteEditorHost.vue",
        template_anchor: "<section class=\"en-editor-panel\">",
        script_anchor: "persistNoteMarkdown/handleEditorDrop/updateCurrentFileMarkdown",
        style_sources: NOTE_EDITOR_HOST_STYLES,
    },
    semantics: NOTE_EDITOR_HOST_SEMANTICS,
    dimensions: NOTE_EDITOR_HOST_DIMENSIONS,
    states: NOTE_EDITOR_HOST_STATES,
    events: NOTE_EDITOR_HOST_EVENTS,
};

/// The exact active component set. Removing an entry is a contract break.
pub const ACTIVE_COMPONENT_CONTRACTS: &[ComponentContract] = &[
    APP_SHELL,
    TOP_VAULT_BAR,
    ICON_RAIL,
    SIDEBAR_NAV,
    MAIN_CONTENT,
    LIBRARY_TOOLBAR,
    CREATE_ENTRY_MENU,
    LIBRARY_GRID,
    NOTE_CARD,
    NOTE_EDITOR_HOST,
];

/// Source provenance for every component in [`ACTIVE_COMPONENT_CONTRACTS`].
pub const SOURCE_PROVENANCE: &[Provenance] = &[
    APP_SHELL.provenance,
    TOP_VAULT_BAR.provenance,
    ICON_RAIL.provenance,
    SIDEBAR_NAV.provenance,
    MAIN_CONTENT.provenance,
    LIBRARY_TOOLBAR.provenance,
    CREATE_ENTRY_MENU.provenance,
    LIBRARY_GRID.provenance,
    NOTE_CARD.provenance,
    NOTE_EDITOR_HOST.provenance,
];

pub fn contract(id: ComponentId) -> Option<&'static ComponentContract> {
    ACTIVE_COMPONENT_CONTRACTS
        .iter()
        .find(|entry| entry.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_active_components_are_present_exactly_once() {
        assert_eq!(
            ACTIVE_COMPONENT_CONTRACTS.len(),
            ComponentId::ALL.len(),
            "a required source contract was removed"
        );
        for required in ComponentId::ALL {
            let matches = ACTIVE_COMPONENT_CONTRACTS
                .iter()
                .filter(|entry| entry.id == required)
                .count();
            assert_eq!(matches, 1, "missing or duplicated {:?} contract", required);
        }
    }

    #[test]
    fn every_required_contract_has_traceable_provenance() {
        assert_eq!(SOURCE_PROVENANCE.len(), ComponentId::ALL.len());
        for provenance in SOURCE_PROVENANCE {
            assert_eq!(
                provenance.source_path,
                format_source_path(provenance.component)
            );
            assert!(!provenance.template_anchor.is_empty());
            assert!(!provenance.script_anchor.is_empty());
            assert!(!provenance.style_sources.is_empty());
            for style_source in provenance.style_sources {
                assert!(!style_source.is_empty());
            }
        }
    }

    #[test]
    fn every_required_contract_declares_observable_surface() {
        for entry in ACTIVE_COMPONENT_CONTRACTS {
            assert!(
                !entry.semantics.is_empty(),
                "{:?} has no labels/selectors",
                entry.id
            );
            assert!(
                !entry.dimensions.is_empty(),
                "{:?} has no dimensions",
                entry.id
            );
            assert!(!entry.states.is_empty(), "{:?} has no states", entry.id);
            assert!(!entry.events.is_empty(), "{:?} has no events", entry.id);
        }
    }

    #[test]
    fn required_accessibility_and_test_hooks_are_locked() {
        let toolbar = contract(ComponentId::LibraryToolbar).expect("toolbar contract");
        assert!(has_value(toolbar, SemanticSurface::AriaLabel, "Create"));
        assert!(has_value(
            toolbar,
            SemanticSurface::DataAttribute,
            "data-sort={updated-newest|updated-oldest|title-az|title-za}"
        ));

        let create_menu = contract(ComponentId::CreateEntryMenu).expect("create menu contract");
        assert!(has_value(
            create_menu,
            SemanticSurface::DataTestId,
            "excalidraw-logo"
        ));
        assert!(has_value(create_menu, SemanticSurface::Role, "menuitem"));

        let shell = contract(ComponentId::AppShell).expect("shell contract");
        assert!(has_value(
            shell,
            SemanticSurface::AriaLabel,
            "Resize sidebar"
        ));
        assert!(has_value(
            shell,
            SemanticSurface::DataAttribute,
            "data-sidebar-resizer"
        ));
    }

    #[test]
    fn active_runtime_geometry_is_not_replaced_by_obsolete_values() {
        let topbar = contract(ComponentId::TopVaultBar).expect("topbar contract");
        assert!(has_dimension(topbar, ".en-topstrip", "height", "28px"));

        let rail = contract(ComponentId::IconRail).expect("rail contract");
        assert!(has_dimension(
            rail,
            ".en-rail",
            "width / flex-basis",
            "56px / 56px"
        ));

        let cards = contract(ComponentId::NoteCard).expect("card contract");
        assert!(has_dimension(
            cards,
            ".en-card",
            "min-height / padding / border-radius",
            "176px / 10px / 10px"
        ));
        assert!(has_dimension(
            cards,
            ".en-library-grid.list .en-note-card",
            "min-height",
            "58px"
        ));
    }

    fn has_value(contract: &ComponentContract, surface: SemanticSurface, value: &str) -> bool {
        contract
            .semantics
            .iter()
            .any(|item| item.surface == surface && item.value == value)
    }

    fn has_dimension(
        contract: &ComponentContract,
        selector: &str,
        property: &str,
        resolved_value: &str,
    ) -> bool {
        contract.dimensions.iter().any(|item| {
            item.selector == selector
                && item.property == property
                && item.resolved_value == resolved_value
        })
    }

    fn format_source_path(component: ComponentId) -> &'static str {
        match component {
            ComponentId::AppShell => "Elephant/frontend/app/components/shell/AppShell.vue",
            ComponentId::TopVaultBar => "Elephant/frontend/app/components/shell/TopVaultBar.vue",
            ComponentId::IconRail => "Elephant/frontend/app/components/navigation/IconRail.vue",
            ComponentId::SidebarNav => "Elephant/frontend/app/components/navigation/SidebarNav.vue",
            ComponentId::MainContent => "Elephant/frontend/app/components/shell/MainContent.vue",
            ComponentId::LibraryToolbar => {
                "Elephant/frontend/app/components/library/LibraryToolbar.vue"
            }
            ComponentId::CreateEntryMenu => {
                "Elephant/frontend/app/components/library/CreateEntryMenu.vue"
            }
            ComponentId::LibraryGrid => "Elephant/frontend/app/components/library/LibraryGrid.vue",
            ComponentId::NoteCard => "Elephant/frontend/app/components/library/NoteCard.vue",
            ComponentId::NoteEditorHost => {
                "Elephant/frontend/app/components/editor/NoteEditorHost.vue"
            }
        }
    }
}
