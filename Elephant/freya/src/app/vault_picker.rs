//! Native vault selection for the Freya development shell.
//!
//! Vault identity and startup selection now live exclusively in the shared
//! `tauri-vaults.json` registry. `freya-startup.json` is retained only as a
//! one-shot legacy migration source so existing development profiles are not
//! stranded.

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

#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub(super) fn pick_vault() -> Result<Option<PathBuf>, String> {
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
    Ok(picked)
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub(super) fn pick_vault() -> Result<Option<PathBuf>, String> {
    Err(format!(
        "Native vault selection is not implemented for {} yet; the mobile storage adapter must provide it.",
        env::consts::OS
    ))
}

/// Consume the pre-registry Freya startup hint once.
///
/// `ShellState::load` immediately registers a returned vault in the canonical
/// `tauri-vaults.json`, so leaving this file behind would recreate two sources
/// of truth. Malformed legacy data remains visible as an error instead of being
/// silently discarded.
pub(super) fn remembered_vault() -> Result<Option<PathBuf>, String> {
    let path = startup_state_path()?;
    let remembered = read_remembered_vault_from(&path)?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("remove migrated startup state {}: {error}", path.display()))?;
        eprintln!(
            "[freya][vault-picker] legacy-startup-migrated path={}",
            path.display()
        );
    }
    Ok(remembered)
}

fn read_remembered_vault_from(path: &Path) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
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

/// Compatibility hook for older callers. The canonical registry has already
/// been persisted before this is called, so the correct action is to ensure
/// the legacy hint no longer exists rather than write another copy.
pub(super) fn remember_vault(_root: &Path) -> Result<(), String> {
    forget_vault()
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
        .ok_or_else(|| "APPDATA is not set; cannot migrate the selected vault".to_string())?;
    Ok(PathBuf::from(app_data).join("Elephant"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_config_dir() -> Result<PathBuf, String> {
    if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config).join("elephant"));
    }
    let home = env::var_os("HOME").ok_or_else(|| {
        "Neither XDG_CONFIG_HOME nor HOME is set; cannot migrate legacy Freya state".to_string()
    })?;
    Ok(PathBuf::from(home).join(".config").join("elephant"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "elephant-freya-vault-picker-{name}-{}-{stamp}",
            std::process::id()
        ))
    }

    fn write_legacy(path: &Path, vault: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            serde_json::to_vec_pretty(&StartupState {
                version: 1,
                last_vault: vault.to_path_buf(),
            })
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn legacy_state_round_trips_unicode_paths_semantically() {
        let state = StartupState {
            version: 1,
            last_vault: PathBuf::from("Café Notes").join("Überblick").join("東京"),
        };
        let encoded = serde_json::to_vec(&state).expect("serialize unicode startup state");
        let decoded: StartupState =
            serde_json::from_slice(&encoded).expect("deserialize unicode startup state");
        assert_eq!(decoded.version, state.version);
        assert_eq!(decoded.last_vault, state.last_vault);
    }

    #[test]
    fn legacy_hint_can_be_read_for_one_shot_migration() {
        let root = test_root("legacy");
        let vault = root.join("Vault With Spaces");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(&vault).unwrap();
        write_legacy(&state_file, &vault);
        assert_eq!(
            read_remembered_vault_from(&state_file).unwrap().as_deref(),
            Some(vault.as_path())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_legacy_hint_does_not_become_an_active_vault() {
        let root = test_root("stale");
        let state_file = root.join("config").join(STATE_FILE);
        let missing_vault = root.join("Missing Vault");
        write_legacy(&state_file, &missing_vault);
        assert_eq!(read_remembered_vault_from(&state_file).unwrap(), None);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_legacy_state_reports_a_parse_error() {
        let root = test_root("malformed");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(state_file.parent().unwrap()).unwrap();
        fs::write(&state_file, "{broken").unwrap();
        let error = read_remembered_vault_from(&state_file).expect_err("malformed state must fail");
        assert!(error.contains("parse startup state"));
        let _ = fs::remove_dir_all(root);
    }
}
