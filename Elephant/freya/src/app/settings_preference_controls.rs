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
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text(title))
                .child(label().color(theme::color(theme::MUTED)).text(description)),
        )
        .child(
            rect()
                .height(Size::px(30.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .center()
                .with_corner_radius(15.)
                .background(theme::color(if active {
                    theme::PRIMARY
                } else {
                    theme::SURFACE
                }))
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
    let value = State::create(current);
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text(title))
                .child(label().color(theme::color(theme::MUTED)).text(description)),
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
    let mut state = state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .child(
            rect()
                .spacing(3.)
                .child(label().font_weight(FontWeight::BOLD).text("Autosave delay"))
                .child(
                    label()
                        .color(theme::color(theme::MUTED))
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

pub(super) fn setting_row(entry: &SettingIndexEntry) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::color(theme::BG))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .spacing(4.)
        .a11y_alt(entry.label)
        .child(label().font_weight(FontWeight::BOLD).text(entry.label))
        .child(
            label()
                .color(theme::color(theme::MUTED))
                .text(entry.description),
        )
        .into_element()
}
