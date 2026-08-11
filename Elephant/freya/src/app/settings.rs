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
};

#[path = "settings_controls.rs"]
mod settings_controls;
#[path = "settings_runtime.rs"]
mod settings_runtime;

pub(super) use settings_runtime::SettingsRuntimeState;

/// Visible state supplied by the host while a section-owned surface loads.
///
/// The text is supplied rather than invented by this renderer.  The helpers
/// below preserve the exact Vaults messages currently used by SettingsPanel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsSurfaceState {
    Ready,
    Loading { title: String, detail: String },
    Error { title: String, detail: String },
    Empty { title: String, detail: String },
}

impl SettingsSurfaceState {
    /// Exact loading copy from `SettingsPanel.vue`'s Vaults branch.
    pub fn loading_vaults() -> Self {
        Self::Loading {
            title: "Loading vaults".to_owned(),
            detail: "Reading the registered workspaces.".to_owned(),
        }
    }

    /// Exact error title from `SettingsPanel.vue`; the detail is the store's
    /// visible error text.
    pub fn vaults_error(detail: impl Into<String>) -> Self {
        Self::Error {
            title: "Vaults could not be loaded".to_owned(),
            detail: detail.into(),
        }
    }

    /// Exact empty-state copy from `SettingsPanel.vue`'s Vaults branch.
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

/// State boundary for the native view and its real vault/profile persistence.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsViewState {
    pub settings: SettingsState,
    pub surface: SettingsSurfaceState,
    pub runtime: SettingsRuntimeState,
}

impl SettingsViewState {
    pub fn load_from(path: impl Into<PathBuf>) -> Self {
        let runtime = SettingsRuntimeState::load_from(path);
        let mut state = Self {
            settings: SettingsState::default(),
            surface: SettingsSurfaceState::Ready,
            runtime,
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

    pub(super) fn toggle_rail_hidden(&mut self, item: &str) {
        self.runtime
            .toggle_string_list_value("iconRailHidden", item);
    }

    pub fn cycle_auto_save_delay(&mut self) {
        self.runtime.cycle_auto_save_delay();
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
        }
    }
}

/// Render the native settings panel.
///
/// This is deliberately a standalone entry point: the caller owns the
/// `State<SettingsViewState>` and can later connect real section actions when
/// the corresponding host contracts are integrated.
pub fn settings_panel(state: State<SettingsViewState>) -> Element {
    let snapshot = state.read().clone();
    let palette = snapshot.effects().palette();
    let active_section = snapshot.settings.active_section.clone();
    let search_value = State::create(snapshot.settings.query.clone());
    let mut search_state = state;
    let search = Input::new(search_value)
        .width(Size::px(220.))
        .placeholder("Search all settings")
        .on_submit(move |query: String| {
            search_state.write().settings.set_query(&query);
        });

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .color(theme::token_color(palette, theme::ThemeToken::Text))
        .padding(Gaps::new_all(16.))
        .spacing(14.)
        .a11y_alt("ElephantNote settings")
        .child(settings_controls::settings_header(search, palette))
        .maybe_child(snapshot.runtime.feedback.clone().map(|message| {
            rect()
                .width(Size::fill())
                .padding(Gaps::new(6., 10., 6., 10.))
                .a11y_alt("Settings feedback")
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(message),
                )
                .into_element()
        }))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .horizontal()
                .spacing(14.)
                .child(settings_controls::section_navigation(
                    &active_section,
                    state,
                    palette,
                ))
                .child(settings_controls::section_content(
                    state,
                    &active_section,
                    &snapshot.surface,
                    &snapshot.settings.query,
                )),
        )
        .into_element()
}

/// Pure helper for the eventual host's settings search integration.
///
/// Keeping the query routed through `settings_contract` avoids a second
/// implementation of the Vue search matching rules in this view module.
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
