//! Bootstrap for the native shell runtime.
//!
//! State transitions live in the focused history, gestures and preferences
//! modules. This file only loads a vault and composes their initial state.

use std::path::PathBuf;

use crate::vault_adapter::VaultAdapter;

use super::{
    shell_history::NavigationTarget,
    shell_preferences::{self, ShellPreferences},
    ShellState,
};

pub(super) fn load_from_root(root: PathBuf) -> ShellState {
    let mut state = ShellState::empty();
    match VaultAdapter::open(root) {
        Ok(vault) => {
            let preferences = match shell_preferences::read_shell_preferences(vault.root()) {
                Ok(preferences) => preferences,
                Err(error) => {
                    eprintln!("[freya][shell] action=load-preferences failure={error}");
                    state.error = Some(error);
                    ShellPreferences::default()
                }
            };
            state.library.active_vault_id = Some(crate::library_contract::VaultId::new(
                vault.descriptor().id.clone(),
            ));
            state.sidebar_visible = preferences.sidebar_visible;
            state.sidebar_width = preferences.sidebar_width;
            state.rail_order = preferences.rail_order;
            state.vault = Some(vault);
            let preference_error = state.error.clone();
            state.reload_directory("");
            if preference_error.is_some() {
                state.error = preference_error;
            }
            state.navigation_history = vec![NavigationTarget::Directory(String::new())];
            state.navigation_index = 0;
        }
        Err(error) => state.error = Some(error.to_string()),
    }
    state
}
