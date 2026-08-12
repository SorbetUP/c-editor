use super::drawing_scene::{rgba, DrawingCanvasState, DrawingElement, Viewport};
use freya::prelude::*;

fn excalidraw_purple() -> Color {
    Color::from_rgb(105, 101, 219)
}

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
        "line" | "arrow" | "freedraw" => {
            render_polyline(output, element, index, viewport, stroke, &label)
        }
        "text" => output.push(text_element(element, index, viewport, stroke, &label)),
        _ => {}
    }
    if selected {
        output.push(selection_rect(
            index,
            x,
            y,
            width,
            height,
            element.angle,
            viewport,
        ));
        render_selection_handles(output, index, x, y, width, height, element.angle, viewport);
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
    let radius = if ellipse {
        width.max(height) * viewport.zoom
    } else {
        0.
    };
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
        .rotation(element.angle.to_degrees())
        .a11y_alt(label)
        .into_element()
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
        render_styled_segment(
            output,
            element,
            index,
            segment_index,
            segment[0],
            segment[1],
            viewport,
            stroke,
            label,
        );
    }

    if element.kind != "arrow" || points.len() < 2 {
        return;
    }
    if element
        .start_arrowhead
        .as_deref()
        .is_some_and(|value| value != "none")
    {
        render_arrowhead(
            output,
            index,
            "start",
            points[0],
            points[1],
            element.stroke_width,
            viewport,
            stroke,
            label,
        );
    }
    if element.end_arrowhead.as_deref() != Some("none") {
        let end = points.len() - 1;
        render_arrowhead(
            output,
            index,
            "end",
            points[end],
            points[end - 1],
            element.stroke_width,
            viewport,
            stroke,
            label,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_styled_segment(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    segment_index: usize,
    start: [f32; 2],
    end: [f32; 2],
    viewport: Viewport,
    stroke: Color,
    label: &str,
) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let unit = [dx / length, dy / length];
    let stroke_width = element.stroke_width.max(1.);
    let (dash, gap) = match element.stroke_style.as_str() {
        "dashed" => (stroke_width * 4.5, stroke_width * 3.),
        "dotted" => (stroke_width.max(1.5), stroke_width * 2.5),
        _ => {
            output.push(segment_element(
                format!("drawing-segment-{index}-{segment_index}"),
                start,
                end,
                stroke_width,
                viewport,
                stroke,
                format!("{label} segment {segment_index}"),
            ));
            return;
        }
    };

    let mut offset = 0.;
    let mut piece = 0usize;
    while offset < length {
        let piece_end = (offset + dash).min(length);
        let piece_start_point = [start[0] + unit[0] * offset, start[1] + unit[1] * offset];
        let piece_end_point = [
            start[0] + unit[0] * piece_end,
            start[1] + unit[1] * piece_end,
        ];
        output.push(segment_element(
            format!("drawing-segment-{index}-{segment_index}-{piece}"),
            piece_start_point,
            piece_end_point,
            stroke_width,
            viewport,
            stroke,
            format!("{label} segment {segment_index} piece {piece}"),
        ));
        offset += dash + gap;
        piece += 1;
    }
}

fn render_arrowhead(
    output: &mut Vec<Element>,
    index: usize,
    side: &'static str,
    tip: [f32; 2],
    neighbor: [f32; 2],
    stroke_width: f32,
    viewport: Viewport,
    stroke: Color,
    label: &str,
) {
    let base_angle = (tip[1] - neighbor[1]).atan2(tip[0] - neighbor[0]);
    let length = 12.;
    for (branch, branch_angle) in [
        base_angle + 150_f32.to_radians(),
        base_angle - 150_f32.to_radians(),
    ]
    .into_iter()
    .enumerate()
    {
        let branch_end = [
            tip[0] + length * branch_angle.cos(),
            tip[1] + length * branch_angle.sin(),
        ];
        output.push(segment_element(
            format!("drawing-arrowhead-{index}-{side}-{branch}"),
            tip,
            branch_end,
            stroke_width.max(1.),
            viewport,
            stroke,
            format!("{label} {side} arrowhead {branch}"),
        ));
    }
}

fn segment_element(
    key: String,
    start: [f32; 2],
    end: [f32; 2],
    stroke_width: f32,
    viewport: Viewport,
    stroke: Color,
    alt: String,
) -> Element {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt().max(0.001);
    let angle = dy.atan2(dx).to_degrees();
    let midpoint = [(start[0] + end[0]) / 2., (start[1] + end[1]) / 2.];
    let thickness = stroke_width.max(1.);
    rect()
        .key(key)
        .position(absolute(
            viewport,
            midpoint[0] - length / 2.,
            midpoint[1] - thickness / 2.,
        ))
        .width(Size::px(length * viewport.zoom))
        .height(Size::px(thickness * viewport.zoom))
        .background(stroke)
        .with_corner_radius(thickness * viewport.zoom / 2.)
        .rotation(angle)
        .a11y_alt(alt)
        .into_element()
}

fn text_element(
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    stroke: Color,
    alt: &str,
) -> Element {
    let width = element.width.max(1.) * viewport.zoom;
    let height = element.height.max(element.font_size).max(1.) * viewport.zoom;
    let font_size = if element.font_size > 0. {
        element.font_size * viewport.zoom
    } else {
        20. * viewport.zoom
    };
    rect()
        .key(("drawing-text-container", index))
        .position(absolute(viewport, element.x, element.y))
        .width(Size::px(width))
        .height(Size::px(height))
        .rotation(element.angle.to_degrees())
        .child(
            label()
                .width(Size::fill())
                .height(Size::fill())
                .font_size(font_size)
                .color(stroke)
                .a11y_alt(alt)
                .text(element.text.clone()),
        )
        .into_element()
}

fn selection_rect(
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    angle: f32,
    viewport: Viewport,
) -> Element {
    rect()
        .key(("drawing-selection", index))
        .position(absolute(viewport, x - 4., y - 4.))
        .width(Size::px((width + 8.).max(8.) * viewport.zoom))
        .height(Size::px((height + 8.).max(8.) * viewport.zoom))
        .border(Border::new().fill(excalidraw_purple()).width(1.))
        .rotation(angle.to_degrees())
        .a11y_alt(format!("Drawing selection {index}"))
        .into_element()
}

fn render_selection_handles(
    output: &mut Vec<Element>,
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    angle: f32,
    viewport: Viewport,
) {
    let size = (8. / viewport.zoom).clamp(3., 12.);
    let center = [x + width / 2., y + height / 2.];
    for (handle, point) in [
        [x, y],
        [x + width, y],
        [x, y + height],
        [x + width, y + height],
    ]
    .into_iter()
    .map(|point| rotate_point(point, center, angle))
    .enumerate()
    {
        output.push(
            rect()
                .key(("drawing-selection-handle", index, handle))
                .position(absolute(
                    viewport,
                    point[0] - size / 2.,
                    point[1] - size / 2.,
                ))
                .width(Size::px(size * viewport.zoom))
                .height(Size::px(size * viewport.zoom))
                .background(Color::WHITE)
                .border(Border::new().fill(excalidraw_purple()).width(1.))
                .with_corner_radius(2.)
                .a11y_alt(format!("Drawing selection handle {handle}"))
                .into_element(),
        );
    }
}

fn points(element: &DrawingElement) -> Vec<[f32; 2]> {
    let mut points = if element.points.is_empty() {
        vec![
            [element.x, element.y],
            [element.x + element.width, element.y + element.height],
        ]
    } else {
        element
            .points
            .iter()
            .map(|point| [element.x + point[0], element.y + point[1]])
            .collect()
    };
    if element.angle.abs() <= f32::EPSILON {
        return points;
    }
    let (x, y, width, height) = element.bounds();
    let center = [x + width / 2., y + height / 2.];
    for point in &mut points {
        *point = rotate_point(*point, center, element.angle);
    }
    points
}

fn rotate_point(point: [f32; 2], center: [f32; 2], angle: f32) -> [f32; 2] {
    if angle.abs() <= f32::EPSILON {
        return point;
    }
    let sin = angle.sin();
    let cos = angle.cos();
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn element_with_angle(kind: &str, angle: f32) -> DrawingElement {
        serde_json::from_value(json!({
            "id":"rotated",
            "type":kind,
            "x":0,
            "y":0,
            "width":100,
            "height":20,
            "angle":angle,
            "points":[[0,0],[100,20]],
            "strokeColor":"#000000",
            "backgroundColor":"transparent",
            "strokeWidth":2,
            "opacity":100
        }))
        .unwrap()
    }

    #[test]
    fn polyline_points_follow_excalidraw_radian_rotation() {
        let element = element_with_angle("line", std::f32::consts::FRAC_PI_2);
        let rotated = points(&element);
        assert!((rotated[0][0] - 60.).abs() < 0.001);
        assert!((rotated[0][1] + 40.).abs() < 0.001);
        assert!((rotated[1][0] - 40.).abs() < 0.001);
        assert!((rotated[1][1] - 60.).abs() < 0.001);
    }

    #[test]
    fn midpoint_geometry_keeps_segment_center_on_the_requested_line() {
        let start: [f32; 2] = [10., 20.];
        let end: [f32; 2] = [50., 60.];
        let midpoint = [(start[0] + end[0]) / 2., (start[1] + end[1]) / 2.];
        assert_eq!(midpoint, [30., 40.]);
        let length = ((end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2)).sqrt();
        assert!((length - 56.56854).abs() < 0.001);
    }
}
