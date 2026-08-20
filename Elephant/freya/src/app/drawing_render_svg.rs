use bytes::Bytes;
use elephant_draw::{Arrowhead, ArrowheadPrimitive};
use freya::prelude::*;
use serde_json::Value;
use std::fmt::Write;

use super::drawing_scene::{DrawingElement, Viewport};

pub(super) fn shape(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
    ellipse: bool,
) -> Element {
    let (x, y, width, height) = element.bounds();
    let width = width.max(1.0);
    let height = height.max(1.0);
    let tag = if ellipse { "ellipse" } else { "rect" };
    let geometry = if ellipse {
        format!(
            "cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"",
            width / 2.0,
            height / 2.0,
            width / 2.0,
            height / 2.0
        )
    } else {
        format!("x=\"0\" y=\"0\" width=\"{width}\" height=\"{height}\"")
    };
    let content = format!("<{tag} {geometry} {}/>", style(element, true));
    rotated_surface(
        ("drawing-shape", index),
        element,
        viewport,
        x,
        y,
        width,
        height,
        content,
        label,
    )
}

pub(super) fn diamond(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
) -> Element {
    let (x, y, width, height) = element.bounds();
    let width = width.max(1.0);
    let height = height.max(1.0);
    let points = format!(
        "{},0 {},{} {},{} 0,{}",
        width / 2.0,
        width,
        height / 2.0,
        width / 2.0,
        height,
        height / 2.0
    );
    rotated_surface(
        ("drawing-diamond", index),
        element,
        viewport,
        x,
        y,
        width,
        height,
        format!("<polygon points=\"{points}\" {}/>", style(element, true)),
        label,
    )
}

pub(super) fn polyline(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
) -> Element {
    let points = absolute_points(element);
    let (x, y, width, height) = element.bounds();
    let width = width.max(1.0);
    let height = height.max(1.0);
    let mut path = String::new();
    for (point_index, [px, py]) in points.iter().enumerate() {
        let command = if point_index == 0 { 'M' } else { 'L' };
        let _ = write!(path, "{command} {} {} ", px - x, py - y);
    }
    let mut content = format!("<path d=\"{path}\" {}/>", style(element, false));
    if element.kind == "arrow" {
        if let Some(kind) = element
            .start_arrowhead
            .as_deref()
            .filter(|kind| *kind != "none")
        {
            if let Some(window) = points.windows(2).next() {
                content.push_str(&arrowhead(element, kind, window[1], window[0], x, y));
            }
        }
        if let Some(kind) = element
            .end_arrowhead
            .as_deref()
            .filter(|kind| *kind != "none")
        {
            if let Some(window) = points.windows(2).last() {
                content.push_str(&arrowhead(element, kind, window[0], window[1], x, y));
            }
        }
    }
    rotated_padded_surface(
        ("drawing-polyline", index),
        element,
        viewport,
        x,
        y,
        width,
        height,
        content,
        label,
        24.0,
    )
}

pub(super) fn text(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
) -> Element {
    let width = element
        .width
        .max(element.text.chars().count() as f32 * element.font_size.max(1.0) * 0.6)
        .max(1.0);
    let height = element.height.max(element.font_size).max(1.0);
    let content = format!(
        "<text x=\"0\" y=\"{}\" fill=\"{}\" opacity=\"{}\" font-family=\"sans-serif\" font-size=\"{}\">{}</text>",
        element.font_size.max(1.0),
        escape(&element.stroke_color),
        opacity(element),
        element.font_size.max(1.0),
        escape(&element.text)
    );
    rotated_surface(
        ("drawing-text", index),
        element,
        viewport,
        element.x,
        element.y,
        width,
        height,
        content,
        label,
    )
}

pub(super) fn image(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
    files: &Value,
) -> Element {
    let width = element.width.abs().max(1.0);
    let height = element.height.abs().max(1.0);
    let content = data_url(element, files).map_or_else(
        || format!(
            "<rect width=\"{width}\" height=\"{height}\" fill=\"#e9ecef\" stroke=\"#868e96\"/><path d=\"M 0 {height} L {} {} L {width} {height}\" fill=\"none\" stroke=\"#868e96\"/>",
            width * 0.4,
            height * 0.55
        ),
        |url| format!(
            "<image href=\"{}\" width=\"{width}\" height=\"{height}\" opacity=\"{}\" preserveAspectRatio=\"none\"/>",
            escape(url),
            opacity(element)
        ),
    );
    rotated_surface(
        ("drawing-image", index),
        element,
        viewport,
        element.x,
        element.y,
        width,
        height,
        content,
        label,
    )
}

pub(super) fn selection(index: usize, element: &DrawingElement, viewport: Viewport) -> Element {
    let (x, y, width, height) = element.bounds();
    let width = (width + 8.0).max(8.0);
    let height = (height + 8.0).max(8.0);
    surface(
        ("drawing-selection", index),
        viewport,
        x - 4.0,
        y - 4.0,
        width,
        height,
        format!("<rect x=\"1\" y=\"1\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"rgb(105,101,219)\" stroke-width=\"1\" stroke-dasharray=\"4 3\"/>", width - 2.0, height - 2.0),
        &format!("Drawing selection {index}"),
    )
}

fn style(element: &DrawingElement, fill: bool) -> String {
    let fill = if fill {
        escape(&element.background_color)
    } else {
        "none".to_owned()
    };
    let dash = match element.stroke_style.as_str() {
        "dashed" => " stroke-dasharray=\"8 6\"",
        "dotted" => " stroke-dasharray=\"2 5\"",
        _ => "",
    };
    format!(
        "fill=\"{fill}\" stroke=\"{}\" stroke-width=\"{}\"{dash} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        opacity(element)
    )
}

fn arrowhead(
    element: &DrawingElement,
    kind: &str,
    previous: [f32; 2],
    end: [f32; 2],
    x: f32,
    y: f32,
) -> String {
    let arrowhead = Arrowhead::from_id(kind).or_else(|| match kind {
        "dot" => Some(Arrowhead::Circle),
        "crowfoot_one" => Some(Arrowhead::CardinalityOne),
        "crowfoot_many" => Some(Arrowhead::CardinalityMany),
        "crowfoot_one_or_many" => Some(Arrowhead::CardinalityOneOrMany),
        _ => None,
    });
    let Some(arrowhead) = arrowhead else {
        return String::new();
    };
    let previous = [previous[0] - x, previous[1] - y];
    let end = [end[0] - x, end[1] - y];
    let mut output = String::new();
    for primitive in arrowhead.primitives(previous, end) {
        match primitive {
            ArrowheadPrimitive::Line { from, to } => {
                let _ = write!(
                    output,
                    "<path data-excalidraw-arrowhead=\"{}\" d=\"M {} {} L {} {}\" {}/>",
                    escape(kind),
                    from[0],
                    from[1],
                    to[0],
                    to[1],
                    arrowhead_style(element, false)
                );
            }
            ArrowheadPrimitive::Polygon { points, filled } => {
                let mut serialized = String::new();
                for (index, [px, py]) in points.iter().enumerate() {
                    if index > 0 {
                        serialized.push(' ');
                    }
                    let _ = write!(serialized, "{px},{py}");
                }
                let _ = write!(
                    output,
                    "<polygon data-excalidraw-arrowhead=\"{}\" points=\"{}\" {}/>",
                    escape(kind),
                    serialized,
                    arrowhead_style(element, filled)
                );
            }
            ArrowheadPrimitive::Circle {
                center,
                radius,
                filled,
            } => {
                let _ = write!(
                    output,
                    "<circle data-excalidraw-arrowhead=\"{}\" cx=\"{}\" cy=\"{}\" r=\"{}\" {}/>",
                    escape(kind),
                    center[0],
                    center[1],
                    radius,
                    arrowhead_style(element, filled)
                );
            }
        }
    }
    output
}

fn arrowhead_style(element: &DrawingElement, filled: bool) -> String {
    let fill = if filled {
        escape(&element.stroke_color)
    } else {
        "none".to_owned()
    };
    format!(
        "fill=\"{fill}\" stroke=\"{}\" stroke-width=\"{}\" opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        opacity(element)
    )
}

#[allow(clippy::too_many_arguments)]
fn rotated_padded_surface(
    key: impl std::hash::Hash,
    element: &DrawingElement,
    viewport: Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    label: &str,
    padding: f32,
) -> Element {
    if element.angle.abs() <= f32::EPSILON {
        return padded_surface(key, viewport, x, y, width, height, content, label, padding);
    }
    let (sin, cos) = (element.angle.sin().abs(), element.angle.cos().abs());
    let (rw, rh) = (width * cos + height * sin, width * sin + height * cos);
    let (ox, oy) = ((rw - width) / 2.0, (rh - height) / 2.0);
    let content = format!(
        "<g transform=\"translate({ox} {oy}) rotate({} {} {})\">{content}</g>",
        element.angle.to_degrees(),
        width / 2.0,
        height / 2.0
    );
    padded_surface(
        key,
        viewport,
        x + width / 2.0 - rw / 2.0,
        y + height / 2.0 - rh / 2.0,
        rw,
        rh,
        content,
        label,
        padding,
    )
}

#[allow(clippy::too_many_arguments)]
fn padded_surface(
    key: impl std::hash::Hash,
    viewport: Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    label: &str,
    padding: f32,
) -> Element {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let padded_width = width + padding * 2.0;
    let padded_height = height + padding * 2.0;
    let source = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">{content}</svg>",
        -padding, -padding, padded_width, padded_height
    );
    rect()
        .position(
            Position::new_absolute()
                .left(viewport.pan[0] + (x - padding) * viewport.zoom)
                .top(viewport.pan[1] + (y - padding) * viewport.zoom),
        )
        .width(Size::px(padded_width * viewport.zoom))
        .height(Size::px(padded_height * viewport.zoom))
        .a11y_alt(label.to_owned())
        .child(
            SvgViewer::new((key, Bytes::from(source.into_bytes())))
                .width(Size::fill())
                .height(Size::fill())
                .show_loader(false),
        )
        .into_element()
}

fn rotated_surface(
    key: impl std::hash::Hash,
    element: &DrawingElement,
    viewport: Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    label: &str,
) -> Element {
    if element.angle.abs() <= f32::EPSILON {
        return surface(key, viewport, x, y, width, height, content, label);
    }
    let (sin, cos) = (element.angle.sin().abs(), element.angle.cos().abs());
    let (rw, rh) = (width * cos + height * sin, width * sin + height * cos);
    let (ox, oy) = ((rw - width) / 2.0, (rh - height) / 2.0);
    let content = format!(
        "<g transform=\"translate({ox} {oy}) rotate({} {} {})\">{content}</g>",
        element.angle.to_degrees(),
        width / 2.0,
        height / 2.0
    );
    surface(
        key,
        viewport,
        x + width / 2.0 - rw / 2.0,
        y + height / 2.0 - rh / 2.0,
        rw,
        rh,
        content,
        label,
    )
}

fn surface(
    key: impl std::hash::Hash,
    viewport: Viewport,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    content: String,
    label: &str,
) -> Element {
    let (width, height) = (width.max(1.0), height.max(1.0));
    let source = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\">{content}</svg>"
    );
    rect()
        .position(
            Position::new_absolute()
                .left(viewport.pan[0] + x * viewport.zoom)
                .top(viewport.pan[1] + y * viewport.zoom),
        )
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

fn absolute_points(element: &DrawingElement) -> Vec<[f32; 2]> {
    if element.points.is_empty() {
        return vec![
            [element.x, element.y],
            [element.x + element.width, element.y + element.height],
        ];
    }
    element
        .points
        .iter()
        .map(|[x, y]| [element.x + x, element.y + y])
        .collect()
}

fn data_url<'a>(element: &DrawingElement, files: &'a Value) -> Option<&'a str> {
    let id = element.extra.get("fileId")?.as_str()?;
    files.as_object()?.get(id)?.get("dataURL")?.as_str()
}

fn opacity(element: &DrawingElement) -> f32 {
    (element.opacity / 100.0).clamp(0.0, 1.0)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
