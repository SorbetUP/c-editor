use bytes::Bytes;
use freya::prelude::*;
use serde_json::Value;
use std::fmt::Write;

use super::drawing_scene::{DrawingCanvasState, DrawingElement, Viewport};

pub fn render(state: &DrawingCanvasState) -> Vec<Element> {
    let mut elements = Vec::new();
    for (index, element) in state.document.elements.iter().enumerate() {
        if element.is_deleted {
            continue;
        }
        let selected = state.selected_element_id() == Some(element.id.as_str());
        render_element(
            &mut elements,
            element,
            index,
            state.viewport,
            selected,
            &state.document.files,
        );
    }
    elements
}

fn render_element(
    output: &mut Vec<Element>,
    element: &DrawingElement,
    index: usize,
    viewport: Viewport,
    selected: bool,
    files: &Value,
) {
    let label = format!("Drawing element {} {}", element.id, element.kind);
    let (x, y, width, height) = element.bounds();
    match element.kind.as_str() {
        "rectangle" => output.push(shape_svg(
            element, index, &label, x, y, width, height, viewport, false,
        )),
        "ellipse" => output.push(shape_svg(
            element, index, &label, x, y, width, height, viewport, true,
        )),
        "diamond" => output.push(diamond_svg(
            element, index, &label, x, y, width, height, viewport,
        )),
        "line" | "arrow" | "freedraw" => output.push(polyline_svg(
            element, index, &label, x, y, width, height, viewport,
        )),
        "text" => output.push(text_svg(element, index, &label, viewport)),
        "image" => output.push(image_svg(element, index, &label, viewport, files)),
        _ => {}
    }
    if selected {
        output.push(selection_svg(index, x, y, width, height, viewport));
    }
}

fn shape_svg(
    element: &DrawingElement,
    index: usize,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
    ellipse: bool,
) -> Element {
    let width = width.max(1.);
    let height = height.max(1.);
    let stroke = svg_color(&element.stroke_color);
    let fill = svg_color(&element.background_color);
    let opacity = (element.opacity / 100.).clamp(0., 1.);
    let dash = stroke_dash_attribute(&element.stroke_style);
    let content = if ellipse {
        format!(
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"{} opacity="{}"/>"#,
            width / 2.,
            height / 2.,
            width / 2.,
            height / 2.,
            fill,
            stroke,
            element.stroke_width.max(0.5),
            dash,
            opacity
        )
    } else {
        format!(
            r#"<rect x="0" y="0" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="{}"{} opacity="{}"/>"#,
            width,
            height,
            fill,
            stroke,
            element.stroke_width.max(0.5),
            dash,
            opacity
        )
    };
    let (surface_x, surface_y, surface_width, surface_height, content) =
        rotate_content(element, x, y, width, height, content);
    svg_surface(
        ("drawing-shape", index),
        viewport,
        surface_x,
        surface_y,
        surface_width,
        surface_height,
        content,
        label,
    )
}

fn diamond_svg(
    element: &DrawingElement,
    index: usize,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
) -> Element {
    let width = width.max(1.);
    let height = height.max(1.);
    let content = format!(
        r#"<polygon points="{},{} {},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="{}"{} opacity="{}" stroke-linejoin="round"/>"#,
        width / 2.,
        0.,
        width,
        height / 2.,
        width / 2.,
        height,
        0.,
        height / 2.,
        svg_color(&element.background_color),
        svg_color(&element.stroke_color),
        element.stroke_width.max(0.5),
        stroke_dash_attribute(&element.stroke_style),
        (element.opacity / 100.).clamp(0., 1.)
    );
    let (surface_x, surface_y, surface_width, surface_height, content) =
        rotate_content(element, x, y, width, height, content);
    svg_surface(
        ("drawing-diamond", index),
        viewport,
        surface_x,
        surface_y,
        surface_width,
        surface_height,
        content,
        label,
    )
}

fn polyline_svg(
    element: &DrawingElement,
    index: usize,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
) -> Element {
    let points = points(element);
    let width = width.max(1.);
    let height = height.max(1.);
    let mut path = String::new();
    for (point_index, [px, py]) in points.iter().enumerate() {
        let command = if point_index == 0 { "M" } else { "L" };
        let _ = write!(path, "{} {} {} ", command, px - x, py - y);
    }
    let stroke = svg_color(&element.stroke_color);
    let opacity = (element.opacity / 100.).clamp(0., 1.);
    let dash = stroke_dash_attribute(&element.stroke_style);
    let mut content = format!(
        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}"{} opacity="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
        path,
        stroke,
        element.stroke_width.max(0.5),
        dash,
        opacity
    );
    if element.kind == "arrow" && element.end_arrowhead.as_deref() != Some("none") {
        if let Some([previous, end]) = points
            .windows(2)
            .last()
            .map(|window| [window[0], window[1]])
        {
            let length = ((end[0] - previous[0]).powi(2) + (end[1] - previous[1]).powi(2))
                .sqrt()
                .max(1.);
            let ux = (end[0] - previous[0]) / length;
            let uy = (end[1] - previous[1]) / length;
            let base_x = end[0] - x - ux * 12.;
            let base_y = end[1] - y - uy * 12.;
            let wing_x = -uy * 5.;
            let wing_y = ux * 5.;
            let _ = write!(
                content,
                r#"<path d="M {} {} L {} {} M {} {} L {} {}" fill="none" stroke="{}" stroke-width="{}"{} opacity="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                end[0] - x,
                end[1] - y,
                base_x + wing_x,
                base_y + wing_y,
                end[0] - x,
                end[1] - y,
                base_x - wing_x,
                base_y - wing_y,
                stroke,
                element.stroke_width.max(0.5),
                dash,
                opacity
            );
        }
    }
    let (surface_x, surface_y, surface_width, surface_height, content) =
        rotate_content(element, x, y, width, height, content);
    svg_surface(
        ("drawing-polyline", index),
        viewport,
        surface_x,
        surface_y,
        surface_width,
        surface_height,
        content,
        label,
    )
}

fn text_svg(element: &DrawingElement, index: usize, label: &str, viewport: Viewport) -> Element {
    let width = element
        .width
        .max(element.text.chars().count() as f32 * element.font_size * 0.6)
        .max(1.);
    let height = element.height.max(element.font_size).max(1.);
    let content = format!(
        r#"<text x="0" y="{}" fill="{}" opacity="{}" font-family="sans-serif" font-size="{}">{}</text>"#,
        element.font_size.max(1.),
        svg_color(&element.stroke_color),
        (element.opacity / 100.).clamp(0., 1.),
        element.font_size.max(1.),
        svg_escape(&element.text)
    );
    let (surface_x, surface_y, surface_width, surface_height, content) = rotate_content(
        element,
        element.x,
        element.y,
        width,
        height,
        content,
    );
    svg_surface(
        ("drawing-text", index),
        viewport,
        surface_x,
        surface_y,
        surface_width,
        surface_height,
        content,
        label,
    )
}

fn image_svg(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
    files: &Value,
) -> Element {
    let width = element.width.abs().max(1.);
    let height = element.height.abs().max(1.);
    let content = image_data_url(element, files).map_or_else(
        || {
            format!(
                r#"<rect x="0" y="0" width="{}" height="{}" fill="rgb(233,236,239)" stroke="rgb(134,142,150)"/><path d="M 0 {} L {} {} L {} {}" fill="none" stroke="rgb(134,142,150)"/>"#,
                width,
                height,
                height,
                width * 0.4,
                height * 0.55,
                width,
                height
            )
        },
        |data_url| {
            format!(
                r#"<image href="{}" x="0" y="0" width="{}" height="{}" opacity="{}" preserveAspectRatio="none"/>"#,
                svg_escape(data_url),
                width,
                height,
                (element.opacity / 100.).clamp(0., 1.)
            )
        },
    );
    let (surface_x, surface_y, surface_width, surface_height, content) = rotate_content(
        element,
        element.x,
        element.y,
        width,
        height,
        content,
    );
    svg_surface(
        ("drawing-image", index),
        viewport,
        surface_x,
        surface_y,
        surface_width,
        surface_height,
        content,
        label,
    )
}

fn image_data_url<'a>(element: &DrawingElement, files: &'a Value) -> Option<&'a str> {
    let file_id = element.extra.get("fileId")?.as_str()?;
    files
        .as_object()?
        .get(file_id)?
        .get("dataURL")?
        .as_str()
}

fn selection_svg(
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    viewport: Viewport,
) -> Element {
    let width = (width + 8.).max(8.);
    let height = (height + 8.).max(8.);
    let content = format!(
        r#"<rect x="1" y="1" width="{}" height="{}" fill="none" stroke="rgb(105,101,219)" stroke-width="1" stroke-dasharray="4 3"/>"#,
        width - 2.,
        height - 2.
    );
    svg_surface(
        ("drawing-selection", index),
        viewport,
        x - 4.,
        y - 4.,
        width,
        height,
        content,
        &format!("Drawing selection {index}"),
    )
}

fn rotate_content(
    element: &DrawingElement,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
) -> (f32, f32, f32, f32, String) {
    if element.angle.abs() <= f32::EPSILON {
        return (x, y, width, height, content);
    }
    let sin = element.angle.sin().abs();
    let cos = element.angle.cos().abs();
    let rotated_width = (width * cos + height * sin).max(1.);
    let rotated_height = (width * sin + height * cos).max(1.);
    let center_x = x + width / 2.;
    let center_y = y + height / 2.;
    let surface_x = center_x - rotated_width / 2.;
    let surface_y = center_y - rotated_height / 2.;
    let offset_x = (rotated_width - width) / 2.;
    let offset_y = (rotated_height - height) / 2.;
    let degrees = element.angle.to_degrees();
    (
        surface_x,
        surface_y,
        rotated_width,
        rotated_height,
        format!(
            r#"<g transform="translate({} {}) rotate({} {} {})">{}</g>"#,
            offset_x,
            offset_y,
            degrees,
            width / 2.,
            height / 2.,
            content
        ),
    )
}

fn stroke_dash_attribute(style: &str) -> &'static str {
    match style {
        "dashed" => r#" stroke-dasharray="8 6""#,
        "dotted" => r#" stroke-dasharray="2 5""#,
        _ => "",
    }
}

fn svg_surface(
    key: impl std::hash::Hash,
    viewport: Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    label: &str,
) -> Element {
    let width = width.max(1.);
    let height = height.max(1.);
    let source = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">{}</svg>"#,
        width, height, content
    );
    rect()
        .position(absolute(viewport, x, y))
        .width(Size::px(width * viewport.zoom))
        .height(Size::px(height * viewport.zoom))
        .a11y_alt(label.to_owned())
        .child(
            SvgViewer::new((key, Bytes::from(source.into_bytes())))
                .width(Size::fill())
                .height(Size::fill())
                .show_loader(false),
        )
        .into_element()
}

fn svg_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn svg_color(value: &str) -> String {
    svg_escape(if value.trim().is_empty() {
        "transparent"
    } else {
        value
    })
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
