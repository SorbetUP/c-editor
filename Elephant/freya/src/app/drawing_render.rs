use super::drawing_scene::{rgba, DrawingCanvasState, DrawingElement, Viewport};
use freya::prelude::*;

pub fn render(state: &DrawingCanvasState) -> Vec<Element> {
    let mut elements = Vec::new();
    for (index, element) in state.document.elements.iter().enumerate() {
        if element.is_deleted {
            continue;
        }
        let selected = state.selected_element_id() == Some(element.id.as_str());
        render_element(&mut elements, element, index, state.viewport, selected);
    }
    elements
}

fn render_element(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    selected: bool,
) {
    let label = format!("Drawing element {} {}", element.id, element.kind);
    let (x, y, width, height) = element.bounds();
    let stroke = color(&element.stroke_color, element.opacity);
    let fill = color(&element.background_color, element.opacity);
    match element.kind.as_str() {
        "rectangle" => output.push(shape_rect(
            element, index, label, x, y, width, height, viewport, stroke, fill, false,
        )),
        "ellipse" => output.push(shape_rect(
            element, index, label, x, y, width, height, viewport, stroke, fill, true,
        )),
        "diamond" => render_diamond(output, element, index, viewport, stroke, fill, &label),
        "line" | "arrow" => render_polyline(output, element, index, viewport, stroke, &label),
        "freedraw" => render_polyline(output, element, index, viewport, stroke, &label),
        "text" => output.push(text_element(element, index, viewport, stroke, &label)),
        _ => {}
    }
    if selected {
        output.push(selection_rect(index, x, y, width, height, viewport));
    }
}

fn shape_rect(
    element: &DrawingElement,
    index: usize,
    label: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
    stroke: Color,
    fill: Color,
    ellipse: bool,
) -> Element {
    let radius = if ellipse { (width.min(height) / 2.).max(2.) * viewport.zoom } else { 0. };
    rect()
        .key(("drawing-element", index))
        .position(absolute(viewport, x, y))
        .width(Size::px(width.max(1.) * viewport.zoom))
        .height(Size::px(height.max(1.) * viewport.zoom))
        .background(fill)
        .border(
            Border::new()
                .fill(stroke)
                .width(element.stroke_width.max(0.5) * viewport.zoom),
        )
        .with_corner_radius(radius)
        .a11y_alt(label)
        .into_element()
}

fn render_diamond(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    stroke: Color,
    _fill: Color,
    label: &str,
) {
    let (x, y, width, height) = element.bounds();
    let cx = width / 2.;
    let cy = height / 2.;
    let pts = [
        [x + cx, y],
        [x + width, y + cy],
        [x + cx, y + height],
        [x, y + cy],
        [x + cx, y],
    ];
    for (seg_idx, seg) in pts.windows(2).enumerate() {
        let start = seg[0];
        let end = seg[1];
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let length = (dx * dx + dy * dy).sqrt().max(1.);
        let angle = dy.atan2(dx).to_degrees();
        output.push(
            rect()
                .key(("drawing-diamond-seg", index, seg_idx))
                .position(absolute(viewport, start[0], start[1]))
                .width(Size::px(length * viewport.zoom))
                .height(Size::px(element.stroke_width.max(1.) * viewport.zoom))
                .background(stroke)
                .with_corner_radius(element.stroke_width.max(1.) * viewport.zoom / 2.)
                .rotation(angle)
                .a11y_alt(format!("{label} diamond segment {seg_idx}"))
                .into_element(),
        );
    }
}

fn render_polyline(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    stroke: Color,
    label: &str,
) {
    let points = points(element);
    for (segment_index, segment) in points.windows(2).enumerate() {
        let start = segment[0];
        let end = segment[1];
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let length = (dx * dx + dy * dy).sqrt().max(1.);
        let angle = dy.atan2(dx).to_degrees();
        let line = rect()
            .key(("drawing-segment", index, segment_index))
            .position(absolute(viewport, start[0], start[1]))
            .width(Size::px(length * viewport.zoom))
            .height(Size::px(element.stroke_width.max(1.) * viewport.zoom))
            .background(stroke)
            .with_corner_radius(element.stroke_width.max(1.) * viewport.zoom / 2.)
            .rotation(angle)
            .a11y_alt(format!("{label} segment {segment_index}"));
        output.push(line.into_element());
    }
    if element.kind == "arrow" && element.end_arrowhead.as_deref() != Some("none") {
        if let Some([previous, end]) = points
            .windows(2)
            .last()
            .map(|window| [window[0], window[1]])
        {
            let dx = end[0] - previous[0];
            let dy = end[1] - previous[1];
            let angle = dy.atan2(dx).to_degrees();
            for (branch, rotation) in [-150., 150.].into_iter().enumerate() {
                output.push(
                    rect()
                        .key(("drawing-arrowhead", index, branch))
                        .position(absolute(viewport, end[0], end[1]))
                        .width(Size::px(12. * viewport.zoom))
                        .height(Size::px(element.stroke_width.max(1.) * viewport.zoom))
                        .background(stroke)
                        .rotation(angle + rotation)
                        .a11y_alt(format!("{label} arrowhead {branch}"))
                        .into_element(),
                );
            }
        }
    }
}

fn text_element(
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    stroke: Color,
    alt: &str,
) -> Element {
    label()
        .key(("drawing-text", index))
        .position(absolute(viewport, element.x, element.y))
        .width(Size::px(element.width.max(1.) * viewport.zoom))
        .height(Size::px(
            element.height.max(element.font_size).max(1.) * viewport.zoom,
        ))
        .font_size(if element.font_size > 0. {
            element.font_size * viewport.zoom
        } else {
            20. * viewport.zoom
        })
        .color(stroke)
        .a11y_alt(alt)
        .text(element.text.clone())
        .into_element()
}

fn selection_rect(
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
) -> Element {
    rect()
        .key(("drawing-selection", index))
        .position(absolute(viewport, x - 4., y - 4.))
        .width(Size::px((width + 8.).max(8.) * viewport.zoom))
        .height(Size::px((height + 8.).max(8.) * viewport.zoom))
        .border(Border::new().fill(Color::from_rgb(105, 101, 219)).width(1.))
        .a11y_alt(format!("Drawing selection {index}"))
        .into_element()
}

fn points(element: &DrawingElement) -> Vec<[f32; 2]> {
    if element.points.is_empty() {
        return vec![
            [element.x, element.y],
            [element.x + element.width, element.y + element.height],
        ];
    }
    element
        .points
        .iter()
        .map(|point| [element.x + point[0], element.y + point[1]])
        .collect()
}

fn absolute(viewport: Viewport, x: f32, y: f32) -> Position {
    Position::new_absolute()
        .left(viewport.pan[0] + x * viewport.zoom)
        .top(viewport.pan[1] + y * viewport.zoom)
}

fn color(value: &str, opacity: f32) -> Color {
    let [r, g, b, a] = rgba(value, opacity);
    Color::from_argb(a, r, g, b)
}
