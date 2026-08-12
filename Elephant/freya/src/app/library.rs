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

#[path = "drawing.rs"]
mod drawing;

#[path = "library_actions.rs"]
mod library_actions;
use library_actions::{card_action_menu, CardMenuState};

pub(super) fn main_content(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    if let Some(panel) = drawing::active_panel(state) {
        return panel;
    }
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
                .map(|error| library_error_notice(&error)),
        )
        .child(body)
        .into_element()
}

fn library_error_notice(error: &str) -> Element {
    let accessibility_label = drawing::error_accessibility_label(error);
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(10.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .a11y_alt(accessibility_label)
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(error.to_owned()),
        )
        .into_element()
}

fn library_toolbar(mut state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let create_hovered = snapshot.hovered_target.as_deref() == Some("toolbar:create");
    let mut create_enter_state = state;
    let mut create_leave_state = state;
    let create = rect()
        .width(Size::px(56.))
        .height(Size::px(56.))
        .center()
        .background(theme::color(if create_hovered {
            theme::TEXT
        } else {
            theme::PRIMARY
        }))
        .with_corner_radius(11.)
        .on_mouse_up(move |_| state.write().menu_open = true)
        .on_pointer_enter(move |_| {
            create_enter_state
                .write()
                .set_hovered_target("toolbar:create")
        })
        .on_pointer_leave(move |_| {
            create_leave_state
                .write()
                .clear_hovered_target("toolbar:create")
        })
        .a11y_alt("Create")
        .child(
            label()
                .font_size(28.)
                .color(theme::color(theme::TEXT))
                .text("+"),
        );
    let sort_hovered = snapshot.hovered_target.as_deref() == Some("toolbar:sort");
    let mut sort_enter_state = state;
    let mut sort_leave_state = state;
    let sort = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(if sort_hovered {
            theme::SOFT
        } else {
            theme::SURFACE
        }))
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().library.cycle_sort())
        .on_pointer_enter(move |_| sort_enter_state.write().set_hovered_target("toolbar:sort"))
        .on_pointer_leave(move |_| {
            sort_leave_state
                .write()
                .clear_hovered_target("toolbar:sort")
        })
        .a11y_alt(format!("Sort: {}", snapshot.library.sort.as_contract()))
        .child(label().font_size(19.).text("↕"));
    let view_hovered = snapshot.hovered_target.as_deref() == Some("toolbar:view");
    let mut view_enter_state = state;
    let mut view_leave_state = state;
    let view = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(if view_hovered {
            theme::SOFT
        } else {
            theme::SURFACE
        }))
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().library.cycle_view())
        .on_pointer_enter(move |_| view_enter_state.write().set_hovered_target("toolbar:view"))
        .on_pointer_leave(move |_| {
            view_leave_state
                .write()
                .clear_hovered_target("toolbar:view")
        })
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
        let hover_key = format!("create-menu:{title}");
        let mut enter_state = state;
        let mut leave_state = state;
        let enter_key = hover_key.clone();
        let leave_key = hover_key.clone();
        let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
        rect()
            .width(Size::fill())
            .height(Size::px(64.))
            .padding(Gaps::new_all(10.))
            .horizontal()
            .spacing(12.)
            .with_corner_radius(10.)
            .background(theme::color(if hovered {
                theme::SOFT
            } else {
                theme::SURFACE
            }))
            .on_mouse_up(move |_| {
                state.write().menu_open = false;
                if action == crate::library_contract::CreateAction::Drawing {
                    drawing::request_create(state);
                } else {
                    state.write().create(action);
                }
            })
            .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
            .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
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
    let mut close_state = state;
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
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Escape) {
                close_state.write().menu_open = false;
            }
        })
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
        .map(|entry| {
            LibraryCard {
                entry: entry.clone(),
                mode: snapshot.library.view_mode,
                state,
            }
            .into_element()
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new(72., 12., 12., 12.))
            .a11y_alt("No visible notes")
            .child(label().font_size(18.).text("No visible notes"))
            .into_element();
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

#[derive(PartialEq)]
struct LibraryCard {
    entry: LibraryEntry,
    mode: ViewMode,
    state: State<ShellState>,
}

impl Component for LibraryCard {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.entry.path)
    }

    fn render(&self) -> impl IntoElement {
        let card_menu_state = use_state(CardMenuState::default);
        let rename_value = use_state(String::new);
        render_library_card(
            &self.entry,
            self.mode,
            self.state,
            card_menu_state,
            rename_value,
        )
    }
}

fn render_library_card(
    entry: &LibraryEntry,
    mode: ViewMode,
    state: State<ShellState>,
    mut card_menu_state: State<CardMenuState>,
    rename_value: State<String>,
) -> Element {
    let path = entry.path.as_str().to_string();
    let is_drawing =
        is_drawing_path(&path) || matches!(entry.effective_kind(), ContractKind::Drawing);
    let title = display_title(entry.title.as_str(), is_drawing);
    let is_folder = matches!(entry.effective_kind(), ContractKind::Folder);
    let height = if mode == ViewMode::Grid {
        theme::CARD_HEIGHT
    } else {
        theme::LIST_CARD_HEIGHT
    };
    let hover_key = format!("card:{path}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    let mut state_for_open = state;
    let path_for_open = path.clone();
    let menu_snapshot = card_menu_state.read().clone();
    let card_menu = if menu_snapshot.open {
        Some(card_action_menu(
            path.clone(),
            title.clone(),
            is_folder,
            state,
            card_menu_state,
            rename_value,
            menu_snapshot.renaming,
        ))
    } else {
        None
    };
    let menu_trigger = if hovered || menu_snapshot.open {
        let mut trigger_state = card_menu_state;
        Some(
            rect()
                .position(Position::new_absolute().top(8.).right(8.))
                .width(Size::px(30.))
                .height(Size::px(30.))
                .center()
                .background(theme::color(theme::BG))
                .with_corner_radius(6.)
                .a11y_alt(if is_folder {
                    "Folder actions"
                } else {
                    "Note actions"
                })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    let mut menu = trigger_state.write();
                    menu.open = true;
                    menu.renaming = false;
                })
                .child(label().text("⋯")),
        )
    } else {
        None
    };
    let mut menu_state_for_secondary = card_menu_state;
    rect()
        .width(if mode == ViewMode::Grid {
            Size::px(240.)
        } else {
            Size::fill()
        })
        .height(Size::px(height))
        .padding(Gaps::new_all(10.))
        .background(theme::color(if hovered {
            theme::SOFT
        } else {
            theme::SURFACE
        }))
        .border(
            Border::new()
                .fill(theme::color(if hovered {
                    theme::BORDER_STRONG
                } else {
                    theme::BORDER
                }))
                .width(1.),
        )
        .with_corner_radius(10.)
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
        .on_secondary_down(move |_| {
            let mut menu = menu_state_for_secondary.write();
            menu.open = true;
            menu.renaming = false;
        })
        .on_mouse_up(move |event: Event<MouseEventData>| {
            if event.button == Some(MouseButton::Right) {
                return;
            }
            let mut menu = card_menu_state.write();
            menu.open = false;
            menu.renaming = false;
            drop(menu);
            if is_drawing {
                drawing::open_existing(state_for_open, &path_for_open);
            } else if is_folder {
                state_for_open.write().open_directory(path_for_open.clone());
            } else {
                let root = {
                    let snapshot = state_for_open.read();
                    snapshot
                        .vault
                        .as_ref()
                        .map(|vault| vault.root().to_path_buf())
                };
                if let Some(root) = root {
                    let full = root.join(&path_for_open);
                    match EditorDocument::load(full) {
                        Ok(document) => state_for_open.write().editor = Some(document),
                        Err(error) => state_for_open.write().error = Some(error.to_string()),
                    }
                }
            }
        })
        .a11y_alt(title.clone())
        .maybe_child(menu_trigger)
        .maybe_child(card_menu)
        .child(
            label()
                .font_size(14.)
                .font_weight(FontWeight::BOLD)
                .text(format!(
                    "{}  {title}",
                    if is_folder {
                        "▱"
                    } else if is_drawing {
                        "✎"
                    } else {
                        "▤"
                    }
                )),
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
                        } else if is_drawing {
                            "Excalidraw preview unavailable in native Freya".to_string()
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
    let drawing_path = is_drawing_path(&entry.path);
    let entry_type = if drawing_path {
        EntryType::Drawing
    } else {
        match &entry.entry_type {
            EntryKind::Note => EntryType::Note,
            EntryKind::Folder => EntryType::Folder,
            EntryKind::Drawing => EntryType::Drawing,
            EntryKind::File => EntryType::File,
            EntryKind::Other(_) => EntryType::Custom(entry.entry_type.as_str().to_string()),
        }
    };
    let kind = Some(if drawing_path {
        ContractKind::Drawing
    } else {
        match &entry.kind {
            EntryKind::Note => ContractKind::Note,
            EntryKind::Folder => ContractKind::Folder,
            EntryKind::Drawing => ContractKind::Drawing,
            EntryKind::File => ContractKind::File,
            EntryKind::Other(value) => ContractKind::Custom(value.clone()),
        }
    });
    LibraryEntry::new(
        entry_type,
        kind,
        EntryTitle::new(display_title(&entry.title, drawing_path)),
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

fn is_drawing_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".excalidraw") || lower.ends_with(".excalidraw.png")
}

fn display_title(title: &str, drawing: bool) -> String {
    if !drawing {
        return title.to_owned();
    }
    title
        .strip_suffix(".excalidraw")
        .or_else(|| title.strip_suffix(".excalidraw.png"))
        .unwrap_or(title)
        .to_owned()
}
