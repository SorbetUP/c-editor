//! Shared preference row renderers aligned with Settings visual primitives.

use freya::prelude::*;

use crate::{settings_contract::SettingIndexEntry, theme};

use super::super::SettingsViewState;

fn row_copy(
    title: impl Into<String>,
    description: impl Into<String>,
    palette: theme::ThemePalette,
) -> Element {
    rect()
        .width(Size::fill())
        .spacing(3.)
        .child(
            label()
                .font_size(13.)
                .font_weight(FontWeight::BOLD)
                .text(title.into()),
        )
        .child(
            label()
                .font_size(11.5)
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(description.into()),
        )
        .into_element()
}

macro_rules! settings_row {
    ($palette:expr) => {
        rect()
            .width(Size::fill())
            .height(Size::px(68.))
            .padding(Gaps::new(13., 18., 13., 18.))
            .background(theme::token_color($palette, theme::ThemeToken::Surface))
            .horizontal()
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
    };
}

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
    settings_row!(palette)
        .child(row_copy(title, description, palette))
        .child(
            rect()
                .width(Size::px(42.))
                .height(Size::px(24.))
                .padding(Gaps::new_all(2.))
                .horizontal()
                .main_align(if active {
                    Alignment::End
                } else {
                    Alignment::Start
                })
                .cross_align(Alignment::Center)
                .background(theme::token_color(
                    palette,
                    if active {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::BorderStrong
                    },
                ))
                .with_corner_radius(99.)
                .a11y_alt(alt)
                .on_press(move |event: Event<PressEventData>| {
                    event.stop_propagation();
                    state.write().toggle_bool(key);
                })
                .child(
                    rect()
                        .width(Size::px(20.))
                        .height(Size::px(20.))
                        .background(theme::token_color(palette, theme::ThemeToken::Surface))
                        .with_corner_radius(99.),
                ),
        )
        .into_element()
}

#[derive(PartialEq)]
struct TextPreferenceControl {
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    current: String,
}

impl Component for TextPreferenceControl {
    fn render(&self) -> impl IntoElement {
        let palette = self.state.read().effects().palette();
        let initial = self.current.clone();
        let value = use_state(move || initial);
        let mut state = self.state;
        settings_row!(palette)
            .child(row_copy(self.title, self.description, palette))
            .child(
                rect()
                    .width(Size::px(52.))
                    .height(Size::px(36.))
                    .padding(Gaps::new(0., 10., 0., 10.))
                    .border(
                        Border::new()
                            .fill(theme::token_color(palette, theme::ThemeToken::Border))
                            .width(1.),
                    )
                    .with_corner_radius(9.)
                    .center()
                    .child(
                        Input::new(value)
                            .width(Size::fill())
                            .placeholder("/")
                            .on_submit(move |value: String| {
                                state
                                    .write()
                                    .set_text_preference("quickInsertTrigger", value);
                            }),
                    ),
            )
            .a11y_alt(self.title)
    }
}

pub(super) fn text_preference(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    current: String,
) -> Element {
    TextPreferenceControl {
        state,
        title,
        description,
        current,
    }
    .into_element()
}

pub(super) fn delay_preference(
    state: State<SettingsViewState>,
    delay: u64,
    enabled: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let mut state = state;
    settings_row!(palette)
        .child(row_copy(
            "Autosave delay",
            "How long ElephantNote waits after the last edit.",
            palette,
        ))
        .child(
            rect()
                .height(Size::px(36.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .center()
                .opacity(if enabled { 1. } else { 0.52 })
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(9.)
                .a11y_alt("Autosave delay")
                .on_press(move |_| {
                    if enabled {
                        state.write().cycle_auto_save_delay();
                    }
                })
                .child(label().font_size(11.).text(format_delay(delay))),
        )
        .into_element()
}

fn format_delay(delay: u64) -> String {
    match delay {
        250 => "Instant · 250 ms".to_owned(),
        500 => "Fast · 500 ms".to_owned(),
        1000 => "Balanced · 1 s".to_owned(),
        2000 => "Relaxed · 2 s".to_owned(),
        5000 => "Battery saver · 5 s".to_owned(),
        other => format!("{other} ms"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn integer_stepper(
    state: State<SettingsViewState>,
    title: &'static str,
    description: &'static str,
    key: &'static str,
    value: i64,
    minimum: i64,
    maximum: i64,
    step: i64,
    suffix: &'static str,
) -> Element {
    let palette = state.read().effects().palette();
    let mut lower_state = state;
    let mut upper_state = state;
    let lower = (value - step).max(minimum);
    let upper = (value + step).min(maximum);
    settings_row!(palette)
        .child(row_copy(title, description, palette))
        .child(
            rect()
                .horizontal()
                .spacing(7.)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .width(Size::px(29.))
                        .height(Size::px(29.))
                        .center()
                        .border(
                            Border::new()
                                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                                .width(1.),
                        )
                        .with_corner_radius(8.)
                        .opacity(if value <= minimum { 0.48 } else { 1.0 })
                        .a11y_alt(format!("Decrease {title}"))
                        .on_press(move |_| {
                            if value > minimum {
                                lower_state.write().set_integer_preference(key, lower);
                            }
                        })
                        .child(label().text("−")),
                )
                .child(
                    rect()
                        .height(Size::px(30.))
                        .padding(Gaps::new(0., 8., 0., 8.))
                        .center()
                        .border(
                            Border::new()
                                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                                .width(1.),
                        )
                        .with_corner_radius(8.)
                        .child(
                            label()
                                .font_size(10.5)
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text(format!("{value}{suffix}")),
                        ),
                )
                .child(
                    rect()
                        .width(Size::px(29.))
                        .height(Size::px(29.))
                        .center()
                        .border(
                            Border::new()
                                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                                .width(1.),
                        )
                        .with_corner_radius(8.)
                        .opacity(if value >= maximum { 0.48 } else { 1.0 })
                        .a11y_alt(format!("Increase {title}"))
                        .on_press(move |_| {
                            if value < maximum {
                                upper_state.write().set_integer_preference(key, upper);
                            }
                        })
                        .child(label().text("+")),
                ),
        )
        .into_element()
}

pub(super) fn theme_variant(
    state: State<SettingsViewState>,
    shell_state: State<super::super::super::ShellState>,
    label_text: &'static str,
    description: &'static str,
    theme_id: &'static str,
    active: bool,
) -> Element {
    let palette = state.read().effects().palette();
    let preview = theme::palette_for(theme_id);
    let mut state = state;
    let mut shell_state = shell_state;
    rect()
        .width(Size::fill())
        .height(Size::px(86.))
        .padding(Gaps::new_all(10.))
        .horizontal()
        .spacing(12.)
        .cross_align(Alignment::Center)
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .border(
            Border::new()
                .fill(theme::token_color(
                    palette,
                    if active {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::Border
                    },
                ))
                .width(1.),
        )
        .with_corner_radius(11.)
        .a11y_alt(format!("Use {label_text} theme"))
        .on_mouse_up(move |_| {
            state
                .write()
                .set_text_preference("theme", theme_id.to_owned());
            shell_state.write().mark_settings_changed();
        })
        .child(
            rect()
                .width(Size::px(84.))
                .height(Size::px(58.))
                .padding(Gaps::new_all(5.))
                .horizontal()
                .spacing(4.)
                .background(theme::token_color(preview, theme::ThemeToken::Bg))
                .border(
                    Border::new()
                        .fill(theme::token_color(preview, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(8.)
                .child(
                    rect()
                        .width(Size::px(22.))
                        .height(Size::fill())
                        .background(theme::token_color(preview, theme::ThemeToken::Sidebar))
                        .with_corner_radius(4.),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
                        .background(theme::token_color(preview, theme::ThemeToken::Surface))
                        .with_corner_radius(4.),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .spacing(4.)
                .child(
                    label()
                        .font_size(12.5)
                        .font_weight(FontWeight::BOLD)
                        .text(label_text),
                )
                .child(
                    label()
                        .font_size(10.5)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(description),
                ),
        )
        .into_element()
}

pub(super) fn navigation_visibility(
    state: State<SettingsViewState>,
    shell_state: State<super::super::super::ShellState>,
    label_text: &'static str,
    item_id: &'static str,
    hidden_ids: Vec<String>,
) -> Element {
    let palette = state.read().effects().palette();
    let hidden = hidden_ids.iter().any(|id| id == item_id);
    let visible = !hidden;
    let accessibility_label = if visible {
        format!("Hide {label_text} in navigation")
    } else {
        format!("Show {label_text} in navigation")
    };
    let mut state = state;
    let mut shell_state = shell_state;
    settings_row!(palette)
        .child(row_copy(
            label_text,
            "Show this action in the vertical icon bar.",
            palette,
        ))
        .child(
            rect()
                .width(Size::px(42.))
                .height(Size::px(24.))
                .padding(Gaps::new_all(2.))
                .horizontal()
                .main_align(if visible {
                    Alignment::End
                } else {
                    Alignment::Start
                })
                .cross_align(Alignment::Center)
                .background(theme::token_color(
                    palette,
                    if visible {
                        theme::ThemeToken::Primary
                    } else {
                        theme::ThemeToken::BorderStrong
                    },
                ))
                .with_corner_radius(99.)
                .a11y_alt(accessibility_label)
                .on_mouse_up(move |event: Event<MouseEventData>| {
                    event.stop_propagation();
                    state.write().toggle_rail_hidden(item_id);
                    shell_state.write().mark_settings_changed();
                })
                .child(
                    rect()
                        .width(Size::px(20.))
                        .height(Size::px(20.))
                        .background(theme::token_color(palette, theme::ThemeToken::Surface))
                        .with_corner_radius(99.),
                ),
        )
        .into_element()
}

pub(super) fn setting_row(entry: &SettingIndexEntry, palette: theme::ThemePalette) -> Element {
    settings_row!(palette)
        .a11y_alt(entry.label)
        .child(row_copy(entry.label, entry.description, palette))
        .into_element()
}
