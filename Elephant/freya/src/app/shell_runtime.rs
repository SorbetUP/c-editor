//! Runtime state transitions shared by the native shell renderers.
//!
//! This module owns navigation history, pointer gesture state and the durable
//! shell preferences stored in the native Tauri workspace file. Rendering
//! modules only call the `pub(super)` transitions below.

use freya::prelude::*;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    editor::EditorDocument,
    library_contract::RelativePath,
    navigation_contract::SidebarWidth,
    vault_adapter::{VaultAdapter, VaultEntry},
    vault_layout,
};

use super::ShellState;

pub(super) const DEFAULT_RAIL_ORDER: [&str; 2] = ["sidebar-toggle", "search"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NavigationTarget {
    Directory(String),
    Note(String),
}

#[derive(Clone, Debug)]
pub(super) struct RailDragState {
    pub(super) source: String,
    pub(super) start_x: f64,
    pub(super) start_y: f64,
    pub(super) moved: bool,
}

#[derive(Clone, Debug)]
pub(super) struct SidebarResizeState {
    pub(super) start_x: f64,
    pub(super) start_width: SidebarWidth,
}

#[derive(Clone, Debug)]
struct ShellPreferences {
    sidebar_visible: bool,
    sidebar_width: SidebarWidth,
    rail_order: Vec<String>,
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

pub(super) fn load_from_root(root: PathBuf) -> ShellState {
    let mut state = ShellState::empty();
    match VaultAdapter::open(root) {
        Ok(vault) => {
            let preferences = match read_shell_preferences(vault.root()) {
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

impl ShellState {
    pub(super) fn open_directory(&mut self, path: String) {
        self.open_directory_with_history(path, true);
    }

    fn open_directory_with_history(&mut self, path: String, record: bool) {
        self.view = crate::navigation_contract::WorkspaceView::Notes;
        self.editor = None;
        self.library.current_path = RelativePath::from(path.as_str());
        self.reload_directory(&path);
        if record {
            self.record_navigation(NavigationTarget::Directory(path));
        }
    }

    pub(super) fn open_note(&mut self, entry: &VaultEntry) {
        self.open_note_with_history(entry, true);
    }

    fn open_note_with_history(&mut self, entry: &VaultEntry, record: bool) {
        eprintln!(
            "[freya][editor] action:start action=open_note path={}",
            entry.path
        );
        let Some(vault) = self.vault.as_ref() else {
            self.error = Some("No vault selected.".to_string());
            eprintln!("[freya][editor] action:failure action=open_note reason=no_vault");
            return;
        };
        let path = vault.root().join(&entry.path);
        match EditorDocument::load(&path) {
            Ok(document) => {
                self.editor = Some(document);
                self.error = None;
                if record {
                    self.record_navigation(NavigationTarget::Note(entry.path.clone()));
                }
                eprintln!(
                    "[freya][editor] action:complete action=open_note path={}",
                    entry.path
                );
            }
            Err(error) => {
                eprintln!(
                    "[freya][editor] action:failure action=open_note path={} error={error}",
                    entry.path
                );
                self.error = Some(error.to_string());
            }
        }
    }

    fn record_navigation(&mut self, target: NavigationTarget) {
        if self.navigation_history.get(self.navigation_index) == Some(&target) {
            return;
        }
        self.navigation_history.truncate(self.navigation_index + 1);
        self.navigation_history.push(target);
        if self.navigation_history.len() > 100 {
            self.navigation_history.drain(..20);
        }
        self.navigation_index = self.navigation_history.len().saturating_sub(1);
        eprintln!(
            "[freya][navigation] action:record index={} length={}",
            self.navigation_index,
            self.navigation_history.len()
        );
    }

    pub(super) fn can_go_back(&self) -> bool {
        self.navigation_index > 0
    }

    pub(super) fn can_go_forward(&self) -> bool {
        self.navigation_index + 1 < self.navigation_history.len()
    }

    pub(super) fn navigate_back(&mut self) {
        if !self.can_go_back() {
            eprintln!("[freya][navigation] action:back-disabled");
            return;
        }
        self.navigation_index -= 1;
        self.open_navigation_target();
    }

    pub(super) fn navigate_forward(&mut self) {
        if !self.can_go_forward() {
            eprintln!("[freya][navigation] action:forward-disabled");
            return;
        }
        self.navigation_index += 1;
        self.open_navigation_target();
    }

    fn open_navigation_target(&mut self) {
        let Some(target) = self.navigation_history.get(self.navigation_index).cloned() else {
            return;
        };
        eprintln!(
            "[freya][navigation] action:open index={} target={target:?}",
            self.navigation_index
        );
        match target {
            NavigationTarget::Directory(path) => self.open_directory_with_history(path, false),
            NavigationTarget::Note(path) => {
                let parent = path
                    .rsplit_once('/')
                    .map(|(parent, _)| parent)
                    .unwrap_or("");
                self.library.current_path = RelativePath::from(parent);
                self.reload_directory(parent);
                let entry = self
                    .page
                    .as_ref()
                    .and_then(|page| page.entries.iter().find(|entry| entry.path == path))
                    .cloned();
                if let Some(entry) = entry {
                    self.open_note_with_history(&entry, false);
                } else {
                    self.error = Some(format!("Navigation target is no longer present: {path}"));
                }
            }
        }
    }

    pub(super) fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        self.persist_shell_preferences();
    }

    pub(super) fn begin_sidebar_resize(&mut self, x: f64) {
        self.sidebar_resize = Some(SidebarResizeState {
            start_x: x,
            start_width: self.sidebar_width,
        });
        eprintln!(
            "[freya][sidebar] action:resize-start width={} x={x}",
            self.sidebar_width.get()
        );
    }

    pub(super) fn update_sidebar_resize(&mut self, x: f64) {
        let Some(resize) = self.sidebar_resize.as_ref() else {
            return;
        };
        self.sidebar_width =
            SidebarWidth::from_number(f64::from(resize.start_width.get()) + x - resize.start_x);
    }

    pub(super) fn finish_sidebar_resize(&mut self, x: f64) {
        self.update_sidebar_resize(x);
        if self.sidebar_resize.take().is_some() {
            eprintln!(
                "[freya][sidebar] action:resize-complete width={} x={x}",
                self.sidebar_width.get()
            );
            self.persist_shell_preferences();
        }
    }

    pub(super) fn resize_sidebar_by(&mut self, delta: f64) {
        self.sidebar_width = SidebarWidth::from_number(f64::from(self.sidebar_width.get()) + delta);
        eprintln!(
            "[freya][sidebar] action:resize-keyboard width={}",
            self.sidebar_width.get()
        );
        self.persist_shell_preferences();
    }

    pub(super) fn begin_rail_drag(&mut self, source: &str, x: f64, y: f64) {
        self.rail_drag = Some(RailDragState {
            source: source.to_string(),
            start_x: x,
            start_y: y,
            moved: false,
        });
        self.rail_drop_target = None;
        eprintln!("[freya][rail] action:drag-start source={source}");
    }

    pub(super) fn update_rail_drag(&mut self, target: &str, x: f64, y: f64) {
        let Some(drag) = self.rail_drag.as_mut() else {
            return;
        };
        if drag.source == target {
            return;
        }
        if (x - drag.start_x).abs() >= 4. || (y - drag.start_y).abs() >= 4. {
            drag.moved = true;
            self.rail_drop_target = Some(target.to_string());
        }
    }

    pub(super) fn finish_rail_drag(&mut self, target: &str) -> bool {
        let Some(drag) = self.rail_drag.take() else {
            return false;
        };
        let was_drag = drag.moved;
        if was_drag && drag.source != target {
            if let Some(source_index) = self.rail_order.iter().position(|id| id == &drag.source) {
                let source = self.rail_order.remove(source_index);
                let target_index = self
                    .rail_order
                    .iter()
                    .position(|id| id == target)
                    .unwrap_or(self.rail_order.len());
                self.rail_order.insert(target_index, source);
                eprintln!(
                    "[freya][rail] action:drag-complete source={} target={target}",
                    drag.source
                );
                self.persist_shell_preferences();
            }
        }
        self.rail_drop_target = None;
        was_drag
    }

    fn persist_shell_preferences(&mut self) {
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

fn read_shell_preferences(root: &Path) -> Result<ShellPreferences, String> {
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
