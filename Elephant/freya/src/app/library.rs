//! Freya conversion of LibraryToolbar, CreateEntryMenu, LibraryGrid and NoteCard.

use freya::prelude::*;

use crate::{
    library_contract::{
        EntryKind as ContractKind, EntryTitle, EntryType, LibraryEntry, RelativePath, SortMode,
        ViewMode,
    },
    navigation_contract::WorkspaceView,
    theme,
    vault_adapter::{EntryKind, VaultEntry},
};

use super::{
    editor_view,
    navigation_icons::{svg_icon, Icon},
    route_notice,
    ShellState,
};

#[path = "drawing.rs"]
mod drawing;

#[path = "library_actions.rs"]
mod library_actions;
use library_actions::{card_action_menu, CardMenuState};

const GRID_CARD_WIDTH: f32 = 317.;

pub(super) fn main_content(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let body = if snapshot.editor.is_some() {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new(0., 0., 12., 10.))
            .child(editor_view::note_editor_host(state))
            .into_element()
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
    let mut dismiss_menu_state = state;
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .on_mouse_up(move |_| dismiss_menu_state.write().menu_open = false)
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
            theme::toolbar_button_background()
        }))
        .with_corner_radius(12.)
        .border(
            Border::new()
                .fill(theme::color(theme::BORDER))
                .width(1.),
        )
        .shadow(Shadow::new().y(8.).blur(22.).color(Color::from_argb(56, 0, 0, 0)))
        .on_mouse_up(move |_| state.write().library.cycle_sort())
        .on_pointer_enter(move |_| sort_enter_state.write().set_hovered_target("toolbar:sort"))
        .on_pointer_leave(move |_| {
            sort_leave_state
                .write()
                .clear_hovered_target("toolbar:sort")
        })
        .a11y_alt(format!("Sort: {}", snapshot.library.sort.as_contract()))
        .child(svg_icon(
            match snapshot.library.sort {
                SortMode::UpdatedNewest => Icon::ArrowDownNarrowWide,
                SortMode::UpdatedOldest => Icon::ArrowUpNarrowWide,
                SortMode::TitleAz => Icon::ArrowDownAz,
                SortMode::TitleZa => Icon::ArrowDownZa,
            },
            theme::color(theme::TEXT),
            22.,
        ));
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
            theme::toolbar_button_background()
        }))
        .with_corner_radius(12.)
        .border(
            Border::new()
                .fill(theme::color(theme::BORDER))
                .width(1.),
        )
        .shadow(Shadow::new().y(8.).blur(22.).color(Color::from_argb(56, 0, 0, 0)))
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
        .child(svg_icon(
            if snapshot.library.view_mode == ViewMode::Grid {
                Icon::List
            } else {
                Icon::Grid3x3
            },
            theme::color(theme::TEXT),
            22.,
        ));
    rect()
        .position(Position::new_absolute().left(0.).top(0.))
        .width(Size::fill())
        .height(Size::px(72.))
        .padding(Gaps::new(10., 12., 10., 12.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(rect().width(Size::fill()))
        .child(rect().horizontal().spacing(14.).child(sort).child(view))
        .into_element()
}

pub(super) fn create_fab(mut state: State<ShellState>) -> Element {
    let hovered = state.read().hovered_target.as_deref() == Some("create-fab");
    let mut enter_state = state;
    let mut leave_state = state;
    rect()
        .position(Position::new_absolute().right(20.).bottom(20.))
        .width(Size::px(56.))
        .height(Size::px(56.))
        .center()
        .background(theme::color(if hovered {
            theme::BORDER_STRONG
        } else {
            theme::PRIMARY
        }))
        .with_corner_radius(11.)
        .layer(Layer::OverlayLevel(10))
        .on_mouse_up(move |_| {
            let mut shell = state.write();
            shell.menu_open = !shell.menu_open;
        })
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target("create-fab"))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target("create-fab"))
        .a11y_alt("Create")
        .child(svg_icon(
            Icon::Plus,
            theme::color(theme::SURFACE),
            27.,
        ))
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
                } else if action == crate::library_contract::CreateAction::Note {
                    create_note_from_library(state);
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
                .right(20.)
                .bottom(88.),
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
    let visible_entries = snapshot.library.visible_entries();
    let fallback_menu_path = visible_entries
        .iter()
        .find(|entry| !matches!(entry.effective_kind(), ContractKind::Folder))
        .map(|entry| entry.path.clone());
    let entries = visible_entries
        .into_iter()
        .map(|entry| {
            LibraryCard {
                entry: entry.clone(),
                mode: snapshot.library.view_mode,
                fallback_menu: fallback_menu_path.as_ref() == Some(&entry.path),
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
            .padding(Gaps::new(0., 10., 10., 10.))
            .content(Content::wrap_spacing(10.))
            .spacing(10.)
            .children(entries)
    } else {
        rect()
            .padding(Gaps::new(0., 10., 10., 10.))
            .spacing(8.)
            .children(entries)
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
    fallback_menu: bool,
    state: State<ShellState>,
}

impl Component for LibraryCard {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.entry.path)
    }

    fn render(&self) -> impl IntoElement {
        let card_menu_state = use_state(CardMenuState::default);
        let rename_value = use_state(String::new);
        let hover_state = use_state(|| false);
        render_library_card(
            &self.entry,
            self.mode,
            self.fallback_menu,
            self.state,
            card_menu_state,
            rename_value,
            hover_state,
        )
    }
}

fn render_library_card(
    entry: &LibraryEntry,
    mode: ViewMode,
    fallback_menu: bool,
    state: State<ShellState>,
    mut card_menu_state: State<CardMenuState>,
    rename_value: State<String>,
    hover_state: State<bool>,
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
    let hovered = *hover_state.read()
        || state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let card_action_active = state.read().card_action_target.as_deref() == Some(hover_key.as_str());
    let card_selected = hovered || card_action_active;
    let show_accessible_menu = card_selected
        || (fallback_menu && state.read().card_action_target.is_none());
    let is_pinned = state
        .read()
        .library
        .pinned_paths
        .iter()
        .any(|pinned| pinned == &entry.path);
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    let mut enter_hover_state = hover_state;
    let mut leave_hover_state = hover_state;
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
    let menu_trigger = {
        let trigger = rect()
            .position(Position::new_absolute().top(8.).right(-2.))
            .width(Size::px(30.))
            .height(Size::px(30.))
            .center()
            .background(Color::TRANSPARENT)
            .with_corner_radius(6.);
        let icon = svg_icon(Icon::MoreHorizontal, theme::color(theme::MUTED), 18.);
        if show_accessible_menu {
            let mut trigger_state = card_menu_state;
            Some(
                trigger
                    .a11y_alt(if is_folder {
                        "Folder actions"
                    } else {
                        "Note actions"
                    })
                    .on_mouse_up(move |event: Event<MouseEventData>| {
                        event.stop_propagation();
                        let mut menu = trigger_state.write();
                        menu.open = !menu.open;
                        menu.renaming = false;
                    })
                    .child(icon),
            )
        } else {
            Some(trigger.child(icon))
        }
    };
    let pin_trigger = if !is_folder && (card_selected || is_pinned) {
        let mut pin_state = state;
        let mut pin_menu_state = card_menu_state;
        let path_for_pin = entry.path.clone();
        Some(
            rect()
                .position(Position::new_absolute().top(8.).right(34.))
                .width(Size::px(30.))
                .height(Size::px(30.))
                .center()
                .background(Color::TRANSPARENT)
                .with_corner_radius(6.)
                .a11y_alt(if is_pinned { "Unpin entry" } else { "Pin entry" })
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    pin_state.write().toggle_pinned(path_for_pin.clone());
                    let mut menu = pin_menu_state.write();
                    menu.open = false;
                    menu.renaming = false;
                })
                .child(svg_icon(Icon::Pin, theme::color(theme::MUTED), 18.)),
        )
    } else {
        None
    };
    let title_icon = if is_folder {
        Icon::Folder
    } else {
        Icon::FileText
    };
    let preview = if is_folder {
        let children = entry
            .children_preview
            .iter()
            .map(|child| {
                rect()
                    .width(Size::fill())
                    .height(Size::px(40.))
                    .padding(Gaps::new(7., 8., 7., 8.))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(6.)
                    .background(theme::color(theme::card_preview_background()))
                    .border(
                        Border::new()
                            .fill(theme::color(theme::mix(
                                theme::BORDER,
                                theme::SURFACE,
                                0.70,
                            )))
                            .width(1.),
                    )
                    .with_corner_radius(8.)
                    .child(svg_icon(
                        Icon::FileText,
                        theme::color(theme::MUTED),
                        15.,
                    ))
                    .child(
                        label()
                            .font_size(13.)
                            .color(theme::color(theme::MUTED))
                            .text(child.title.as_str().to_owned()),
                    )
                    .into_element()
            })
            .collect::<Vec<_>>();
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .main_align(Alignment::End)
            .padding(Gaps::new(0., 0., 2., 0.))
            .spacing(4.)
            .children(children)
    } else if is_drawing {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new(6., 0., 0., 0.))
            .child(
                label()
                    .font_size(16.)
                    .color(theme::color(theme::MUTED))
                    .text("Excalidraw preview unavailable in native Freya"),
            )
    } else {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new(6., 0., 0., 0.))
            .child(
                label()
                    .font_size(16.)
                    .color(theme::color(theme::TEXT))
                    .text(entry.excerpt.clone()),
            )
    };
    let mut menu_state_for_secondary = card_menu_state;
    rect()
        .width(if mode == ViewMode::Grid {
            Size::px(GRID_CARD_WIDTH)
        } else {
            Size::fill()
        })
        .min_width(if mode == ViewMode::Grid {
            Size::px(240.)
        } else {
            Size::px(0.)
        })
        .height(Size::px(height))
        .padding(Gaps::new_all(10.))
        .background(theme::color(theme::card_background()))
        .border(
            Border::new()
                .fill(theme::color(if card_selected {
                    theme::BORDER_STRONG
                } else {
                    theme::BORDER
                }))
                .width(1.),
        )
        .with_corner_radius(10.)
        .on_pointer_enter(move |_| {
            *enter_hover_state.write() = true;
            enter_state.write().set_hovered_target(enter_key.clone());
            enter_state.write().set_card_action_target(enter_key.clone());
        })
        .on_pointer_leave(move |_| {
            *leave_hover_state.write() = false;
            // Keep the last card active while an overlay takes focus. The
            // source library keeps its card actions mounted across search
            // open/close; a later card enter replaces this target naturally.
            leave_state.write().clear_hovered_target(&leave_key);
        })
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
                open_note_from_library(state_for_open, &path_for_open);
            }
        })
        .a11y_alt(title.clone())
        .maybe_child(pin_trigger)
        .maybe_child(menu_trigger)
        .maybe_child(card_menu)
        .child(
            rect()
                .height(Size::px(30.))
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .child(svg_icon(
                    title_icon,
                    theme::color(theme::TEXT),
                    22.,
                ))
                .child(
                    label()
                        .font_size(20.)
                        .font_weight(FontWeight::BOLD)
                        .text(title.clone()),
                ),
        )
        .child(if mode == ViewMode::Grid {
            preview.into_element()
        } else {
            rect().height(Size::px(0.)).into_element()
        })
        .into_element()
}

fn create_note_from_library(mut state: State<ShellState>) {
    let (vault, directory) = {
        let snapshot = state.read();
        (
            snapshot.vault.clone(),
            snapshot.library.current_path.as_str().to_string(),
        )
    };
    let Some(vault) = vault else {
        state.write().error = Some("No vault selected.".to_string());
        return;
    };

    eprintln!(
        "[freya][library] action:start action=Note directory={directory}"
    );
    match vault.create_note(Some(directory.clone()), None, None) {
        Ok(entry) => {
            let mut next = state.write();
            next.reload_directory(&directory);
            next.open_note(&entry);
            eprintln!(
                "[freya][library] action:complete action=Note path={}",
                entry.path
            );
        }
        Err(error) => {
            eprintln!(
                "[freya][library] action:failure action=Note directory={directory} error={error}"
            );
            state.write().error = Some(error.to_string());
        }
    }
}

fn open_note_from_library(mut state: State<ShellState>, path: &str) {
    let vault = state.read().vault.clone();
    let Some(vault) = vault else {
        state.write().error = Some("No vault selected.".to_string());
        return;
    };
    match vault.find_entry(path) {
        Ok(entry) => state.write().open_note(&entry),
        Err(error) => state.write().error = Some(error.to_string()),
    }
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
