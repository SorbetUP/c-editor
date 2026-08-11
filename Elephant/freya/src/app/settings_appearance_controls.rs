//! Appearance preference rows from the source settings panel.

use freya::prelude::*;

use super::super::SettingsViewState;

pub(super) fn appearance_settings(state: State<SettingsViewState>) -> Vec<Element> {
    let snapshot = state.read().clone();
    vec![super::preference_switch(
        state,
        "Floating surfaces",
        "Lift navigation, controls and the writing surface above the background.",
        "floatingSurfaces",
        snapshot.runtime.bool_value("floatingSurfaces"),
        "Floating surfaces",
    )]
}
