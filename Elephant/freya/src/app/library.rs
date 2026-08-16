//! Freya conversion of the current Tauri LibraryToolbar, CreateEntryMenu,
//! LibraryGrid and NoteCard surface.

use freya::prelude::*;
use std::time::Duration;

use crate::{
    editor::Delay,
    library_contract::{
        EntryKind as ContractKind, EntryOpenTarget, EntryTitle, EntryType, LibraryEntry,
        RelativePath, ScrollAction, SortMode, ViewMode,
    },
    navigation_contract::WorkspaceView,
    theme,
    vault_adapter::{EntryKind, VaultEntry},
};

use super::{
    calendar_view, chat_view, drawing, editor_view, models_view, route_notice, sync_view,
    wiki_view, ShellState,
};

#[path = "library_icons.rs"]
mod library_icons;

#[path = "library_actions.rs"]
mod library_actions;

use library_actions::{
    card_action_menu, create_note_and_open, load_more_library_entries, move_library_entry,
    rename_library_entry, CardMenuState,
};
use library_icons::{svg_icon, Icon as LibraryIcon};

const LIBRARY_DRAG_THRESHOLD: f64 = 4.;
const CARD_OPEN_DELAY: Duration = Duration::from_millis(220);
const GRID_GAP: f32 = 10.;
const GRID_MIN_CARD_WIDTH: f32 = 240.;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct LibraryCardDrag {
    source: Option<String>,
    source_is_directory: bool,
    start_x: f64,
    start_y: f64,
    moved: bool,
    target: Option<String>,
    target_allowed: bool,
}

impl LibraryCardDrag {
    pub(super) fn begin(&mut self, source: &str, source_is_directory: bool, x: f64, y: f64) {
        self.source = Some(source.to_owned());
        self.source_is_directory = source_is_directory;
        self.start_x = x;
        self.start_y = y;
        self.moved = false;
        self.target = None;
        self.target_allowed = false;
    }

    pub(super) fn update(&mut self, x: f64, y: f64) {
        if self.source.is_some()
            && ((x - self.start_x).abs() >= LIBRARY_DRAG_THRESHOLD
                || (y - self.start_y).abs() >= LIBRARY_DRAG_THRESHOLD)
        {
            self.moved = true;
        }
    }

    pub(super) fn enter_target(&mut self, target: &str, target_is_directory: bool) {
        let Some(source) = self.source.as_deref() else {
            return;
        };
        let allowed = target_is_directory
            && source != target
            && !target.starts_with(&format!("{source}/"))
            && parent_path(source) != target;
        self.target = Some(target.to_owned());
        self.target_allowed = allowed;
    }

    pub(super) fn leave_target(&mut self, target: &str) {
        if self.target.as_deref() == Some(target) {
            self.target = None;
            self.target_allowed = false;
        }
    }

    pub(super) fn target_state(&self, target: &str) -> Option<bool> {
        (self.source.is_some() && self.target.as_deref() == Some(target))
            .then_some(self.target_allowed)
    }

    pub(super) fn is_dragging(&self, path: &str) -> bool {
        self.moved && self.source.as_deref() == Some(path)
    }

    pub(super) fn has_moved(&self) -> bool {
        self.moved
    }

    pub(super) fn finish(&mut self) -> Option<(String, String)> {
        let result = if self.moved && self.target_allowed {
            self.source.clone().zip(self.target.clone())
        } else {
            None
        };
        *self = Self::default();
        result
    }
}

fn parent_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

pub(super) fn main_content(
    state: State<ShellState>,
    wiki_view_state: State<wiki_view::WikiViewState>,
    palette: theme::ThemePalette,
) -> Element {
    let snapshot = state.read().clone();
    let showing_library = snapshot.editor.is_none()
        && snapshot.drawing.is_none()
        && snapshot.view == WorkspaceView::Notes;
    let body = if snapshot.drawing.is_some() {
        drawing::drawing_view(state)
    } else if snapshot.editor.is_some() {
        editor_view::note_editor_host(state)
    } else if snapshot.view == WorkspaceView::Notes {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(library_toolbar(state))
            .child(LibraryGrid { state })
            .child(library_create_button(state))
            .into_element()
    } else if snapshot.view == WorkspaceView::Wiki {
        wiki_view::wiki_workspace(state, wiki_view_state, palette)
    } else if snapshot.view == WorkspaceView::Calendar {
        calendar_view::workspace(state, palette)
    } else if snapshot.view == WorkspaceView::Chat {
        chat_view::workspace(state, palette)
    } else if snapshot.view == WorkspaceView::Models {
        models_view::workspace(state, palette)
    } else if snapshot.view == WorkspaceView::Sync {
        sync_view::workspace(state, palette)
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
        .border(
            Border::new()
                .fill(theme::color(theme::BORDER_STRONG))
                .width(1.),
        )
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
        .child(
            rect()
                .position(Position::new_absolute().right(0.).top(0.))
                .horizontal()
                .spacing(14.)
                .child(sort)
                .child(view),
        )
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

fn run_create_action(mut state: State<ShellState>, action: crate::library_contract::CreateAction) {
    state.write().menu_open = false;
    match action {
        crate::library_contract::CreateAction::Note => {
            create_note_and_open(state);
        }
        crate::library_contract::CreateAction::Drawing => {
            drawing::request_create(state);
        }
        crate::library_contract::CreateAction::Folder => {
            state.write().create(action);
        }
    }
}

#[derive(PartialEq)]
struct CreateMenuItem {
    state: State<ShellState>,
    action: crate::library_contract::CreateAction,
    icon: LibraryIcon,
    title: &'static str,
    description: &'static str,
}

impl Component for CreateMenuItem {
    fn render(&self) -> impl IntoElement {
        let area = use_state(|| Option::<Area>::None);
        let mut area_state = area;
        let action_state = self.state;
        let action = self.action;
        let hover_key = format!("create-menu:{}", self.title);
        let hovered = self.state.read().hovered_target.as_deref() == Some(hover_key.as_str());
        let move_key = hover_key.clone();
        let move_area = area;
        let mut move_state = self.state;

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
            .layer(Layer::OverlayLevel(22))
            .on_sized(move |event: Event<SizedEventData>| area_state.set(Some(event.area)))
            // Keep both paths: direct mouse-up handles the icon/root hit
            // target, while the coordinate-checked global fallback handles
            // text descendants that do not bubble mouse-up in Freya.  The
            // menu-open guard makes the two paths idempotent when both fire.
            .on_mouse_up(move |_| {
                if action_state.read().menu_open {
                    run_create_action(action_state, action);
                }
            })
            .on_global_pointer_press(move |event: Event<PointerEventData>| {
                let Some(area) = *area.read() else {
                    return;
                };
                let point = event.global_location();
                if point.x < f64::from(area.min_x())
                    || point.x > f64::from(area.max_x())
                    || point.y < f64::from(area.min_y())
                    || point.y > f64::from(area.max_y())
                    || !action_state.read().menu_open
                {
                    return;
                }
                run_create_action(action_state, action);
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let Some(area) = *move_area.read() else {
                    return;
                };
                let point = event.global_location();
                let inside = point.x >= f64::from(area.min_x())
                    && point.x <= f64::from(area.max_x())
                    && point.y >= f64::from(area.min_y())
                    && point.y <= f64::from(area.max_y());
                let current = move_state.read().hovered_target.clone();
                if inside && current.as_deref() != Some(move_key.as_str()) {
                    move_state.write().set_hovered_target(move_key.clone());
                } else if !inside && current.as_deref() == Some(move_key.as_str()) {
                    move_state.write().clear_hovered_target(&move_key);
                }
            })
            .a11y_alt(self.title)
            .child(svg_icon(self.icon, theme::color(theme::PRIMARY), 20.))
            .child(
                rect()
                    .spacing(2.)
                    .child(
                        label()
                            .font_size(14.)
                            .font_weight(FontWeight::BOLD)
                            .text(self.title),
                    )
                    .child(
                        label()
                            .font_size(12.)
                            .color(theme::color(theme::MUTED))
                            .text(self.description),
                    ),
            )
            .into_element()
    }
}

pub(super) fn create_entry_menu(state: State<ShellState>) -> Element {
    let item = |action, icon, title, description| {
        CreateMenuItem {
            state,
            action,
            icon,
            title,
            description,
        }
        .into_element()
    };

    let mut close_state = state;
    let mut escape_state = state;
    let backdrop = rect()
        .position(
            Position::new_absolute()
                .left(0.)
                .right(0.)
                .top(0.)
                .bottom(0.),
        )
        .width(Size::fill())
        .height(Size::fill())
        .on_mouse_up(move |_| close_state.write().menu_open = false);

    let popover = rect()
        .position(Position::new_global().right(20.).bottom(86.))
        .width(Size::px(280.))
        .padding(Gaps::new_all(8.))
        .background(theme::color(theme::SURFACE))
        .border(
            Border::new()
                .fill(theme::color(theme::BORDER_STRONG))
                .width(1.),
        )
        .with_corner_radius(14.)
        .layer(Layer::OverlayLevel(21))
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
        ));

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
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::Escape) {
                escape_state.write().menu_open = false;
            }
        })
        .child(backdrop)
        .child(popover)
        .into_element()
}

#[derive(PartialEq)]
struct LibraryGrid {
    state: State<ShellState>,
}

impl Component for LibraryGrid {
    fn render(&self) -> impl IntoElement {
        library_grid(self.state)
    }
}

fn library_grid(state: State<ShellState>) -> Element {
    let scroll_position = use_state(|| (0_i32, 0_i32));
    let scroll_notifier = use_state(|| ());
    let scroll_requests = use_state(Vec::<ScrollRequest>::new);
    let viewport_height = use_state(|| 0_f32);
    let content_height = use_state(|| 0_f32);
    let on_scroll = use_state(|| {
        let mut position_state = scroll_position;
        let mut notifier = scroll_notifier;
        let paging_state = state;
        Callback::new(move |event: ScrollEvent| {
            let (changed, vertical_offset) = {
                let mut position = position_state.write();
                let previous = *position;
                let vertical_offset = match event {
                    ScrollEvent::X(x) => {
                        position.0 = x;
                        None
                    }
                    ScrollEvent::Y(y) => {
                        position.1 = y;
                        Some(y)
                    }
                };
                (previous != *position, vertical_offset)
            };
            if changed {
                notifier.write();
            }
            if changed {
                if let Some(offset) = vertical_offset {
                    let viewport = *viewport_height.read();
                    let content = *content_height.read();
                    if viewport > 0. && content > 0. {
                        let scroll_top = offset.saturating_neg() as f32;
                        let distance_from_bottom =
                            (content - scroll_top - viewport).max(0.).ceil() as usize;
                        if paging_state.read().library.on_scroll(distance_from_bottom)
                            == ScrollAction::RequestMore
                        {
                            let _ = load_more_library_entries(paging_state);
                        }
                    }
                }
            }
            changed
        })
    });
    let get_scroll = use_state(|| {
        let position_state = scroll_position;
        Callback::new(move |_| *position_state.read())
    });
    let scroll_controller =
        ScrollController::managed(scroll_notifier, scroll_requests, on_scroll, get_scroll);

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

    let mut drag_move_state = state;
    let mut drag_release_state = state;
    let mut drag_action_state = state;
    let mut viewport_height_state = viewport_height;
    let mut content_height_state = content_height;
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .on_sized(move |event: Event<SizedEventData>| {
            viewport_height_state.set_if_modified(event.area.height());
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                let location = event.global_location();
                drag_move_state
                    .write()
                    .library_drag
                    .update(location.x, location.y);
            }
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
            if !event.is_primary() {
                return;
            }
            let moved = {
                let mut drag = drag_release_state.write();
                drag.library_drag.finish()
            };
            if let Some((source, target)) = moved {
                let _ = move_library_entry(&mut drag_action_state, &source, &target);
            }
        })
        .child(
            ScrollView::new_controlled(scroll_controller)
                .width(Size::fill())
                .height(Size::fill())
                .child(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(72., 10., 10., 10.))
                        .on_sized(move |event: Event<SizedEventData>| {
                            content_height_state.set_if_modified(event.area.height());
                        })
                        .child(surface),
                ),
        )
        .into_element()
}

fn grid_card_width_for_parent(available: f32) -> f32 {
    let available = available.max(0.);
    if available <= GRID_MIN_CARD_WIDTH {
        return available;
    }

    let columns = ((available + GRID_GAP) / (GRID_MIN_CARD_WIDTH + GRID_GAP))
        .floor()
        .max(1.);
    ((available - GRID_GAP * (columns - 1.)) / columns).max(0.)
}

fn grid_card_width() -> Size {
    Size::func(|context| Some(grid_card_width_for_parent(context.available_parent)))
}

fn activate_library_entry(mut state: State<ShellState>, target: EntryOpenTarget) {
    match target {
        EntryOpenTarget::Drawing(target) => {
            drawing::open_existing(state, target.as_str());
        }
        EntryOpenTarget::Folder(target) => {
            state.write().open_directory(target.as_str().to_string());
        }
        EntryOpenTarget::Note(target) => {
            let vault_entry = {
                let snapshot = state.read();
                snapshot
                    .page
                    .as_ref()
                    .and_then(|page| {
                        page.entries
                            .iter()
                            .find(|entry| entry.path == target.as_str())
                    })
                    .cloned()
            };
            if let Some(vault_entry) = vault_entry {
                state.write().open_note(&vault_entry);
            } else {
                eprintln!(
                    "[freya][library] action:failure action=open path={} reason=missing_page_entry",
                    target.as_str()
                );
                state.write().error = Some(format!(
                    "Library entry is no longer present in the current directory: {}",
                    target.as_str()
                ));
            }
        }
        EntryOpenTarget::Ignored => {}
    }
}

fn begin_library_entry_rename(
    mut card_menu_state: State<CardMenuState>,
    mut rename_value: State<String>,
    title: String,
) {
    rename_value.set(title);
    let mut menu = card_menu_state.write();
    menu.open = false;
    menu.renaming = true;
}

fn schedule_card_activation(
    state: State<ShellState>,
    target: EntryOpenTarget,
    generation: State<u64>,
    pending_generation: u64,
) {
    spawn(async move {
        Delay::new(CARD_OPEN_DELAY).await;
        if *generation.read() == pending_generation {
            activate_library_entry(state, target);
        }
    });
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
        let rename_a11y_id = use_a11y();
        let card_area = use_state(|| None::<Area>);
        let card_open_generation = use_state(|| 0_u64);
        let should_focus_rename = card_menu_state.read().renaming;
        use_side_effect(move || {
            if should_focus_rename {
                rename_a11y_id.request_focus();
            }
        });
        render_library_card(
            &self.entry,
            self.mode,
            self.featured,
            self.state,
            card_menu_state,
            rename_value,
            rename_a11y_id,
            card_area,
            card_open_generation,
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
    rename_a11y_id: AccessibilityId,
    card_area: State<Option<Area>>,
    card_open_generation: State<u64>,
) -> Element {
    let path = entry.path.as_str().to_string();
    let is_drawing =
        is_drawing_path(&path) || matches!(entry.effective_kind(), ContractKind::Drawing);
    let is_folder = matches!(entry.effective_kind(), ContractKind::Folder);
    let open_target = state.read().library.open_entry(entry);
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
    let drag_snapshot = state.read().library_drag.clone();
    let dragging = drag_snapshot.is_dragging(&path);
    let drop_state = drag_snapshot.target_state(&path);
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    let drag_source_path = path.clone();
    let mut drag_start_state = state;
    let drag_start_area = card_area;
    let drag_target_path = path.clone();
    let drag_leave_path = path.clone();
    let mut drag_enter_state = state;
    let mut drag_leave_state = state;
    let mut card_area_state = card_area;
    let drag_move_path = path.clone();
    let mut drag_move_state = state;
    let mut hover_move_state = state;
    let hover_move_key = hover_key.clone();
    let hover_move_area = card_area;

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
        rename_a11y_id,
        open_target.clone(),
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
    let mut click_menu_state = card_menu_state;
    let mut body_open_generation = card_open_generation;

    rect()
        .width(if mode == ViewMode::Grid {
            grid_card_width()
        } else {
            Size::fill()
        })
        .height(Size::px(height))
        .padding(if mode == ViewMode::Grid {
            Gaps::new_all(10.)
        } else {
            Gaps::new(8., 10., 8., 10.)
        })
        .background(theme::color(theme::mix(theme::SURFACE, theme::BG, 0.34)))
        .border(
            Border::new()
                .fill(theme::color(if hovered {
                    theme::BORDER_STRONG
                } else if drop_state == Some(true) {
                    theme::PRIMARY
                } else {
                    theme::BORDER
                }))
                .width(1.),
        )
        .with_corner_radius(10.)
        .opacity(if dragging { 0.45 } else { 1. })
        .on_sized(move |event: Event<SizedEventData>| {
            card_area_state.set(Some(event.area));
        })
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
        .on_global_pointer_down(move |event: Event<PointerEventData>| {
            if event.button() == Some(MouseButton::Left) {
                let location = event.global_location();
                let Some(area) = *drag_start_area.read() else {
                    return;
                };
                if location.x < f64::from(area.min_x())
                    || location.x > f64::from(area.max_x())
                    || location.y < f64::from(area.min_y())
                    || location.y > f64::from(area.max_y())
                {
                    return;
                }
                drag_start_state.write().library_drag.begin(
                    &drag_source_path,
                    is_folder,
                    location.x,
                    location.y,
                );
            }
        })
        .on_pointer_enter(move |_| {
            drag_enter_state
                .write()
                .library_drag
                .enter_target(&drag_target_path, is_folder);
        })
        .on_pointer_leave(move |_| {
            drag_leave_state
                .write()
                .library_drag
                .leave_target(&drag_leave_path);
        })
        .on_capture_global_pointer_move(move |event: Event<PointerEventData>| {
            let Some(area) = *card_area.read() else {
                return;
            };
            let point = event.global_location();
            let inside = point.x >= f64::from(area.min_x())
                && point.x <= f64::from(area.max_x())
                && point.y >= f64::from(area.min_y())
                && point.y <= f64::from(area.max_y());
            if inside {
                drag_move_state
                    .write()
                    .library_drag
                    .enter_target(&drag_move_path, is_folder);
            } else {
                drag_move_state
                    .write()
                    .library_drag
                    .leave_target(&drag_move_path);
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            let Some(area) = *hover_move_area.read() else {
                return;
            };
            let point = event.global_location();
            let inside = point.x >= f64::from(area.min_x())
                && point.x <= f64::from(area.max_x())
                && point.y >= f64::from(area.min_y())
                && point.y <= f64::from(area.max_y());
            if inside {
                if hover_move_state.read().hovered_target.as_deref()
                    != Some(hover_move_key.as_str())
                {
                    hover_move_state
                        .write()
                        .set_hovered_target(hover_move_key.clone());
                }
            } else if hover_move_state.read().hovered_target.as_deref()
                == Some(hover_move_key.as_str())
            {
                hover_move_state
                    .write()
                    .clear_hovered_target(&hover_move_key);
            }
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
            let menu_snapshot = click_menu_state.read().clone();
            if menu_snapshot.open
                || menu_snapshot.renaming
                || state_for_open.read().library_drag.has_moved()
            {
                return;
            }

            let mut menu = click_menu_state.write();
            menu.open = false;
            menu.renaming = false;
            drop(menu);

            // Match NoteCard.vue's CLICK_OPEN_DELAY_MS=220. Keeping the card
            // mounted during the double-click window prevents the second click
            // from landing on a newly mounted child at the same coordinates.
            // A later click on a different logical card gets its own component
            // generation and remains an independent activation.
            let pending_generation = (*body_open_generation.read()).wrapping_add(1);
            body_open_generation.set(pending_generation);
            schedule_card_activation(
                state_for_open,
                open_target.clone(),
                body_open_generation,
                pending_generation,
            );
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
    rename_a11y_id: AccessibilityId,
    open_target: EntryOpenTarget,
    renaming: bool,
) -> Element {
    let title_open_generation = use_state(|| 0_u64);
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
            .on_mouse_down(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                rename_a11y_id.request_focus();
            })
            .on_mouse_up(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                rename_a11y_id.request_focus();
            })
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
                    .border(Border::new().fill(theme::color(theme::PRIMARY)).width(1.))
                    .with_corner_radius(7.)
                    .child(
                        Input::new(rename_value)
                            .width(Size::fill())
                            .a11y_id(rename_a11y_id)
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
        let mut title_generation = title_open_generation;
        let mut title_menu_state = card_menu_state;
        let mut title_rename_value = rename_value;
        let title_for_rename = title.to_string();
        let title_open_state = state;
        label()
            .font_size(title_size)
            .font_weight(FontWeight::BOLD)
            .text(title.to_string())
            .on_mouse_up(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                let pending_generation = (*title_generation.read()).wrapping_add(1);
                title_generation.set(pending_generation);
                if EventsCombos::pressed(event.global_location).is_double() {
                    begin_library_entry_rename(
                        title_menu_state,
                        title_rename_value,
                        title_for_rename.clone(),
                    );
                    return;
                }
                schedule_card_activation(
                    title_open_state,
                    open_target.clone(),
                    title_generation,
                    pending_generation,
                );
            })
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
            .background(theme::color(theme::mix(theme::SURFACE, theme::BG, 0.55)))
            .border(
                Border::new()
                    .fill(theme::color(theme::mix(theme::BORDER, theme::BG, 0.70)))
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
            .background(theme::color(theme::mix(theme::SURFACE, theme::BG, 0.55)))
            .border(
                Border::new()
                    .fill(theme::color(theme::mix(theme::BORDER, theme::BG, 0.70)))
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

    #[test]
    fn grid_card_width_matches_tauri_auto_fill_columns() {
        assert!((grid_card_width_for_parent(972.) - 317.33334).abs() < 0.01);
        assert!((grid_card_width_for_parent(600.) - 295.).abs() < 0.01);
        assert!((grid_card_width_for_parent(230.) - 230.).abs() < 0.01);
    }
}
