//! Bootstrap for the native shell runtime.
//!
//! State transitions live in the focused history, gestures and preferences
//! modules. This file only loads a vault and composes their initial state.

use std::path::PathBuf;

use crate::vault_adapter::{metadata, VaultAdapter};

use super::{
    shell_history::NavigationTarget,
    shell_preferences::{self, ShellPreferences},
    ShellState,
};

/// Re-open an already selected/initialized vault.
///
/// This is the restart/test path and mirrors Tauri's `tauri_vaults_get` path:
/// opening must not recreate user content every launch.
pub(super) fn load_from_root(root: PathBuf) -> ShellState {
    match VaultAdapter::open(root) {
        Ok(vault) => load_from_vault(vault),
        Err(error) => error_state(error.to_string()),
    }
}

/// Select a folder as an Elephant vault.
///
/// Tauri's `tauri_vaults_select_path` calls `initialize_vault` before exposing
/// the selected vault. The native shell must preserve that contract rather
/// than treating an arbitrary directory as if it were already an Elephant
/// vault.
pub(super) fn select_root(root: PathBuf) -> ShellState {
    let vault = match VaultAdapter::open(root) {
        Ok(vault) => vault,
        Err(error) => return error_state(error.to_string()),
    };

    let root_string = vault.root().to_string_lossy().into_owned();
    if let Err(error) = metadata::initialize_vault(&root_string) {
        eprintln!(
            "[freya][vault] action=initialize failure path={} error={error}",
            vault.root().display()
        );
        return error_state(format!(
            "Unable to initialize Elephant vault {}: {error}",
            vault.root().display()
        ));
    }

    eprintln!(
        "[freya][vault] action=initialize complete path={}",
        vault.root().display()
    );
    load_from_vault(vault)
}

fn load_from_vault(vault: VaultAdapter) -> ShellState {
    let mut state = ShellState::empty();
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
    state.rail_order = if preferences.rail_order_persisted {
        preferences.rail_order.clone()
    } else {
        let effects = super::settings::SettingsViewState::default().effects();
        effects.startup_rail_order(&preferences.rail_order)
    };
    state.vault = Some(vault);
    let preference_error = state.error.clone();
    state.reload_directory("");
    if preference_error.is_some() {
        state.error = preference_error;
    }
    state.navigation_history = vec![NavigationTarget::Directory(String::new())];
    state.navigation_index = 0;
    state
}

fn error_state(error: String) -> ShellState {
    let mut state = ShellState::empty();
    state.error = Some(error);
    state
}
