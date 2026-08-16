//! Appearance controls aligned with the current Tauri SettingsPanel surface.

use freya::prelude::*;

use crate::{
    settings_contract::{CORE_ICON_RAIL_ITEMS, THEME_FAMILIES},
    theme,
};

use super::super::SettingsViewState;

pub(super) fn appearance_settings(
    state: State<SettingsViewState>,
    shell_state: State<super::super::super::ShellState>,
) -> Vec<Element> {
    let snapshot = state.read().clone();
    let current_theme = snapshot.runtime.text_value("theme");
    let active_family = THEME_FAMILIES
        .iter()
        .find(|family| current_theme == family.light || current_theme == family.dark)
        .unwrap_or(&THEME_FAMILIES[0]);
    let dark = current_theme == active_family.dark;

    let mut controls = vec![
        color_mode(state, shell_state, active_family.light, active_family.dark, dark),
        theme_selector(
            state,
            shell_state,
            &current_theme,
            dark,
            snapshot.settings.theme_expanded,
        ),
    ];

    for item in CORE_ICON_RAIL_ITEMS {
        controls.push(super::navigation_visibility(
            state,
            shell_state,
            item.label,
            item.id,
            snapshot.runtime.string_list_value("iconRailHidden"),
        ));
    }

    controls.push(super::preference_switch(
        state,
        "Floating surfaces",
        "Lift navigation, controls and the writing surface above the background.",
        "floatingSurfaces",
        snapshot.runtime.bool_value("floatingSurfaces"),
        "Floating surfaces",
    ));
    controls
}

fn color_mode(
    state: State<SettingsViewState>,
    shell_state: State<super::super::super::ShellState>,
    light_theme: &'static str,
    dark_theme: &'static str,
    dark: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let mut light_state = state;
    let mut dark_state = state;
    let mut light_shell = shell_state;
    let mut dark_shell = shell_state;
    rect()
        .width(Size::fill())
        .height(Size::px(68.))
        .padding(Gaps::new(13., 18., 13., 18.))
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .spacing(3.)
                .child(
                    label()
                        .font_size(13.)
                        .font_weight(FontWeight::BOLD)
                        .text("Color mode"),
                )
                .child(
                    label()
                        .font_size(11.5)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("Use the light or dark variant of the selected theme."),
                ),
        )
        .child(
            rect()
                .padding(Gaps::new_all(3.))
                .horizontal()
                .spacing(3.)
                .background(theme::token_color(palette, theme::ThemeToken::Soft))
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(10.)
                .child(mode_button("Light", !dark, palette, move || {
                    light_state
                        .write()
                        .set_text_preference("theme", light_theme.to_owned());
                    light_shell.write().mark_settings_changed();
                }))
                .child(mode_button("Dark", dark, palette, move || {
                    dark_state
                        .write()
                        .set_text_preference("theme", dark_theme.to_owned());
                    dark_shell.write().mark_settings_changed();
                })),
        )
        .into_element()
}

fn theme_selector(
    state: State<SettingsViewState>,
    shell_state: State<super::super::super::ShellState>,
    current_theme: &str,
    dark: bool,
    expanded: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let mut toggle_state = state;
    let toggle_label = if expanded {
        "Collapse themes"
    } else {
        "Expand themes"
    };
    let mut selector = rect()
        .width(Size::fill())
        .padding(Gaps::new(13., 18., 13., 18.))
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .spacing(10.)
        .a11y_alt("Theme choices")
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(30.))
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .font_size(13.)
                        .font_weight(FontWeight::BOLD)
                        .text("Theme"),
                )
                .child(
                    rect()
                        .width(Size::px(29.))
                        .height(Size::px(29.))
                        .center()
                        .with_corner_radius(8.)
                        .a11y_alt(toggle_label)
                        .on_mouse_up(move |_| {
                            toggle_state.write().settings.toggle_theme_expansion();
                        })
                        .child(
                            label()
                                .font_size(14.)
                                .text(if expanded { "⌃" } else { "⌄" }),
                        ),
                ),
        );

    if expanded {
        let mut grid = rect().width(Size::fill()).spacing(9.);
        for pair in THEME_FAMILIES.chunks(2) {
            let mut row = rect().width(Size::fill()).horizontal().spacing(9.);
            for family in pair {
                let theme_id = if dark { family.dark } else { family.light };
                row = row.child(rect().width(Size::fill()).child(super::theme_variant(
                    state,
                    shell_state,
                    family.name,
                    family.description,
                    theme_id,
                    current_theme == theme_id,
                )));
            }
            if pair.len() == 1 {
                row = row.child(rect().width(Size::fill()));
            }
            grid = grid.child(row);
        }
        selector = selector.child(grid);
    }

    selector.into_element()
}

fn mode_button(
    text: &'static str,
    active: bool,
    palette: theme::ThemePalette,
    on_press: impl FnMut() + 'static,
) -> Element {
    let mut on_press = on_press;
    rect()
        .width(Size::px(78.))
        .height(Size::px(31.))
        .center()
        .background(theme::token_color(
            palette,
            if active {
                theme::ThemeToken::Surface
            } else {
                theme::ThemeToken::Soft
            },
        ))
        .with_corner_radius(7.)
        .a11y_alt(format!("Use {text} color mode"))
        .on_mouse_up(move |_| on_press())
        .child(
            label()
                .font_size(12.)
                .color(theme::token_color(
                    palette,
                    if active {
                        theme::ThemeToken::Text
                    } else {
                        theme::ThemeToken::Muted
                    },
                ))
                .text(text),
        )
        .into_element()
}
