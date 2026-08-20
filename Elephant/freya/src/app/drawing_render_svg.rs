use bytes::Bytes;
use elephant_draw::{font_family_css, layout_text, Arrowhead, ArrowheadPrimitive};
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
    let pattern_id = format!("draw-fill-{index}");
    let (defs, fill) = fill_paint(element, &pattern_id);
    let geometry = if ellipse {
        format!(
            "cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"",
            width / 2.0,
            height / 2.0,
            width / 2.0,
            height / 2.0
        )
    } else {
        let radius = roundness_radius(element, width, height);
        format!(
            "x=\"0\" y=\"0\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\" ry=\"{radius}\""
        )
    };
    let tag = if ellipse { "ellipse" } else { "rect" };
    let mut content = defs;
    let _ = write!(
        content,
        "<{tag} {geometry} {}/>",
        shape_style(element, &fill)
    );
    append_rough_shape(&mut content, tag, &geometry, element);

    let mut padding = rough_padding(element);
    if matches!(element.kind.as_str(), "frame" | "magicframe") {
        if let Some(name) = element.extra.get("name").and_then(Value::as_str) {
            if !name.is_empty() {
                let _ = write!(
                    content,
                    "<text x=\"2\" y=\"-7\" font-family=\"sans-serif\" font-size=\"14\" fill=\"{}\" opacity=\"{}\">{}</text>",
                    escape(&element.stroke_color),
                    opacity(element),
                    escape(name)
                );
                padding = padding.max(24.0);
            }
        }
    }

    rotated_padded_surface(
        ("drawing-shape", index),
        element,
        viewport,
        x,
        y,
        width,
        height,
        content,
        label,
        padding,
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
    let pattern_id = format!("draw-fill-{index}");
    let (defs, fill) = fill_paint(element, &pattern_id);
    let mut content = defs;
    let _ = write!(
        content,
        "<polygon points=\"{points}\" {}/>",
        shape_style(element, &fill)
    );
    append_rough_shape(
        &mut content,
        "polygon",
        &format!("points=\"{points}\""),
        element,
    );
    rotated_padded_surface(
        ("drawing-diamond", index),
        element,
        viewport,
        x,
        y,
        width,
        height,
        content,
        label,
        rough_padding(element),
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
    let local = points
        .iter()
        .map(|point| [point[0] - x, point[1] - y])
        .collect::<Vec<_>>();
    let path = if element.kind == "freedraw" {
        smooth_path(&local)
    } else {
        straight_path(&local)
    };
    let mut content = format!("<path d=\"{path}\" {}/>", linear_style(element));
    append_rough_path(&mut content, &path, element);

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
        28.0_f32.max(rough_padding(element)),
    )
}

pub(super) fn text(
    element: &DrawingElement,
    index: usize,
    label: &str,
    viewport: Viewport,
) -> Element {
    let layout = layout_text(element);
    let font_family = element
        .extra
        .get("fontFamily")
        .and_then(Value::as_u64)
        .unwrap_or(5);
    let mut content = String::new();
    for line in &layout.lines {
        let _ = write!(
            content,
            "<text x=\"{}\" y=\"{}\" fill=\"{}\" opacity=\"{}\" font-family=\"{}\" font-size=\"{}\" xml:space=\"preserve\">{}</text>",
            line.x,
            line.baseline_y,
            escape(&element.stroke_color),
            opacity(element),
            escape(font_family_css(font_family)),
            element.font_size.max(1.0),
            escape(&line.text)
        );
    }
    rotated_padded_surface(
        ("drawing-text", index),
        element,
        viewport,
        element.x,
        element.y,
        layout.width.max(1.0),
        layout.height.max(element.font_size).max(1.0),
        content,
        label,
        2.0,
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
        || image_placeholder(width, height),
        |url| image_markup(element, index, url, width, height),
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
        format!(
            "<rect x=\"1\" y=\"1\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"rgb(105,101,219)\" stroke-width=\"1\" stroke-dasharray=\"4 3\"/>",
            width - 2.0,
            height - 2.0
        ),
        &format!("Drawing selection {index}"),
    )
}

fn fill_paint(element: &DrawingElement, pattern_id: &str) -> (String, String) {
    if element.background_color.eq_ignore_ascii_case("transparent")
        || element.background_color.is_empty()
    {
        return (String::new(), "none".to_owned());
    }
    if element.fill_style == "solid" || element.fill_style.is_empty() {
        return (String::new(), escape(&element.background_color));
    }

    let color = escape(&element.background_color);
    let weight = (element.stroke_width * 0.7).clamp(0.7, 2.5);
    let pattern = match element.fill_style.as_str() {
        "cross-hatch" => format!(
            "<pattern id=\"{pattern_id}\" patternUnits=\"userSpaceOnUse\" width=\"9\" height=\"9\"><path d=\"M -2 2 L 7 11 M 2 -2 L 11 7 M 11 -2 L -2 11\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{weight}\" opacity=\"0.82\"/></pattern>"
        ),
        "zigzag" => format!(
            "<pattern id=\"{pattern_id}\" patternUnits=\"userSpaceOnUse\" width=\"12\" height=\"8\"><path d=\"M 0 6 L 3 2 L 6 6 L 9 2 L 12 6\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{weight}\" opacity=\"0.82\"/></pattern>"
        ),
        _ => format!(
            "<pattern id=\"{pattern_id}\" patternUnits=\"userSpaceOnUse\" width=\"8\" height=\"8\" patternTransform=\"rotate(-41)\"><path d=\"M 0 0 L 0 8\" stroke=\"{color}\" stroke-width=\"{weight}\" opacity=\"0.82\"/></pattern>"
        ),
    };
    (format!("<defs>{pattern}</defs>"), format!("url(#{pattern_id})"))
}

fn shape_style(element: &DrawingElement, fill: &str) -> String {
    format!(
        "fill=\"{fill}\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}

fn linear_style(element: &DrawingElement) -> String {
    format!(
        "fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}

fn append_rough_shape(output: &mut String, tag: &str, geometry: &str, element: &DrawingElement) {
    let roughness = roughness(element);
    if roughness <= f32::EPSILON {
        return;
    }
    let [dx, dy] = rough_offset(element, roughness);
    let _ = write!(
        output,
        "<{tag} {geometry} transform=\"translate({dx} {dy})\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        escape(&element.stroke_color),
        (element.stroke_width * 0.75).max(0.5),
        dash_attribute(&element.stroke_style),
        (opacity(element) * 0.42).clamp(0.0, 1.0)
    );
}

fn append_rough_path(output: &mut String, path: &str, element: &DrawingElement) {
    let roughness = roughness(element);
    if roughness <= f32::EPSILON {
        return;
    }
    let [dx, dy] = rough_offset(element, roughness);
    let _ = write!(
        output,
        "<path d=\"{path}\" transform=\"translate({dx} {dy})\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        escape(&element.stroke_color),
        (element.stroke_width * 0.7).max(0.5),
        dash_attribute(&element.stroke_style),
        (opacity(element) * 0.38).clamp(0.0, 1.0)
    );
}

fn roughness(element: &DrawingElement) -> f32 {
    element
        .extra
        .get("roughness")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.0, 2.0) as f32
}

fn rough_offset(element: &DrawingElement, roughness: f32) -> [f32; 2] {
    let seed = element
        .extra
        .get("seed")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let a = ((seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) >> 16) & 0x7fff) as f32;
    let b = ((seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223) >> 16) & 0x7fff) as f32;
    let scale = roughness * 0.9;
    [((a / 32767.0) - 0.5) * scale, ((b / 32767.0) - 0.5) * scale]
}

fn rough_padding(element: &DrawingElement) -> f32 {
    3.0 + roughness(element) * 2.0 + element.stroke_width.max(0.0)
}

fn roundness_radius(element: &DrawingElement, width: f32, height: f32) -> f32 {
    let Some(roundness) = element.extra.get("roundness") else {
        return 0.0;
    };
    if roundness.is_null() {
        return 0.0;
    }
    let kind = roundness
        .get("type")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    match kind {
        3 => 32.0_f32.min(width.min(height) * 0.25),
        _ => width.max(height) * 0.25,
    }
    .max(0.0)
}

fn straight_path(points: &[[f32; 2]]) -> String {
    let mut path = String::new();
    for (index, [x, y]) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(path, "{command} {x} {y}");
        if index + 1 < points.len() {
            path.push(' ');
        }
    }
    path
}

fn smooth_path(points: &[[f32; 2]]) -> String {
    match points {
        [] => String::new(),
        [[x, y]] => format!("M {x} {y}"),
        [first, second] => format!("M {} {} L {} {}", first[0], first[1], second[0], second[1]),
        _ => {
            let mut path = format!("M {} {}", points[0][0], points[0][1]);
            for index in 1..points.len() - 1 {
                let current = points[index];
                let next = points[index + 1];
                let mid = [(current[0] + next[0]) / 2.0, (current[1] + next[1]) / 2.0];
                let _ = write!(
                    path,
                    " Q {} {} {} {}",
                    current[0], current[1], mid[0], mid[1]
                );
            }
            let last = points[points.len() - 1];
            let penultimate = points[points.len() - 2];
            let _ = write!(
                path,
                " Q {} {} {} {}",
                penultimate[0], penultimate[1], last[0], last[1]
            );
            path
        }
    }
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

fn image_placeholder(width: f32, height: f32) -> String {
    format!(
        "<rect width=\"{width}\" height=\"{height}\" fill=\"#e9ecef\" stroke=\"#868e96\"/><path d=\"M 0 {height} L {} {} L {width} {height}\" fill=\"none\" stroke=\"#868e96\"/>",
        width * 0.4,
        height * 0.55
    )
}

fn image_markup(
    element: &DrawingElement,
    index: usize,
    url: &str,
    width: f32,
    height: f32,
) -> String {
    let [scale_x, scale_y] = image_scale(element);
    let crop = image_crop(element);
    let (image_x, image_y, image_width, image_height) = crop.map_or(
        (0.0, 0.0, width, height),
        |[crop_x, crop_y, crop_width, crop_height, natural_width, natural_height]| {
            let source_width = width * natural_width / crop_width;
            let source_height = height * natural_height / crop_height;
            (
                -(crop_x / natural_width) * source_width,
                -(crop_y / natural_height) * source_height,
                source_width,
                source_height,
            )
        },
    );
    let clip_id = format!("draw-image-clip-{index}");
    let translate_x = if scale_x < 0.0 { width } else { 0.0 };
    let translate_y = if scale_y < 0.0 { height } else { 0.0 };
    format!(
        "<defs><clipPath id=\"{clip_id}\"><rect x=\"0\" y=\"0\" width=\"{width}\" height=\"{height}\"/></clipPath></defs><g clip-path=\"url(#{clip_id})\" transform=\"translate({translate_x} {translate_y}) scale({scale_x} {scale_y})\"><image href=\"{}\" x=\"{image_x}\" y=\"{image_y}\" width=\"{image_width}\" height=\"{image_height}\" opacity=\"{}\" preserveAspectRatio=\"none\"/></g>",
        escape(url),
        opacity(element)
    )
}

fn image_scale(element: &DrawingElement) -> [f32; 2] {
    let Some(scale) = element.extra.get("scale").and_then(Value::as_array) else {
        return [1.0, 1.0];
    };
    if scale.len() != 2 {
        return [1.0, 1.0];
    }
    let x = scale[0].as_f64().unwrap_or(1.0) as f32;
    let y = scale[1].as_f64().unwrap_or(1.0) as f32;
    [if x < 0.0 { -1.0 } else { 1.0 }, if y < 0.0 { -1.0 } else { 1.0 }]
}

fn image_crop(element: &DrawingElement) -> Option<[f32; 6]> {
    let crop = element.extra.get("crop")?.as_object()?;
    let values = [
        crop.get("x")?.as_f64()? as f32,
        crop.get("y")?.as_f64()? as f32,
        crop.get("width")?.as_f64()? as f32,
        crop.get("height")?.as_f64()? as f32,
        crop.get("naturalWidth")?.as_f64()? as f32,
        crop.get("naturalHeight")?.as_f64()? as f32,
    ];
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(values)
        .filter(|values| values[2] > 0.0 && values[3] > 0.0 && values[4] > 0.0 && values[5] > 0.0)
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

fn dash_attribute(stroke_style: &str) -> &'static str {
    match stroke_style {
        "dashed" => " stroke-dasharray=\"8 6\"",
        "dotted" => " stroke-dasharray=\"2 5\"",
        _ => "",
    }
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
        .replace('\'', "&apos;")
}
