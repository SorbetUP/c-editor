//! Header and section navigation aligned with the source SettingsPanel chrome.

use freya::prelude::*;

use crate::{
    app::navigation_icons::{svg_icon, Icon},
    settings_contract::CORE_SECTIONS,
    theme,
};

use super::super::SettingsViewState;

pub(super) fn settings_header(
    search: Input,
    mut shell_state: State<super::super::super::ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(64.))
        .padding(Gaps::new(0., 18., 0., 22.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .child(
            label()
                .font_size(18.)
                .font_weight(FontWeight::BOLD)
                .text("Settings"),
        )
        .child(
            rect()
                .horizontal()
                .spacing(10.)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .width(Size::percent(44.))
                        .max_width(Size::px(350.))
                        .height(Size::px(36.))
                        .padding(Gaps::new(0., 10., 0., 10.))
                        .background(theme::token_color(palette, theme::ThemeToken::Bg))
                        .border(
                            Border::new()
                                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                                .width(1.),
                        )
                        .with_corner_radius(10.)
                        .cross_align(Alignment::Center)
                        .a11y_alt("Search all settings")
                        .child(search),
                )
                .child(
                    rect()
                        .width(Size::px(32.))
                        .height(Size::px(32.))
                        .center()
                        .with_corner_radius(8.)
                        .a11y_alt("Close settings")
                        .on_mouse_up(move |_| shell_state.write().settings_open = false)
                        .child(svg_icon(
                            Icon::X,
                            theme::token_color(palette, theme::ThemeToken::Muted),
                            17.,
                        )),
                ),
        )
        .into_element()
}

pub(super) fn section_navigation(
    active_section: &str,
    state: State<SettingsViewState>,
    search_query: State<String>,
    palette: theme::ThemePalette,
) -> Element {
    let items = CORE_SECTIONS
        .iter()
        .map(|section| {
            section_button(
                section.id,
                section.label,
                active_section,
                state,
                search_query,
                palette,
            )
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::px(196.))
        .height(Size::fill())
        .padding(Gaps::new(14., 10., 12., 10.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .spacing(3.)
        .a11y_alt("Settings sections")
        .children(items)
        .child(
            rect()
                .height(Size::fill())
                .main_align(Alignment::End)
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(1.))
                        .background(theme::token_color(palette, theme::ThemeToken::Border)),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(11., 9., 0., 9.))
                        .horizontal()
                        .main_align(Alignment::SpaceBetween)
                        .child(
                            label()
                                .font_size(9.5)
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text("Local-first"),
                        )
                        .child(
                            label()
                                .font_size(9.5)
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text("v0.1.0"),
                        ),
                ),
        )
        .into_element()
}

fn section_button(
    id: &'static str,
    label_text: &'static str,
    active_section: &str,
    state: State<SettingsViewState>,
    search_query: State<String>,
    palette: theme::ThemePalette,
) -> Element {
    let selected = active_section == id && search_query.read().trim().is_empty();
    let mut state = state;
    let mut query_state = search_query;
    rect()
        .width(Size::fill())
        .height(Size::px(38.))
        .padding(Gaps::new(0., 10., 0., 10.))
        .background(theme::token_color(
            palette,
            if selected {
                theme::ThemeToken::Soft
            } else {
                theme::ThemeToken::Bg
            },
        ))
        .border(
            Border::new()
                .fill(theme::token_color(
                    palette,
                    if selected {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Bg
                    },
                ))
                .width(1.),
        )
        .with_corner_radius(9.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .on_mouse_up(move |_| {
            *query_state.write() = String::new();
            state.write().select_section(id);
        })
        .a11y_alt(format!("Select {label_text} settings"))
        .child(
            label()
                .font_size(12.5)
                .font_weight(FontWeight::BOLD)
                .color(theme::token_color(
                    palette,
                    if selected {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Muted
                    },
                ))
                .text(label_text),
        )
        .child(
            label()
                .font_size(15.)
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("›"),
        )
        .into_element()
}
