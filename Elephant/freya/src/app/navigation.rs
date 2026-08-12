//! Freya conversion of AppShell's TopVaultBar, IconRail and SidebarNav.

use std::collections::HashSet;

use freya::prelude::*;

use crate::{theme, vault_adapter::VaultEntry};

use super::{
    navigation_icons::{svg_icon, Icon},
    settings_effects::SettingsEffects,
    ShellState,
};

// `app-shell-runtime-fixes.css` is imported after `app-shell.css`, so these are
// the final runtime dimensions rather than the earlier parity overrides.
const TAURI_TOPBAR_HEIGHT: f32 = 28.;
const TAURI_TOPBAR_NAV_WIDTH: f32 = 92.;
const TAURI_RAIL_WIDTH: f32 = 48.;
const TREE_ROW_HEIGHT: f32 = 36.;
const TREE_DEPTH_INDENT: f32 = 14.;
const TREE_ROW_HORIZONTAL_PADDING: f32 = 10.;
const TREE_TOGGLE_SIZE: f32 = 22.;
const TAGS_HEADER_HEIGHT: f32 = 36.;
const SIDEBAR_DRAG_THRESHOLD: f64 = 4.;

#[derive(Clone, Debug, Default, PartialEq)]
struct SidebarEntryDrag {
    source: Option<String>,
    source_is_directory: bool,
    start_x: f64,
    start_y: f64,
    moved: bool,
    target: Option<String>,
    target_allowed: bool,
}

impl SidebarEntryDrag {
    fn begin(&mut self, source: &str, source_is_directory: bool, x: f64, y: f64) {
        self.source = Some(normalize_sidebar_path(source));
        self.source_is_directory = source_is_directory;
        self.start_x = x;
        self.start_y = y;
        self.moved = false;
        self.target = None;
        self.target_allowed = false;
    }

    fn update_pointer(&mut self, x: f64, y: f64) {
        if self.source.is_none() {
            return;
        }
        if (x - self.start_x).abs() >= SIDEBAR_DRAG_THRESHOLD
            || (y - self.start_y).abs() >= SIDEBAR_DRAG_THRESHOLD
        {
            self.moved = true;
        }
    }

    fn enter_target(&mut self, target: &str) {
        let Some(source) = self.source.as_deref() else {
            return;
        };
        let target = normalize_sidebar_path(target);
        self.target_allowed = sidebar_can_drop(source, self.source_is_directory, &target);
        self.target = Some(target);
    }

    fn leave_target(&mut self, target: &str) {
        let target = normalize_sidebar_path(target);
        if self.target.as_deref() == Some(target.as_str()) {
            self.target = None;
            self.target_allowed = false;
        }
    }

    fn is_source(&self, path: &str) -> bool {
        self.source.as_deref() == Some(normalize_sidebar_path(path).as_str()) && self.moved
    }

    fn target_state(&self, path: &str) -> Option<bool> {
        let path = normalize_sidebar_path(path);
        (self.source.is_some() && self.target.as_deref() == Some(path.as_str()))
            .then_some(self.target_allowed)
    }

    fn finish(&mut self) -> Option<(String, String)> {
        let result = if self.moved && self.target_allowed {
            self.source
                .clone()
                .zip(self.target.clone())
                .filter(|(source, target)| {
                    sidebar_can_drop(source, self.source_is_directory, target)
                })
        } else {
            None
        };
        *self = Self::default();
        result
    }
}

pub(super) fn top_vault_bar(state: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let can_go_back = state.read().can_go_back();
    let can_go_forward = state.read().can_go_forward();
    let back_hovered = state.read().hovered_target.as_deref() == Some("topnav:Retour");
    let forward_hovered = state.read().hovered_target.as_deref() == Some("topnav:Avancer");
    let mut back_state = state;
    let mut forward_state = state;
    let mut back_enter_state = state;
    let mut back_leave_state = state;
    let mut forward_enter_state = state;
    let mut forward_leave_state = state;

    rect()
        .height(Size::px(TAURI_TOPBAR_HEIGHT))
        .width(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .left(if cfg!(target_os = "macos") { 84. } else { 56. })
                        .top(theme::TOPBAR_NAV_TOP),
                )
                .width(Size::px(TAURI_TOPBAR_NAV_WIDTH))
                .height(Size::px(theme::TOPBAR_NAV_BUTTON_SIZE))
                .horizontal()
                .spacing(theme::RAIL_GAP)
                .child(
                    rect()
                        .width(Size::px(theme::TOPBAR_NAV_BUTTON_SIZE))
                        .height(Size::px(theme::TOPBAR_NAV_BUTTON_SIZE))
                        .center()
                        .background(theme::token_color(
                            palette,
                            if can_go_back && back_hovered {
                                theme::ThemeToken::Soft
                            } else {
                                theme::ThemeToken::Bg
                            },
                        ))
                        .with_corner_radius(5.)
                        .opacity(if can_go_back { 1. } else { 0.3 })
                        .on_mouse_up(move |_| {
                            if can_go_back {
                                back_state.write().navigate_back();
                            }
                        })
                        .on_pointer_enter(move |_| {
                            back_enter_state.write().set_hovered_target("topnav:Retour")
                        })
                        .on_pointer_leave(move |_| {
                            back_leave_state
                                .write()
                                .clear_hovered_target("topnav:Retour")
                        })
                        .a11y_alt("Retour")
                        .child(svg_icon(
                            Icon::ChevronLeft,
                            theme::token_color(
                                palette,
                                if can_go_back && back_hovered {
                                    theme::ThemeToken::Text
                                } else {
                                    theme::ThemeToken::Muted
                                },
                            ),
                            18.,
                        )),
                )
                .child(
                    rect()
                        .width(Size::px(theme::TOPBAR_NAV_BUTTON_SIZE))
                        .height(Size::px(theme::TOPBAR_NAV_BUTTON_SIZE))
                        .center()
                        .background(theme::token_color(
                            palette,
                            if can_go_forward && forward_hovered {
                                theme::ThemeToken::Soft
                            } else {
                                theme::ThemeToken::Bg
                            },
                        ))
                        .with_corner_radius(5.)
                        .opacity(if can_go_forward { 1. } else { 0.3 })
                        .on_mouse_up(move |_| {
                            if can_go_forward {
                                forward_state.write().navigate_forward();
                            }
                        })
                        .on_pointer_enter(move |_| {
                            forward_enter_state
                                .write()
                                .set_hovered_target("topnav:Avancer")
                        })
                        .on_pointer_leave(move |_| {
                            forward_leave_state
                                .write()
                                .clear_hovered_target("topnav:Avancer")
                        })
                        .a11y_alt("Avancer")
                        .child(svg_icon(
                            Icon::ChevronRight,
                            theme::token_color(
                                palette,
                                if can_go_forward && forward_hovered {
                                    theme::ThemeToken::Text
                                } else {
                                    theme::ThemeToken::Muted
                                },
                            ),
                            18.,
                        )),
                ),
        )
        .child(
            rect()
                .position(Position::new_absolute().left(180.).top(0.))
                .width(Size::fill())
                .height(Size::px(TAURI_TOPBAR_HEIGHT)),
        )
        .a11y_alt("TopVaultBar")
        .into_element()
}

pub(super) fn icon_rail(
    state: State<ShellState>,
    palette: theme::ThemePalette,
    effects: &SettingsEffects,
) -> Element {
    let snapshot = state.read().clone();
    let rail_padding_top = if cfg!(target_os = "macos") {
        theme::RAIL_PADDING_TOP_MACOS
    } else {
        theme::RAIL_PADDING_TOP_DESKTOP
    };
    let nav = rect()
        .width(Size::fill())
        .expanded()
        .cross_align(Alignment::Center)
        .spacing(theme::RAIL_GAP)
        .children(
            effects
                .visible_rail_order(&snapshot.rail_order)
                .iter()
                .filter_map(|item| match item.as_str() {
                    "sidebar-toggle" => Some((
                        "sidebar-toggle",
                        if snapshot.sidebar_visible {
                            "Hide sidebar"
                        } else {
                            "Show sidebar"
                        },
                        if snapshot.sidebar_visible {
                            Icon::PanelLeftClose
                        } else {
                            Icon::PanelLeftOpen
                        },
                    )),
                    "search" => Some(("search", "Search", Icon::Search)),
                    _ => None,
                })
                .map(|(item_id, label_text, icon)| {
                    rail_action(item_id, label_text, icon, state, palette)
                })
                .collect::<Vec<_>>(),
        )
        .child(rect().expanded());
    let bottom = rect()
        .position(Position::new_absolute().left(0.).right(0.).bottom(0.))
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .spacing(theme::RAIL_GAP)
        .padding(Gaps::new(
            theme::RAIL_BOTTOM_PADDING_TOP,
            0.,
            theme::RAIL_BOTTOM_PADDING_BOTTOM,
            0.,
        ))
        .child(vault_action(state, palette))
        .child(rail_action(
            "settings",
            "Settings",
            Icon::Settings,
            state,
            palette,
        ));

    rect()
        .width(Size::px(TAURI_RAIL_WIDTH))
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
        .padding(Gaps::new(rail_padding_top, 0., 0., 0.))
        .a11y_alt("Workspace navigation")
        .child(nav)
        .child(bottom)
        .child(
            rect()
                .position(Position::new_absolute().right(0.).top(0.))
                .width(Size::px(1.))
                .height(Size::fill())
                .background(theme::token_color(palette, theme::ThemeToken::Border)),
        )
        .maybe_child(
            state
                .read()
                .vault_menu_open
                .then(|| vault_switcher(state, palette)),
        )
        .into_element()
}

fn vault_action(mut state: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let title = state
        .read()
        .vault
        .as_ref()
        .map(|vault| format!("{} - open vault switcher", vault.descriptor().name))
        .unwrap_or_else(|| "No vault - open vault switcher".to_string());
    let hovered = state.read().hovered_target.as_deref() == Some("rail:vault");
    let mut enter_state = state;
    let mut leave_state = state;
    rect()
        .width(Size::px(theme::RAIL_ACTION_SIZE))
        .height(Size::px(theme::RAIL_ACTION_SIZE))
        .center()
        .background(theme::token_color(palette, theme::ThemeToken::Soft))
        .opacity(if hovered { 0.85 } else { 1. })
        .with_corner_radius(10.)
        .on_mouse_up(move |_| state.write().vault_menu_open = true)
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target("rail:vault"))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target("rail:vault"))
        .a11y_alt(title)
        .child(svg_icon(
            Icon::Vault,
            theme::token_color(palette, theme::ThemeToken::Text),
            19.,
        ))
        .into_element()
}

fn rail_action(
    item_id: &'static str,
    label_text: &'static str,
    icon: Icon,
    state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let hover_key = format!("rail:{label_text}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let drop_target = state.read().rail_drop_target.as_deref() == Some(item_id);
    let dragging = state
        .read()
        .rail_drag
        .as_ref()
        .is_some_and(|drag| drag.source == item_id && drag.moved);
    let shown_icon = if item_id == "sidebar-toggle" && !hovered {
        Icon::PanelLeft
    } else {
        icon
    };
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    let mut drag_state = state;
    let mut move_state = state;
    let mut release_state = state;
    let mut focus_state = state;
    rect()
        .width(Size::px(theme::RAIL_ACTION_SIZE))
        .height(Size::px(theme::RAIL_ACTION_SIZE))
        .center()
        .background(theme::token_color(
            palette,
            if drop_target {
                theme::ThemeToken::BorderStrong
            } else if hovered {
                theme::ThemeToken::Soft
            } else {
                theme::ThemeToken::Sidebar
            },
        ))
        .with_corner_radius(8.)
        .opacity(if dragging { 0.55 } else { 1. })
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                focus_state.write().begin_rail_drag(
                    item_id,
                    event.global_location().x,
                    event.global_location().y,
                );
            }
        })
        .on_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                move_state.write().update_rail_drag(
                    item_id,
                    event.global_location().x,
                    event.global_location().y,
                );
            }
        })
        .on_mouse_up(move |event: Event<MouseEventData>| {
            let was_drag = release_state.write().finish_rail_drag(item_id);
            if was_drag {
                return;
            }
            match item_id {
                "search" => {
                    let mut shell = release_state.write();
                    shell.search_open = !shell.search_open;
                    shell.settings_open = false;
                }
                "settings" => {
                    let mut shell = release_state.write();
                    shell.settings_open = !shell.settings_open;
                    shell.search_open = false;
                }
                "sidebar-toggle" => release_state.write().toggle_sidebar(),
                _ => {}
            }
            let _ = event;
        })
        .on_pointer_enter(move |event: Event<PointerEventData>| {
            enter_state.write().set_hovered_target(enter_key.clone());
            if event.is_primary() {
                drag_state.write().update_rail_drag(
                    item_id,
                    event.global_location().x,
                    event.global_location().y,
                );
            }
        })
        .on_pointer_leave(move |_| {
            leave_state.write().clear_hovered_target(&leave_key);
        })
        .a11y_alt(label_text)
        .child(svg_icon(
            shown_icon,
            theme::token_color(
                palette,
                if hovered {
                    theme::ThemeToken::Text
                } else {
                    theme::ThemeToken::Muted
                },
            ),
            18.,
        ))
        .into_element()
}

fn vault_switcher(mut state: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let vault_name = state
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.descriptor().name.clone())
        .unwrap_or_else(|| "No vault".to_string());
    rect()
        .position(Position::new_absolute().left(52.).bottom(42.))
        .width(Size::px(250.))
        .padding(Gaps::new_all(6.))
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(10.)
        .layer(Layer::OverlayLevel(20))
        .child(
            label()
                .padding(Gaps::new(6., 10., 8., 10.))
                .font_size(11.)
                .font_weight(FontWeight::BOLD)
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("VAULTS"),
        )
        .child(
            rect()
                .height(Size::px(34.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .horizontal()
                .cross_align(Alignment::Center)
                .background(theme::token_color(palette, theme::ThemeToken::Soft))
                .with_corner_radius(6.)
                .a11y_alt(vault_name.clone())
                .child(svg_icon(
                    Icon::Vault,
                    theme::token_color(palette, theme::ThemeToken::Text),
                    15.,
                ))
                .child(
                    label()
                        .padding(Gaps::new(0., 0., 0., 10.))
                        .font_size(13.)
                        .text(vault_name),
                ),
        )
        .child(
            rect()
                .height(Size::px(34.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .horizontal()
                .cross_align(Alignment::Center)
                .on_mouse_up(move |_| {
                    let mut shell = state.write();
                    shell.settings_open = true;
                    shell.search_open = false;
                    shell.vault_menu_open = false;
                })
                .a11y_alt("Manage vaults")
                .child(svg_icon(
                    Icon::Settings,
                    theme::token_color(palette, theme::ThemeToken::Muted),
                    16.,
                ))
                .child(
                    label()
                        .padding(Gaps::new(0., 0., 0., 10.))
                        .font_size(13.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("Manage vaults"),
                ),
        )
        .into_element()
}

pub(super) fn sidebar_nav(mut state: State<ShellState>, palette: theme::ThemePalette) -> Element {
    // Hooks stay unconditional: the host component owns this lifecycle even
    // while the sidebar is hidden.
    let resizer_a11y_id = use_a11y();
    let expanded_paths = use_state(HashSet::<String>::new);
    let sidebar_drag = use_state(SidebarEntryDrag::default);
    let snapshot = state.read().clone();
    if !snapshot.sidebar_visible {
        return rect()
            .width(Size::px(0.))
            .height(Size::fill())
            .into_element();
    }

    let drag_snapshot = sidebar_drag.read().clone();
    let all_notes_hovered = snapshot.hovered_target.as_deref() == Some("sidebar:all");
    let all_notes_active = snapshot.view == crate::navigation_contract::WorkspaceView::Notes
        && snapshot.library.current_path.as_str().is_empty()
        && snapshot.editor.is_none()
        && !snapshot.search_open
        && !snapshot.settings_open;
    let all_notes_drop = drag_snapshot.target_state("");
    let sidebar_width = f32::from(snapshot.sidebar_width.get());
    let mut all_notes_open_state = state;
    let mut all_notes_enter = state;
    let mut all_notes_leave = state;
    let mut all_notes_drag_enter = sidebar_drag;
    let mut all_notes_drag_leave = sidebar_drag;
    let all_notes = rect()
        .width(Size::px(sidebar_width - 16.))
        .height(Size::px(theme::SIDEBAR_ALL_NOTES_HEIGHT))
        .padding(Gaps::new(0., 12., 0., 12.))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(10.)
        .background(theme::color(match all_notes_drop {
            Some(true) => theme::mix(palette.primary, palette.soft, 0.16),
            Some(false) => palette.soft,
            None if all_notes_active => theme::mix(palette.primary, palette.soft, 0.20),
            None if all_notes_hovered => theme::mix(palette.primary, palette.soft, 0.10),
            None => palette.soft,
        }))
        .border(sidebar_drop_border(all_notes_drop, palette))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            if !sidebar_drag.read().moved {
                all_notes_open_state.write().open_directory("".to_string());
            }
        })
        .on_pointer_enter(move |_| {
            all_notes_enter.write().set_hovered_target("sidebar:all");
            all_notes_drag_enter.write().enter_target("");
        })
        .on_pointer_leave(move |_| {
            all_notes_leave.write().clear_hovered_target("sidebar:all");
            all_notes_drag_leave.write().leave_target("");
        })
        .a11y_alt("All notes")
        .child(svg_icon(
            Icon::Inbox,
            theme::token_color(palette, theme::ThemeToken::Text),
            18.,
        ))
        .child(label().font_size(14.).text("All notes"));

    // SidebarNav.vue keeps rootSidebarEntries stable while directory contents
    // are loaded lazily by each expanded tree node. Do the same here instead
    // of reusing `page.entries`, which represents the current library page.
    let root_entries = snapshot
        .vault
        .as_ref()
        .and_then(|vault| vault.list_directory("").ok())
        .map(|page| page.entries)
        .or_else(|| snapshot.page.as_ref().map(|page| page.entries.clone()))
        .unwrap_or_default();
    let entries = root_entries
        .iter()
        .filter(|entry| sidebar_entry_visible(entry))
        .map(|entry| {
            sidebar_entry(
                entry,
                0,
                state,
                palette,
                expanded_paths,
                sidebar_drag,
            )
        })
        .collect::<Vec<_>>();

    let mut resize_press_state = state;
    let mut resize_move_state = state;
    let mut resize_release_state = state;
    let mut resize_key_state = state;
    let mut resize_enter_state = state;
    let mut resize_leave_state = state;
    let resizer = rect()
        .position(Position::new_absolute().right(-12.).top(0.))
        .width(Size::px(theme::SIDEBAR_RESIZER_WIDTH))
        .height(Size::fill())
        .center()
        .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
        .child(
            rect()
                .width(Size::px(3.))
                .height(Size::px(64.))
                .background(theme::token_color(
                    palette,
                    if snapshot.hovered_target.as_deref() == Some("sidebar:resizer") {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Bg
                    },
                ))
                .with_corner_radius(999.),
        )
        .a11y_id(resizer_a11y_id)
        .a11y_alt("Resize sidebar")
        .on_pointer_enter(move |_| {
            resize_enter_state
                .write()
                .set_hovered_target("sidebar:resizer");
        })
        .on_pointer_leave(move |_| {
            resize_leave_state
                .write()
                .clear_hovered_target("sidebar:resizer");
        })
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                resizer_a11y_id.request_focus();
                resize_press_state
                    .write()
                    .begin_sidebar_resize(event.global_location().x);
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                resize_move_state
                    .write()
                    .update_sidebar_resize(event.global_location().x);
            }
        })
        .on_global_pointer_up(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                resize_release_state
                    .write()
                    .finish_sidebar_resize(event.global_location().x);
            }
        })
        .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
            Key::Named(NamedKey::ArrowLeft) => resize_key_state.write().resize_sidebar_by(-16.),
            Key::Named(NamedKey::ArrowRight) => resize_key_state.write().resize_sidebar_by(16.),
            _ => {}
        });

    let entries = ScrollView::new()
        .width(Size::fill())
        .height(Size::fill())
        .show_scrollbar(true)
        .scroll_with_arrows(true)
        .drag_scrolling(false)
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(0., 6., 0., 6.))
                .spacing(theme::RAIL_GAP)
                .children(entries),
        );
    let mut search_state = state;
    let sidebar_scroll = rect()
        .width(Size::fill())
        .height(Size::fill())
        .cross_align(Alignment::Center)
        .padding(Gaps::new(theme::SIDEBAR_SCROLL_PADDING_TOP, 0., 0., 0.))
        .child(all_notes)
        .child(rect().height(Size::px(8.)))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(TAGS_HEADER_HEIGHT))
                .padding(Gaps::new(4., 14., 8., 14.))
                .horizontal()
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(11.)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .a11y_alt("Notes")
                        .text("NOTES"),
                )
                .child(
                    rect()
                        .width(Size::px(24.))
                        .height(Size::px(24.))
                        .center()
                        .with_corner_radius(6.)
                        .on_mouse_up(move |_| {
                            let mut shell = search_state.write();
                            shell.search_open = true;
                            shell.settings_open = false;
                        })
                        .a11y_alt("Search notes")
                        .child(svg_icon(
                            Icon::Search,
                            theme::token_color(palette, theme::ThemeToken::Muted),
                            14.,
                        )),
                ),
        )
        .child(entries);
    let sidebar = rect()
        .width(Size::px(sidebar_width))
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
        .a11y_alt("Sidebar")
        .child(sidebar_scroll)
        .child(
            rect()
                .position(Position::new_absolute().right(0.).top(0.))
                .width(Size::px(1.))
                .height(Size::fill())
                .background(theme::token_color(palette, theme::ThemeToken::Border)),
        );

    let mut drag_move = sidebar_drag;
    let mut drag_release = sidebar_drag;
    let mut drop_state = state;
    let mut drop_expanded_paths = expanded_paths;
    rect()
        .width(Size::px(sidebar_width))
        .height(Size::fill())
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                drag_move
                    .write()
                    .update_pointer(event.global_location().x, event.global_location().y);
            }
        })
        .on_global_pointer_up(move |event: Event<PointerEventData>| {
            if !event.is_primary() {
                return;
            }
            let drop = drag_release.write().finish();
            if let Some((source, target)) = drop {
                if let Some(new_path) = move_sidebar_entry(&mut drop_state.write(), &source, &target)
                {
                    rebase_expanded_paths(
                        &mut drop_expanded_paths.write(),
                        &source,
                        &new_path,
                    );
                }
            }
        })
        .child(sidebar)
        .child(resizer)
        .into_element()
}

fn sidebar_entry(
    entry: &VaultEntry,
    depth: usize,
    state: State<ShellState>,
    palette: theme::ThemePalette,
    expanded_paths: State<HashSet<String>>,
    sidebar_drag: State<SidebarEntryDrag>,
) -> Element {
    let entry = entry.clone();
    let path = entry.path.clone();
    let title = entry.title.clone();
    let is_directory = entry.is_directory;
    let snapshot = state.read().clone();
    let drag_snapshot = sidebar_drag.read().clone();
    let folder_active = is_directory
        && folder_path_is_active(snapshot.library.current_path.as_str(), path.as_str());
    let note_active = !is_directory && note_is_active(&snapshot, &entry);
    let explicitly_expanded = expanded_paths.read().contains(path.as_str());
    // Active paths are expanded just like SidebarTreeEntry's immediate watcher,
    // so a restored/reloaded navigation state reconstructs the visible branch.
    let expanded = is_directory && (explicitly_expanded || folder_active);
    let hover_key = format!("sidebar:{path}");
    let hovered = snapshot.hovered_target.as_deref() == Some(hover_key.as_str());
    let active = folder_active || note_active;
    let dragging = drag_snapshot.is_source(&path);
    let drop_state = is_directory
        .then(|| drag_snapshot.target_state(&path))
        .flatten();
    let row_color = if active || hovered || drop_state == Some(true) {
        theme::ThemeToken::Text
    } else {
        theme::ThemeToken::Muted
    };
    let left_padding = TREE_ROW_HORIZONTAL_PADDING + depth as f32 * TREE_DEPTH_INDENT;
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    let mut drag_enter = sidebar_drag;
    let mut drag_leave = sidebar_drag;

    let row = if is_directory {
        let toggle_path = path.clone();
        let mut toggle_expanded = expanded_paths;
        let open_path = path.clone();
        let expand_on_open_path = path.clone();
        let mut open_state = state;
        let mut expand_on_open = expanded_paths;
        let mut start_drag = sidebar_drag;
        let click_drag = sidebar_drag;
        let count = entry.note_count;
        let drag_path = path.clone();
        let target_path = path.clone();
        let leave_target_path = path.clone();
        rect()
            .width(Size::fill())
            .height(Size::px(TREE_ROW_HEIGHT))
            .padding(Gaps::new(0., TREE_ROW_HORIZONTAL_PADDING, 0., left_padding))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(6.)
            .background(sidebar_row_background(
                active,
                hovered,
                drop_state,
                palette,
            ))
            .border(sidebar_drop_border(drop_state, palette))
            .with_corner_radius(8.)
            .opacity(if dragging { 0.45 } else { 1. })
            .on_pointer_enter(move |_| {
                enter_state.write().set_hovered_target(enter_key.clone());
                drag_enter.write().enter_target(&target_path);
            })
            .on_pointer_leave(move |_| {
                leave_state.write().clear_hovered_target(&leave_key);
                drag_leave.write().leave_target(&leave_target_path);
            })
            .a11y_alt(title.clone())
            .child(
                rect()
                    .width(Size::px(TREE_TOGGLE_SIZE))
                    .height(Size::px(TREE_TOGGLE_SIZE))
                    .center()
                    .a11y_alt(if expanded {
                        format!("Collapse {title}")
                    } else {
                        format!("Expand {title}")
                    })
                    .on_mouse_up(move |_| {
                        let mut paths = toggle_expanded.write();
                        if !paths.remove(toggle_path.as_str()) {
                            paths.insert(toggle_path.clone());
                        }
                    })
                    .child(svg_icon(
                        if expanded {
                            Icon::ChevronDown
                        } else {
                            Icon::ChevronRight
                        },
                        theme::token_color(palette, row_color),
                        15.,
                    )),
            )
            .child(
                rect()
                    .expanded()
                    .height(Size::fill())
                    .cross_align(Alignment::Center)
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        if event.is_primary() {
                            start_drag.write().begin(
                                &drag_path,
                                true,
                                event.global_location().x,
                                event.global_location().y,
                            );
                        }
                    })
                    .on_mouse_up(move |_| {
                        if click_drag.read().moved {
                            return;
                        }
                        expand_on_open.write().insert(expand_on_open_path.clone());
                        open_state.write().open_directory(open_path.clone());
                    })
                    .a11y_alt(title.clone())
                    .child(
                        label()
                            .font_size(14.)
                            .color(theme::token_color(palette, row_color))
                            .text(title),
                    ),
            )
            .maybe_child((count > 0).then(|| {
                label()
                    .font_size(12.)
                    .color(theme::token_color(palette, theme::ThemeToken::Muted))
                    .text(count.to_string())
            }))
            .into_element()
    } else {
        let mut open_state = state;
        let mut start_drag = sidebar_drag;
        let click_drag = sidebar_drag;
        let drag_path = path.clone();
        rect()
            .width(Size::fill())
            .height(Size::px(TREE_ROW_HEIGHT))
            .padding(Gaps::new(0., TREE_ROW_HORIZONTAL_PADDING, 0., left_padding))
            .horizontal()
            .cross_align(Alignment::Center)
            .background(sidebar_row_background(active, hovered, None, palette))
            .with_corner_radius(8.)
            .opacity(if dragging { 0.45 } else { 1. })
            .on_pointer_down(move |event: Event<PointerEventData>| {
                if event.is_primary() {
                    start_drag.write().begin(
                        &drag_path,
                        false,
                        event.global_location().x,
                        event.global_location().y,
                    );
                }
            })
            .on_mouse_up(move |_| {
                if !click_drag.read().moved {
                    open_state.write().open_note(&entry);
                }
            })
            .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
            .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
            .a11y_alt(title.clone())
            .child(
                label()
                    .font_size(14.)
                    .color(theme::token_color(palette, row_color))
                    .text(title),
            )
            .into_element()
    };

    let children = if expanded {
        snapshot
            .vault
            .as_ref()
            .and_then(|vault| vault.list_directory(path.clone()).ok())
            .map(|page| {
                page.entries
                    .iter()
                    .filter(|child| sidebar_entry_visible(child))
                    .map(|child| {
                        sidebar_entry(
                            child,
                            depth + 1,
                            state,
                            palette,
                            expanded_paths,
                            sidebar_drag,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    rect()
        .width(Size::fill())
        .child(row)
        .children(children)
        .into_element()
}

fn sidebar_row_background(
    active: bool,
    hovered: bool,
    drop_state: Option<bool>,
    palette: theme::ThemePalette,
) -> Color {
    match drop_state {
        Some(true) => theme::color(theme::mix(palette.primary, palette.soft, 0.18)),
        _ if active || hovered => theme::token_color(palette, theme::ThemeToken::Soft),
        _ => theme::token_color(palette, theme::ThemeToken::Sidebar),
    }
}

fn sidebar_drop_border(drop_state: Option<bool>, palette: theme::ThemePalette) -> Border {
    match drop_state {
        Some(true) => Border::new()
            .fill(theme::token_color(palette, theme::ThemeToken::Primary))
            .width(1.),
        Some(false) => Border::new()
            .fill(theme::token_color(palette, theme::ThemeToken::Danger))
            .width(1.),
        None => Border::new()
            .fill(theme::token_color(palette, theme::ThemeToken::Sidebar))
            .width(0.),
    }
}

fn normalize_sidebar_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn sidebar_parent_path(path: &str) -> String {
    let path = normalize_sidebar_path(path);
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn sidebar_can_drop(source: &str, source_is_directory: bool, target_directory: &str) -> bool {
    let source = normalize_sidebar_path(source);
    let target = normalize_sidebar_path(target_directory);
    if source.is_empty() || sidebar_parent_path(&source) == target {
        return false;
    }
    if source_is_directory
        && (target == source
            || target
                .strip_prefix(source.as_str())
                .is_some_and(|suffix| suffix.starts_with('/')))
    {
        return false;
    }
    true
}

fn rebase_path(path: &str, source: &str, destination: &str) -> String {
    let path = normalize_sidebar_path(path);
    let source = normalize_sidebar_path(source);
    let destination = normalize_sidebar_path(destination);
    if path == source {
        return destination;
    }
    path.strip_prefix(source.as_str())
        .filter(|suffix| suffix.starts_with('/'))
        .map(|suffix| format!("{destination}{suffix}"))
        .unwrap_or(path)
}

fn rebase_expanded_paths(paths: &mut HashSet<String>, source: &str, destination: &str) {
    let rebased = paths
        .iter()
        .map(|path| rebase_path(path, source, destination))
        .collect::<HashSet<_>>();
    *paths = rebased;
}

fn move_sidebar_entry(
    shell: &mut ShellState,
    source: &str,
    target_directory: &str,
) -> Option<String> {
    let Some(vault) = shell.vault.clone() else {
        shell.error = Some("No vault selected.".to_string());
        return None;
    };
    let source = normalize_sidebar_path(source);
    let target_directory = normalize_sidebar_path(target_directory);
    let opened_relative = shell
        .editor
        .as_ref()
        .and_then(|document| document.path())
        .and_then(|path| path.strip_prefix(vault.root()).ok())
        .map(|path| normalize_sidebar_path(path.to_string_lossy().as_ref()));

    eprintln!(
        "[freya][sidebar] action:move-start source={} target={}",
        source, target_directory
    );
    match vault.move_entry(&source, &target_directory) {
        Ok(new_path) => {
            let new_path = normalize_sidebar_path(&new_path);
            let current = shell.library.current_path.as_str().to_string();
            let next_current = rebase_path(&current, &source, &new_path);
            shell.library.current_path =
                crate::library_contract::RelativePath::from(next_current.as_str());
            shell.reload_directory(&next_current);

            if let Some(opened_relative) = opened_relative {
                let next_opened = rebase_path(&opened_relative, &source, &new_path);
                if next_opened != opened_relative {
                    match crate::editor::EditorDocument::load(&vault.root().join(&next_opened)) {
                        Ok(document) => shell.editor = Some(document),
                        Err(error) => {
                            shell.editor = None;
                            shell.error = Some(error.to_string());
                        }
                    }
                }
            }

            eprintln!(
                "[freya][sidebar] action:move-complete source={} destination={}",
                source, new_path
            );
            Some(new_path)
        }
        Err(error) => {
            eprintln!(
                "[freya][sidebar] action:move-failure source={} target={} error={error}",
                source, target_directory
            );
            shell.error = Some(error.to_string());
            None
        }
    }
}

fn sidebar_entry_visible(entry: &VaultEntry) -> bool {
    if entry
        .path
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .any(|part| part.starts_with('.'))
    {
        return false;
    }
    entry.is_directory || entry.path.to_ascii_lowercase().ends_with(".md")
}

fn folder_path_is_active(current_path: &str, folder_path: &str) -> bool {
    if folder_path.is_empty() {
        return false;
    }
    current_path == folder_path
        || current_path
            .strip_prefix(folder_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn note_is_active(snapshot: &ShellState, entry: &VaultEntry) -> bool {
    let Some(document_path) = snapshot.editor.as_ref().and_then(|document| document.path()) else {
        return false;
    };
    let Some(vault) = snapshot.vault.as_ref() else {
        return false;
    };
    let expected = vault.root().join(&entry.path);
    document_path == expected.as_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault_adapter::EntryKind;

    fn entry(path: &str, is_directory: bool) -> VaultEntry {
        VaultEntry {
            path: path.to_string(),
            filename: path.rsplit('/').next().unwrap_or(path).to_string(),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            title: path.rsplit('/').next().unwrap_or(path).to_string(),
            entry_type: if is_directory {
                EntryKind::Folder
            } else {
                EntryKind::Note
            },
            kind: if is_directory {
                EntryKind::Folder
            } else {
                EntryKind::Note
            },
            is_directory,
            note_count: 0,
            excerpt: String::new(),
            preview: String::new(),
            tags: Vec::new(),
            updated_at: String::new(),
            children_preview: Vec::new(),
            drawing_preview: None,
            full_path: path.to_string(),
        }
    }

    #[test]
    fn sidebar_only_keeps_folders_and_markdown_notes() {
        assert!(sidebar_entry_visible(&entry("Projects", true)));
        assert!(sidebar_entry_visible(&entry("Projects/Plan.MD", false)));
        assert!(!sidebar_entry_visible(&entry("Projects/image.png", false)));
        assert!(!sidebar_entry_visible(&entry(".assets/private.md", false)));
    }

    #[test]
    fn active_folder_matches_tauri_descendant_rule() {
        assert!(folder_path_is_active("Projects", "Projects"));
        assert!(folder_path_is_active("Projects/Elephant", "Projects"));
        assert!(!folder_path_is_active("Projector", "Projects"));
        assert!(!folder_path_is_active("", "Projects"));
    }

    #[test]
    fn sidebar_drop_matches_tauri_move_guards() {
        assert!(sidebar_can_drop("Alpha.md", false, "Projects"));
        assert!(sidebar_can_drop("Projects/Plan.md", false, "Archive"));
        assert!(!sidebar_can_drop("Projects/Plan.md", false, "Projects"));
        assert!(!sidebar_can_drop("Projects", true, "Projects"));
        assert!(!sidebar_can_drop("Projects", true, "Projects/Nested"));
        assert!(!sidebar_can_drop("Projects", true, ""));
        assert!(sidebar_can_drop("Projects/Nested", true, ""));
    }

    #[test]
    fn moved_paths_rebase_current_and_descendants() {
        assert_eq!(
            rebase_path("Projects", "Projects", "Archive/Projects"),
            "Archive/Projects"
        );
        assert_eq!(
            rebase_path("Projects/Nested", "Projects", "Archive/Projects"),
            "Archive/Projects/Nested"
        );
        assert_eq!(
            rebase_path("Other", "Projects", "Archive/Projects"),
            "Other"
        );
    }

    #[test]
    fn sidebar_drag_requires_real_motion_and_valid_target() {
        let mut drag = SidebarEntryDrag::default();
        drag.begin("Projects/Plan.md", false, 10., 10.);
        drag.enter_target("Archive");
        drag.update_pointer(12., 12.);
        assert_eq!(drag.finish(), None);

        drag.begin("Projects/Plan.md", false, 10., 10.);
        drag.enter_target("Archive");
        drag.update_pointer(16., 10.);
        assert_eq!(
            drag.finish(),
            Some(("Projects/Plan.md".to_string(), "Archive".to_string()))
        );
    }

    #[test]
    fn tauri_navigation_metrics_are_kept_explicit() {
        assert_eq!(TAURI_TOPBAR_HEIGHT, 28.);
        assert_eq!(TAURI_TOPBAR_NAV_WIDTH, 92.);
        assert_eq!(TAURI_RAIL_WIDTH, 48.);
        assert_eq!(TREE_ROW_HEIGHT, 36.);
        assert_eq!(TREE_DEPTH_INDENT, 14.);
        assert_eq!(TREE_TOGGLE_SIZE, 22.);
        assert_eq!(TAGS_HEADER_HEIGHT, 36.);
    }
}
