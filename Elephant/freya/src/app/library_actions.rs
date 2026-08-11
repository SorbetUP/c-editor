//! Overlay card actions for the native library surface.
//!
//! Card composition stays in `library.rs`; this module owns the source
//! NoteCard/FolderCard menu and its real vault mutations.

use freya::prelude::*;

use crate::theme;

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
    renaming: bool,
) -> Element {
    if renaming {
        let mut rename_state = state;
        let mut rename_menu_state = card_menu_state;
        let rename_input = rename_value;
        let path_for_rename = path.clone();
        let mut cancel_menu_state = card_menu_state;
        return rect()
            .position(Position::new_absolute().top(42.).right(8.))
            .width(Size::px(220.))
            .padding(Gaps::new_all(8.))
            .background(theme::color(theme::SURFACE))
            .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
            .with_corner_radius(7.)
            .a11y_alt("Rename menu")
            .child(
                rect()
                    .width(Size::fill())
                    .on_mouse_up(|event: Event<MouseEventData>| event.stop_propagation())
                    .child(
                        Input::new(rename_input)
                            .width(Size::fill())
                            .placeholder(format!("Rename {title}"))
                            .on_submit(move |next_title: String| {
                                if rename_library_entry(
                                    &mut rename_state,
                                    &path_for_rename,
                                    &next_title,
                                ) {
                                    let mut menu = rename_menu_state.write();
                                    menu.open = false;
                                    menu.renaming = false;
                                }
                            }),
                    ),
            )
            .child(
                rect()
                    .height(Size::px(28.))
                    .padding(Gaps::new(0., 8., 0., 8.))
                    .center()
                    .a11y_alt("Cancel rename")
                    .on_mouse_up(move |event: Event<MouseEventData>| {
                        event.stop_propagation();
                        let mut menu = cancel_menu_state.write();
                        menu.open = false;
                        menu.renaming = false;
                    })
                    .child(label().text("Cancel")),
            )
            .into_element();
    }

    let mut rename_menu_state = card_menu_state;
    let mut rename_value_state = rename_value;
    let mut delete_menu_state = card_menu_state;
    let mut delete_state = state;
    let path_for_delete = path.clone();
    let action_label = if is_folder {
        "Close folder actions"
    } else {
        "Note actions"
    };
    rect()
        .position(Position::new_absolute().top(42.).right(8.))
        .width(Size::px(if is_folder { 118. } else { 82. }))
        .horizontal()
        .spacing(6.)
        .padding(Gaps::new_all(6.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(7.)
        .a11y_alt(action_label)
        .child(
            rect()
                .height(Size::px(28.))
                .padding(Gaps::new(0., 8., 0., 8.))
                .center()
                .a11y_alt(if is_folder { "Rename folder" } else { "Rename" })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    let mut menu = rename_menu_state.write();
                    menu.open = true;
                    menu.renaming = true;
                    rename_value_state.set(String::new());
                })
                .child(label().text("Rename")),
        )
        .child(
            rect()
                .height(Size::px(28.))
                .padding(Gaps::new(0., 8., 0., 8.))
                .center()
                .a11y_alt(if is_folder { "Delete folder" } else { "Delete" })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    if delete_library_entry(&mut delete_state, &path_for_delete) {
                        delete_menu_state.write().open = false;
                    }
                })
                .child(label().text("Delete")),
        )
        .into_element()
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
