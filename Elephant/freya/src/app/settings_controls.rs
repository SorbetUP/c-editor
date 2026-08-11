//! Composition boundary for the native settings controls.

use freya::prelude::*;

use crate::{
    settings_contract::{CORE_SECTIONS, CORE_SETTINGS_INDEX},
    theme,
};

use super::{SettingsSurfaceState, SettingsViewState};

#[path = "settings_appearance_controls.rs"]
mod settings_appearance_controls;
#[path = "settings_editor_controls.rs"]
mod settings_editor_controls;
#[path = "settings_navigation.rs"]
mod settings_navigation;
#[path = "settings_preference_controls.rs"]
mod settings_preference_controls;
#[path = "settings_search.rs"]
mod settings_search;
#[path = "settings_surface.rs"]
mod settings_surface;

pub(super) fn settings_header(search: Input) -> Element {
    settings_navigation::settings_header(search)
}

pub(super) fn section_navigation(active_section: &str, state: State<SettingsViewState>) -> Element {
    settings_navigation::section_navigation(active_section, state)
}

pub(super) fn preference_switch(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    key: &'static str,
    active: bool,
    alt: &'static str,
) -> Element {
    settings_preference_controls::preference_switch(state, title, description, key, active, alt)
}

pub(super) fn text_preference(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    current: String,
) -> Element {
    settings_preference_controls::text_preference(state, title, description, current)
}

pub(super) fn delay_preference(
    state: State<SettingsViewState>,
    delay: u64,
    enabled: bool,
) -> Element {
    settings_preference_controls::delay_preference(state, delay, enabled)
}

pub(super) fn search_results_content(state: State<SettingsViewState>, query: &str) -> Element {
    settings_search::search_results_content(state, query)
}

pub(super) fn surface_state(surface: &SettingsSurfaceState) -> Element {
    settings_surface::surface_state(surface)
}

pub(super) fn unavailable_section(section_label: &str) -> Element {
    settings_surface::unavailable_section(section_label)
}

pub(super) fn section_content(
    state: State<SettingsViewState>,
    active_section: &str,
    surface: &SettingsSurfaceState,
    query: &str,
) -> Element {
    if !query.trim().is_empty() {
        return search_results_content(state, query);
    }
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

    let entries = match active_section {
        "appearance" => settings_appearance_controls::appearance_settings(state),
        "editor" => settings_editor_controls::editor_settings(state),
        _ => CORE_SETTINGS_INDEX
            .iter()
            .filter(|entry| entry.section == active_section)
            .map(settings_preference_controls::setting_row)
            .collect::<Vec<_>>(),
    };

    if entries.is_empty() {
        return content.child(unavailable_section(&title)).into_element();
    }

    content.children(entries).into_element()
}
