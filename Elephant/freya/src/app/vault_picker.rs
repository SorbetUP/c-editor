//! Native vault selection and app-level startup persistence for the Freya shell.
//!
//! This deliberately lives outside the vault itself: a user must be able to
//! choose a vault before Elephant knows where that vault's `.elephantnote`
//! metadata is. The selected path is stored in the platform's normal config
//! directory and remains only a convenience hint; `VaultAdapter::open` still
//! validates/canonicalizes the directory every time it is opened.

use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const STATE_FILE: &str = "freya-startup.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartupState {
    version: u32,
    last_vault: PathBuf,
}

/// Open the operating-system's native directory chooser.
///
/// `rfd` uses the platform native dialog on macOS/Windows and the desktop
/// portal/native backend on Linux. Cancellation is intentionally not an error.
pub(super) fn pick_vault() -> Option<PathBuf> {
    eprintln!("[freya][vault-picker] action:start");
    let picked = rfd::FileDialog::new()
        .set_title("Choose an Elephant vault")
        .set_can_create_directories(true)
        .pick_folder();
    match picked.as_ref() {
        Some(path) => eprintln!(
            "[freya][vault-picker] action:complete path={}",
            path.display()
        ),
        None => eprintln!("[freya][vault-picker] action:cancel"),
    }
    picked
}

/// Read the last successfully opened vault, if one was persisted and still
/// exists. A stale path never prevents the picker from being displayed.
pub(super) fn remembered_vault() -> Result<Option<PathBuf>, String> {
    let path = startup_state_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("read startup state {}: {error}", path.display()))?;
    let state: StartupState = serde_json::from_str(&raw)
        .map_err(|error| format!("parse startup state {}: {error}", path.display()))?;
    if state.version != 1 {
        return Ok(None);
    }
    if !state.last_vault.is_dir() {
        eprintln!(
            "[freya][vault-picker] remembered-vault-stale path={}",
            state.last_vault.display()
        );
        return Ok(None);
    }
    Ok(Some(state.last_vault))
}

/// Persist only after `VaultAdapter::open` has succeeded.
pub(super) fn remember_vault(root: &Path) -> Result<(), String> {
    let path = startup_state_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("startup state path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("create startup state directory {}: {error}", parent.display()))?;

    let state = StartupState {
        version: 1,
        last_vault: root.to_path_buf(),
    };
    let encoded = serde_json::to_vec_pretty(&state)
        .map_err(|error| format!("encode startup state: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, encoded)
        .map_err(|error| format!("write startup state {}: {error}", temporary.display()))?;

    // Windows does not replace an existing destination with rename(). Remove
    // the tiny previous state file first; failure is surfaced rather than
    // silently ignoring persistence.
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("replace startup state {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, &path)
        .map_err(|error| format!("install startup state {}: {error}", path.display()))?;
    eprintln!(
        "[freya][vault-picker] remember-complete path={} vault={}",
        path.display(),
        root.display()
    );
    Ok(())
}

#[allow(dead_code)]
pub(super) fn forget_vault() -> Result<(), String> {
    let path = startup_state_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("remove startup state {}: {error}", path.display()))?;
    }
    Ok(())
}

fn startup_state_path() -> Result<PathBuf, String> {
    Ok(platform_config_dir()?.join(STATE_FILE))
}

#[cfg(target_os = "macos")]
fn platform_config_dir() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Elephant"))
}

#[cfg(target_os = "windows")]
fn platform_config_dir() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .ok_or_else(|| "APPDATA is not set; cannot persist the selected vault".to_string())?;
    Ok(PathBuf::from(app_data).join("Elephant"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_config_dir() -> Result<PathBuf, String> {
    if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config).join("elephant"));
    }
    let home = env::var_os("HOME").ok_or_else(|| {
        "Neither XDG_CONFIG_HOME nor HOME is set; cannot persist the selected vault".to_string()
    })?;
    Ok(PathBuf::from(home).join(".config").join("elephant"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_state_round_trips_paths_with_spaces() {
        let state = StartupState {
            version: 1,
            last_vault: PathBuf::from("/tmp/Elephant Vault/Test"),
        };
        let encoded = serde_json::to_string(&state).expect("serialize startup state");
        let decoded: StartupState =
            serde_json::from_str(&encoded).expect("deserialize startup state");
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.last_vault, state.last_vault);
    }
}
