//! Native Freya read/selection view for the existing SettingsPanel surface.
//!
//! This module intentionally owns no preferences, dialogs, addon manager, or
//! Tauri calls.  It renders the section/navigation contracts and lets the
//! eventual host decide how to wire persistence and section-owned controls.
//! The labels and descriptions come from `settings_contract`, which in turn
//! records the production Vue sources.

use freya::prelude::*;

use crate::{
    settings_contract::{
        search_core_settings, SectionTransition, SettingIndexEntry, SettingsState, CORE_SECTIONS,
        CORE_SETTINGS_INDEX,
    },
    theme,
};

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

/// State boundary for the native view.
///
/// `SettingsState` is the already-typed contract state.  This wrapper adds
/// only the render status needed to keep loading/error/empty states visible.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsViewState {
    pub settings: SettingsState,
    pub surface: SettingsSurfaceState,
}

impl SettingsViewState {
    pub fn select_section(&mut self, section: &str) -> SectionTransition {
        self.settings.select_section(section)
    }

    pub fn set_surface_state(&mut self, surface: SettingsSurfaceState) {
        self.surface = surface;
    }
}

/// Render the native settings panel.
///
/// This is deliberately a standalone entry point: the caller owns the
/// `State<SettingsViewState>` and can later connect real section actions when
/// the corresponding host contracts are integrated.
pub fn settings_panel(state: State<SettingsViewState>) -> Element {
    let snapshot = state.read().clone();
    let active_section = snapshot.settings.active_section.clone();

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .color(theme::color(theme::TEXT))
        .padding(Gaps::new_all(16.))
        .spacing(14.)
        .a11y_alt("ElephantNote settings")
        .child(settings_header())
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .horizontal()
                .spacing(14.)
                .child(section_navigation(&active_section, state))
                .child(section_content(&active_section, &snapshot.surface)),
        )
        .into_element()
}

fn settings_header() -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(48.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .font_size(20.)
                .font_weight(FontWeight::BOLD)
                .text("Settings"),
        )
        .child(
            rect()
                .width(Size::px(220.))
                .height(Size::px(34.))
                .padding(Gaps::new(8., 12., 8., 12.))
                .background(theme::color(theme::SURFACE))
                .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
                .with_corner_radius(8.)
                .a11y_alt("Search all settings")
                .child(
                    label()
                        .color(theme::color(theme::MUTED))
                        .text("Search all settings"),
                ),
        )
        .into_element()
}

fn section_navigation(active_section: &str, state: State<SettingsViewState>) -> Element {
    let items = CORE_SECTIONS
        .iter()
        .map(|section| section_button(section.id, section.label, active_section, state))
        .collect::<Vec<_>>();

    rect()
        .width(Size::px(196.))
        .height(Size::fill())
        .padding(Gaps::new_all(8.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .spacing(3.)
        .a11y_alt("Settings sections")
        .children(items)
        .child(
            rect()
                .height(Size::fill())
                .main_align(Alignment::End)
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(11.)
                        .color(theme::color(theme::MUTED))
                        .text("Local-first"),
                )
                .child(
                    label()
                        .font_size(11.)
                        .color(theme::color(theme::MUTED))
                        .text("v0.1.0"),
                ),
        )
        .into_element()
}

fn section_button(
    id: &'static str,
    label_text: &'static str,
    active_section: &str,
    state: State<SettingsViewState>,
) -> Element {
    let selected = active_section == id;
    let mut state = state;
    rect()
        .width(Size::fill())
        .height(Size::px(38.))
        .padding(Gaps::new(8., 10., 8., 10.))
        .background(if selected {
            theme::color(theme::SOFT)
        } else {
            theme::color(theme::SURFACE)
        })
        .with_corner_radius(7.)
        .horizontal()
        .cross_align(Alignment::Center)
        .on_mouse_up(move |_| {
            state.write().select_section(id);
        })
        .a11y_alt(format!("Select {label_text} settings"))
        .child(label().text(label_text))
        .child(label().color(theme::color(theme::MUTED)).text("›"))
        .into_element()
}

fn section_content(active_section: &str, surface: &SettingsSurfaceState) -> Element {
    let title = CORE_SECTIONS
        .iter()
        .find(|section| section.id == active_section)
        .map(|section| section.label.to_owned())
        .unwrap_or_else(|| active_section.to_owned());

    let content = rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new_all(14.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .spacing(12.)
        .a11y_alt(format!("Settings section {active_section}"))
        .child(
            label()
                .font_size(18.)
                .font_weight(FontWeight::BOLD)
                .text(title.clone()),
        );

    if !matches!(surface, SettingsSurfaceState::Ready) {
        return content.child(surface_state(surface)).into_element();
    }

    let entries = CORE_SETTINGS_INDEX
        .iter()
        .filter(|entry| entry.section == active_section)
        .map(setting_row)
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return content.child(unavailable_section(&title)).into_element();
    }

    content.children(entries).into_element()
}

fn setting_row(entry: &SettingIndexEntry) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .spacing(4.)
        .a11y_alt(entry.label)
        .child(label().font_weight(FontWeight::BOLD).text(entry.label))
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(entry.description),
        )
        .into_element()
}

fn surface_state(surface: &SettingsSurfaceState) -> Element {
    let (title, detail, alt) = match surface {
        SettingsSurfaceState::Loading { title, detail } => {
            (title.clone(), detail.clone(), "Settings loading")
        }
        SettingsSurfaceState::Error { title, detail } => {
            (title.clone(), detail.clone(), "Settings error")
        }
        SettingsSurfaceState::Empty { title, detail } => {
            (title.clone(), detail.clone(), "Settings empty")
        }
        SettingsSurfaceState::Ready => return rect().into_element(),
    };

    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(24.))
        .background(theme::color(theme::BG))
        .with_corner_radius(8.)
        .spacing(8.)
        .a11y_alt(alt)
        .child(label().font_weight(FontWeight::BOLD).text(title))
        .child(label().color(theme::color(theme::MUTED)).text(detail))
        .into_element()
}

fn unavailable_section(section_label: &str) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(24.))
        .background(theme::color(theme::BG))
        .with_corner_radius(8.)
        .spacing(8.)
        .a11y_alt(format!("{section_label} is unavailable"))
        .child(
            label()
                .font_weight(FontWeight::BOLD)
                .text(format!("{section_label} is unavailable")),
        )
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text("The addon is being reloaded or has been disabled. Elephant keeps this page selected instead of moving you to another menu."),
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
