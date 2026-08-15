//! Native Freya rendering for the existing SettingsPanel surface.
//!
//! Persistence and preference transforms live in `settings_runtime`; this
//! module keeps the visible settings layout, controls, and state transitions.

use freya::prelude::*;
use std::path::PathBuf;

use crate::{
    settings_contract::{
        search_core_settings, SectionTransition, SettingIndexEntry, SettingsState,
    },
    theme,
    vault_adapter::TrashEntry,
};

#[path = "settings_controls.rs"]
mod settings_controls;
#[path = "settings_runtime.rs"]
mod settings_runtime;

pub(super) use settings_runtime::SettingsRuntimeState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSurfaceState {
    Ready,
    Loading { title: String, detail: String },
    Error { title: String, detail: String },
    Empty { title: String, detail: String },
}

impl SettingsSurfaceState {
    pub fn loading_vaults() -> Self {
        Self::Loading {
            title: "Loading vaults".to_owned(),
            detail: "Reading the registered workspaces.".to_owned(),
        }
    }

    pub fn vaults_error(detail: impl Into<String>) -> Self {
        Self::Error {
            title: "Vaults could not be loaded".to_owned(),
            detail: detail.into(),
        }
    }

    pub fn no_vault_registered() -> Self {
        Self::Empty {
            title: "No vault registered".to_owned(),
            detail: "Open a folder from the main workspace to add it. Existing folders stay on disk when removed from this list.".to_owned(),
        }
    }
}

impl Default for SettingsSurfaceState {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct VaultTrashState {
    pub(super) items: Vec<TrashEntry>,
    pub(super) loaded: bool,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    pub(super) action: Option<String>,
    pub(super) empty_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct AddonsViewState {
    pub(super) items: Vec<crate::addon_adapter::InstalledAddon>,
    pub(super) loaded: bool,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    pub(super) action: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsViewState {
    pub settings: SettingsState,
    pub surface: SettingsSurfaceState,
    pub runtime: SettingsRuntimeState,
    pub(super) trash: VaultTrashState,
    pub(super) addons: AddonsViewState,
}

impl SettingsViewState {
    pub fn load_from(path: impl Into<PathBuf>) -> Self {
        let runtime = SettingsRuntimeState::load_from(path);
        let mut state = Self {
            settings: SettingsState::default(),
            surface: SettingsSurfaceState::Ready,
            runtime,
            trash: VaultTrashState::default(),
            addons: AddonsViewState::default(),
        };
        if let Some(error) = state.runtime.load_error.clone() {
            state.surface = SettingsSurfaceState::Error {
                title: "Settings could not be loaded".to_owned(),
                detail: error,
            };
        }
        state
    }

    pub fn select_section(&mut self, section: &str) -> SectionTransition {
        self.settings.select_section(section)
    }

    pub fn set_surface_state(&mut self, surface: SettingsSurfaceState) {
        self.surface = surface;
    }

    pub fn toggle_bool(&mut self, key: &str) {
        self.runtime.toggle_bool(key);
    }

    pub fn set_text_preference(&mut self, key: &str, value: String) {
        self.runtime.set_text_preference(key, value);
    }

    pub(super) fn set_integer_preference(&mut self, key: &str, value: i64) {
        self.runtime.set_integer_preference(key, value);
    }

    pub(super) fn toggle_rail_hidden(&mut self, item: &str) {
        self.runtime
            .toggle_string_list_value("iconRailHidden", item);
    }

    pub fn cycle_auto_save_delay(&mut self) {
        self.runtime.cycle_auto_save_delay();
    }

    pub(super) fn begin_trash_load(&mut self) {
        self.trash.loading = true;
        self.trash.error = None;
    }

    pub(super) fn apply_trash_result(&mut self, result: Result<Vec<TrashEntry>, String>) {
        self.trash.loading = false;
        self.trash.loaded = true;
        match result {
            Ok(items) => {
                self.trash.items = items;
                self.trash.error = None;
            }
            Err(error) => {
                self.trash.error = Some(error);
            }
        }
    }

    pub(super) fn begin_trash_action(&mut self, action: impl Into<String>) {
        self.trash.action = Some(action.into());
        self.trash.error = None;
    }

    pub(super) fn finish_trash_action(&mut self, result: Result<(), String>) {
        self.trash.action = None;
        match result {
            Ok(()) => self.trash.error = None,
            Err(error) => self.trash.error = Some(error),
        }
        self.trash.empty_confirmation = false;
    }

    pub(super) fn toggle_trash_expanded(&mut self) {
        self.settings.toggle_trash();
    }

    pub(super) fn request_empty_trash(&mut self) {
        self.trash.empty_confirmation = !self.trash.empty_confirmation;
    }

    pub(super) fn begin_addons_load(&mut self) {
        self.addons.loading = true;
        self.addons.error = None;
    }

    pub(super) fn apply_addons_result(
        &mut self,
        result: Result<Vec<crate::addon_adapter::InstalledAddon>, String>,
    ) {
        self.addons.loading = false;
        self.addons.loaded = true;
        match result {
            Ok(items) => {
                self.addons.items = items;
                self.addons.error = None;
            }
            Err(error) => self.addons.error = Some(error),
        }
    }

    pub(super) fn begin_addon_action(&mut self, addon_id: impl Into<String>) {
        self.addons.action = Some(addon_id.into());
        self.addons.error = None;
    }

    pub(super) fn finish_addon_action(&mut self, result: Result<(), String>) {
        self.addons.action = None;
        if let Err(error) = result {
            self.addons.error = Some(error);
        }
    }

    pub(super) fn effects(&self) -> super::settings_effects::SettingsEffects {
        super::settings_effects::SettingsEffects::from_runtime(&self.runtime)
    }
}

impl Default for SettingsViewState {
    fn default() -> Self {
        let runtime = SettingsRuntimeState::default();
        let surface = runtime
            .load_error
            .clone()
            .map(|detail| SettingsSurfaceState::Error {
                title: "Settings could not be loaded".to_owned(),
                detail,
            })
            .unwrap_or_default();
        Self {
            settings: SettingsState::default(),
            surface,
            runtime,
            trash: VaultTrashState::default(),
            addons: AddonsViewState::default(),
        }
    }
}

#[derive(PartialEq)]
struct SettingsPanelComponent {
    state: State<SettingsViewState>,
    shell_state: State<super::ShellState>,
}

impl Component for SettingsPanelComponent {
    fn render(&self) -> impl IntoElement {
        let state = self.state;
        let snapshot = state.read().clone();
        let palette = snapshot.effects().palette();
        let active_section = snapshot.settings.active_section.clone();
        let initial_query = snapshot.settings.query.clone();
        let search_value = use_state(move || initial_query);
        let query = search_value.read().clone();
        let mut search_state = state;
        let search = Input::new(search_value)
            .width(Size::fill())
            .placeholder("Search all settings")
            .on_submit(move |query: String| {
                search_state.write().settings.set_query(&query);
            });

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new_all(16.))
            .background(theme::token_color(palette, theme::ThemeToken::Bg))
            .color(theme::token_color(palette, theme::ThemeToken::Text))
            .center()
            .a11y_alt("Settings backdrop")
            .child(
                rect()
                    .width(Size::fill())
                    .max_width(Size::px(1020.))
                    .height(Size::fill())
                    .max_height(Size::px(780.))
                    .background(theme::token_color(palette, theme::ThemeToken::Surface))
                    .border(
                        Border::new()
                            .fill(theme::token_color(palette, theme::ThemeToken::Border))
                            .width(1.),
                    )
                    .with_corner_radius(22.)
                    .a11y_alt("ElephantNote settings")
                    .child(settings_controls::settings_header(search, palette))
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(1.))
                            .background(theme::token_color(palette, theme::ThemeToken::Border)),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::fill())
                            .horizontal()
                            .child(settings_controls::section_navigation(
                                &active_section,
                                state,
                                search_value,
                                palette,
                            ))
                            .child(
                                rect().width(Size::px(1.)).height(Size::fill()).background(
                                    theme::token_color(palette, theme::ThemeToken::Border),
                                ),
                            )
                            .child(settings_controls::section_content(
                                state,
                                self.shell_state,
                                &active_section,
                                &snapshot.surface,
                                &query,
                                search_value,
                            )),
                    ),
            )
    }
}

/// Render Settings with the same major geometry as `settings-redesign.css`:
/// up to 1020x780 with a 16px viewport gutter, a 64px header, 196px left
/// navigation and a 22px radius. The search input lives inside a dedicated
/// component scope so its writable state survives normal rerenders without
/// relying on a conditional host hook.
pub fn settings_panel(
    state: State<SettingsViewState>,
    shell_state: State<super::ShellState>,
) -> Element {
    SettingsPanelComponent { state, shell_state }.into_element()
}

pub fn search_labels(query: &str) -> Vec<&'static SettingIndexEntry> {
    search_core_settings(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_contract::{CORE_SECTIONS, CORE_SETTINGS_INDEX};

    #[test]
    fn selection_uses_the_existing_contract_transition() {
        let mut state = SettingsViewState::default();
        assert_eq!(state.settings.active_section, "appearance");
        assert_eq!(state.select_section("editor"), SectionTransition::Selected);
        assert_eq!(state.settings.active_section, "editor");
    }

    #[test]
    fn rendered_labels_are_the_contract_labels() {
        assert_eq!(search_labels("autosave").len(), 2);
        assert_eq!(CORE_SECTIONS[0].label, "Appearance");
        assert_eq!(CORE_SETTINGS_INDEX[0].label, "Color mode");
    }

    #[test]
    fn vault_status_helpers_keep_vue_copy() {
        assert_eq!(
            SettingsSurfaceState::loading_vaults(),
            SettingsSurfaceState::Loading {
                title: "Loading vaults".to_owned(),
                detail: "Reading the registered workspaces.".to_owned(),
            }
        );
        assert!(matches!(
            SettingsSurfaceState::no_vault_registered(),
            SettingsSurfaceState::Empty { .. }
        ));
    }
}
