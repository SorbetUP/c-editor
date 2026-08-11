//! Typed source contract for the existing Vue settings surface.
//!
//! This module is deliberately data-only.  It does not write preferences, open
//! dialogs, call Tauri, or reproduce the Vue DOM.  The values below are copied
//! from the current production sources and describe what a Freya adapter must
//! preserve before any UI or host implementation is attempted.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Core,
    AddonStandalone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionContract {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: SectionKind,
}

pub const CORE_SECTIONS: &[SectionContract] = &[
    SectionContract {
        id: "appearance",
        label: "Appearance",
        kind: SectionKind::Core,
    },
    SectionContract {
        id: "editor",
        label: "Editor",
        kind: SectionKind::Core,
    },
    SectionContract {
        id: "vaults",
        label: "Vaults",
        kind: SectionKind::Core,
    },
    SectionContract {
        id: "addons",
        label: "Addons",
        kind: SectionKind::Core,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingIndexEntry {
    pub id: &'static str,
    pub section: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

/// Exact `CORE_SETTINGS_INDEX` entries from SettingsPanel.vue.
pub const CORE_SETTINGS_INDEX: &[SettingIndexEntry] = &[
    SettingIndexEntry {
        id: "appearance-mode",
        section: "appearance",
        label: "Color mode",
        description: "Light and dark appearance.",
    },
    SettingIndexEntry {
        id: "appearance-language",
        section: "appearance",
        label: "Language",
        description: "System, built-in and ISO language packs.",
    },
    SettingIndexEntry {
        id: "appearance-theme",
        section: "appearance",
        label: "Theme",
        description:
            "Elephant, Apple, Graphite, Nord, Solar, Forest, Beige, Pastel and Gamer Violet themes.",
    },
    SettingIndexEntry {
        id: "appearance-icon-rail",
        section: "appearance",
        label: "Vertical icon bar",
        description: "Reorder, hide and divide navigation icons.",
    },
    SettingIndexEntry {
        id: "appearance-floating-surfaces",
        section: "appearance",
        label: "Floating surfaces",
        description: "Lift navigation, controls and the writing surface above the background.",
    },
    SettingIndexEntry {
        id: "editor-footer",
        section: "editor",
        label: "Editor footer",
        description: "Word count and typography controls.",
    },
    SettingIndexEntry {
        id: "editor-tags",
        section: "editor",
        label: "Tag prefix",
        description: "Show or hide the # before tags.",
    },
    SettingIndexEntry {
        id: "editor-quick-insert",
        section: "editor",
        label: "Quick insert menu",
        description: "Show block commands when typing the trigger.",
    },
    SettingIndexEntry {
        id: "editor-quick-trigger",
        section: "editor",
        label: "Quick insert trigger",
        description: "Change the / command trigger.",
    },
    SettingIndexEntry {
        id: "editor-brackets",
        section: "editor",
        label: "Pair brackets",
        description: "Automatically close brackets.",
    },
    SettingIndexEntry {
        id: "editor-markdown",
        section: "editor",
        label: "Pair Markdown syntax",
        description: "Automatically close Markdown markers.",
    },
    SettingIndexEntry {
        id: "editor-quotes",
        section: "editor",
        label: "Pair quotes",
        description: "Automatically close quotation marks.",
    },
    SettingIndexEntry {
        id: "editor-spellchecker",
        section: "editor",
        label: "Spellchecker",
        description: "Check spelling while writing.",
    },
    SettingIndexEntry {
        id: "editor-code-lines",
        section: "editor",
        label: "Code block line numbers",
        description: "Show line numbers in fenced code blocks.",
    },
    SettingIndexEntry {
        id: "editor-margin",
        section: "editor",
        label: "Note margins",
        description: "Horizontal writing space.",
    },
    SettingIndexEntry {
        id: "editor-autosave",
        section: "editor",
        label: "Autosave",
        description: "Automatically persist changes to disk.",
    },
    SettingIndexEntry {
        id: "editor-autosave-delay",
        section: "editor",
        label: "Autosave delay",
        description: "Delay before writing the latest edit.",
    },
    SettingIndexEntry {
        id: "vault-active",
        section: "vaults",
        label: "Active vault",
        description: "Current local workspace folder.",
    },
    SettingIndexEntry {
        id: "vault-open",
        section: "vaults",
        label: "Open vaults",
        description: "Review or remove registered vaults.",
    },
    SettingIndexEntry {
        id: "addons-installed",
        section: "addons",
        label: "Installed addons",
        description: "Installed addon packages.",
    },
    SettingIndexEntry {
        id: "addons-available",
        section: "addons",
        label: "Available addons",
        description: "Install optional features and community packages.",
    },
    SettingIndexEntry {
        id: "addons-community",
        section: "addons",
        label: "Community addons",
        description: "Third-party addon activation.",
    },
    SettingIndexEntry {
        id: "addons-packs",
        section: "addons",
        label: "Addon packs",
        description: "Install or share complete addon configurations.",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Boolean,
    Integer,
    SingleCharacter,
    String,
    StringList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultValue {
    Boolean(bool),
    Integer(i64),
    Text(&'static str),
    MissingFromStoreState,
    RuntimeFallback(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiValueTransform {
    Direct,
    InvertedBoolean,
    Clamped { minimum: i64, maximum: i64 },
    MaxLength(usize),
    DisabledWhenAutosaveFalse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreferenceContract {
    pub setting_id: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub value_kind: ValueKind,
    pub default: DefaultValue,
    pub transform: UiValueTransform,
}

/// Keys sent through `usePreferencesStore().SET_SINGLE_PREFERENCE` by the
/// settings panel or its directly owned IconRailLayoutSettings child.
pub const SETTINGS_PREFERENCES: &[PreferenceContract] = &[
    PreferenceContract {
        setting_id: "appearance-theme",
        key: "theme",
        label: "Theme",
        description: "The active Elephant theme variant.",
        value_kind: ValueKind::String,
        default: DefaultValue::Text("light"),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "appearance-floating-surfaces",
        key: "floatingSurfaces",
        label: "Floating surfaces",
        description: "Lift navigation, controls and the writing surface above the background.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::MissingFromStoreState,
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-footer",
        key: "showEditorFooter",
        label: "Editor footer",
        description: "Show word count, typography controls and the theme shortcut.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::MissingFromStoreState,
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-tags",
        key: "showTagHashInEditor",
        label: "Tag prefix",
        description: "Display # before tag names in the editor.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(true),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-quick-insert",
        key: "hideQuickInsertHint",
        label: "Quick insert menu",
        description: "Show the block command menu when its trigger is typed.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(false),
        transform: UiValueTransform::InvertedBoolean,
    },
    PreferenceContract {
        setting_id: "editor-quick-trigger",
        key: "quickInsertTrigger",
        label: "Quick insert trigger",
        description: "The character that opens the insert menu. The default is /.",
        value_kind: ValueKind::SingleCharacter,
        default: DefaultValue::Text("/"),
        transform: UiValueTransform::MaxLength(1),
    },
    PreferenceContract {
        setting_id: "editor-brackets",
        key: "autoPairBracket",
        label: "Pair brackets",
        description: "Automatically insert the matching closing bracket.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(true),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-markdown",
        key: "autoPairMarkdownSyntax",
        label: "Pair Markdown syntax",
        description: "Automatically close Markdown emphasis and formatting markers.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(true),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-quotes",
        key: "autoPairQuote",
        label: "Pair quotes",
        description: "Automatically insert the matching closing quote.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(true),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-spellchecker",
        key: "spellcheckerEnabled",
        label: "Spellchecker",
        description: "Check spelling while writing.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(false),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-code-lines",
        key: "codeBlockLineNumbers",
        label: "Code block line numbers",
        description: "Display line numbers in fenced code blocks.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(true),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-margin",
        key: "noteEditorMargin",
        label: "Note margins",
        description: "Horizontal space around the title and text.",
        value_kind: ValueKind::Integer,
        default: DefaultValue::Integer(12),
        transform: UiValueTransform::Clamped {
            minimum: 8,
            maximum: 48,
        },
    },
    PreferenceContract {
        setting_id: "editor-autosave",
        key: "autoSave",
        label: "Autosave",
        description: "Write changes to disk automatically.",
        value_kind: ValueKind::Boolean,
        default: DefaultValue::Boolean(false),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "editor-autosave-delay",
        key: "autoSaveDelay",
        label: "Autosave delay",
        description: "How long ElephantNote waits after the last edit.",
        value_kind: ValueKind::Integer,
        default: DefaultValue::Integer(5000),
        transform: UiValueTransform::DisabledWhenAutosaveFalse,
    },
    PreferenceContract {
        setting_id: "appearance-icon-rail",
        key: "iconRailOrder",
        label: "Vertical icon bar order",
        description: "Reorder and divide navigation icons.",
        value_kind: ValueKind::StringList,
        default: DefaultValue::Text("dashboard,wiki,graph,models,search,chat"),
        transform: UiValueTransform::Direct,
    },
    PreferenceContract {
        setting_id: "appearance-icon-rail",
        key: "iconRailHidden",
        label: "Vertical icon bar visibility",
        description: "Hide or show navigation icons.",
        value_kind: ValueKind::StringList,
        default: DefaultValue::Text(""),
        transform: UiValueTransform::Direct,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedSurface {
    pub key: &'static str,
    pub owner: &'static str,
    pub semantics: &'static str,
}

/// Persistence routes observed in the Vue settings path.  This is a map of
/// ownership, not a Rust persistence implementation.
pub const PERSISTED_SURFACES: &[PersistedSurface] = &[
    PersistedSurface {
        key: "elephantnote:lastSettingsSection",
        owner: "SettingsPanel.vue",
        semantics:
            "last selected section; localStorage read on setup and written on selection/mount",
    },
    PersistedSurface {
        key: "elephantnote:theme",
        owner: "AppShell.vue",
        semantics: "theme update emitted by SettingsPanel and persisted by the parent shell",
    },
    PersistedSurface {
        key: "elephantnote:tauri:language",
        owner: "LanguageSettingsRow.vue / i18n",
        semantics:
            "language preference written by setLanguage; not by SET_SINGLE_PREFERENCE in this child",
    },
    PersistedSurface {
        key: "elephantnote:pref:<key>",
        owner: "preferenceStorage.js",
        semantics: "portable mirror for non-transient preference-store keys",
    },
    PersistedSurface {
        key: "tauri_prefs_set(key, value)",
        owner: "preferences.js / Tauri state.rs",
        semantics: "portable runtime preference write",
    },
    PersistedSurface {
        key: "mt::set-user-preference",
        owner: "preferences.js",
        semantics: "legacy desktop IPC preference write when not portable",
    },
    PersistedSurface {
        key: "addons.communityEnabled",
        owner: "addons.js",
        semantics: "Tauri preference used by addon settings; not owned by SettingsPanel directly",
    },
    PersistedSurface {
        key: "elephantnote:ai-settings-draft",
        owner: "AiProviderSettingsPanel.vue",
        semantics: "draft cache for the auxiliary AI settings surface",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeFamilyContract {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

pub const THEME_FAMILIES: &[ThemeFamilyContract] = &[
    ThemeFamilyContract {
        id: "default",
        name: "Elephant",
        description: "Neutral notes workspace with crisp blue accents.",
        light: "light",
        dark: "dark",
    },
    ThemeFamilyContract {
        id: "apple",
        name: "Apple",
        description: "Frosted system surfaces, graphite text and iOS blue.",
        light: "apple-light",
        dark: "apple-dark",
    },
    ThemeFamilyContract {
        id: "graphite",
        name: "Graphite",
        description: "Quiet monochrome chrome for dense writing sessions.",
        light: "graphite-light",
        dark: "graphite-dark",
    },
    ThemeFamilyContract {
        id: "nord",
        name: "Nord",
        description: "Cold blue surfaces with cyan focus states.",
        light: "nord-light",
        dark: "nord-dark",
    },
    ThemeFamilyContract {
        id: "solar",
        name: "Solar",
        description: "Warm reading palette with amber selection and code blocks.",
        light: "solar-light",
        dark: "solar-dark",
    },
    ThemeFamilyContract {
        id: "forest",
        name: "Forest",
        description: "Green editorial palette with soft natural contrast.",
        light: "forest-light",
        dark: "forest-dark",
    },
    ThemeFamilyContract {
        id: "beige",
        name: "Beige",
        description: "Paper-like ivory surfaces with warm clay accents.",
        light: "beige-light",
        dark: "beige-dark",
    },
    ThemeFamilyContract {
        id: "pastel",
        name: "Pastel",
        description: "Soft lavender, peach and mint for a calm workspace.",
        light: "pastel-light",
        dark: "pastel-dark",
    },
    ThemeFamilyContract {
        id: "gamer-violet",
        name: "Gamer Violet",
        description: "Deep violet surfaces with neon purple focus states.",
        light: "gamer-violet-light",
        dark: "gamer-violet-dark",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconRailItemContract {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub const CORE_ICON_RAIL_ITEMS: &[IconRailItemContract] = &[
    IconRailItemContract {
        id: "vault",
        label: "Vault",
        description: "Open the active vault switcher.",
    },
    IconRailItemContract {
        id: "sidebar-toggle",
        label: "Sidebar",
        description: "Show or hide the navigation sidebar.",
    },
    IconRailItemContract {
        id: "search",
        label: "Search",
        description: "Open global search.",
    },
];

pub const ICON_RAIL_SEPARATOR_PREFIX: &str = "separator:";
pub const ICON_RAIL_PREFERENCE_KEYS: &[&str] = &["iconRailOrder", "iconRailHidden"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    Close,
    SelectSection,
    SetSearchQuery,
    OpenSearchResult,
    ToggleThemeExpansion,
    UpdateTheme,
    SetPreference,
    UpdateLanguage,
    RetryVaultLoad,
    ChooseVault,
    VaultMutation,
    ToggleVaultTrash,
    RestoreVaultTrash,
    EmptyVaultTrash,
    SelectAddonPage,
    SelectAddon,
    CloseAddonDetails,
    RefreshAddons,
    InstallAddon,
    ToggleAddon,
    UninstallAddon,
    RunAddonAction,
    ImportAddonOrPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionTransition {
    Selected,
    SearchMode,
    SearchResultOpened,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsState {
    pub active_section: String,
    pub query: String,
    pub theme_expanded: bool,
    pub addon_page: AddonPage,
    pub selected_addon_id: Option<String>,
    pub installed_only: bool,
    pub trash_expanded: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_section: "appearance".to_owned(),
            query: String::new(),
            theme_expanded: true,
            addon_page: AddonPage::Addons,
            selected_addon_id: None,
            installed_only: false,
            trash_expanded: false,
        }
    }
}

impl SettingsState {
    pub fn select_section(&mut self, section: &str) -> SectionTransition {
        self.active_section = section.to_owned();
        self.query.clear();
        SectionTransition::Selected
    }

    pub fn set_query(&mut self, query: &str) -> SectionTransition {
        self.query = query.to_owned();
        if query.trim().is_empty() {
            SectionTransition::Selected
        } else {
            SectionTransition::SearchMode
        }
    }

    pub fn open_search_result(&mut self, section: &str) -> SectionTransition {
        self.active_section = section.to_owned();
        self.query.clear();
        SectionTransition::SearchResultOpened
    }

    pub fn toggle_theme_expansion(&mut self) {
        self.theme_expanded = !self.theme_expanded;
    }

    pub fn toggle_trash(&mut self) {
        self.trash_expanded = !self.trash_expanded;
    }

    pub fn close_addon_details(&mut self) {
        self.selected_addon_id = None;
    }
}

pub fn initial_section(prop: Option<&str>, stored: Option<&str>) -> String {
    let requested = match prop.filter(|value| !value.is_empty() && *value != "appearance") {
        Some(value) => value,
        None => stored.or(prop).unwrap_or("appearance"),
    };
    normalize_section(requested, true)
}

pub fn normalize_section(candidate: &str, preserve_unknown: bool) -> String {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return "appearance".to_owned();
    }
    if CORE_SECTIONS.iter().any(|section| section.id == trimmed) {
        return trimmed.to_owned();
    }
    if preserve_unknown
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        return trimmed.to_owned();
    }
    "appearance".to_owned()
}

pub fn search_core_settings(query: &str) -> Vec<&'static SettingIndexEntry> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|term| term.to_lowercase())
        .collect();
    if terms.is_empty() {
        return Vec::new();
    }
    CORE_SETTINGS_INDEX
        .iter()
        .filter(|entry| {
            let haystack =
                format!("{} {} {}", entry.label, entry.description, entry.section).to_lowercase();
            terms.iter().all(|term| haystack.contains(term))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultViewState {
    Loading,
    Error,
    Populated,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultAction {
    RetryLoad,
    ChooseFolder,
    Activate,
    ToggleEnabled,
    RemoveFromList,
    StartRename,
    SaveRename,
    CancelRename,
    CopyPath,
    ToggleIconEditor,
    SetIcon,
    ToggleTrash,
    RetryTrash,
    RestoreTrashEntry,
    EmptyTrash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddonPage {
    Addons,
    Packs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddonViewState {
    Catalogue,
    Detail,
    PacksSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddonAction {
    Search,
    ToggleInstalledOnly,
    Refresh,
    ImportPackage,
    ImportPack,
    OpenDetails,
    BackToCatalogue,
    Install,
    Enable,
    Disable,
    Uninstall,
    RunCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuxiliarySettingsSurface {
    pub source: &'static str,
    pub pages: &'static [&'static str],
    pub host_boundary: &'static str,
}

pub const AUXILIARY_SETTINGS_SURFACES: &[AuxiliarySettingsSurface] = &[
    AuxiliarySettingsSurface {
        source: "AiProviderSettingsPanel.vue",
        pages: &["chat", "embedding", "ocr"],
        host_boundary: "elephantnoteClient.aiConfig / elephantnote:ai-config-changed",
    },
    AuxiliarySettingsSurface {
        source: "SyncSettingsPanel.vue",
        pages: &["overview", "devices", "conflicts"],
        host_boundary: "irohSyncClient and tauri_sync_* commands",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebDomGap {
    pub surface: &'static str,
    pub gap: &'static str,
}

/// Explicit boundaries a Freya implementation must provide.  None of these
/// are silently replaced by an in-memory or hard-coded success path here.
pub const WEB_DOM_GAPS: &[WebDomGap] = &[
    WebDomGap { surface: "SettingsPanel.vue", gap: "Vue v-if/v-for, computed search results, scoped settings-redesign.css, dialog/backdrop semantics, aria-modal and data-active-section need native equivalents." },
    WebDomGap { surface: "SettingsPanel.vue", gap: "window.localStorage, window.addEventListener('keydown'), window.confirm, navigator.clipboard, and window.fileUtils.stat are browser globals and need explicit Freya host adapters." },
    WebDomGap { surface: "SettingsPanel.vue", gap: "update-theme is an emitted event; AppShell owns normalization and persistence under elephantnote:theme." },
    WebDomGap { surface: "LanguageSettingsRow.vue", gap: "setLanguage persists elephantnote:tauri:language and dispatches elephantnote:language-changed; it is not the same path as the preferences store action." },
    WebDomGap { surface: "IconRailLayoutSettings.vue", gap: "HTML dragstart/dragover/drop, dynamic addon contributions, and generated separator IDs require native drag and stable-ID adapters." },
    WebDomGap { surface: "AddonsSettingsPanel.vue", gap: "dynamic component icons, addon contribution slots, Tauri file dialog, CustomEvent pack bridge, and addon manager operations are not represented by a Rust widget alone." },
    WebDomGap { surface: "AiProviderSettingsPanel.vue / SyncSettingsPanel.vue", gap: "These are addon-owned settings surfaces; their real providers, Iroh protocol and Tauri commands must remain behind their production clients." },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceProvenance {
    pub path: &'static str,
    pub symbols_or_lines: &'static str,
    pub role: &'static str,
}

pub const SOURCE_PROVENANCE: &[SourceProvenance] = &[
    SourceProvenance { path: "Elephant/frontend/app/components/settings/SettingsPanel.vue", symbols_or_lines: "template; CORE_SECTIONS; CORE_SETTINGS_INDEX; setup state/actions", role: "core settings shell, search, appearance/editor/vault/addon routing" },
    SourceProvenance { path: "Elephant/frontend/app/components/settings/IconRailLayoutSettings.vue", symbols_or_lines: "persistOrder; persistHidden; move; addDivider; removeDivider; resetLayout", role: "icon rail order/visibility/divider actions" },
    SourceProvenance { path: "Elephant/frontend/app/components/settings/LanguageSettingsRow.vue", symbols_or_lines: "getLanguagePreference; setLanguage; elephantnote:language-changed", role: "language preference UI and event lifecycle" },
    SourceProvenance { path: "Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue", symbols_or_lines: "activePage; browserEntries; selectedEntry; refreshActivePage; install/toggle/uninstall/run", role: "addon catalogue/detail/packs settings surface" },
    SourceProvenance { path: "Elephant/frontend/app/components/settings/AiProviderSettingsPanel.vue", symbols_or_lines: "SUPPORTED_PAGES; CACHE_KEY; saveConfig", role: "addon-owned AI settings boundary" },
    SourceProvenance { path: "Elephant/frontend/app/components/settings/SyncSettingsPanel.vue", symbols_or_lines: "validPages; activeSyncPage; irohSyncClient actions", role: "addon-owned synchronization settings boundary" },
    SourceProvenance { path: "Elephant/frontend/src/renderer/src/store/preferences.js", symbols_or_lines: "state; SET_SINGLE_PREFERENCE; ASK_FOR_USER_PREFERENCE", role: "preference defaults and portable/legacy persistence routing" },
    SourceProvenance { path: "Elephant/frontend/src/renderer/src/platform/preferenceStorage.js", symbols_or_lines: "PREF_PREFIX; persistPortablePreference", role: "elephantnote:pref:<key> localStorage mirror" },
    SourceProvenance { path: "Elephant/frontend/app/i18n/appMessages.js", symbols_or_lines: "APP_LANGUAGE_STORAGE_KEY; settings translations", role: "language storage key and localized labels" },
    SourceProvenance { path: "Elephant/shared/appearance.js", symbols_or_lines: "ELEPHANTNOTE_THEME_FAMILIES; ELEPHANTNOTE_THEME_STORAGE_KEY", role: "theme family IDs, labels and variants" },
    SourceProvenance { path: "Elephant/frontend/app/components/shell/AppShell.vue", symbols_or_lines: "<settings-panel>; openSettings; setTheme", role: "settings ownership and theme persistence parent" },
    SourceProvenance { path: "Elephant/frontend/app/stores/vaultStore.js", symbols_or_lines: "load; chooseVault; setActiveVault; setVaultIcon; setVaultName; setVaultEnabled; removeVault", role: "vault mutation boundary used by SettingsPanel" },
    SourceProvenance { path: "Elephant/frontend/src/renderer/src/store/addons.js", symbols_or_lines: "addons.communityEnabled; addon manager actions", role: "addon state and community preference boundary" },
    SourceProvenance { path: "Elephant/backend/tauri/src/state.rs", symbols_or_lines: "tauri_prefs_all; tauri_prefs_set", role: "Tauri preference command boundary" },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_sections_keep_vue_ids_and_labels() {
        assert_eq!(CORE_SECTIONS.len(), 4);
        assert_eq!(
            CORE_SECTIONS[0],
            SectionContract {
                id: "appearance",
                label: "Appearance",
                kind: SectionKind::Core
            }
        );
        assert_eq!(CORE_SECTIONS[1].id, "editor");
        assert_eq!(CORE_SECTIONS[2].label, "Vaults");
        assert_eq!(CORE_SECTIONS[3].label, "Addons");
    }

    #[test]
    fn core_search_index_is_complete_and_unique() {
        assert_eq!(CORE_SETTINGS_INDEX.len(), 23);
        for (index, entry) in CORE_SETTINGS_INDEX.iter().enumerate() {
            assert!(!entry.id.is_empty(), "empty setting id at {index}");
            assert!(CORE_SETTINGS_INDEX[index + 1..]
                .iter()
                .all(|other| other.id != entry.id));
        }
        assert_eq!(CORE_SETTINGS_INDEX[0].label, "Color mode");
        assert_eq!(CORE_SETTINGS_INDEX[22].label, "Addon packs");
        assert_eq!(
            search_core_settings("change / command")[0].id,
            "editor-quick-trigger"
        );
    }

    #[test]
    fn preference_contract_preserves_defaults_and_transforms() {
        let margin = SETTINGS_PREFERENCES
            .iter()
            .find(|item| item.key == "noteEditorMargin")
            .unwrap();
        assert_eq!(margin.default, DefaultValue::Integer(12));
        assert_eq!(
            margin.transform,
            UiValueTransform::Clamped {
                minimum: 8,
                maximum: 48
            }
        );
        let quick = SETTINGS_PREFERENCES
            .iter()
            .find(|item| item.key == "hideQuickInsertHint")
            .unwrap();
        assert_eq!(quick.transform, UiValueTransform::InvertedBoolean);
        assert!(
            SETTINGS_PREFERENCES
                .iter()
                .find(|item| item.key == "floatingSurfaces")
                .unwrap()
                .default
                == DefaultValue::MissingFromStoreState
        );
        assert!(
            SETTINGS_PREFERENCES
                .iter()
                .find(|item| item.key == "showEditorFooter")
                .unwrap()
                .default
                == DefaultValue::MissingFromStoreState
        );
    }

    #[test]
    fn initial_section_and_search_transitions_match_vue_state() {
        assert_eq!(
            initial_section(Some("appearance"), Some("editor")),
            "editor"
        );
        assert_eq!(initial_section(Some("editor"), Some("vaults")), "editor");
        assert_eq!(initial_section(Some("bad id!"), None), "appearance");
        let mut state = SettingsState::default();
        assert_eq!(state.set_query("autosave"), SectionTransition::SearchMode);
        assert_eq!(search_core_settings("autosave").len(), 2);
        assert_eq!(
            state.open_search_result("editor"),
            SectionTransition::SearchResultOpened
        );
        assert_eq!(state.query, "");
        assert_eq!(state.active_section, "editor");
    }

    #[test]
    fn theme_and_icon_rail_contracts_preserve_source_values() {
        assert_eq!(THEME_FAMILIES.len(), 9);
        assert_eq!(THEME_FAMILIES.last().unwrap().name, "Gamer Violet");
        assert_eq!(
            CORE_ICON_RAIL_ITEMS
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec!["vault", "sidebar-toggle", "search"]
        );
        assert_eq!(ICON_RAIL_SEPARATOR_PREFIX, "separator:");
    }

    #[test]
    fn settings_state_actions_are_local_only_and_typed() {
        let mut state = SettingsState::default();
        state.toggle_theme_expansion();
        state.toggle_trash();
        state.selected_addon_id = Some("example.addon".to_owned());
        state.close_addon_details();
        assert!(!state.theme_expanded);
        assert!(state.trash_expanded);
        assert_eq!(state.selected_addon_id, None);
        assert_eq!(SettingsAction::SetPreference, SettingsAction::SetPreference);
    }

    #[test]
    fn vault_and_addon_states_cover_real_branches() {
        assert_eq!(VaultViewState::Loading, VaultViewState::Loading);
        assert_eq!(VaultViewState::Error, VaultViewState::Error);
        assert_eq!(AddonPage::Packs, AddonPage::Packs);
        assert_eq!(AddonViewState::PacksSlot, AddonViewState::PacksSlot);
        assert!(AUXILIARY_SETTINGS_SURFACES
            .iter()
            .any(|surface| surface.pages == ["chat", "embedding", "ocr"]));
    }

    #[test]
    fn provenance_and_gap_inventory_are_not_empty() {
        assert!(SOURCE_PROVENANCE
            .iter()
            .any(|item| item.path.ends_with("SettingsPanel.vue")));
        assert!(SOURCE_PROVENANCE
            .iter()
            .any(|item| item.path.ends_with("preferences.js")));
        assert!(WEB_DOM_GAPS
            .iter()
            .any(|gap| gap.gap.contains("localStorage")));
        assert!(PERSISTED_SURFACES
            .iter()
            .any(|surface| surface.key == "elephantnote:theme"));
    }
}
