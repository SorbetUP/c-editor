//! Shared preference row renderers for appearance and editor sections.

use freya::prelude::*;

use crate::{settings_contract::SettingIndexEntry, theme};

use super::super::SettingsViewState;

pub(super) fn preference_switch(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    key: &'static str,
    active: bool,
    alt: &'static str,
) -> Element {
    let palette = state.read().effects().palette();
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text(title))
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(description),
                ),
        )
        .child(
            rect()
                .height(Size::px(30.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .center()
                .with_corner_radius(15.)
                .background(theme::token_color(
                    palette,
                    if active {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Surface
                    },
                ))
                .a11y_alt(alt)
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    state.write().toggle_bool(key);
                })
                .child(label().text(if active { "On" } else { "Off" })),
        )
        .into_element()
}

pub(super) fn text_preference(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    current: String,
) -> Element {
    let palette = state.read().effects().palette();
    let value = State::create(current);
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text(title))
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(description),
                ),
        )
        .child(
            Input::new(value)
                .width(Size::px(48.))
                .placeholder("/")
                .on_submit(move |value: String| {
                    state
                        .write()
                        .set_text_preference("quickInsertTrigger", value);
                }),
        )
        .a11y_alt(title)
        .into_element()
}

pub(super) fn delay_preference(
    state: State<SettingsViewState>,
    delay: u64,
    enabled: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text("Autosave delay"))
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text("How long ElephantNote waits after the last edit."),
                ),
        )
        .child(
            rect()
                .height(Size::px(30.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .center()
                .opacity(if enabled { 1. } else { 0.5 })
                .a11y_alt("Autosave delay")
                .on_mouse_up(move |_| {
                    if enabled {
                        state.write().cycle_auto_save_delay();
                    }
                })
                .child(label().text(format!("{delay} ms"))),
        )
        .into_element()
}

pub(super) fn theme_variant(
    state: State<SettingsViewState>,
    label_text: &'static str,
    theme_id: &'static str,
    active: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(8., 12., 8., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(label().font_weight(FontWeight::BOLD).text(label_text))
        .child(
            rect()
                .padding(Gaps::new(6., 12., 6., 12.))
                .with_corner_radius(7.)
                .background(theme::token_color(
                    palette,
                    if active {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Surface
                    },
                ))
                .a11y_alt(label_text)
                .on_mouse_up(move |_| {
                    state
                        .write()
                        .set_text_preference("theme", theme_id.to_owned());
                })
                .child(label().text(if active { "Active" } else { "Use" })),
        )
        .into_element()
}

pub(super) fn navigation_visibility(
    state: State<SettingsViewState>,
    label_text: &'static str,
    item_id: &'static str,
    hidden_ids: Vec<String>,
) -> Element {
    let palette = state.read().effects().palette();
    let mut state = state;
    let hidden = hidden_ids.iter().any(|id| id == item_id);
    let title = if hidden {
        format!("Show {label_text} in navigation")
    } else {
        format!("Hide {label_text} in navigation")
    };
    let accessibility_label = if hidden {
        format!("Show {label_text} in navigation")
    } else {
        format!("Hide {label_text} in navigation")
    };
    rect()
        .width(Size::fill())
        .padding(Gaps::new(8., 12., 8., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(label().font_weight(FontWeight::BOLD).text(title))
        .child(
            rect()
                .padding(Gaps::new(6., 12., 6., 12.))
                .with_corner_radius(7.)
                .background(theme::token_color(
                    palette,
                    if hidden {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Surface
                    },
                ))
                .a11y_alt(accessibility_label)
                .on_mouse_up(move |_| state.write().toggle_rail_hidden(item_id))
                .child(label().text(if hidden { "Hidden" } else { "Visible" })),
        )
        .into_element()
}

pub(super) fn setting_row(entry: &SettingIndexEntry, palette: theme::ThemePalette) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(8.)
        .spacing(4.)
        .a11y_alt(entry.label)
        .child(label().font_weight(FontWeight::BOLD).text(entry.label))
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(entry.description),
        )
        .into_element()
}
