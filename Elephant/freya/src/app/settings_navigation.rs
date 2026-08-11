//! Header and section navigation copied from the source settings surface.

use freya::prelude::*;

use crate::{settings_contract::CORE_SECTIONS, theme};

use super::super::SettingsViewState;

pub(super) fn settings_header(search: Input, palette: theme::ThemePalette) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(48.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .font_size(20.)
                .font_weight(FontWeight::BOLD)
                .text("Settings"),
        )
        .child(
            rect()
                .width(Size::px(220.))
                .height(Size::px(34.))
                .padding(Gaps::new(8., 12., 8., 12.))
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(8.)
                .child(search),
        )
        .into_element()
}

pub(super) fn section_navigation(
    active_section: &str,
    state: State<SettingsViewState>,
    palette: theme::ThemePalette,
) -> Element {
    let items = CORE_SECTIONS
        .iter()
        .map(|section| section_button(section.id, section.label, active_section, state, palette))
        .collect::<Vec<_>>();

    rect()
        .width(Size::px(196.))
        .height(Size::fill())
        .padding(Gaps::new_all(8.))
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(10.)
        .spacing(3.)
        .a11y_alt("Settings sections")
        .children(items)
        .child(
            rect()
                .height(Size::fill())
                .main_align(Alignment::End)
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(11.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("Local-first"),
                )
                .child(
                    label()
                        .font_size(11.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("v0.1.0"),
                ),
        )
        .into_element()
}

fn section_button(
    id: &'static str,
    label_text: &'static str,
    active_section: &str,
    state: State<SettingsViewState>,
    palette: theme::ThemePalette,
) -> Element {
    let selected = active_section == id;
    let mut state = state;
    rect()
        .width(Size::fill())
        .height(Size::px(38.))
        .padding(Gaps::new(8., 10., 8., 10.))
        .background(if selected {
            theme::token_color(palette, theme::ThemeToken::Soft)
        } else {
            theme::token_color(palette, theme::ThemeToken::Surface)
        })
        .with_corner_radius(7.)
        .horizontal()
        .cross_align(Alignment::Center)
        .on_mouse_up(move |_| {
            state.write().select_section(id);
        })
        .a11y_alt(format!("Select {label_text} settings"))
        .child(label().text(label_text))
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("›"),
        )
        .into_element()
}
