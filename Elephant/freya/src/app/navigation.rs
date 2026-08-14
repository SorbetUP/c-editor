//! Freya conversion of AppShell's TopVaultBar, IconRail and SidebarNav.

use freya::prelude::*;

use crate::{theme, vault_adapter::VaultEntry};

use super::{
    navigation_icons::{svg_icon, Icon},
    settings_effects::SettingsEffects,
    ShellState,
};

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
        .height(Size::px(theme::TOPBAR_HEIGHT))
        .width(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .left(if cfg!(target_os = "macos") { 84. } else { 56. })
                        .top(theme::TOPBAR_NAV_TOP),
                )
                .width(Size::px(76.))
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
                        .on_mouse_up(move |_| back_state.write().navigate_back())
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
                        .on_mouse_up(move |_| forward_state.write().navigate_forward())
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
                .height(Size::px(theme::TOPBAR_HEIGHT)),
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
        .width(Size::px(theme::RAIL_WIDTH))
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
        .padding(Gaps::new(rail_padding_top, 0., 0., 0.))
        .a11y_alt("Workspace navigation")
        .child(nav)
        .child(bottom)
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
        .with_corner_radius(7.)
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
        .with_corner_radius(7.)
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
            match label_text {
                "Search" => {
                    let mut shell = release_state.write();
                    shell.search_open = !shell.search_open;
                    shell.settings_open = false;
                }
                "Settings" => {
                    let mut shell = release_state.write();
                    shell.settings_open = !shell.settings_open;
                    shell.search_open = false;
                }
                _ => release_state.write().toggle_sidebar(),
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
            icon,
            theme::token_color(palette, theme::ThemeToken::Muted),
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
        .padding(Gaps::new_all(8.))
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
                .with_corner_radius(7.)
                .a11y_alt(vault_name.clone())
                .child(label().text(vault_name)),
        )
        .child(
            rect()
                .height(Size::px(34.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .horizontal()
                .cross_align(Alignment::Center)
                .on_mouse_up(move |_| state.write().settings_open = true)
                .a11y_alt("Manage vaults")
                .child(svg_icon(
                    Icon::Settings,
                    theme::token_color(palette, theme::ThemeToken::Muted),
                    16.,
                ))
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("Manage vaults"),
                ),
        )
        .into_element()
}

pub(super) fn sidebar_nav(mut state: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let resizer_a11y_id = use_a11y();
    let snapshot = state.read().clone();
    if !snapshot.sidebar_visible {
        return rect()
            .width(Size::px(0.))
            .height(Size::fill())
            .into_element();
    }
    let all_notes_hovered = snapshot.hovered_target.as_deref() == Some("sidebar:all");
    let all_notes_active = snapshot.view == crate::navigation_contract::WorkspaceView::Notes
        && snapshot.library.current_path.as_str().is_empty()
        && snapshot.editor.is_none()
        && !snapshot.search_open
        && !snapshot.settings_open;
    let sidebar_width = f32::from(snapshot.sidebar_width.get());
    let mut all_notes_enter = state;
    let mut all_notes_leave = state;
    let all_notes = rect()
        .width(Size::px(sidebar_width - 16.))
        .height(Size::px(theme::SIDEBAR_ALL_NOTES_HEIGHT))
        .padding(Gaps::new(0., 12., 0., 12.))
        .horizontal()
        .cross_align(Alignment::Center)
        .background(theme::color(if all_notes_hovered {
            theme::mix(palette.primary, palette.soft, 0.24)
        } else if all_notes_active {
            theme::mix(palette.primary, palette.soft, 0.20)
        } else {
            palette.soft
        }))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| state.write().open_directory("".to_string()))
        .on_pointer_enter(move |_| all_notes_enter.write().set_hovered_target("sidebar:all"))
        .on_pointer_leave(move |_| all_notes_leave.write().clear_hovered_target("sidebar:all"))
        .a11y_alt("All notes")
        .child(svg_icon(
            Icon::Inbox,
            theme::token_color(palette, theme::ThemeToken::Text),
            18.,
        ))
        .child(
            label()
                .padding(Gaps::new(0., 0., 0., 10.))
                .font_size(14.)
                .text("All notes"),
        );

    let entries = snapshot
        .page
        .as_ref()
        .map(|page| {
            page.entries
                .iter()
                .map(|entry| sidebar_entry(entry, state, palette))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut resize_press_state = state;
    let mut resize_move_state = state;
    let mut resize_release_state = state;
    let mut resize_key_state = state;
    let mut resize_enter_state = state;
    let mut resize_leave_state = state;
    let resizer = rect()
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
            resize_move_state
                .write()
                .update_sidebar_resize(event.global_location().x);
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
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
    let entries = rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(0., 6., 0., 6.))
        .spacing(theme::RAIL_GAP)
        .children(entries);
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
                .height(Size::px(28.))
                .padding(Gaps::new(4., 14., 8., 14.))
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(11.)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .a11y_alt("Notes")
                        .text("Notes"),
                )
                .child(rect().a11y_alt("Search notes").child(svg_icon(
                    Icon::Search,
                    theme::token_color(palette, theme::ThemeToken::Muted),
                    14.,
                ))),
        )
        .child(entries);
    let sidebar = rect()
        .width(Size::px(sidebar_width))
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Sidebar))
        .a11y_alt("Sidebar")
        .child(sidebar_scroll);
    rect()
        .width(Size::px(sidebar_width + theme::SIDEBAR_RESIZER_WIDTH))
        .height(Size::fill())
        .horizontal()
        .child(sidebar)
        .child(resizer)
        .into_element()
}

fn sidebar_entry(
    entry: &VaultEntry,
    mut state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let entry = entry.clone();
    let path = entry.path.clone();
    let title = entry.title.clone();
    let is_directory = entry.is_directory;
    let hover_key = format!("sidebar:{path}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let enter_key = hover_key.clone();
    let leave_key = hover_key.clone();
    let mut enter_state = state;
    let mut leave_state = state;
    rect()
        .width(Size::fill())
        .height(Size::px(36.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .horizontal()
        .cross_align(Alignment::Center)
        .background(theme::token_color(
            palette,
            if hovered {
                theme::ThemeToken::Soft
            } else {
                theme::ThemeToken::Sidebar
            },
        ))
        .on_mouse_up(move |_| {
            if is_directory {
                state.write().open_directory(path.clone());
            } else {
                state.write().open_note(&entry);
            }
        })
        .on_pointer_enter(move |_| enter_state.write().set_hovered_target(enter_key.clone()))
        .on_pointer_leave(move |_| leave_state.write().clear_hovered_target(&leave_key))
        .a11y_alt(title.clone())
        .child(if is_directory {
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(4.)
                .child(svg_icon(
                    Icon::ChevronRight,
                    theme::token_color(palette, theme::ThemeToken::Muted),
                    15.,
                ))
                .child(
                    label()
                        .font_size(14.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(title),
                )
                .into_element()
        } else {
            label()
                .font_size(14.)
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(title)
                .into_element()
        })
        .into_element()
}
