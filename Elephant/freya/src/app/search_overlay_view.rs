//! Presentation-only renderer for the workspace search overlay.
//!
//! Search state and commands live in `explorer.rs`. This module owns only the
//! Freya tree, visual tokens, and callbacks supplied by the state owner.

use freya::prelude::*;

use crate::{
    app::{
        explorer::ExplorerState,
        navigation_icons::{svg_icon, Icon},
    },
    search_graph_contract::SearchMatchType,
};

pub(super) const SEARCH_BACKDROP_LAYER: u8 = 12;
pub(super) const SEARCH_MODAL_LAYER: u8 = 13;
pub(super) const SEARCH_PANEL_LAYER: u8 = 14;
pub(super) const SEARCH_CONTENT_LAYER: u8 = 15;
pub(super) const SEARCH_TEXT_LAYER: u8 = 16;

/// The source search shell is a translucent glass surface. Keeping this fill
/// in the presentation module lets the explorer own only search state and
/// geometry while the renderer owns the visual contract.
pub(super) fn glass_surface() -> impl Into<Fill> {
    LinearGradient::new()
        .angle(135.)
        .stop((Color::from_argb(122, 255, 255, 255), 0.))
        .stop((Color::from_argb(87, 225, 240, 255), 58.))
        .stop((Color::from_argb(102, 255, 255, 255), 100.))
}

pub(super) fn render(state: State<ExplorerState>, snapshot: &ExplorerState) -> Element {
    let mut view = rect()
        .width(Size::fill())
        .height(Size::fill())
        .a11y_alt("Searching locally…");
    if let Some(concept) = snapshot.search.concepts.first() {
        let concept = concept.clone();
        let evidence_count = concept.evidence_chunks.len();
        let source = concept
            .evidence_chunks
            .first()
            .map(|chunk| {
                if chunk.heading_path.is_empty() {
                    if chunk.relative_path.is_empty() {
                        chunk.document_path.clone()
                    } else {
                        chunk.relative_path.clone()
                    }
                } else {
                    chunk.heading_path.join(" › ")
                }
            })
            .unwrap_or_else(|| "source chunk".to_owned());
        let score = format!("{}%", (concept.score.clamp(0., 1.) * 100.).round() as u8);
        let mut concept_state = state;
        let concept_for_action = concept.clone();
        view = view
            .child(section_title("WIKIS & CONCEPTS", 7.))
            .child(
                rect()
                    .position(Position::new_absolute().left(18.).top(28.))
                    .width(Size::px(654.))
                    .height(Size::px(58.))
                    .padding(Gaps::new(10., 12., 10., 12.))
                    .horizontal()
                    .main_align(Alignment::SpaceBetween)
                    .background(Color::from_argb(18, 37, 99, 235))
                    .border(
                        Border::new()
                            .fill(Color::from_argb(46, 37, 99, 235))
                            .width(1.),
                    )
                    .with_corner_radius(14.)
                    .layer(Layer::OverlayLevel(SEARCH_CONTENT_LAYER))
                    .on_mouse_up(move |_| concept_state.write().open_concept(&concept_for_action))
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .height(Size::fill())
                            .vertical()
                            .spacing(2.)
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .child(
                                label()
                                    .font_size(14.)
                                    .font_weight(FontWeight::BOLD)
                                    .color(Color::from_rgb(16, 24, 40))
                                    .text(concept.title.clone()),
                            )
                            .child(
                                label()
                                    .font_size(12.)
                                    .color(Color::from_rgb(100, 116, 139))
                                    .text(format!(
                                        "{} source chunk{} · {source}",
                                        evidence_count,
                                        if evidence_count == 1 { "" } else { "s" }
                                    )),
                            ),
                    )
                    .child(
                        label()
                            .font_size(12.)
                            .font_weight(FontWeight::BOLD)
                            .color(Color::from_rgb(37, 99, 235))
                            .width(Size::px(36.))
                            .height(Size::px(20.))
                            .position(Position::new_absolute().right(14.).top(10.))
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .text(score),
                    ),
            );
    }
    if let Some(result) = snapshot.search.results.first() {
        let result = result.clone();
        let mut result_state = state;
        let result_title = highlighted_title(&result.title, &snapshot.search.query);
        view = view
            .child(section_title("NOTES & PASSAGES", 94.))
            .child(
                rect()
                    .position(Position::new_absolute().left(18.).top(120.))
                    .width(Size::px(654.))
                    .height(Size::px(70.))
                    .padding(Gaps::new(12., 14., 12., 14.))
                    .background(Color::from_argb(51, 37, 99, 235))
                    .with_corner_radius(16.)
                    .layer(Layer::OverlayLevel(SEARCH_CONTENT_LAYER))
                    .a11y_alt(format!("Open note {}", result.title))
                    .on_mouse_up(move |_| result_state.write().open_search_result(0))
                    .horizontal()
                    .spacing(12.)
                    .child(
                        rect()
                            .width(Size::px(38.))
                            .height(Size::px(38.))
                            .center()
                            .background(Color::from_argb(87, 255, 255, 255))
                            .with_corner_radius(14.)
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .child(svg_icon(Icon::FileText, Color::from_rgb(37, 99, 235), 18.)),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .height(Size::fill())
                            .spacing(4.)
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .child(result_title)
                            .child(
                                label()
                                    .font_size(12.)
                                    .font_weight(FontWeight::BOLD)
                                    .color(Color::from_rgb(37, 99, 235))
                                    .text(result.relative_path.clone()),
                            ),
                    )
                    .child(
                        rect()
                            .position(Position::new_absolute().right(50.).top(12.))
                            .width(Size::px(86.))
                            .height(Size::px(22.))
                            .padding(Gaps::new(0., 9., 0., 9.))
                            .center()
                            .background(Color::from_argb(61, 37, 99, 235))
                            .with_corner_radius(999.)
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .child(
                                label()
                                    .font_size(11.)
                                    .font_weight(FontWeight::BOLD)
                                    .color(Color::from_rgb(37, 99, 235))
                                    .text(match_label(result.match_type)),
                            ),
                    )
                    .child(
                        rect()
                            .position(Position::new_absolute().right(14.).top(20.))
                            .width(Size::px(30.))
                            .height(Size::px(30.))
                            .center()
                            .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                            .child(
                                label()
                                    .font_size(18.)
                                    .color(Color::from_rgb(100, 116, 139))
                                    .text("↗"),
                            ),
                    ),
            );
    }
    view.into_element()
}

fn section_title(title: &'static str, top: f32) -> Element {
    label()
        .position(Position::new_absolute().left(26.).top(top))
        .font_size(11.)
        .font_weight(FontWeight::BOLD)
        .color(Color::from_rgb(71, 84, 103))
        .a11y_alt(title)
        .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
        .text(title)
        .into_element()
}

fn highlighted_title(title: &str, query: &str) -> Element {
    let token = query
        .split_whitespace()
        .find(|part| part.chars().count() >= 2)
        .unwrap_or_default();
    if token.is_empty() {
        return plain_title(title);
    }
    let Some(start) = title.to_lowercase().find(&token.to_lowercase()) else {
        return plain_title(title);
    };
    let end = start + token.len();
    let mut row = rect().height(Size::px(20.)).horizontal();
    if start > 0 {
        row = row.child(plain_title(&title[..start]));
    }
    row = row.child(
        rect()
            .padding(Gaps::new(0., 2., 0., 2.))
            .with_corner_radius(4.)
            .background(Color::from_argb(66, 37, 99, 235))
            .child(
                label()
                    .font_size(15.)
                    .font_weight(FontWeight::BOLD)
                    .color(Color::from_rgb(37, 99, 235))
                    .text(title[start..end].to_owned()),
            ),
    );
    if end < title.len() {
        row = row.child(plain_title(&title[end..]));
    }
    row.into_element()
}

fn plain_title(title: &str) -> Element {
    label()
        .font_size(15.)
        .font_weight(FontWeight::BOLD)
        .color(Color::from_rgb(16, 24, 40))
        .text(title.to_owned())
        .into_element()
}

fn match_label(kind: SearchMatchType) -> &'static str {
    match kind {
        SearchMatchType::Hybrid => "Semantic + keyword",
        SearchMatchType::Semantic => "Semantic",
        SearchMatchType::Keyword => "Keyword",
        SearchMatchType::Concept => "Concept",
        SearchMatchType::Unknown => "Local match",
    }
}
