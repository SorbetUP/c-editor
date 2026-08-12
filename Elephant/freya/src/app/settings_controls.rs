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

pub(super) fn settings_header(search: Input, palette: theme::ThemePalette) -> Element {
    settings_navigation::settings_header(search, palette)
}

pub(super) fn section_navigation(
    active_section: &str,
    state: State<SettingsViewState>,
    search_query: State<String>,
    palette: theme::ThemePalette,
) -> Element {
    settings_navigation::section_navigation(active_section, state, search_query, palette)
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

#[allow(clippy::too_many_arguments)]
pub(super) fn integer_stepper(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    key: &'static str,
    value: i64,
    minimum: i64,
    maximum: i64,
    step: i64,
    suffix: &'static str,
) -> Element {
    settings_preference_controls::integer_stepper(
        state, title, description, key, value, minimum, maximum, step, suffix,
    )
}

pub(super) fn theme_variant(
    state: State<SettingsViewState>,
    label_text: &'static str,
    description: &'static str,
    theme_id: &'static str,
    active: bool,
) -> Element {
    settings_preference_controls::theme_variant(
        state, label_text, description, theme_id, active,
    )
}

pub(super) fn navigation_visibility(
    state: State<SettingsViewState>,
    label_text: &'static str,
    item_id: &'static str,
    hidden_ids: Vec<String>,
) -> Element {
    settings_preference_controls::navigation_visibility(state, label_text, item_id, hidden_ids)
}

pub(super) fn search_results_content(
    state: State<SettingsViewState>,
    search_query: State<String>,
    query: &str,
) -> Element {
    settings_search::search_results_content(state, search_query, query)
}

pub(super) fn section_content(
    state: State<SettingsViewState>,
    active_section: &str,
    surface: &SettingsSurfaceState,
    query: &str,
    search_query: State<String>,
) -> Element {
    let palette = state.read().effects().palette();
    if !query.trim().is_empty() {
        return search_results_content(state, search_query, query);
    }

    let title = CORE_SECTIONS
        .iter()
        .find(|section| section.id == active_section)
        .map(|section| section.label.to_owned())
        .unwrap_or_else(|| active_section.to_owned());

    let content = rect()
        .width(Size::fill())
        .padding(Gaps::new(28., 34., 46., 34.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .spacing(16.)
        .a11y_alt(format!("Settings section {active_section}"))
        .child(
            rect()
                .height(Size::px(38.))
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .font_size(24.)
                        .font_weight(FontWeight::BOLD)
                        .text(title.clone()),
                ),
        );

    let content = if !matches!(surface, SettingsSurfaceState::Ready) {
        content.child(settings_surface::surface_state(surface, palette))
    } else {
        let entries = match active_section {
            "appearance" => settings_appearance_controls::appearance_settings(state),
            "editor" => settings_editor_controls::editor_settings(state),
            "vaults" | "addons" => Vec::new(),
            _ => CORE_SETTINGS_INDEX
                .iter()
                .filter(|entry| entry.section == active_section)
                .map(|entry| settings_preference_controls::setting_row(entry, palette))
                .collect::<Vec<_>>(),
        };

        if entries.is_empty() {
            content.child(settings_surface::unavailable_section(&title, palette))
        } else {
            let mut group = rect()
                .width(Size::fill())
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(14.);

            for (index, entry) in entries.into_iter().enumerate() {
                if index > 0 {
                    group = group.child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(1.))
                            .background(theme::token_color(palette, theme::ThemeToken::Border)),
                    );
                }
                group = group.child(entry);
            }

            content.child(group)
        }
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .show_scrollbar(true)
                .child(content),
        )
        .into_element()
}
