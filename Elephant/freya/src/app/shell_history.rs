//! Shell navigation history and production vault transitions.

use crate::{
    editor::EditorDocument, library_contract::RelativePath, navigation_contract::WorkspaceView,
    vault_adapter::VaultEntry,
};
use std::{fs, path::Path};

use super::ShellState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NavigationTarget {
    Directory(String),
    Note(String),
}

impl ShellState {
    pub(super) fn open_workspace(&mut self, view: WorkspaceView) {
        eprintln!(
            "[freya][navigation] action:workspace-open view={}",
            view.source_id()
        );
        self.editor = None;
        self.code_execution = Default::default();
        self.drawing = None;
        self.drawing_path = None;
        if view != WorkspaceView::Canvas {
            self.canvas = None;
        }
        self.menu_open = false;
        self.search_open = false;
        self.settings_open = false;
        if view == WorkspaceView::Calendar {
            if let Some(vault) = self.vault.as_ref() {
                self.calendar.load_for(vault.root());
            }
        }
        if view == WorkspaceView::Chat {
            if let Some(vault) = self.vault.as_ref() {
                self.chat.load_for(vault.root());
            }
        }
        if view == WorkspaceView::Models {
            if let Some(vault) = self.vault.as_ref() {
                self.models.load_for(vault.root());
            }
        }
        self.view = view;
    }

    pub(super) fn open_dashboard(&mut self) {
        let Some(vault) = self.vault.as_ref() else {
            self.error = Some("No vault selected.".to_owned());
            return;
        };
        let path = vault.root().join(".elephantnote").join("Dashboard.md");
        let result = (|| {
            if !path.exists() {
                fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
                fs::write(&path, "# Dashboard\n\n")?;
            }
            Ok::<(), std::io::Error>(())
        })();
        match result {
            Ok(()) => self.open_note_path(".elephantnote/Dashboard.md"),
            Err(error) => {
                self.error = Some(format!("Unable to open Dashboard: {error}"));
                eprintln!("[freya][dashboard] action=open-failure error={error}");
            }
        }
    }

    pub(super) fn open_directory(&mut self, path: String) {
        self.open_directory_with_history(path, true);
    }

    fn open_directory_with_history(&mut self, path: String, record: bool) {
        self.view = WorkspaceView::Notes;
        self.editor = None;
        self.code_execution = Default::default();
        self.editor_tag_draft = None;
        self.library.current_path = RelativePath::from(path.as_str());
        self.reload_directory(&path);
        if record {
            self.record_navigation(NavigationTarget::Directory(path));
        }
    }

    pub(super) fn open_note(&mut self, entry: &VaultEntry) {
        self.open_note_with_history(&entry, true);
    }

    pub(super) fn open_note_path(&mut self, relative_path: &str) {
        let filename = relative_path
            .rsplit('/')
            .next()
            .unwrap_or(relative_path)
            .to_owned();
        let entry = VaultEntry {
            path: relative_path.to_owned(),
            filename: filename.clone(),
            name: filename.clone(),
            title: filename.trim_end_matches(".md").to_owned(),
            entry_type: crate::vault_adapter::EntryKind::Note,
            kind: crate::vault_adapter::EntryKind::Note,
            is_directory: false,
            note_count: 0,
            excerpt: String::new(),
            preview: String::new(),
            tags: Vec::new(),
            updated_at: String::new(),
            children_preview: Vec::new(),
            drawing_preview: None,
            full_path: relative_path.to_owned(),
        };
        self.open_note_with_history(&entry, true);
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
                self.code_execution = Default::default();
                self.drawing = None;
                self.drawing_path = None;
                self.editor_tag_draft = None;
                self.view = crate::navigation_contract::WorkspaceView::Notes;
                self.search_open = false;
                self.settings_open = false;
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
                match self
                    .vault
                    .as_ref()
                    .and_then(|vault| vault.find_entry(&path).ok())
                {
                    Some(entry) => self.open_note_with_history(&entry, false),
                    None => {
                        self.error =
                            Some(format!("Navigation target is no longer present: {path}"));
                    }
                }
            }
        }
    }
}
