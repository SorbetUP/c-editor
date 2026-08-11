//! Freya conversion of LibraryToolbar, CreateEntryMenu, LibraryGrid and NoteCard.

use freya::prelude::*;

use crate::{
    editor::EditorDocument,
    library_contract::{
        EntryKind as ContractKind, EntryTitle, EntryType, LibraryEntry, RelativePath, ViewMode,
    },
    navigation_contract::WorkspaceView,
    theme,
    vault_adapter::{EntryKind, VaultEntry},
};

use super::{editor_view, route_notice, ShellState};

pub(super) fn main_content(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let body = if snapshot.editor.is_some() {
        editor_view::note_editor_host(state)
    } else if snapshot.view == WorkspaceView::Notes {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(library_toolbar(state))
            .child(library_grid(state))
            .into_element()
    } else if snapshot.search_open {
        route_notice("Search", "Search notes")
    } else if snapshot.settings_open {
        route_notice("Settings", "Settings")
    } else {
        route_notice(snapshot.view.source_id(), snapshot.view.source_id())
    };
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .padding(Gaps::new(8., 12., 12., 12.))
        .maybe_child(
            snapshot
                .error
                .clone()
                .map(|error| route_notice("Library error", &error)),
        )
        .child(body)
        .into_element()
}

fn library_toolbar(mut state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let create = rect()
        .width(Size::px(56.))
        .height(Size::px(56.))
        .center()
        .background(theme::color(theme::PRIMARY))
        .with_corner_radius(11.)
        .on_mouse_up(move |_| state.write().menu_open = true)
        .a11y_alt("Create")
        .child(
            label()
                .font_size(28.)
                .color(theme::color(theme::TEXT))
                .text("+"),
        );
    let sort = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().library.cycle_sort())
        .a11y_alt(format!("Sort: {}", snapshot.library.sort.as_contract()))
        .child(label().font_size(19.).text("↕"));
    let view = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().library.cycle_view())
        .a11y_alt(if snapshot.library.view_mode == ViewMode::Grid {
            "Show notes as list"
        } else {
            "Show notes as grid"
        })
        .child(
            label()
                .font_size(19.)
                .text(if snapshot.library.view_mode == ViewMode::Grid {
                    "☷"
                } else {
                    "▦"
                }),
        );
    rect()
        .position(Position::new_absolute().left(0.).top(0.))
        .width(Size::fill())
        .height(Size::px(72.))
        .padding(Gaps::new(8., 12., 8., 12.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(create)
        .child(rect().horizontal().spacing(10.).child(sort).child(view))
        .into_element()
}

pub(super) fn create_entry_menu(state: State<ShellState>) -> Element {
    let item = |action, icon, title, description| {
        let mut state = state;
        rect()
            .width(Size::fill())
            .height(Size::px(64.))
            .padding(Gaps::new_all(10.))
            .horizontal()
            .spacing(12.)
            .with_corner_radius(10.)
            .on_mouse_up(move |_| {
                state.write().menu_open = false;
                state.write().create(action);
            })
            .a11y_alt(title)
            .child(label().font_size(21.).text(icon))
            .child(
                rect()
                    .spacing(2.)
                    .child(label().font_weight(FontWeight::BOLD).text(title))
                    .child(label().color(theme::color(theme::MUTED)).text(description)),
            )
            .into_element()
    };
    rect()
        .position(
            Position::new_absolute()
                .left(78.)
                .top(theme::TOPBAR_HEIGHT + 68.),
        )
        .width(Size::px(280.))
        .height(Size::px(248.))
        .padding(Gaps::new_all(8.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(14.)
        .layer(Layer::OverlayLevel(10))
        .child(
            label()
                .padding(Gaps::new_all(8.))
                .font_size(12.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(theme::MUTED))
                .text("CREATE"),
        )
        .child(item(
            crate::library_contract::CreateAction::Note,
            "▤",
            "Note",
            "Create a new note",
        ))
        .child(item(
            crate::library_contract::CreateAction::Drawing,
            "✎",
            "Drawing",
            "Open a new Excalidraw canvas",
        ))
        .child(item(
            crate::library_contract::CreateAction::Folder,
            "▱",
            "Folder",
            "Organize notes in a folder",
        ))
        .into_element()
}

fn library_grid(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let entries = snapshot
        .library
        .visible_entries()
        .into_iter()
        .map(|entry| library_card(entry, snapshot.library.view_mode, state))
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return route_notice("LibraryGrid", "No visible notes");
    }
    let surface = if snapshot.library.view_mode == ViewMode::Grid {
        rect()
            .horizontal()
            .content(Content::wrap_spacing(10.))
            .spacing(10.)
            .children(entries)
    } else {
        rect().spacing(8.).children(entries)
    };
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(72., 0., 0., 0.))
        .child(surface)
        .into_element()
}

fn library_card(entry: &LibraryEntry, mode: ViewMode, state: State<ShellState>) -> Element {
    let path = entry.path.as_str().to_string();
    let title = entry.title.as_str().to_string();
    let is_folder = matches!(entry.effective_kind(), ContractKind::Folder);
    let height = if mode == ViewMode::Grid {
        theme::CARD_HEIGHT
    } else {
        theme::LIST_CARD_HEIGHT
    };
    let mut state_for_open = state;
    let path_for_open = path.clone();
    rect()
        .width(if mode == ViewMode::Grid {
            Size::px(240.)
        } else {
            Size::fill()
        })
        .height(Size::px(height))
        .padding(Gaps::new_all(10.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .on_mouse_up(move |_| {
            if is_folder {
                state_for_open.write().open_directory(path_for_open.clone());
            } else if let Some(vault) = state_for_open.read().vault.clone() {
                let full = vault.root().join(&path_for_open);
                match EditorDocument::load(full) {
                    Ok(document) => state_for_open.write().editor = Some(document),
                    Err(error) => state_for_open.write().error = Some(error.to_string()),
                }
            }
        })
        .a11y_alt(title.clone())
        .child(
            label()
                .font_size(14.)
                .font_weight(FontWeight::BOLD)
                .text(format!("{}  {title}", if is_folder { "▱" } else { "▤" })),
        )
        .child(if mode == ViewMode::Grid {
            rect()
                .height(Size::fill())
                .padding(Gaps::new_all(10.))
                .background(theme::color(theme::BG))
                .with_corner_radius(8.)
                .child(
                    label()
                        .color(theme::color(theme::MUTED))
                        .text(if is_folder {
                            entry
                                .children_preview
                                .iter()
                                .map(|child| child.title.as_str())
                                .collect::<Vec<_>>()
                                .join("  ·  ")
                        } else {
                            entry.excerpt.clone()
                        }),
                )
                .into_element()
        } else {
            rect().height(Size::px(0.)).into_element()
        })
        .into_element()
}

pub(super) fn to_library_entry(entry: &VaultEntry) -> LibraryEntry {
    let entry_type = match &entry.entry_type {
        EntryKind::Note => EntryType::Note,
        EntryKind::Folder => EntryType::Folder,
        EntryKind::Drawing => EntryType::Drawing,
        EntryKind::File => EntryType::File,
        EntryKind::Other(_) => EntryType::Custom(entry.entry_type.as_str().to_string()),
    };
    let kind = Some(match &entry.kind {
        EntryKind::Note => ContractKind::Note,
        EntryKind::Folder => ContractKind::Folder,
        EntryKind::Drawing => ContractKind::Drawing,
        EntryKind::File => ContractKind::File,
        EntryKind::Other(value) => ContractKind::Custom(value.clone()),
    });
    LibraryEntry::new(
        entry_type,
        kind,
        EntryTitle::new(entry.title.clone()),
        RelativePath::new(entry.path.clone()),
        entry
            .children_preview
            .iter()
            .map(|child| crate::library_contract::ChildPreview {
                title: EntryTitle::new(child.title.clone()),
                entry_type: match &child.entry_type {
                    EntryKind::Note => EntryType::Note,
                    EntryKind::Folder => EntryType::Folder,
                    EntryKind::Drawing => EntryType::Drawing,
                    EntryKind::File => EntryType::File,
                    EntryKind::Other(value) => EntryType::Custom(value.clone()),
                },
            })
            .collect(),
        entry.excerpt.clone(),
        entry.tags.clone(),
        crate::library_contract::UpdatedAt::new(entry.updated_at.clone()),
    )
}
