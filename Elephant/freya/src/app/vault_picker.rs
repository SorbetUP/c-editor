//! Native vault selection and app-level startup persistence for the Freya shell.
//!
//! The chooser remains the operating-system picker. Elephant only persists a
//! successfully opened path as a startup hint; the vault adapter still owns
//! canonicalization and validation when the shell opens it.

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

pub(super) fn remembered_vault() -> Result<Option<PathBuf>, String> {
    read_remembered_vault_from(&startup_state_path()?)
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

/// Persist only after `VaultAdapter::open` has succeeded.
pub(super) fn remember_vault(root: &Path) -> Result<(), String> {
    remember_vault_at(&startup_state_path()?, root)
}

fn remember_vault_at(path: &Path, root: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("startup state path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create startup state directory {}: {error}",
            parent.display()
        )
    })?;

    let state = StartupState {
        version: 1,
        last_vault: root.to_path_buf(),
    };
    let encoded = serde_json::to_vec_pretty(&state)
        .map_err(|error| format!("encode startup state: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, encoded)
        .map_err(|error| format!("write startup state {}: {error}", temporary.display()))?;

    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("replace startup state {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, path)
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

    #[test]
    fn startup_state_round_trips_paths_with_spaces() {
        let state = StartupState {
            version: 1,
            last_vault: PathBuf::from("Elephant Vault").join("Test"),
        };
        let encoded = serde_json::to_string(&state).expect("serialize startup state");
        let decoded: StartupState =
            serde_json::from_str(&encoded).expect("deserialize startup state");
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.last_vault, state.last_vault);
    }

    #[test]
    fn startup_state_round_trips_unicode_paths_semantically() {
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
    fn successful_selection_hint_is_reloaded_from_real_state_file() {
        let root = test_root("round-trip");
        let vault = root.join("Vault With Spaces");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(&vault).expect("create vault fixture");

        remember_vault_at(&state_file, &vault).expect("persist startup hint");
        let restored = read_remembered_vault_from(&state_file).expect("reload startup hint");
        assert_eq!(restored.as_deref(), Some(vault.as_path()));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn repeated_selection_replaces_the_hint_with_the_latest_valid_vault() {
        let root = test_root("replace");
        let first = root.join("First Vault");
        let second = root.join("Second Vault");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(&first).expect("create first vault");
        fs::create_dir_all(&second).expect("create second vault");

        remember_vault_at(&state_file, &first).expect("persist first startup hint");
        remember_vault_at(&state_file, &second).expect("replace startup hint");

        assert_eq!(
            read_remembered_vault_from(&state_file).expect("reload replaced hint"),
            Some(second)
        );
        assert!(
            !state_file.with_extension("json.tmp").exists(),
            "successful replacement must not leave a temporary file"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_selection_hint_returns_to_picker_instead_of_failing_startup() {
        let root = test_root("stale");
        let state_file = root.join("config").join(STATE_FILE);
        let missing_vault = root.join("Missing Vault");
        fs::create_dir_all(state_file.parent().expect("config parent")).expect("create config");
        remember_vault_at(&state_file, &missing_vault).expect("persist stale hint");

        assert_eq!(
            read_remembered_vault_from(&state_file).expect("read stale hint"),
            None
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unsupported_startup_state_version_is_ignored_safely() {
        let root = test_root("future-version");
        let vault = root.join("Vault");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(&vault).expect("create vault");
        fs::create_dir_all(state_file.parent().expect("config parent")).expect("create config");
        fs::write(
            &state_file,
            serde_json::json!({"version": 99, "lastVault": vault}).to_string(),
        )
        .expect("write future state");

        assert_eq!(
            read_remembered_vault_from(&state_file).expect("future versions are ignored"),
            None
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_startup_state_reports_a_parse_error() {
        let root = test_root("malformed");
        let state_file = root.join("config").join(STATE_FILE);
        fs::create_dir_all(state_file.parent().expect("config parent")).expect("create config");
        fs::write(&state_file, "{broken").expect("write malformed state");

        let error = read_remembered_vault_from(&state_file).expect_err("malformed state must fail");
        assert!(error.contains("parse startup state"));

        let _ = fs::remove_dir_all(root);
    }
}
