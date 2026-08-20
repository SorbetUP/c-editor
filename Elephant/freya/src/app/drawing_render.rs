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
            state.selected_element_id() == Some(element.id.as_str()),
            &state.document.files,
        );
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
