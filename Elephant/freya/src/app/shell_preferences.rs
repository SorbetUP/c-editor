//! Durable shell preferences stored in the native Tauri workspace metadata.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{navigation_contract::SidebarWidth, vault_layout};

use super::ShellState;

pub(super) const DEFAULT_RAIL_ORDER: [&str; 2] = ["sidebar-toggle", "search"];

#[derive(Clone, Debug)]
pub(super) struct ShellPreferences {
    pub(super) sidebar_visible: bool,
    pub(super) sidebar_width: SidebarWidth,
    pub(super) rail_order: Vec<String>,
}

impl Default for ShellPreferences {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            sidebar_width: SidebarWidth::default(),
            rail_order: default_rail_order(),
        }
    }
}

pub(super) fn default_rail_order() -> Vec<String> {
    DEFAULT_RAIL_ORDER
        .iter()
        .map(|id| (*id).to_string())
        .collect()
}

pub(super) fn read_shell_preferences(root: &Path) -> Result<ShellPreferences, String> {
    let canonical = shell_preferences_path(root);
    let legacy = legacy_shell_preferences_path(root);
    let Some(path) = [canonical, legacy].into_iter().find(|path| path.exists()) else {
        return Ok(ShellPreferences::default());
    };
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("read workspace preferences {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("parse workspace preferences {}: {error}", path.display()))?;
    let object = value.as_object().ok_or_else(|| {
        format!(
            "workspace preferences must be a JSON object: {}",
            path.display()
        )
    })?;
    let Some(shell) = object.get("freyaShell") else {
        return Ok(ShellPreferences::default());
    };
    let shell = shell.as_object().ok_or_else(|| {
        format!(
            "workspace freyaShell preferences must be a JSON object: {}",
            path.display()
        )
    })?;
    let mut preferences = ShellPreferences::default();
    if let Some(visible) = shell.get("sidebarVisible").and_then(Value::as_bool) {
        preferences.sidebar_visible = visible;
    }
    if let Some(width) = shell.get("sidebarWidth").and_then(Value::as_f64) {
        preferences.sidebar_width = SidebarWidth::from_number(width);
    }
    if let Some(order) = shell.get("railOrder").and_then(Value::as_array) {
        let mut normalized = Vec::new();
        for id in order.iter().filter_map(Value::as_str) {
            if DEFAULT_RAIL_ORDER.contains(&id) && !normalized.iter().any(|item| item == id) {
                normalized.push(id.to_string());
            }
        }
        for id in DEFAULT_RAIL_ORDER {
            if !normalized.iter().any(|item| item == id) {
                normalized.push(id.to_string());
            }
        }
        preferences.rail_order = normalized;
    }
    Ok(preferences)
}

impl ShellState {
    pub(super) fn persist_shell_preferences(&mut self) {
        let Some(vault) = self.vault.as_ref() else {
            return;
        };
        if let Err(error) = write_shell_preferences(
            vault.root(),
            self.sidebar_visible,
            self.sidebar_width,
            &self.rail_order,
        ) {
            eprintln!("[freya][shell] action:persist-failure error={error}");
            self.error = Some(format!("Unable to persist shell preferences: {error}"));
        }
    }
}

fn shell_preferences_path(root: &Path) -> PathBuf {
    vault_layout::config_file(root, vault_layout::WORKSPACE_FILE)
}

fn legacy_shell_preferences_path(root: &Path) -> PathBuf {
    vault_layout::hidden_root(root).join(vault_layout::WORKSPACE_FILE)
}

fn write_shell_preferences(
    root: &Path,
    sidebar_visible: bool,
    sidebar_width: SidebarWidth,
    rail_order: &[String],
) -> Result<(), String> {
    let path = shell_preferences_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| format!("workspace path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    let mut document = if path.exists() {
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("read workspace preferences {}: {error}", path.display()))?;
        serde_json::from_str::<Value>(&raw)
            .map_err(|error| format!("parse workspace preferences {}: {error}", path.display()))?
    } else {
        json!({
            "version": 1,
            "schemaVersion": vault_layout::SCHEMA_VERSION,
        })
    };
    let object = document.as_object_mut().ok_or_else(|| {
        format!(
            "refusing to replace non-object workspace preferences: {}",
            path.display()
        )
    })?;
    object.insert(
        "freyaShell".to_string(),
        json!({
            "sidebarVisible": sidebar_visible,
            "sidebarWidth": sidebar_width.get(),
            "railOrder": rail_order,
        }),
    );
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("replace {}: {error}", path.display()))?;
    eprintln!(
        "[freya][shell] action:persist-complete path={} width={} visible={} railOrder={:?}",
        path.display(),
        sidebar_width.get(),
        sidebar_visible,
        rail_order
    );
    Ok(())
}
