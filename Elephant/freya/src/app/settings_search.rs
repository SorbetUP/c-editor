//! Search results for the source settings index.

use freya::prelude::*;

use crate::{settings_contract::search_core_settings, theme};

use super::super::SettingsViewState;

pub(super) fn search_results_content(state: State<SettingsViewState>, query: &str) -> Element {
    let results = search_core_settings(query);
    let count = results.len();
    let body = if results.is_empty() {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(24.))
            .a11y_alt("No setting found")
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text("No setting found"),
            )
            .child(
                label()
                    .color(theme::color(theme::MUTED))
                    .text("Try another word, feature name or control."),
            )
            .into_element()
    } else {
        rect()
            .width(Size::fill())
            .spacing(8.)
            .children(results.into_iter().map(|entry| {
                let section = entry.section.to_owned();
                let label_text = entry.label.to_owned();
                let mut result_state = state;
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(10.))
                    .background(theme::color(theme::BG))
                    .with_corner_radius(8.)
                    .a11y_alt(format!("Open setting {label_text}"))
                    .on_mouse_up(move |_| {
                        result_state.write().settings.open_search_result(&section);
                    })
                    .child(label().font_weight(FontWeight::BOLD).text(label_text))
                    .child(
                        label()
                            .color(theme::color(theme::MUTED))
                            .text(entry.description),
                    )
                    .into_element()
            }))
            .into_element()
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new_all(14.))
        .background(theme::color(theme::SURFACE))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(10.)
        .spacing(12.)
        .a11y_alt("Settings search results")
        .child(
            label()
                .font_size(18.)
                .font_weight(FontWeight::BOLD)
                .text("Search"),
        )
        .child(label().color(theme::color(theme::MUTED)).text(format!(
            "{count} result{}",
            if count == 1 { "" } else { "s" }
        )))
        .child(body)
        .into_element()
}
