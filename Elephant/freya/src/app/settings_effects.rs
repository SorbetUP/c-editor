//! Effective shell values derived from the existing portable preference keys.
//!
//! This is an adapter, not a second preference schema. It only resolves keys
//! already declared in `settings_contract` and keeps the shell's workspace
//! order as the fallback for older profiles that predate portable rail prefs.

use crate::{settings_contract::THEME_FAMILIES, theme};

use super::{settings::SettingsRuntimeState, shell_preferences};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SettingsEffects {
    pub(super) theme_id: String,
    pub(super) rail_order: Option<Vec<String>>,
    pub(super) rail_hidden: Vec<String>,
    pub(super) floating_surfaces: bool,
}

impl SettingsEffects {
    pub(super) fn from_runtime(runtime: &SettingsRuntimeState) -> Self {
        let requested_theme = runtime.text_value("theme");
        let theme_id = known_theme(&requested_theme)
            .then_some(requested_theme)
            .unwrap_or_else(|| "light".to_owned());
        let rail_hidden = runtime.string_list_value("iconRailHidden");
        Self {
            theme_id,
            rail_order: runtime
                .has_key("iconRailOrder")
                .then(|| runtime.string_list_value("iconRailOrder")),
            rail_hidden,
            floating_surfaces: runtime.bool_value("floatingSurfaces"),
        }
    }

    pub(super) fn palette(&self) -> theme::ThemePalette {
        theme::palette_for(&self.theme_id)
    }

    pub(super) fn resolved_rail_order(&self, shell_order: &[String]) -> Vec<String> {
        let preferred = shell_order.iter().map(String::as_str);
        self.normalize_rail_order(preferred)
    }

    pub(super) fn startup_rail_order(&self, shell_order: &[String]) -> Vec<String> {
        let preferred = self
            .rail_order
            .as_deref()
            .unwrap_or(shell_order)
            .iter()
            .map(String::as_str);
        self.normalize_rail_order(preferred.chain(shell_order.iter().map(String::as_str)))
    }

    fn normalize_rail_order<'a>(&self, preferred: impl Iterator<Item = &'a str>) -> Vec<String> {
        let mut resolved = Vec::new();
        for id in preferred {
            if shell_preferences::DEFAULT_RAIL_ORDER.contains(&id)
                && !resolved.iter().any(|current| current == id)
            {
                resolved.push(id.to_owned());
            }
        }
        for id in shell_preferences::DEFAULT_RAIL_ORDER {
            if !resolved.iter().any(|current| current == id) {
                resolved.push(id.to_owned());
            }
        }
        resolved
    }

    pub(super) fn visible_rail_order(&self, shell_order: &[String]) -> Vec<String> {
        self.resolved_rail_order(shell_order)
            .into_iter()
            .filter(|id| !self.rail_hidden.iter().any(|hidden| hidden == id))
            .collect()
    }
}

fn known_theme(theme_id: &str) -> bool {
    THEME_FAMILIES
        .iter()
        .any(|family| theme_id == family.light || theme_id == family.dark)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn runtime(seed: &str) -> SettingsRuntimeState {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("elephant-freya-effects-{stamp}.json"));
        fs::write(&path, seed).expect("seed preferences");
        let state = SettingsRuntimeState::load_from(path.clone());
        let _ = fs::remove_file(PathBuf::from(path));
        state
    }

    #[test]
    fn invalid_theme_uses_source_light_default() {
        assert_eq!(
            SettingsEffects::from_runtime(&runtime(r#"{"theme":"missing"}"#)).theme_id,
            "light"
        );
    }

    #[test]
    fn portable_rail_preferences_extend_legacy_shell_actions() {
        let effects = SettingsEffects::from_runtime(&runtime(
            r#"{"iconRailOrder":["search"],"iconRailHidden":["search"]}"#,
        ));
        assert_eq!(
            effects.visible_rail_order(&["sidebar-toggle".to_owned(), "search".to_owned()]),
            vec!["sidebar-toggle"]
        );
    }
}
