//! Component-by-component Freya assembly for the active Elephant Vue shell.
//!
//! The state/effect boundary stays here. Renderers are split by the source
//! component ownership they convert: navigation, library, and editor view.

mod editor_view;
mod explorer;
mod library;
mod navigation;
mod settings;
mod shell_gestures;
mod shell_history;
mod shell_preferences;
mod shell_runtime;

use freya::prelude::*;
use std::{env, path::PathBuf};

use crate::{
    editor::EditorDocument,
    library_contract::LibraryState,
    navigation_contract::{SidebarWidth, WorkspaceView},
    source_contracts::{self, ComponentId},
    theme,
    vault_adapter::{PageRequest, VaultAdapter, VaultPage},
};

use shell_gestures::{RailDragState, SidebarResizeState};
use shell_history::NavigationTarget;

#[derive(Clone, Debug)]
struct ShellState {
    view: WorkspaceView,
    sidebar_visible: bool,
    sidebar_width: SidebarWidth,
    vault: Option<VaultAdapter>,
    page: Option<VaultPage>,
    library: LibraryState,
    menu_open: bool,
    hovered_target: Option<String>,
    vault_menu_open: bool,
    search_open: bool,
    settings_open: bool,
    editor: Option<EditorDocument>,
    error: Option<String>,
    navigation_history: Vec<NavigationTarget>,
    navigation_index: usize,
    rail_order: Vec<String>,
    rail_drag: Option<RailDragState>,
    rail_drop_target: Option<String>,
    sidebar_resize: Option<SidebarResizeState>,
}

impl ShellState {
    fn empty() -> Self {
        Self {
            view: WorkspaceView::Notes,
            sidebar_visible: true,
            sidebar_width: SidebarWidth::default(),
            vault: None,
            page: None,
            library: LibraryState::default(),
            menu_open: false,
            hovered_target: None,
            vault_menu_open: false,
            search_open: false,
            settings_open: false,
            editor: None,
            error: None,
            navigation_history: Vec::new(),
            navigation_index: 0,
            rail_order: shell_preferences::default_rail_order(),
            rail_drag: None,
            rail_drop_target: None,
            sidebar_resize: None,
        }
    }

    fn load() -> Self {
        let Some(raw_root) = env::var_os("ELEPHANT_FREYA_VAULT") else {
            let mut state = Self::empty();
            state.error = Some("No vault selected. Set ELEPHANT_FREYA_VAULT.".to_string());
            return state;
        };
        shell_runtime::load_from_root(PathBuf::from(raw_root))
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
                .create_folder(Some(self.library.current_path.as_str().to_string()))
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
}

pub fn app() -> impl IntoElement {
    let state = use_state(ShellState::load);
    app_shell(state)
}

/// Test/runtime entry point that uses the same shell and production vault
/// adapter while injecting a clean fixture root.
pub fn app_with_vault(root: impl Into<PathBuf>) -> impl IntoElement {
    let root = root.into();
    let state = use_state(move || shell_runtime::load_from_root(root.clone()));
    app_shell(state)
}

fn app_shell(state: State<ShellState>) -> Element {
    let settings_state = use_state(settings::SettingsViewState::default);
    let explorer_state = use_state(explorer::ExplorerState::new);
    let explorer_query = use_state(String::new);
    let explorer_graph_query = use_state(String::new);
    let snapshot = state.read().clone();
    if snapshot.vault.is_none() {
        return empty_vault_picker(snapshot.error.as_deref());
    }
    let contract = source_contracts::contract(ComponentId::AppShell)
        .expect("AppShell source contract must remain registered");
    let content = if snapshot.settings_open {
        settings::settings_panel(settings_state)
    } else if snapshot.search_open || snapshot.view == WorkspaceView::Graph {
        explorer::explorer_view(explorer_state, explorer_query, explorer_graph_query)
    } else {
        library::main_content(state)
    };
    let shell = rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .color(theme::color(theme::TEXT))
        .child(navigation::top_vault_bar(state))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .horizontal()
                .child(navigation::icon_rail(state))
                .child(navigation::sidebar_nav(state))
                .child(content),
        )
        .a11y_alt(contract.provenance.component.source_name());

    if snapshot.menu_open {
        shell
            .child(library::create_entry_menu(state))
            .into_element()
    } else {
        shell.into_element()
    }
}

fn empty_vault_picker(error: Option<&str>) -> Element {
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
                    label().color(theme::color(theme::MUTED)).text(
                        error
                            .unwrap_or("Set ELEPHANT_FREYA_VAULT to open an existing vault.")
                            .to_string(),
                    ),
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
