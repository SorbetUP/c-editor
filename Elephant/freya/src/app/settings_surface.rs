//! Loading, error, empty, and unavailable settings surfaces.

use freya::prelude::*;

use crate::theme;

use super::super::SettingsSurfaceState;

pub(super) fn surface_state(
    surface: &SettingsSurfaceState,
    palette: theme::ThemePalette,
) -> Element {
    let (title, detail, alt) = match surface {
        SettingsSurfaceState::Loading { title, detail } => {
            (title.clone(), detail.clone(), "Settings loading")
        }
        SettingsSurfaceState::Error { title, detail } => {
            (title.clone(), detail.clone(), "Settings error")
        }
        SettingsSurfaceState::Empty { title, detail } => {
            (title.clone(), detail.clone(), "Settings empty")
        }
        SettingsSurfaceState::Ready => return rect().into_element(),
    };

    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(24.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .with_corner_radius(8.)
        .spacing(8.)
        .a11y_alt(alt)
        .child(label().font_weight(FontWeight::BOLD).text(title))
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(detail),
        )
        .into_element()
}

pub(super) fn unavailable_section(
    section_label: &str,
    palette: theme::ThemePalette,
) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(24.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .with_corner_radius(8.)
        .spacing(8.)
        .a11y_alt(format!("{section_label} is unavailable"))
        .child(
            label()
                .font_weight(FontWeight::BOLD)
                .text(format!("{section_label} is unavailable")),
        )
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("The addon is being reloaded or has been disabled. Elephant keeps this page selected instead of moving you to another menu."),
        )
        .into_element()
}
