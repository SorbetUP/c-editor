use freya::prelude::*;

use super::drawing_scene::{DrawingCanvasState, DrawingElement, Viewport};

mod drawing_scene {
    pub(super) use super::super::drawing_scene::{DrawingElement, Viewport};
}

#[path = "drawing_render_svg.rs"]
mod svg;

pub fn render(state: &DrawingCanvasState) -> Vec<Element> {
    let mut output = Vec::new();
    for (index, element) in state.document.elements.iter().enumerate() {
        if element.is_deleted {
            continue;
        }
        render_element(
            &mut output,
            element,
            index,
            state.viewport,
            state.is_element_selected(&element.id),
            &state.document.files,
        );
    }

    if state.selected_element_ids().len() > 1 {
        if let Some(bounds) = state.selection_bounds() {
            output.push(selection_overlay(
                bounds,
                state.viewport,
                "Drawing group selection",
                10,
            ));
        }
    }
    if let Some(bounds) = state.selection_marquee_world() {
        output.push(selection_overlay(
            bounds,
            state.viewport,
            "Drawing selection marquee",
            24,
        ));
    }
    output
}

fn render_element(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    selected: bool,
    files: &serde_json::Value,
) {
    let label = format!("Drawing element {} {}", element.id, element.kind);
    match element.kind.as_str() {
        "rectangle" | "frame" | "magicframe" | "embeddable" | "iframe" => {
            output.push(svg::shape(element, index, &label, viewport, false));
        }
        "ellipse" => output.push(svg::shape(element, index, &label, viewport, true)),
        "diamond" => output.push(svg::diamond(element, index, &label, viewport)),
        "line" | "arrow" | "freedraw" => {
            output.push(svg::polyline(element, index, &label, viewport));
        }
        "text" => output.push(svg::text(element, index, &label, viewport)),
        "image" => output.push(svg::image(element, index, &label, viewport, files)),
        _ => {}
    }
    if selected {
        output.push(svg::selection(index, element, viewport));
    }
}

fn selection_overlay(
    bounds: (f32, f32, f32, f32),
    viewport: Viewport,
    label: &'static str,
    alpha: u8,
) -> Element {
    let (x, y, width, height) = bounds;
    rect()
        .position(
            Position::new_absolute()
                .left(viewport.pan[0] + x * viewport.zoom)
                .top(viewport.pan[1] + y * viewport.zoom),
        )
        .width(Size::px(width.max(1.0) * viewport.zoom))
        .height(Size::px(height.max(1.0) * viewport.zoom))
        .background(Color::from_argb(alpha, 105, 101, 219))
        .a11y_alt(label)
        .into_element()
}
