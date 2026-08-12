//! Freya conversion of the current Tauri LibraryToolbar, CreateEntryMenu,
//! LibraryGrid and NoteCard surface.

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

use super::{editor_view, route_notice, ShellState};

#[path = "drawing.rs"]
mod drawing;

#[path = "library_icons.rs"]
mod library_icons;

#[path = "library_actions.rs"]
mod library_actions;

use library_actions::{
    card_action_menu, create_note_and_open, rename_library_entry, CardMenuState,
};
use library_icons::{svg_icon, Icon as LibraryIcon};

pub(super) fn main_content(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let showing_library = snapshot.editor.is_none() && snapshot.view == WorkspaceView::Notes;
    let body = if snapshot.editor.is_some() {
        editor_view::note_editor_host(state)
    } else if snapshot.view == WorkspaceView::Notes {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(library_toolbar(state))
            .child(library_grid(state))
            .child(library_create_button(state))
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
        .padding(if showing_library {
            Gaps::new_all(0.)
        } else {
            Gaps::new(8., 12., 12., 12.)
        })
        .maybe_child(snapshot.error.as_deref().map(library_error_notice))
        .child(body)
        .into_element()
}

fn library_error_notice(error: &str) -> Element {
    let accessibility_label = drawing::error_accessibility_label(error);
    rect()
        .position(Position::new_absolute().left(12.).right(12.).top(80.))
        .padding(Gaps::new_all(10.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER_STRONG)).width(1.))
        .with_corner_radius(8.)
        .layer(Layer::OverlayLevel(20))
        .a11y_alt(accessibility_label)
        .child(
            label()
                .color(theme::color(theme::DANGER))
                .text(error.to_owned()),
        )
        .into_element()
}

fn library_toolbar(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let sort_hovered = snapshot.hovered_target.as_deref() == Some("toolbar:sort");
    let mut sort_state = state;
    let mut sort_enter_state = state;
    let mut sort_leave_state = state;
    let sort = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(if sort_hovered {
            theme::SOFT
        } else {
            theme::mix(theme::SURFACE, theme::BG, 0.52)
        }))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(12.)
        .on_mouse_up(move |_| sort_state.write().library.cycle_sort())
        .on_pointer_enter(move |_| sort_enter_state.write().set_hovered_target("toolbar:sort"))
        .on_pointer_leave(move |_| {
            sort_leave_state
                .write()
                .clear_hovered_target("toolbar:sort")
        })
        .a11y_alt(format!("Sort: {}", sort_label(snapshot.library.sort)))
        .child(svg_icon(
            sort_icon(snapshot.library.sort),
            theme::color(theme::TEXT),
            22.,
        ));

    let view_hovered = snapshot.hovered_target.as_deref() == Some("toolbar:view");
    let mut view_state = state;
    let mut view_enter_state = state;
    let mut view_leave_state = state;
    let view_label = if snapshot.library.view_mode == ViewMode::Grid {
        "Show notes as list"
    } else {
        "Show notes as grid"
    };
    let view = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .center()
        .background(theme::color(if view_hovered {
            theme::SOFT
        } else {
            theme::mix(theme::SURFACE, theme::BG, 0.52)
        }))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(12.)
        .on_mouse_up(move |_| view_state.write().library.cycle_view())
        .on_pointer_enter(move |_| view_enter_state.write().set_hovered_target("toolbar:view"))
        .on_pointer_leave(move |_| {
            view_leave_state
                .write()
                .clear_hovered_target("toolbar:view")
        })
        .a11y_alt(view_label)
        .child(svg_icon(
            if snapshot.library.view_mode == ViewMode::Grid {
                LibraryIcon::List
            } else {
                LibraryIcon::Grid3x3
            },
            theme::color(theme::TEXT),
            22.,
        ));

    rect()
        .position(Position::new_absolute().left(0.).right(0.).top(0.))
        .width(Size::fill())
        .height(Size::px(72.))
        .padding(Gaps::new(10., 12., 10., 12.))
        .horizontal()
        .child(rect().width(Size::fill()))
        .child(rect().horizontal().spacing(14.).child(sort).child(view))
        .into_element()
}

fn library_create_button(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let hovered = snapshot.hovered_target.as_deref() == Some("toolbar:create");
    let mut click_state = state;
    let mut enter_state = state;
    let mut leave_state = state;

    rect()
        .position(Position::new_absolute().right(20.).bottom(20.))
        .width(Size::px(56.))
        .height(Size::px(56.))
        .center()
        .background(theme::color(if hovered {
            theme::mix(theme::PRIMARY, (0, 0, 0, 255), 0.88)
        } else {
            theme::PRIMARY
        }))
        .border(
            Border::new()
                .fill(theme::color(theme::mix(
                    theme::PRIMARY,
                    theme::BORDER,
                    0.64,
                )))
                .width(1.),
        )
        .with_corner_radius(11.)
        .layer(Layer::OverlayLevel(10))
        .on_mouse_up(move |_| {
            let open = click_state.read().menu_open;
            click_state.write().menu_open = !open;
        })
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target("toolbar:create"))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target("toolbar:create"))
        .a11y_alt("Create")
        .child(svg_icon(
            LibraryIcon::Plus,
            theme::color((255, 255, 255, 255)),
            27.,
        ))
        .into_element()
}

fn sort_label(sort: SortMode) -> &'static str {
    match sort {
        SortMode::UpdatedNewest => "Updated newest",
        SortMode::UpdatedOldest => "Updated oldest",
        SortMode::TitleAz => "Title A-Z",
        SortMode::TitleZa => "Title Z-A",
    }
}

fn sort_icon(sort: SortMode) -> LibraryIcon {
    match sort {
        SortMode::UpdatedNewest => LibraryIcon::ArrowDownNarrowWide,
        SortMode::UpdatedOldest => LibraryIcon::ArrowUpNarrowWide,
        SortMode::TitleAz => LibraryIcon::ArrowDownAz,
        SortMode::TitleZa => LibraryIcon::ArrowDownZa,
    }
}

pub(super) fn create_entry_menu(state: State<ShellState>) -> Element {
    let item = |action, icon, title, description| {
        let mut action_state = state;
        let hover_key = format!("create-menu:{title}");
        let mut enter_state = state;
        let mut leave_state = state;
        let enter_key = hover_key.clone();
        let leave_key = hover_key.clone();
        let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());

        rect()
            .width(Size::fill())
            .height(Size::px(52.))
            .padding(Gaps::new_all(10.))
            .horizontal()
            .spacing(12.)
            .with_corner_radius(10.)
            .background(theme::color(if hovered {
                theme::SOFT
            } else {
                theme::SURFACE
            }))
            .on_mouse_up(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                action_state.write().menu_open = false;
                match action {
                    crate::library_contract::CreateAction::Note => {
                        create_note_and_open(action_state);
                    }
                    crate::library_contract::CreateAction::Drawing => {
                        drawing::request_create(action_state);
                    }
                    crate::library_contract::CreateAction::Folder => {
                        action_state.write().create(action);
                    }
                }
            })
            .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
            .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
            .a11y_alt(title)
            .child(svg_icon(icon, theme::color(theme::PRIMARY), 20.))
            .child(
                rect()
                    .spacing(2.)
                    .child(
                        label()
                            .font_size(14.)
                            .font_weight(FontWeight::BOLD)
                            .text(title),
                    )
                    .child(
                        label()
                            .font_size(12.)
                            .color(theme::color(theme::MUTED))
                            .text(description),
                    ),
            )
            .into_element()
    };

    let mut close_state = state;
    let mut escape_state = state;
    rect()
        .position(
            Position::new_absolute()
                .left(0.)
                .right(0.)
                .top(0.)
                .bottom(0.),
        )
        .width(Size::fill())
        .height(Size::fill())
        .layer(Layer::OverlayLevel(20))
        .on_mouse_up(move |_| close_state.write().menu_open = false)
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Escape) {
                escape_state.write().menu_open = false;
            }
        })
        .child(
            rect()
                .position(Position::new_absolute().right(20.).bottom(86.))
                .width(Size::px(280.))
                .padding(Gaps::new_all(8.))
                .background(theme::color(theme::SURFACE))
                .border(
                    Border::new()
                        .fill(theme::color(theme::BORDER_STRONG))
                        .width(1.),
                )
                .with_corner_radius(14.)
                .on_mouse_up(|event: Event<MouseEventData>| event.stop_propagation())
                .a11y_alt("Create")
                .child(
                    label()
                        .padding(Gaps::new(8., 10., 6., 10.))
                        .font_size(12.)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(theme::MUTED))
                        .text("CREATE"),
                )
                .child(item(
                    crate::library_contract::CreateAction::Note,
                    LibraryIcon::FilePlus2,
                    "Note",
                    "Create a new note",
                ))
                .child(item(
                    crate::library_contract::CreateAction::Drawing,
                    LibraryIcon::Excalidraw,
                    "Drawing",
                    "Open a new Excalidraw canvas",
                ))
                .child(item(
                    crate::library_contract::CreateAction::Folder,
                    LibraryIcon::FolderPlus,
                    "Folder",
                    "Organize notes in a folder",
                )),
        )
        .into_element()
}

fn library_grid(state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let visible = snapshot.library.visible_entries();
    if visible.is_empty() {
        return rect()
            .width(Size::fill())
            .height(Size::fill())
            .a11y_alt("Empty library")
            .into_element();
    }

    let count = visible.len();
    let entries = visible
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            LibraryCard {
                entry: entry.clone(),
                mode: snapshot.library.view_mode,
                featured: snapshot.library.view_mode == ViewMode::Grid && index == 0 && count > 3,
                state,
            }
            .into_element()
        })
        .collect::<Vec<_>>();

    let surface = if snapshot.library.view_mode == ViewMode::Grid {
        rect()
            .horizontal()
            .content(Content::wrap_spacing(10.))
            .spacing(10.)
            .children(entries)
    } else {
        rect().spacing(6.).children(entries)
    };

    ScrollView::new()
        .width(Size::fill())
        .height(Size::fill())
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(72., 10., 10., 10.))
                .child(surface),
        )
        .into_element()
}

#[derive(PartialEq)]
struct LibraryCard {
    entry: LibraryEntry,
    mode: ViewMode,
    featured: bool,
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
            self.featured,
            self.state,
            card_menu_state,
            rename_value,
        )
    }
}

fn render_library_card(
    entry: &LibraryEntry,
    mode: ViewMode,
    featured: bool,
    state: State<ShellState>,
    mut card_menu_state: State<CardMenuState>,
    rename_value: State<String>,
) -> Element {
    let path = entry.path.as_str().to_string();
    let is_drawing =
        is_drawing_path(&path) || matches!(entry.effective_kind(), ContractKind::Drawing);
    let is_folder = matches!(entry.effective_kind(), ContractKind::Folder);
    let title = display_title(entry.title.as_str(), is_drawing);
    let menu_snapshot = card_menu_state.read().clone();
    let renaming = menu_snapshot.renaming;
    let height = if mode == ViewMode::List {
        theme::LIST_CARD_HEIGHT
    } else if featured && !is_folder {
        220.
    } else {
        theme::CARD_HEIGHT
    };

    let hover_key = format!("card:{path}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;

    let mut trigger_state = card_menu_state;
    let menu_trigger = rect()
        .position(Position::new_absolute().top(8.).right(8.))
        .width(Size::px(30.))
        .height(Size::px(30.))
        .center()
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
        .child(svg_icon(
            LibraryIcon::MoreHorizontal,
            theme::color(theme::MUTED),
            20.,
        ));

    let card_menu = if menu_snapshot.open && !renaming {
        Some(card_action_menu(
            path.clone(),
            title.clone(),
            is_folder,
            state,
            card_menu_state,
            rename_value,
        ))
    } else {
        None
    };

    let title_row = card_title_row(
        &title,
        &path,
        mode,
        is_folder,
        is_drawing,
        state,
        card_menu_state,
        rename_value,
        renaming,
    );

    let body = if mode == ViewMode::List {
        rect().height(Size::px(0.)).into_element()
    } else if is_folder {
        folder_preview(entry)
    } else if is_drawing {
        drawing_card_body(&title, featured)
    } else {
        note_card_body(entry)
    };

    let mut menu_state_for_secondary = card_menu_state;
    let mut state_for_open = state;
    let path_for_open = path.clone();
    let mut click_menu_state = card_menu_state;
    let click_rename_value = rename_value;

    rect()
        .width(if mode == ViewMode::Grid {
            Size::px(240.)
        } else {
            Size::fill()
        })
        .height(Size::px(height))
        .padding(if mode == ViewMode::Grid {
            Gaps::new_all(10.)
        } else {
            Gaps::new(8., 10., 8., 10.)
        })
        .background(theme::color(theme::mix(
            theme::SURFACE,
            theme::BG,
            0.34,
        )))
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
            if renaming {
                let mut menu = click_menu_state.write();
                menu.open = false;
                menu.renaming = false;
                drop(menu);
                let mut value = click_rename_value;
                value.set(String::new());
                return;
            }

            let mut menu = click_menu_state.write();
            menu.open = false;
            menu.renaming = false;
            drop(menu);

            if is_drawing {
                drawing::open_existing(state_for_open, &path_for_open);
            } else if is_folder {
                state_for_open.write().open_directory(path_for_open.clone());
            } else {
                let vault_entry = {
                    let snapshot = state_for_open.read();
                    snapshot
                        .page
                        .as_ref()
                        .and_then(|page| {
                            page.entries
                                .iter()
                                .find(|entry| entry.path == path_for_open)
                        })
                        .cloned()
                };
                if let Some(vault_entry) = vault_entry {
                    state_for_open.write().open_note(&vault_entry);
                } else {
                    eprintln!(
                        "[freya][library] action:failure action=open path={} reason=missing_page_entry",
                        path_for_open
                    );
                    state_for_open.write().error = Some(format!(
                        "Library entry is no longer present in the current directory: {}",
                        path_for_open
                    ));
                }
            }
        })
        .a11y_alt(title)
        .child(menu_trigger)
        .maybe_child(card_menu)
        .child(title_row)
        .child(body)
        .into_element()
}

#[allow(clippy::too_many_arguments)]
fn card_title_row(
    title: &str,
    path: &str,
    mode: ViewMode,
    is_folder: bool,
    is_drawing: bool,
    state: State<ShellState>,
    card_menu_state: State<CardMenuState>,
    rename_value: State<String>,
    renaming: bool,
) -> Element {
    let icon = if is_folder {
        LibraryIcon::Folder
    } else if is_drawing {
        LibraryIcon::PenLine
    } else {
        LibraryIcon::FileText
    };
    let icon_size = if mode == ViewMode::Grid { 22. } else { 20. };
    let title_size = if mode == ViewMode::Grid { 20. } else { 17. };

    let copy = if renaming {
        let mut rename_state = state;
        let path_for_submit = path.to_string();
        let previous_title = title.to_string();
        let mut submit_menu_state = card_menu_state;
        let mut submit_value_state = rename_value;
        let mut escape_menu_state = card_menu_state;
        let mut escape_value_state = rename_value;

        rect()
            .width(Size::fill())
            .a11y_alt(format!("Rename {title}"))
            .on_mouse_up(|event: Event<MouseEventData>| event.stop_propagation())
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Escape) {
                    let mut menu = escape_menu_state.write();
                    menu.open = false;
                    menu.renaming = false;
                    drop(menu);
                    escape_value_state.set(String::new());
                }
            })
            .child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new(4., 6., 4., 6.))
                    .background(theme::color(theme::SURFACE))
                    .border(
                        Border::new()
                            .fill(theme::color(theme::PRIMARY))
                            .width(1.),
                    )
                    .with_corner_radius(7.)
                    .child(
                        Input::new(rename_value)
                            .width(Size::fill())
                            .auto_focus(true)
                            .on_submit(move |next_title: String| {
                                let next_title = next_title.trim().to_string();
                                if !next_title.is_empty() && next_title != previous_title {
                                    let _ = rename_library_entry(
                                        &mut rename_state,
                                        &path_for_submit,
                                        &next_title,
                                    );
                                }
                                let mut menu = submit_menu_state.write();
                                menu.open = false;
                                menu.renaming = false;
                                drop(menu);
                                submit_value_state.set(String::new());
                            }),
                    ),
            )
            .into_element()
    } else {
        label()
            .font_size(title_size)
            .font_weight(FontWeight::BOLD)
            .text(title.to_string())
            .into_element()
    };

    rect()
        .width(Size::fill())
        .horizontal()
        .spacing(8.)
        .child(svg_icon(icon, theme::color(theme::TEXT), icon_size))
        .child(copy)
        .into_element()
}

fn folder_preview(entry: &LibraryEntry) -> Element {
    let rows = entry
        .children_preview
        .iter()
        .take(3)
        .map(|child| {
            let icon = match &child.entry_type {
                EntryType::Folder => LibraryIcon::Folder,
                EntryType::Drawing => LibraryIcon::PenLine,
                _ => LibraryIcon::FileText,
            };
            rect()
                .width(Size::fill())
                .horizontal()
                .spacing(6.)
                .child(svg_icon(icon, theme::color(theme::MUTED), 15.))
                .child(
                    label()
                        .font_size(13.)
                        .color(theme::color(theme::MUTED))
                        .text(preview_title(child.title.as_str())),
                )
                .into_element()
        })
        .collect::<Vec<_>>();

    let preview = if rows.is_empty() {
        rect()
            .width(Size::fill())
            .height(Size::px(42.))
            .padding(Gaps::new(7., 8., 7., 8.))
            .center()
            .background(theme::color(theme::mix(
                theme::SURFACE,
                theme::BG,
                0.55,
            )))
            .border(
                Border::new()
                    .fill(theme::color(theme::mix(
                        theme::BORDER,
                        theme::BG,
                        0.70,
                    )))
                    .width(1.),
            )
            .with_corner_radius(8.)
            .child(
                label()
                    .font_size(13.)
                    .color(theme::color(theme::MUTED))
                    .text("No items yet"),
            )
            .into_element()
    } else {
        rect()
            .width(Size::fill())
            .padding(Gaps::new(7., 8., 7., 8.))
            .spacing(4.)
            .background(theme::color(theme::mix(
                theme::SURFACE,
                theme::BG,
                0.55,
            )))
            .border(
                Border::new()
                    .fill(theme::color(theme::mix(
                        theme::BORDER,
                        theme::BG,
                        0.70,
                    )))
                    .width(1.),
            )
            .with_corner_radius(8.)
            .children(rows)
            .into_element()
    };

    rect()
        .height(Size::fill())
        .padding(Gaps::new(10., 0., 0., 0.))
        .child(preview)
        .into_element()
}

fn drawing_card_body(title: &str, featured: bool) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(if featured { 156. } else { 112. }))
        .padding(Gaps::new(10., 0., 8., 0.))
        .center()
        .background(theme::color((255, 255, 255, 255)))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .a11y_alt(format!("{title} drawing"))
        .child(svg_icon(
            LibraryIcon::Excalidraw,
            theme::color(theme::PRIMARY),
            42.,
        ))
        .into_element()
}

fn note_card_body(entry: &LibraryEntry) -> Element {
    let tags = entry
        .tags
        .iter()
        .map(|tag| {
            label()
                .font_size(12.)
                .color(theme::color(theme::MUTED))
                .text(format!("#{tag}"))
                .into_element()
        })
        .collect::<Vec<_>>();

    rect()
        .height(Size::fill())
        .padding(Gaps::new(10., 0., 0., 0.))
        .spacing(8.)
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(entry.excerpt.clone()),
        )
        .maybe_child((!tags.is_empty()).then(|| {
            rect()
                .horizontal()
                .content(Content::wrap_spacing(6.))
                .spacing(6.)
                .children(tags)
                .into_element()
        }))
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

fn preview_title(title: &str) -> String {
    title
        .strip_suffix(".md")
        .or_else(|| title.strip_suffix(".excalidraw"))
        .or_else(|| title.strip_suffix(".excalidraw.png"))
        .unwrap_or(title)
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_titles_match_the_vue_card_extension_cleanup() {
        assert_eq!(preview_title("Note.md"), "Note");
        assert_eq!(preview_title("Sketch.excalidraw"), "Sketch");
        assert_eq!(preview_title("Sketch.excalidraw.png"), "Sketch");
        assert_eq!(preview_title("Folder"), "Folder");
    }

    #[test]
    fn toolbar_sort_labels_follow_the_tauri_cycle() {
        assert_eq!(sort_label(SortMode::UpdatedNewest), "Updated newest");
        assert_eq!(sort_label(SortMode::UpdatedOldest), "Updated oldest");
        assert_eq!(sort_label(SortMode::TitleAz), "Title A-Z");
        assert_eq!(sort_label(SortMode::TitleZa), "Title Z-A");
    }
}
