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
    search_graph_contract::{ConceptCandidate, EvidenceChunk, SearchMatchType},
    theme,
};

pub(super) const SEARCH_BACKDROP_LAYER: u8 = 12;
pub(super) const SEARCH_MODAL_LAYER: u8 = 13;
pub(super) const SEARCH_PANEL_LAYER: u8 = 14;
pub(super) const SEARCH_CONTENT_LAYER: u8 = 15;
pub(super) const SEARCH_TEXT_LAYER: u8 = 16;
const SEARCH_RESULTS_CONTENT_HEIGHT: f32 = 560.;

/// The source search shell is a translucent glass surface. Keeping this fill
/// in the presentation module lets the explorer own only search state and
/// geometry while the renderer owns the visual contract.
pub(super) fn glass_surface(palette: theme::ThemePalette) -> impl Into<Fill> {
    let surface = palette.surface;
    let soft = palette.soft;
    LinearGradient::new()
        .angle(135.)
        .stop((Color::from_argb(248, surface.0, surface.1, surface.2), 0.))
        .stop((Color::from_argb(248, soft.0, soft.1, soft.2), 58.))
        .stop((Color::from_argb(248, surface.0, surface.1, surface.2), 100.))
}

pub(super) fn render(
    state: State<ExplorerState>,
    snapshot: &ExplorerState,
    _palette: theme::ThemePalette,
) -> Element {
    let view = rect()
        .width(Size::fill())
        .height(Size::px(SEARCH_RESULTS_CONTENT_HEIGHT))
        .a11y_alt("Searching locally…");
    let selected_index = snapshot.search.selected_index.unwrap_or(0);
    let mut sections = Vec::new();
    if !snapshot.search.concepts.is_empty() {
        sections.push(section_title("WIKIS & CONCEPTS").into_element());
        sections.extend(
            snapshot
                .search
                .concepts
                .iter()
                .map(|concept| concept_card(state, concept)),
        );
    }
    if !snapshot.search.results.is_empty() {
        sections.push(section_title("NOTES & PASSAGES").into_element());
        sections.extend(
            snapshot
                .search
                .results
                .iter()
                .enumerate()
                .map(|(index, result)| {
                    result_card(
                        state,
                        index,
                        result,
                        index == selected_index,
                        &snapshot.search.query,
                    )
                }),
        );
    }

    let results_scroll = rect()
        .width(Size::fill())
        .height(Size::px(SEARCH_RESULTS_CONTENT_HEIGHT))
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .show_scrollbar(true)
                .drag_scrolling(false)
                .child(
                    rect()
                        .width(Size::fill())
                        .vertical()
                        .spacing(4.)
                        .padding(Gaps::new(4., 12., 12., 12.))
                        .children(sections),
                ),
        );
    view.child(results_scroll).into_element()
}

fn concept_card(mut state: State<ExplorerState>, concept: &ConceptCandidate) -> Element {
    let concept = concept.clone();
    let evidence_count = concept.evidence_chunks.len();
    let source = concept
        .evidence_chunks
        .first()
        .map(concept_source_label)
        .unwrap_or_else(|| "source chunk".to_owned());
    let score = format!("{}%", (concept.score.clamp(0., 1.) * 100.).round() as u8);
    let title = concept.title.clone();
    let meta = format!(
        "{} source chunk{} · {source}",
        evidence_count,
        if evidence_count == 1 { "" } else { "s" }
    );
    rect()
        .width(Size::fill())
        .height(Size::px(58.))
        .padding(Gaps::new(10., 12., 10., 12.))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(12.)
        .background(Color::from_argb(18, 37, 99, 235))
        .border(
            Border::new()
                .fill(Color::from_argb(46, 37, 99, 235))
                .width(1.),
        )
        .with_corner_radius(14.)
        .layer(Layer::OverlayLevel(SEARCH_CONTENT_LAYER))
        .on_mouse_up(move |_| state.write().open_concept(&concept))
        .a11y_alt(format!("Open concept {title}"))
        .child(
            rect()
                .width(Size::flex(1.))
                .height(Size::fill())
                .vertical()
                .spacing(2.)
                .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
                .child(
                    label()
                        .width(Size::fill())
                        .font_size(14.)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(16, 24, 40))
                        .text(title),
                )
                .child(
                    label()
                        .width(Size::fill())
                        .font_size(12.)
                        .color(Color::from_rgb(100, 116, 139))
                        .text(meta),
                ),
        )
        .child(
            rect()
                .width(Size::px(48.))
                .height(Size::px(24.))
                .center()
                .child(
                    label()
                        .font_size(12.)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(37, 99, 235))
                        .text(score),
                ),
        )
        .into_element()
}

fn result_card(
    mut state: State<ExplorerState>,
    index: usize,
    result: &crate::search_graph_contract::SearchResult,
    selected: bool,
    query: &str,
) -> Element {
    let result = result.clone();
    let title = result.title.clone();
    let excerpt = result
        .snippets
        .first()
        .map(|snippet| snippet.text.clone())
        .filter(|text| !text.trim().is_empty())
        .or_else(|| (!result.excerpt.trim().is_empty()).then(|| result.excerpt.clone()));
    let title_line = rect()
        .width(Size::flex(1.))
        .height(Size::px(22.))
        .child(highlighted_title(&title, query));
    let match_badge = rect()
        .width(Size::px(match result.match_type {
            SearchMatchType::Hybrid => 112.,
            SearchMatchType::Semantic => 72.,
            SearchMatchType::Keyword => 68.,
            SearchMatchType::Concept => 68.,
            SearchMatchType::Unknown => 78.,
        }))
        .height(Size::px(22.))
        .center()
        .background(Color::from_argb(61, 37, 99, 235))
        .with_corner_radius(999.)
        .child(
            label()
                .font_size(10.)
                .font_weight(FontWeight::BOLD)
                .color(Color::from_rgb(37, 99, 235))
                .text(match_label(result.match_type)),
        );
    let headline = rect()
        .width(Size::fill())
        .height(Size::px(22.))
        .horizontal()
        .cross_align(Alignment::Center)
        .spacing(8.)
        .child(title_line)
        .child(match_badge);
    let mut body = rect()
        .width(Size::flex(1.))
        .height(Size::fill())
        .vertical()
        .spacing(2.)
        .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
        .child(headline)
        .child(
            label()
                .width(Size::fill())
                .font_size(12.)
                .font_weight(FontWeight::BOLD)
                .color(Color::from_rgb(37, 99, 235))
                .text(result.relative_path.clone()),
        );
    if let Some(excerpt) = excerpt {
        body = body.child(
            label()
                .width(Size::fill())
                .font_size(11.)
                .color(Color::from_rgb(100, 116, 139))
                .text(excerpt),
        );
    }
    let selected_marker = selected.then(|| {
        rect()
            .width(Size::px(56.))
            .height(Size::px(22.))
            .center()
            .background(Color::from_argb(61, 37, 99, 235))
            .with_corner_radius(999.)
            .a11y_alt("Selected result")
            .child(
                label()
                    .font_size(10.)
                    .font_weight(FontWeight::BOLD)
                    .color(Color::from_rgb(37, 99, 235))
                    .text("Selected"),
            )
            .into_element()
    });
    rect()
        .width(Size::fill())
        .height(Size::px(84.))
        .padding(Gaps::new(12., 14., 12., 14.))
        .horizontal()
        .spacing(12.)
        .background(if selected {
            Color::from_rgb(215, 225, 249)
        } else {
            Color::from_rgb(246, 248, 252)
        })
        .with_corner_radius(16.)
        .layer(Layer::OverlayLevel(SEARCH_CONTENT_LAYER))
        .a11y_alt(format!("Open note {title}"))
        .on_mouse_up(move |_| state.write().open_search_result(index))
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
        .child(body)
        .maybe_child(selected_marker)
        .child(
            rect()
                .width(Size::px(30.))
                .height(Size::px(30.))
                .center()
                .a11y_alt("Open note")
                .child(
                    label()
                        .font_size(18.)
                        .color(Color::from_rgb(100, 116, 139))
                        .text("↗"),
                ),
        )
        .into_element()
}

fn section_title(title: &'static str) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(22.))
        .padding(Gaps::new(2., 8., 4., 8.))
        .a11y_alt(title)
        .layer(Layer::OverlayLevel(SEARCH_TEXT_LAYER))
        .child(
            label()
                .font_size(11.)
                .font_weight(FontWeight::BOLD)
                .color(Color::from_rgb(71, 84, 103))
                .text(title),
        )
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

fn concept_source_label(chunk: &EvidenceChunk) -> String {
    if chunk.heading_path.is_empty() {
        "source chunk".to_owned()
    } else {
        chunk.heading_path.join(" › ")
    }
}

#[cfg(test)]
mod tests {
    use super::concept_source_label;
    use crate::search_graph_contract::EvidenceChunk;

    fn chunk(heading_path: Vec<&str>) -> EvidenceChunk {
        EvidenceChunk {
            id: "alpha:0".to_owned(),
            document_path: "Alpha.md".to_owned(),
            relative_path: "Alpha.md".to_owned(),
            chunk_index: 0,
            heading_path: heading_path.into_iter().map(str::to_owned).collect(),
            score: 1.,
            preview: "preview".to_owned(),
        }
    }

    #[test]
    fn concept_source_label_matches_the_default_search_contract() {
        assert_eq!(concept_source_label(&chunk(Vec::new())), "source chunk");
        assert_eq!(
            concept_source_label(&chunk(vec!["Projects", "Migration"])),
            "Projects › Migration"
        );
    }
}
