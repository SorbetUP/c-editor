//! Search results for the source settings index.

use freya::prelude::*;

use crate::{settings_contract::search_core_settings, theme};

use super::super::SettingsViewState;

pub(super) fn search_results_content(
    state: State<SettingsViewState>,
    search_query: State<String>,
    query: &str,
) -> Element {
    let palette = state.read().effects().palette();
    let results = search_core_settings(query);
    let count = results.len();

    let body = if results.is_empty() {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(24.))
            .background(theme::token_color(palette, theme::ThemeToken::Surface))
            .border(
                Border::new()
                    .fill(theme::token_color(palette, theme::ThemeToken::Border))
                    .width(1.),
            )
            .with_corner_radius(14.)
            .spacing(5.)
            .a11y_alt("No setting found")
            .child(
                label()
                    .font_size(13.)
                    .font_weight(FontWeight::BOLD)
                    .text("No setting found"),
            )
            .child(
                label()
                    .font_size(11.5)
                    .color(theme::token_color(palette, theme::ThemeToken::Muted))
                    .text("Try another word, feature name or control."),
            )
            .into_element()
    } else {
        let mut group = rect()
            .width(Size::fill())
            .background(theme::token_color(palette, theme::ThemeToken::Surface))
            .border(
                Border::new()
                    .fill(theme::token_color(palette, theme::ThemeToken::Border))
                    .width(1.),
            )
            .with_corner_radius(14.);

        for (index, entry) in results.into_iter().enumerate() {
            if index > 0 {
                group = group.child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(1.))
                        .background(theme::token_color(palette, theme::ThemeToken::Border)),
                );
            }
            let section = entry.section.to_owned();
            let label_text = entry.label.to_owned();
            let section_label = entry
                .section
                .chars()
                .next()
                .map(|first| {
                    first.to_uppercase().collect::<String>() + &entry.section[first.len_utf8()..]
                })
                .unwrap_or_default();
            let mut result_state = state;
            let mut query_state = search_query;
            group = group.child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(68.))
                    .padding(Gaps::new(12., 16., 12., 16.))
                    .horizontal()
                    .spacing(12.)
                    .cross_align(Alignment::Center)
                    .a11y_alt(format!("Open setting {label_text}"))
                    .on_mouse_up(move |_| {
                        *query_state.write() = String::new();
                        result_state.write().settings.open_search_result(&section);
                    })
                    .child(
                        rect()
                            .width(Size::fill())
                            .spacing(3.)
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
                                    .text(entry.description),
                            ),
                    )
                    .child(
                        label()
                            .font_size(9.5)
                            .color(theme::token_color(palette, theme::ThemeToken::Muted))
                            .text(section_label),
                    )
                    .child(
                        label()
                            .font_size(15.)
                            .color(theme::token_color(palette, theme::ThemeToken::Muted))
                            .text("›"),
                    ),
            );
        }
        group.into_element()
    };

    let content = rect()
        .width(Size::fill())
        .padding(Gaps::new(28., 34., 46., 34.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .spacing(16.)
        .child(
            rect()
                .height(Size::px(38.))
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .font_size(24.)
                        .font_weight(FontWeight::BOLD)
                        .text("Search"),
                )
                .child(
                    label()
                        .font_size(10.5)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(format!(
                            "{count} result{}",
                            if count == 1 { "" } else { "s" }
                        )),
                ),
        )
        .child(body);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .a11y_alt("Settings search results")
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .show_scrollbar(true)
                .child(content),
        )
        .into_element()
}
