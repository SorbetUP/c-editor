//! Appearance preference rows from the source settings panel.

use freya::prelude::*;

use crate::settings_contract::THEME_FAMILIES;

use super::super::SettingsViewState;

pub(super) fn appearance_settings(state: State<SettingsViewState>) -> Vec<Element> {
    let snapshot = state.read().clone();
    let current_theme = snapshot.runtime.text_value("theme");
    let mode = if current_theme.ends_with("-dark") || current_theme == "dark" {
        "dark"
    } else {
        "light"
    };
    let mut controls = vec![
        super::theme_variant(state, "Light", "light", current_theme == "light"),
        super::theme_variant(state, "Dark", "dark", current_theme == "dark"),
    ];
    controls.extend(THEME_FAMILIES.iter().map(|family| {
        let theme_id = if mode == "dark" {
            family.dark
        } else {
            family.light
        };
        super::theme_variant(state, family.name, theme_id, current_theme == theme_id)
    }));
    controls.extend([
        super::navigation_visibility(
            state,
            "Search",
            "search",
            snapshot.runtime.string_list_value("iconRailHidden"),
        ),
        super::preference_switch(
            state,
            "Floating surfaces",
            "Lift navigation, controls and the writing surface above the background.",
            "floatingSurfaces",
            snapshot.runtime.bool_value("floatingSurfaces"),
            "Floating surfaces",
        ),
    ]);
    controls
}
