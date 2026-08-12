//! Native library actions backed by the production vault adapter.
//!
//! Card composition and inline rename rendering stay in `library.rs`; this
//! module owns the compact NoteCard action popover and filesystem mutations.

use freya::prelude::*;

use crate::{
    library_contract::{LoadMoreAction, LoadMoreNoop, PageApply},
    theme,
    vault_adapter::PageRequest,
};

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

/// Advance the exact Tauri library window. A single prefetch first reveals any
/// already-buffered render chunk, then may cross one additional backend page
/// boundary. Limiting one wheel dispatch to two backend reads keeps very large
/// vaults incremental while avoiding a Freya ScrollView propagation edge case
/// where the first fetch makes the child scrollable and subsequent wheel events
/// no longer reach the outer pagination sentinel.
///
/// There is deliberately no entry-count cap: later wheel dispatches continue
/// from the accumulated offset until the real backend reports exhaustion.
/// The shell's raw `VaultPage` is extended alongside the typed library state
/// so newly paged notes still open through `ShellState::open_note` rather than
/// a parallel filesystem path.
pub(super) fn load_more_library_entries(mut state: State<ShellState>) -> bool {
    let mut changed = false;
    let mut backend_fetches = 0usize;
    loop {
        let action = state.write().library.load_more();
        match action {
            LoadMoreAction::Noop(LoadMoreNoop::BufferedEntriesRevealed) => {
                eprintln!("[freya][library] pagination:reveal-buffered");
                changed = true;
                continue;
            }
            LoadMoreAction::Noop(LoadMoreNoop::Exhausted) => return changed,
            LoadMoreAction::Noop(LoadMoreNoop::AlreadyLoading) => return changed,
            LoadMoreAction::Noop(LoadMoreNoop::StaleResponse) => return changed,
            LoadMoreAction::Fetch(request) => {
                let vault = state.read().vault.clone();
                let Some(vault) = vault else {
                    let message = "No vault selected.".to_string();
                    let mut next = state.write();
                    next.library.apply_more_error(&request, message.clone());
                    next.error = Some(message);
                    return changed;
                };

                eprintln!(
                    "[freya][library] pagination:fetch-start directory={} offset={} limit={}",
                    request.relative_path.as_str(),
                    request.offset,
                    request.limit
                );
                let adapter_request = PageRequest::new(request.relative_path.as_str().to_string())
                    .with_window(request.offset, request.limit);
                match vault.list(adapter_request) {
                    Ok(page) => {
                        let page_size = request.limit.saturating_sub(1);
                        let backend_has_more = page.has_more;
                        let raw_count = page.entries.len();
                        let raw_entries = page.entries;
                        let contract_entries = raw_entries
                            .iter()
                            .map(super::to_library_entry)
                            .collect::<Vec<_>>();

                        let mut next = state.write();
                        let applied = next.library.apply_more_page(&request, contract_entries);
                        if applied != PageApply::Applied {
                            eprintln!(
                                "[freya][library] pagination:ignored-stale directory={} offset={}",
                                request.relative_path.as_str(),
                                request.offset
                            );
                            return changed;
                        }

                        if let Some(shell_page) = next.page.as_mut() {
                            let existing_paths = shell_page
                                .entries
                                .iter()
                                .map(|entry| entry.path.clone())
                                .collect::<std::collections::HashSet<_>>();
                            shell_page.entries.extend(
                                raw_entries
                                    .into_iter()
                                    .take(page_size)
                                    .filter(|entry| !existing_paths.contains(&entry.path)),
                            );
                            shell_page.has_more = raw_count > page_size || backend_has_more;
                            shell_page.next_offset = shell_page
                                .has_more
                                .then_some(shell_page.entries.len());
                        }
                        next.error = None;
                        eprintln!(
                            "[freya][library] pagination:fetch-complete directory={} offset={} received={} total={}",
                            request.relative_path.as_str(),
                            request.offset,
                            raw_count.min(page_size),
                            next.library.entries.len()
                        );
                        drop(next);

                        changed = true;
                        backend_fetches += 1;
                        let may_have_more = raw_count > page_size || backend_has_more;
                        if backend_fetches < 2 && may_have_more {
                            continue;
                        }
                        return true;
                    }
                    Err(error) => {
                        let message = error.to_string();
                        eprintln!(
                            "[freya][library] pagination:fetch-failure directory={} offset={} error={}",
                            request.relative_path.as_str(),
                            request.offset,
                            message
                        );
                        let mut next = state.write();
                        next.library.apply_more_error(&request, message.clone());
                        next.error = Some(message);
                        return changed;
                    }
                }
            }
        }
    }
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