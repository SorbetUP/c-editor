//! Native library actions backed by the production vault adapter.
//!
//! Card composition and inline rename rendering stay in `library.rs`; this
//! module owns the compact NoteCard action popover and filesystem mutations.

use freya::prelude::*;

use crate::theme;

use super::library_icons::{svg_icon, Icon as LibraryIcon};
use super::super::ShellState;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CardMenuState {
    pub(super) open: bool,
    pub(super) renaming: bool,
}

pub(super) fn card_action_menu(
    path: String,
    title: String,
    is_folder: bool,
    state: State<ShellState>,
    card_menu_state: State<CardMenuState>,
    rename_value: State<String>,
) -> Element {
    let mut rename_menu_state = card_menu_state;
    let mut rename_value_state = rename_value;
    let mut delete_menu_state = card_menu_state;
    let mut delete_state = state;
    let path_for_delete = path;
    let rename_title = title;

    rect()
        .position(Position::new_absolute().top(42.).right(8.))
        .width(Size::px(78.))
        .height(Size::px(42.))
        .horizontal()
        .spacing(4.)
        .padding(Gaps::new_all(5.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .a11y_alt(if is_folder {
            "Folder actions"
        } else {
            "Note actions"
        })
        .on_mouse_up(|event: Event<MouseEventData>| event.stop_propagation())
        .child(
            rect()
                .width(Size::px(32.))
                .height(Size::px(32.))
                .center()
                .with_corner_radius(8.)
                .a11y_alt(if is_folder { "Rename folder" } else { "Rename" })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    rename_value_state.set(rename_title.clone());
                    let mut menu = rename_menu_state.write();
                    menu.open = false;
                    menu.renaming = true;
                })
                .child(svg_icon(
                    LibraryIcon::Pencil,
                    theme::color(theme::TEXT),
                    20.,
                )),
        )
        .child(
            rect()
                .width(Size::px(32.))
                .height(Size::px(32.))
                .center()
                .with_corner_radius(8.)
                .a11y_alt(if is_folder { "Delete folder" } else { "Delete" })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    if delete_library_entry(&mut delete_state, &path_for_delete) {
                        let mut menu = delete_menu_state.write();
                        menu.open = false;
                        menu.renaming = false;
                    }
                })
                .child(svg_icon(
                    LibraryIcon::Trash2,
                    theme::color(theme::DANGER),
                    20.,
                )),
        )
        .into_element()
}

/// Match the Vue `vaultStore.createNote()` orchestration: create through the
/// production backend, refresh the current directory, then open the exact note
/// returned by the backend through the shell's normal note-navigation path.
/// No optimistic card is inserted before filesystem success.
pub(super) fn create_note_and_open(mut state: State<ShellState>) -> bool {
    let (vault, directory) = {
        let snapshot = state.read();
        (
            snapshot.vault.clone(),
            snapshot.library.current_path.as_str().to_string(),
        )
    };
    let Some(vault) = vault else {
        state.write().error = Some("No vault selected.".to_string());
        eprintln!("[freya][library] action:failure action=Note reason=no_vault");
        return false;
    };

    eprintln!("[freya][library] action:start action=Note directory={directory}");
    match vault.create_note(Some(directory.clone()), None, None) {
        Ok(entry) => {
            state.write().reload_directory(&directory);
            state.write().open_note(&entry);
            let opened = {
                let snapshot = state.read();
                snapshot.editor.is_some() && snapshot.error.is_none()
            };
            if opened {
                eprintln!(
                    "[freya][library] action:complete action=Note path={}",
                    entry.path
                );
            } else {
                eprintln!(
                    "[freya][library] action:failure action=open-created-note path={}",
                    entry.path
                );
            }
            opened
        }
        Err(error) => {
            eprintln!("[freya][library] action:failure action=Note error={error}");
            state.write().error = Some(error.to_string());
            false
        }
    }
}

pub(super) fn rename_library_entry(state: &mut State<ShellState>, path: &str, title: &str) -> bool {
    let next_title = title.trim();
    if next_title.is_empty() {
        state.write().error = Some("Entry name cannot be empty.".to_string());
        return false;
    }
    let (vault, directory) = {
        let snapshot = state.read();
        (
            snapshot.vault.clone(),
            snapshot.library.current_path.as_str().to_string(),
        )
    };
    let Some(vault) = vault else {
        state.write().error = Some("No vault selected.".to_string());
        return false;
    };
    eprintln!("[freya][library] action:start action=rename path={path}");
    match vault.rename(path, next_title.to_string()) {
        Ok(()) => {
            let mut next = state.write();
            next.hovered_target = None;
            next.reload_directory(&directory);
            eprintln!("[freya][library] action:complete action=rename path={path}");
            true
        }
        Err(error) => {
            eprintln!("[freya][library] action:failure action=rename path={path} error={error}");
            state.write().error = Some(error.to_string());
            false
        }
    }
}

pub(super) fn delete_library_entry(state: &mut State<ShellState>, path: &str) -> bool {
    let (vault, directory) = {
        let snapshot = state.read();
        (
            snapshot.vault.clone(),
            snapshot.library.current_path.as_str().to_string(),
        )
    };
    let Some(vault) = vault else {
        state.write().error = Some("No vault selected.".to_string());
        return false;
    };
    eprintln!("[freya][library] action:start action=delete path={path}");
    match vault.delete(path) {
        Ok(result) => {
            let mut next = state.write();
            next.hovered_target = None;
            next.reload_directory(&directory);
            eprintln!(
                "[freya][library] action:complete action=delete path={} trash={}",
                path, result.trash_path
            );
            true
        }
        Err(error) => {
            eprintln!("[freya][library] action:failure action=delete path={path} error={error}");
            state.write().error = Some(error.to_string());
            false
        }
    }
}
