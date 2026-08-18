//! Component-by-component Freya assembly for the active Elephant Vue shell.
//!
//! The state/effect boundary stays here. Renderers are split by the source
//! component ownership they convert: navigation, library, and editor view.

mod calendar_view;
mod canvas_view;
mod chat_view;
mod code_execution;
mod drawing;
mod editor_view;
mod explorer;
mod explorer_runtime;
mod graph_canvas;
mod library;
mod models_view;
mod navigation;
mod navigation_icons;
mod search_overlay_view;
mod settings;
mod settings_effects;
mod shell_gestures;
mod shell_history;
mod shell_preferences;
mod shell_runtime;
mod sync_view;
mod vault_picker;
mod vault_watch;
mod visual_transition;
mod wiki_view;

use freya::prelude::*;
use std::{env, path::PathBuf};

use crate::{
    editor::EditorDocument,
    library_contract::LibraryState,
    navigation_contract::{SidebarWidth, WorkspaceView},
    source_contracts::{self, ComponentId},
    theme,
    vault_adapter::{PageRequest, VaultAdapter, VaultPage},
    vault_registry::VaultRegistry,
};

use shell_gestures::{RailDragState, SidebarResizeState};
use shell_history::NavigationTarget;

#[derive(Clone, Debug)]
pub(super) struct ShellState {
    view: WorkspaceView,
    sidebar_visible: bool,
    mobile_navigation_open: bool,
    sidebar_width: SidebarWidth,
    vault: Option<VaultAdapter>,
    vault_registry: VaultRegistry,
    page: Option<VaultPage>,
    library: LibraryState,
    menu_open: bool,
    hovered_target: Option<String>,
    card_action_target: Option<String>,
    vault_menu_open: bool,
    search_open: bool,
    settings_open: bool,
    settings_target_section: Option<String>,
    editor: Option<EditorDocument>,
    editor_tag_draft: Option<String>,
    error: Option<String>,
    navigation_history: Vec<NavigationTarget>,
    navigation_index: usize,
    rail_order: Vec<String>,
    settings_revision: u64,
    rail_drag: Option<RailDragState>,
    rail_drop_target: Option<String>,
    sidebar_resize: Option<SidebarResizeState>,
    library_drag: library::LibraryCardDrag,
    drawing: Option<drawing::DrawingCanvasState>,
    drawing_path: Option<PathBuf>,
    canvas: Option<crate::canvas_runtime::CanvasRuntime>,
    calendar: calendar_view::CalendarState,
    chat: chat_view::ChatState,
    models: models_view::ModelsState,
    sync: crate::sync_adapter::SyncState,
    code_execution: code_execution::CodeExecutionState,
}

impl ShellState {
    fn empty() -> Self {
        Self {
            view: WorkspaceView::Notes,
            sidebar_visible: true,
            mobile_navigation_open: false,
            sidebar_width: SidebarWidth::default(),
            vault: None,
            vault_registry: VaultRegistry::default(),
            page: None,
            library: LibraryState::default(),
            menu_open: false,
            hovered_target: None,
            card_action_target: None,
            vault_menu_open: false,
            search_open: false,
            settings_open: false,
            settings_target_section: None,
            editor: None,
            editor_tag_draft: None,
            error: None,
            navigation_history: Vec::new(),
            navigation_index: 0,
            rail_order: shell_preferences::default_rail_order(),
            settings_revision: 0,
            rail_drag: None,
            rail_drop_target: None,
            sidebar_resize: None,
            library_drag: library::LibraryCardDrag::default(),
            drawing: None,
            drawing_path: None,
            canvas: None,
            calendar: calendar_view::CalendarState::default(),
            chat: chat_view::ChatState::default(),
            models: models_view::ModelsState::default(),
            sync: crate::sync_adapter::SyncState::default(),
            code_execution: code_execution::CodeExecutionState::default(),
        }
    }

    fn load() -> Self {
        // Keep the environment override for deterministic tests and developer
        // workflows, but it is no longer required for normal application use.
        if let Some(raw_root) = env::var_os("ELEPHANT_FREYA_VAULT") {
            return Self::load_root_with_registry(PathBuf::from(raw_root));
        }

        let registry = match VaultRegistry::load() {
            Ok(registry) => registry,
            Err(error) => {
                let mut state = Self::empty();
                state.error = Some(format!("Unable to load the vault registry: {error}"));
                return state;
            }
        };
        if let Some(root) = registry.active().map(|vault| PathBuf::from(&vault.path)) {
            return Self::load_root_with_registry(root);
        }
        match vault_picker::remembered_vault() {
            Ok(Some(root)) => Self::load_root_with_registry(root),
            Ok(None) => {
                let mut state = Self::empty();
                state.vault_registry = registry;
                state
            }
            Err(error) => {
                let mut state = Self::empty();
                state.vault_registry = registry;
                state.error = Some(format!(
                    "Unable to restore the previously selected vault: {error}"
                ));
                state
            }
        }
    }

    fn load_root_with_registry(root: PathBuf) -> Self {
        let mut registry = match VaultRegistry::load() {
            Ok(registry) => registry,
            Err(error) => {
                let mut state = Self::empty();
                state.error = Some(format!("Unable to load the vault registry: {error}"));
                return state;
            }
        };
        let mut loaded = shell_runtime::load_from_root(root);
        if let Some(canonical_root) = loaded
            .vault
            .as_ref()
            .map(|vault| vault.root().to_path_buf())
        {
            if let Err(error) = registry.add_or_activate(&canonical_root) {
                loaded.error = Some(format!("Unable to register the active vault: {error}"));
            } else if let Err(error) = registry.persist() {
                eprintln!("[freya][vault-registry] action=persist-failure error={error}");
            }
        }
        // Keep the registry available even when the selected path disappeared.
        // The recovery picker needs the other registered vaults to remain
        // actionable instead of forcing the user to locate them again.
        loaded.vault_registry = registry;
        loaded
    }

    fn load_root_without_persisted_registry(root: PathBuf) -> Self {
        let mut loaded = shell_runtime::load_from_root(root);
        let mut registry = VaultRegistry::default();
        if let Some(canonical_root) = loaded
            .vault
            .as_ref()
            .map(|vault| vault.root().to_path_buf())
        {
            if let Err(error) = registry.add_or_activate(&canonical_root) {
                loaded.error = Some(format!("Unable to register the active vault: {error}"));
            }
        }
        loaded.vault_registry = registry;
        loaded
    }

    fn open_vault(&mut self, root: PathBuf) {
        eprintln!("[freya][vault] action:open-start path={}", root.display());
        let mut next = shell_runtime::select_root(root);
        if let Some(canonical_root) = next.vault.as_ref().map(|vault| vault.root().to_path_buf()) {
            let mut registry = self.vault_registry.clone();
            if let Err(error) = registry.add_or_activate(&canonical_root) {
                next.error = Some(format!(
                    "Vault opened, but it could not be registered: {error}"
                ));
            } else if let Err(error) = registry.persist() {
                eprintln!("[freya][vault-registry] action=persist-failure error={error}");
                next.error = Some(format!(
                    "Vault opened, but its registry could not be saved: {error}"
                ));
            }
            next.vault_registry = registry;
            if let Err(error) = vault_picker::remember_vault(&canonical_root) {
                eprintln!("[freya][vault] action:remember-failure error={error}");
                next.error = Some(format!(
                    "Vault opened, but Elephant could not remember it for next launch: {error}"
                ));
            }
            eprintln!(
                "[freya][vault] action:open-complete path={}",
                canonical_root.display()
            );
        } else {
            eprintln!(
                "[freya][vault] action:open-failure error={}",
                next.error.as_deref().unwrap_or("unknown error")
            );
        }
        *self = next;
    }

    pub(super) fn activate_vault(&mut self, id: &str) {
        eprintln!("[freya][vault] action=switch-request id={id}");
        let mut registry = self.vault_registry.clone();
        let Ok(descriptor) = registry.activate(id) else {
            self.error = Some(format!("Unknown vault ID: {id}"));
            return;
        };
        eprintln!(
            "[freya][vault] action=switch-start id={} path={}",
            descriptor.id, descriptor.path
        );
        let mut next = shell_runtime::load_from_root(PathBuf::from(&descriptor.path));
        if next.vault.is_none() {
            self.error = next.error;
            eprintln!("[freya][vault] action=switch-failure id={}", descriptor.id);
            return;
        }
        if let Err(error) = registry.persist() {
            next.error = Some(format!(
                "Vault switched, but its registry could not be saved: {error}"
            ));
        }
        let root = next.vault.as_ref().map(|vault| vault.root().to_path_buf());
        next.vault_registry = registry;
        if let Some(root) = root {
            let _ = vault_picker::remember_vault(&root);
        }
        *self = next;
        eprintln!("[freya][vault] action=switch-complete id={}", id);
    }

    pub(super) fn remove_vault(&mut self, id: &str) {
        let was_active = self.vault_registry.active_vault_id.as_deref() == Some(id);
        let mut registry = self.vault_registry.clone();
        if let Err(error) = registry.remove(id) {
            self.error = Some(error);
            return;
        }
        if let Err(error) = registry.persist() {
            self.error = Some(error);
            return;
        }
        if was_active {
            if let Some(next) = registry.active().cloned() {
                let mut loaded = shell_runtime::load_from_root(PathBuf::from(&next.path));
                loaded.vault_registry = registry;
                *self = loaded;
            } else {
                let mut empty = Self::empty();
                empty.vault_registry = registry;
                *self = empty;
            }
        } else {
            self.vault_registry = registry;
        }
    }

    pub(super) fn rename_vault(&mut self, id: &str, name: &str) {
        if let Err(error) = self.vault_registry.set_name(id, name) {
            self.error = Some(error);
        } else if let Err(error) = self.vault_registry.persist() {
            self.error = Some(error);
        }
    }

    pub(super) fn set_vault_icon(&mut self, id: &str, icon: &str) {
        if let Err(error) = self.vault_registry.set_icon(id, icon) {
            self.error = Some(error);
        } else if let Err(error) = self.vault_registry.persist() {
            self.error = Some(error);
        }
    }

    fn reload_directory(&mut self, relative_path: &str) {
        let Some(vault) = self.vault.as_ref() else {
            return;
        };
        match vault.list(PageRequest::new(relative_path)) {
            Ok(page) => {
                self.library.replace_entries(
                    page.entries
                        .iter()
                        .map(library::to_library_entry)
                        .collect::<Vec<_>>(),
                );
                self.page = Some(page);
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn refresh_current_directory_from_external(&mut self) {
        let path = self.library.current_path.as_str().to_owned();
        self.reload_directory(&path);
        self.refresh_open_note_from_external();
    }

    fn refresh_open_note_from_external(&mut self) {
        let Some(note_path) = self
            .editor
            .as_ref()
            .and_then(|editor| editor.path().map(PathBuf::from))
        else {
            return;
        };
        let markdown = match std::fs::read_to_string(&note_path) {
            Ok(markdown) => markdown,
            Err(error) => {
                eprintln!(
                    "[freya][editor] action=external-refresh-failure path={} error={error}",
                    note_path.display()
                );
                self.error = Some(format!(
                    "The open note is no longer readable: {}",
                    note_path.display()
                ));
                return;
            }
        };
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        match editor.reload_external(&markdown) {
            Ok(()) => eprintln!(
                "[freya][editor] action=external-refresh-complete path={}",
                note_path.display()
            ),
            Err(error) => {
                eprintln!(
                    "[freya][editor] action=external-refresh-conflict path={} error={error}",
                    note_path.display()
                );
                self.error = Some(error.to_string());
            }
        }
    }

    fn refresh_library_entry(&mut self, relative_path: &str, directory: &str) {
        let Some(vault) = self.vault.as_ref() else {
            return;
        };
        let page = match vault.list(PageRequest::new(directory)) {
            Ok(page) => page,
            Err(error) => {
                eprintln!(
                    "[freya][library] action:refresh-failure path={relative_path} error={error}"
                );
                self.error = Some(error.to_string());
                return;
            }
        };
        let Some(entry) = page
            .entries
            .into_iter()
            .find(|entry| entry.path == relative_path)
        else {
            let error = format!("Entry disappeared while refreshing: {relative_path}");
            eprintln!("[freya][library] action:refresh-failure path={relative_path} error={error}");
            self.error = Some(error);
            return;
        };
        let mut replacement = library::to_library_entry(&entry);
        if self.library.current_path.as_str() == directory {
            if let Some(existing) = self
                .library
                .entries
                .iter_mut()
                .find(|existing| existing.path.as_str() == relative_path)
            {
                replacement.updated_at = existing.updated_at.clone();
                *existing = replacement;
            }
        }
        if let Some(page) = self
            .page
            .as_mut()
            .filter(|page| page.relative_path == directory)
        {
            if let Some(existing) = page
                .entries
                .iter_mut()
                .find(|existing| existing.path == relative_path)
            {
                let updated_at = existing.updated_at.clone();
                *existing = entry;
                existing.updated_at = updated_at;
            }
        }
        eprintln!(
            "[freya][library] action:refresh-complete path={relative_path} directory={directory}"
        );
    }

    pub(super) fn save_open_editor(&mut self) -> Result<(), crate::editor::EditorError> {
        let result = self
            .editor
            .as_mut()
            .ok_or(crate::editor::EditorError::MissingPath)
            .and_then(EditorDocument::save);
        if result.is_ok() {
            let refreshed = self.editor.as_ref().and_then(|editor| {
                let path = editor.path()?;
                let root = self.vault.as_ref()?.root();
                let relative = path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                let directory = relative
                    .rsplit_once('/')
                    .map(|(parent, _)| parent.to_owned())
                    .unwrap_or_default();
                Some((relative, directory))
            });
            if let Some((relative, directory)) = refreshed {
                self.refresh_library_entry(&relative, &directory);
            }
        }
        result
    }

    fn toggle_pinned(&mut self, path: crate::library_contract::RelativePath) {
        let path_for_log = path.clone();
        self.library.toggle_pinned(path);
        self.persist_shell_preferences();
        eprintln!(
            "[freya][library] action:pin-toggle path={} pinned={}",
            path_for_log.as_str(),
            self.library
                .pinned_paths
                .iter()
                .any(|pinned| pinned == &path_for_log)
        );
    }

    pub(super) fn mark_settings_changed(&mut self) {
        self.settings_revision = self.settings_revision.saturating_add(1);
    }

    fn create(&mut self, action: crate::library_contract::CreateAction) {
        let action_name = format!("{action:?}");
        eprintln!(
            "[freya][library] action:start action={} directory={}",
            action_name,
            self.library.current_path.as_str()
        );
        let Some(vault) = self.vault.as_ref() else {
            self.error = Some("No vault selected.".to_string());
            eprintln!(
                "[freya][library] action:failure action={} reason=no_vault",
                action_name
            );
            return;
        };
        let result = match action {
            crate::library_contract::CreateAction::Note => vault
                .create_note(
                    Some(self.library.current_path.as_str().to_string()),
                    None,
                    None,
                )
                .map(|_| ()),
            crate::library_contract::CreateAction::Folder => vault
                .create_folder(
                    (!self.library.current_path.as_str().is_empty())
                        .then(|| format!("{}/New Folder", self.library.current_path.as_str())),
                )
                .map(|_| ()),
            crate::library_contract::CreateAction::Drawing => {
                Err(crate::vault_adapter::AdapterError::from(
                    "Drawing requires the Excalidraw web island; no fake native fallback is used."
                        .to_string(),
                ))
            }
        };
        match result {
            Ok(()) => {
                let path = self.library.current_path.as_str().to_string();
                self.reload_directory(&path);
                eprintln!(
                    "[freya][library] action:complete action={} directory={}",
                    action_name, path
                );
            }
            Err(error) => {
                eprintln!(
                    "[freya][library] action:failure action={} error={error}",
                    action_name
                );
                self.error = Some(error.to_string());
            }
        }
    }

    fn set_hovered_target(&mut self, target: impl Into<String>) {
        self.hovered_target = Some(target.into());
    }

    fn clear_hovered_target(&mut self, target: &str) {
        if self.hovered_target.as_deref() == Some(target) {
            self.hovered_target = None;
        }
    }

    fn set_card_action_target(&mut self, target: impl Into<String>) {
        self.card_action_target = Some(target.into());
    }

    pub(super) fn clear_card_action_target(&mut self) {
        self.card_action_target = None;
    }
}

pub(super) fn choose_vault(mut state: State<ShellState>) {
    match vault_picker::pick_vault() {
        Ok(Some(root)) => state.write().open_vault(root),
        Ok(None) => {}
        Err(error) => {
            eprintln!("[freya][vault-picker] action:failure error={error}");
            state.write().error = Some(error);
        }
    }
}

pub fn app() -> impl IntoElement {
    let state = use_state(ShellState::load);
    app_shell(state)
}

/// Test/runtime entry point that uses the same shell and production vault
/// adapter while injecting a clean fixture root.
pub fn app_with_vault(root: impl Into<PathBuf>) -> impl IntoElement {
    let root = root.into();
    let state = use_state(move || ShellState::load_root_without_persisted_registry(root.clone()));
    app_shell(state)
}

/// Runtime entry point for hosts that already resolved a workspace view.
///
/// The normal shell starts in Notes, while add-on/host navigation can select
/// another registered view before mounting the same production shell. Keeping
/// that selection at the state boundary makes the Explorer and graph routes
/// testable without adding a test-only renderer or hidden UI control.
pub fn app_with_vault_view(root: impl Into<PathBuf>, view: WorkspaceView) -> impl IntoElement {
    let root = root.into();
    let state = use_state(move || {
        let mut state = ShellState::load_root_without_persisted_registry(root.clone());
        state.view = view.clone();
        state
    });
    app_shell(state)
}

/// Owns the sidebar's hook lifecycle independently from the root shell.
///
/// `sidebar_nav` uses `use_a11y()`. The sidebar is mounted only after a vault
/// exists, so executing that hook directly from `app_shell` changes the root
/// component's hook count when the user picks a vault. Freya correctly treats
/// that as a hook-order violation. A component boundary gives the sidebar its
/// own stable lifecycle and lets it be mounted/unmounted conditionally.
#[derive(PartialEq)]
struct SidebarNavHost {
    state: State<ShellState>,
    palette: theme::ThemePalette,
}

impl Component for SidebarNavHost {
    fn render(&self) -> impl IntoElement {
        navigation::sidebar_nav(self.state, self.palette)
    }
}

fn app_shell(mut state: State<ShellState>) -> Element {
    let mut settings_state = use_state(settings::SettingsViewState::default);
    let settings_effects = settings_state.read().effects();
    let palette = settings_effects.palette();
    let viewport = Platform::get().root_size.read();
    let mobile = viewport.width < 760.;
    let mut explorer_state = use_state(explorer::ExplorerState::new);
    let explorer_query = use_state(String::new);
    let explorer_graph_query = use_state(String::new);
    let graph_canvas_state = use_state(graph_canvas::GraphCanvasState::default);
    let wiki_view_state = use_state(wiki_view::WikiViewState::default);
    explorer::bind_live_search(explorer_state, explorer_query);
    explorer_runtime::drain_explorer_actions(state, explorer_state);
    let snapshot = state.read().clone();
    if snapshot.view == WorkspaceView::Graph || snapshot.view == WorkspaceView::Canvas {
        explorer_state.write().surface = explorer::ExplorerSurface::Graph;
    }
    let search_open = snapshot.search_open;
    let drawing_fullscreen = snapshot.drawing.is_some();
    let overlay_transition = visual_transition::use_search_overlay_transition(search_open);
    if snapshot.vault.is_none() {
        return empty_vault_picker(state);
    }
    let contract = source_contracts::contract(ComponentId::AppShell)
        .expect("AppShell source contract must remain registered");
    if !snapshot.search_open && !overlay_transition.mounted {
        // SearchModal.vue clears its result payload from the unmount hook,
        // after the close transition has finished. Keep the same lifecycle
        // boundary in the functional explorer state.
        explorer_state.write().finish_close();
    }
    if let Some(section) = snapshot.settings_target_section.as_deref() {
        if settings_state.read().settings.active_section != section {
            settings_state.write().select_section(section);
        }
        state.write().settings_target_section = None;
    }
    // Tauri mounts SearchModal as a sibling of MainContent. Opening search must
    // therefore never replace the current workspace; it only overlays it.
    // This also keeps the exact pre-search library/graph state intact for close.
    let content = if snapshot.view == WorkspaceView::Canvas {
        canvas_view::workspace(state, explorer_state, graph_canvas_state, palette)
    } else if snapshot.view == WorkspaceView::Graph {
        explorer::explorer_view(
            explorer_state,
            explorer_query,
            explorer_graph_query,
            graph_canvas_state,
            false,
            palette,
        )
    } else {
        library::main_content(state, wiki_view_state, palette)
    };
    let mut mobile_navigation_state = state;
    let mobile_navigation = snapshot.mobile_navigation_open.then(|| {
        rect()
            .position(Position::new_global().left(0.).top(52.))
            .width(Size::px(300.))
            .height(Size::fill())
            .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
            .layer(Layer::OverlayLevel(40))
            .child(SidebarNavHost { state, palette }.into_element())
            .on_global_pointer_press(move |_| {
                mobile_navigation_state.write().mobile_navigation_open = false;
            })
    });
    let workspace = if mobile || drawing_fullscreen {
        rect()
            .expanded()
            .child(content)
            .into_element()
    } else {
        rect()
            .expanded()
            .horizontal()
            .child(navigation::icon_rail(state, palette, &settings_effects))
            .child(
                rect()
                    .width(Size::px(if snapshot.sidebar_visible {
                        f32::from(snapshot.sidebar_width.get())
                    } else {
                        0.
                    }))
                    .height(Size::fill())
                    .child(SidebarNavHost { state, palette }.into_element()),
            )
            .child(
                rect()
                    .expanded()
                    .height(Size::fill())
                    .child(content),
            )
            .into_element()
    };
    let shell = rect()
        .width(Size::fill())
        .height(Size::fill())
        .font_family(theme::UI_FONT_FAMILY)
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .color(theme::token_color(palette, theme::ThemeToken::Text))
        .maybe_child((!drawing_fullscreen).then(|| {
            if mobile {
                navigation::mobile_top_bar(state, palette)
            } else {
                navigation::top_vault_bar(state, palette)
            }
        }))
        .maybe_child(overlay_transition.mounted.then(|| {
            explorer::search_overlay(
                state,
                explorer_state,
                explorer_query,
                overlay_transition.interactive,
                overlay_transition.backdrop_opacity,
                overlay_transition.content_opacity,
                palette,
            )
        }))
        .child(workspace)
        .maybe_child(snapshot.settings_open.then(|| {
            settings::settings_panel(settings_state, state)
        }))
        .maybe_child(mobile_navigation)
        .child(vault_watch::VaultWatcherHost { state }.into_element())
        .a11y_alt(contract.provenance.component.source_name());

    if snapshot.menu_open {
        shell
            .child(library::create_entry_menu(state, palette))
            .into_element()
    } else {
        shell.into_element()
    }
}

fn empty_vault_picker(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let error = snapshot.error.clone();
    let registered_vaults = snapshot
        .vault_registry
        .vaults
        .into_iter()
        .filter(|vault| vault.enabled)
        .collect::<Vec<_>>();
    let mut recovery_actions = rect().width(Size::fill()).spacing(8.);
    let mut has_recovery_actions = false;
    for vault in registered_vaults {
        has_recovery_actions = true;
        let id = vault.id;
        let name = vault.name;
        let target_label = format!("Open registered vault {name}");
        let mut open_state = state;
        recovery_actions = recovery_actions.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(8., 12., 8., 12.))
                .with_corner_radius(8.)
                .a11y_alt(target_label)
                .on_press(move |_| open_state.write().activate_vault(&id))
                .child(label().text(format!("Open {name}"))),
        );
    }
    let picker_state = state;
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .center()
        .a11y_alt("EmptyVaultPicker")
        .child(
            rect()
                .width(Size::px(420.))
                .padding(Gaps::new_all(28.))
                .background(theme::color(theme::SURFACE))
                .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
                .with_corner_radius(14.)
                .spacing(12.)
                .child(label().font_size(22.).text("Choose a vault"))
                .child(
                    label()
                        .color(theme::color(theme::MUTED))
                        .text("Select an existing Elephant vault folder to continue."),
                )
                .maybe_child(error.map(|message| {
                    rect()
                        .a11y_alt("Vault unavailable")
                        .child(label().color(theme::color(theme::MUTED)).text(message))
                        .into_element()
                }))
                .maybe_child(has_recovery_actions.then(|| {
                    rect()
                        .width(Size::fill())
                        .spacing(8.)
                        .child(
                            label()
                                .font_weight(FontWeight::BOLD)
                                .text("Registered vaults"),
                        )
                        .child(recovery_actions)
                        .into_element()
                }))
                .child(
                    Button::new()
                        .on_press(move |_| choose_vault(picker_state))
                        .child("Choose folder"),
                ),
        )
        .into_element()
}

pub(super) fn route_notice(title: &str, body: &str) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::SURFACE))
        .padding(Gaps::new_all(18.))
        .spacing(10.)
        .child(label().font_size(18.).text(title.to_string()))
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(body.to_string()),
        )
        .into_element()
}
