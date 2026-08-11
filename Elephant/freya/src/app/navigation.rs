//! Freya conversion of AppShell's TopVaultBar, IconRail and SidebarNav.

use freya::prelude::*;

use crate::{theme, vault_adapter::VaultEntry};

use super::ShellState;

pub(super) fn top_vault_bar() -> Element {
    rect()
        .height(Size::px(theme::TOPBAR_HEIGHT))
        .width(Size::fill())
        .background(theme::color(theme::BG))
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .left(if cfg!(target_os = "macos") { 84. } else { 56. })
                        .top(4.),
                )
                .width(Size::px(76.))
                .height(Size::px(24.))
                .horizontal()
                .spacing(2.)
                .child(
                    rect()
                        .width(Size::px(24.))
                        .height(Size::px(24.))
                        .center()
                        .a11y_alt("Retour")
                        .child(label().font_size(18.).text("‹")),
                )
                .child(
                    rect()
                        .width(Size::px(24.))
                        .height(Size::px(24.))
                        .center()
                        .a11y_alt("Avancer")
                        .child(label().font_size(18.).text("›")),
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

pub(super) fn icon_rail(state: State<ShellState>) -> Element {
    let sidebar_visible = state.read().sidebar_visible;
    let mut rail = rect()
        .width(Size::px(theme::RAIL_WIDTH))
        .height(Size::fill())
        .background(theme::color(theme::SURFACE))
        .padding(Gaps::new_all(7.))
        .spacing(6.);
    rail = rail
        .child(rail_action(
            if sidebar_visible {
                "Hide sidebar"
            } else {
                "Show sidebar"
            },
            "◧",
            state,
        ))
        .child(rail_action("Search", "⌕", state))
        .child(rect().expanded())
        .child(vault_action(state))
        .child(rail_action("Settings", "⚙", state))
        .maybe_child(state.read().vault_menu_open.then(|| vault_switcher(state)));
    rail.into_element()
}

fn vault_action(mut state: State<ShellState>) -> Element {
    let title = state
        .read()
        .vault
        .as_ref()
        .map(|vault| format!("{} - open vault switcher", vault.descriptor().name))
        .unwrap_or_else(|| "No vault - open vault switcher".to_string());
    rect()
        .width(Size::fill())
        .height(Size::px(34.))
        .center()
        .background(theme::color(theme::SOFT))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| state.write().vault_menu_open = true)
        .a11y_alt(title)
        .child(label().font_size(18.).text("⌂"))
        .into_element()
}

fn rail_action(
    label_text: &'static str,
    icon: &'static str,
    mut state: State<ShellState>,
) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(34.))
        .center()
        .background(theme::color(theme::SURFACE))
        .with_corner_radius(7.)
        .on_mouse_up(move |_| match label_text {
            "Search" => state.write().search_open = true,
            "Settings" => state.write().settings_open = true,
            _ => {
                let visible = state.read().sidebar_visible;
                state.write().sidebar_visible = !visible;
            }
        })
        .a11y_alt(label_text)
        .child(
            label()
                .font_size(18.)
                .color(theme::color(theme::MUTED))
                .text(icon),
        )
        .into_element()
}

fn vault_switcher(mut state: State<ShellState>) -> Element {
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
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .layer(Layer::OverlayLevel(20))
        .child(
            label()
                .padding(Gaps::new(6., 10., 8., 10.))
                .font_size(11.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(theme::MUTED))
                .text("VAULTS"),
        )
        .child(
            rect()
                .height(Size::px(34.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .horizontal()
                .cross_align(Alignment::Center)
                .background(theme::color(theme::SOFT))
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
                .child(
                    label()
                        .color(theme::color(theme::MUTED))
                        .text("⚙  Manage vaults"),
                ),
        )
        .into_element()
}

pub(super) fn sidebar_nav(mut state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    if !snapshot.sidebar_visible {
        return rect()
            .width(Size::px(0.))
            .height(Size::fill())
            .into_element();
    }
    let all_notes = rect()
        .width(Size::fill())
        .height(Size::px(38.))
        .margin(Gaps::new_all(8.))
        .padding(Gaps::new(0., 12., 0., 12.))
        .horizontal()
        .cross_align(Alignment::Center)
        .background(theme::color(theme::SOFT))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| state.write().open_directory("".to_string()))
        .a11y_alt("All notes")
        .child(label().font_size(17.).text("▣"))
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
                .map(|entry| sidebar_entry(entry, state))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    rect()
        .width(Size::px(f32::from(snapshot.sidebar_width.get())))
        .height(Size::fill())
        .background(theme::color(theme::BG))
        .child(all_notes)
        .child(
            rect()
                .height(Size::px(28.))
                .padding(Gaps::new(4., 14., 8., 14.))
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(11.)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(theme::MUTED))
                        .text("NOTES"),
                )
                .child(
                    label()
                        .a11y_alt("Search notes")
                        .font_size(16.)
                        .color(theme::color(theme::MUTED))
                        .text("⌕"),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .children(entries),
        )
        .into_element()
}

fn sidebar_entry(entry: &VaultEntry, mut state: State<ShellState>) -> Element {
    let entry = entry.clone();
    let path = entry.path.clone();
    let title = entry.title.clone();
    let is_directory = entry.is_directory;
    rect()
        .width(Size::fill())
        .height(Size::px(36.))
        .padding(Gaps::new(0., 8., 0., 14.))
        .horizontal()
        .cross_align(Alignment::Center)
        .on_mouse_up(move |_| {
            if is_directory {
                state.write().open_directory(path.clone());
            } else {
                state.write().open_note(&entry);
            }
        })
        .a11y_alt(title.clone())
        .child(
            label()
                .font_size(13.)
                .color(theme::color(theme::TEXT))
                .text(if is_directory {
                    format!("▸ {title}")
                } else {
                    title
                }),
        )
        .into_element()
}
