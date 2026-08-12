//! Native PNG export for Elephant's Excalidraw-compatible Freya renderer.
//!
//! The PNG is intentionally a derived artifact. The `.excalidraw` JSON stays
//! canonical, while this module produces the preview consumed by existing
//! Tauri notes and embeds the full scene in an Excalidraw-compatible PNG tEXt
//! chunk so the exported image remains recoverable/editable.

use freya_skia_safe::{
    surfaces, Color, EncodedImageFormat, Font, Paint, PaintStyle, Point, Rect,
};
use serde_json::Value;

const EXPORT_PADDING: f32 = 10.;
const DEFAULT_WIDTH: i32 = 512;
const DEFAULT_HEIGHT: i32 = 384;
const MAX_EXPORT_DIMENSION: f32 = 2048.;
const EXCALIDRAW_PNG_KEYWORD: &str = "application/vnd.excalidraw+json";
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Bounds {
    fn from_point(point: [f32; 2]) -> Self {
        Self {
            min_x: point[0],
            min_y: point[1],
            max_x: point[0],
            max_y: point[1],
        }
    }

    fn include(&mut self, point: [f32; 2]) {
        self.min_x = self.min_x.min(point[0]);
        self.min_y = self.min_y.min(point[1]);
        self.max_x = self.max_x.max(point[0]);
        self.max_y = self.max_y.max(point[1]);
    }

    fn width(self) -> f32 {
        (self.max_x - self.min_x).max(1.)
    }

    fn height(self) -> f32 {
        (self.max_y - self.min_y).max(1.)
    }
}

#[derive(Clone, Copy, Debug)]
struct ExportTransform {
    bounds: Bounds,
    scale: f32,
}

impl ExportTransform {
    fn point(self, world: [f32; 2]) -> Point {
        Point::new(
            EXPORT_PADDING + (world[0] - self.bounds.min_x) * self.scale,
            EXPORT_PADDING + (world[1] - self.bounds.min_y) * self.scale,
        )
    }

    fn scalar(self, value: f32) -> f32 {
        value * self.scale
    }
}

/// Render the canonical Excalidraw JSON to a native Skia PNG and embed the
/// scene as Excalidraw-readable metadata.
pub(super) fn render_png(raw: &str) -> Result<Vec<u8>, String> {
    let scene: Value = serde_json::from_str(raw)
        .map_err(|error| format!("Unable to export drawing preview: invalid scene JSON: {error}"))?;
    if scene.get("type").and_then(Value::as_str) != Some("excalidraw") {
        return Err("Unable to export drawing preview: expected type=excalidraw".to_owned());
    }
    let elements = scene
        .get("elements")
        .and_then(Value::as_array)
        .ok_or_else(|| "Unable to export drawing preview: elements must be an array".to_owned())?;

    let visible = elements
        .iter()
        .filter(|element| !bool_value(element, "isDeleted", false))
        .collect::<Vec<_>>();
    let export_bounds = scene_bounds(&visible);
    let (width, height, transform) = export_geometry(export_bounds);
    let mut surface = surfaces::raster_n32_premul((width, height))
        .ok_or_else(|| format!("Unable to allocate drawing preview surface {width}x{height}"))?;
    let canvas = surface.canvas();

    let export_background = scene
        .get("appState")
        .and_then(|app| app.get("exportBackground"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let background = if export_background {
        scene
            .get("appState")
            .and_then(|app| app.get("viewBackgroundColor"))
            .and_then(Value::as_str)
            .map(|value| parse_color(value, 100.))
            .unwrap_or(Color::WHITE)
    } else {
        Color::TRANSPARENT
    };
    canvas.clear(background);

    for element in visible {
        draw_element(canvas, element, transform);
    }

    let image = surface.image_snapshot();
    #[allow(deprecated)]
    let encoded = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or_else(|| "Unable to encode drawing preview as PNG".to_owned())?;
    embed_scene_metadata(encoded.as_bytes(), &scene)
}

fn export_geometry(bounds: Option<Bounds>) -> (i32, i32, ExportTransform) {
    let Some(bounds) = bounds else {
        let default = Bounds {
            min_x: 0.,
            min_y: 0.,
            max_x: (DEFAULT_WIDTH as f32 - EXPORT_PADDING * 2.).max(1.),
            max_y: (DEFAULT_HEIGHT as f32 - EXPORT_PADDING * 2.).max(1.),
        };
        return (
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            ExportTransform {
                bounds: default,
                scale: 1.,
            },
        );
    };

    let max_world = bounds.width().max(bounds.height());
    let scale = if max_world + EXPORT_PADDING * 2. > MAX_EXPORT_DIMENSION {
        ((MAX_EXPORT_DIMENSION - EXPORT_PADDING * 2.) / max_world).max(0.01)
    } else {
        1.
    };
    let width = (bounds.width() * scale + EXPORT_PADDING * 2.)
        .ceil()
        .max(1.) as i32;
    let height = (bounds.height() * scale + EXPORT_PADDING * 2.)
        .ceil()
        .max(1.) as i32;
    (width, height, ExportTransform { bounds, scale })
}

fn scene_bounds(elements: &[&Value]) -> Option<Bounds> {
    let mut bounds = None;
    for element in elements {
        for point in element_outline_points(element) {
            match bounds.as_mut() {
                Some(bounds) => bounds.include(point),
                None => bounds = Some(Bounds::from_point(point)),
            }
        }
    }
    bounds
}

fn element_outline_points(element: &Value) -> Vec<[f32; 2]> {
    let x = number(element, "x", 0.);
    let y = number(element, "y", 0.);
    let width = number(element, "width", 0.);
    let height = number(element, "height", 0.);
    let angle = number(element, "angle", 0.);
    let kind = string_value(element, "type", "");

    if matches!(kind, "line" | "arrow" | "freedraw") {
        let mut points = absolute_points(element);
        if points.is_empty() {
            points = vec![[x, y], [x + width, y + height]];
        }
        return rotate_points(points, [x + width / 2., y + height / 2.], angle);
    }

    let center = [x + width / 2., y + height / 2.];
    rotate_points(
        vec![
            [x, y],
            [x + width, y],
            [x + width, y + height],
            [x, y + height],
        ],
        center,
        angle,
    )
}

fn draw_element(canvas: &freya_skia_safe::Canvas, element: &Value, transform: ExportTransform) {
    let kind = string_value(element, "type", "");
    let opacity = number(element, "opacity", 100.).clamp(0., 100.);
    let stroke = parse_color(string_value(element, "strokeColor", "#1b1b1f"), opacity);
    let fill = parse_color(
        string_value(element, "backgroundColor", "transparent"),
        opacity,
    );
    let stroke_width = number(element, "strokeWidth", 2.).max(0.5);

    match kind {
        "rectangle" => draw_box(canvas, element, transform, stroke, fill, stroke_width, false),
        "ellipse" => draw_box(canvas, element, transform, stroke, fill, stroke_width, true),
        "line" | "freedraw" => draw_polyline(canvas, element, transform, stroke, stroke_width, false),
        "arrow" => draw_polyline(canvas, element, transform, stroke, stroke_width, true),
        "text" => draw_text(canvas, element, transform, stroke),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_box(
    canvas: &freya_skia_safe::Canvas,
    element: &Value,
    transform: ExportTransform,
    stroke: Color,
    fill: Color,
    stroke_width: f32,
    ellipse: bool,
) {
    let x = number(element, "x", 0.);
    let y = number(element, "y", 0.);
    let width = number(element, "width", 0.).max(1.);
    let height = number(element, "height", 0.).max(1.);
    let angle = number(element, "angle", 0.);
    let top_left = transform.point([x, y]);
    let rect = Rect::from_xywh(
        top_left.x,
        top_left.y,
        transform.scalar(width),
        transform.scalar(height),
    );
    let center = transform.point([x + width / 2., y + height / 2.]);

    canvas.save();
    if angle.abs() > f32::EPSILON {
        canvas.rotate(angle.to_degrees(), Some(center));
    }
    if fill != Color::TRANSPARENT {
        let mut paint = Paint::default();
        paint.set_anti_alias(true).set_style(PaintStyle::Fill).set_color(fill);
        if ellipse {
            canvas.draw_oval(rect, &paint);
        } else {
            canvas.draw_rect(rect, &paint);
        }
    }
    let mut paint = Paint::default();
    paint
        .set_anti_alias(true)
        .set_style(PaintStyle::Stroke)
        .set_stroke_width(transform.scalar(stroke_width).max(0.5))
        .set_color(stroke);
    if ellipse {
        canvas.draw_oval(rect, &paint);
    } else {
        canvas.draw_rect(rect, &paint);
    }
    canvas.restore();
}

fn draw_polyline(
    canvas: &freya_skia_safe::Canvas,
    element: &Value,
    transform: ExportTransform,
    stroke: Color,
    stroke_width: f32,
    arrow: bool,
) {
    let x = number(element, "x", 0.);
    let y = number(element, "y", 0.);
    let width = number(element, "width", 0.);
    let height = number(element, "height", 0.);
    let angle = number(element, "angle", 0.);
    let mut points = absolute_points(element);
    if points.is_empty() {
        points = vec![[x, y], [x + width, y + height]];
    }
    let points = rotate_points(points, [x + width / 2., y + height / 2.], angle);
    let style = string_value(element, "strokeStyle", "solid");
    for segment in points.windows(2) {
        draw_styled_segment(
            canvas,
            segment[0],
            segment[1],
            transform,
            stroke,
            stroke_width,
            style,
        );
    }

    if !arrow || points.len() < 2 {
        return;
    }
    let start_arrow = element
        .get("startArrowhead")
        .and_then(Value::as_str)
        .is_some_and(|value| value != "none");
    let end_arrow = element
        .get("endArrowhead")
        .and_then(Value::as_str)
        .map(|value| value != "none")
        .unwrap_or(true);
    if start_arrow {
        draw_arrowhead(canvas, points[0], points[1], transform, stroke, stroke_width);
    }
    if end_arrow {
        let end = points.len() - 1;
        draw_arrowhead(
            canvas,
            points[end],
            points[end - 1],
            transform,
            stroke,
            stroke_width,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_styled_segment(
    canvas: &freya_skia_safe::Canvas,
    start: [f32; 2],
    end: [f32; 2],
    transform: ExportTransform,
    stroke: Color,
    stroke_width: f32,
    style: &str,
) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let mut paint = Paint::default();
    paint
        .set_anti_alias(true)
        .set_style(PaintStyle::Stroke)
        .set_stroke_width(transform.scalar(stroke_width).max(0.5))
        .set_color(stroke);

    if style != "dashed" && style != "dotted" {
        canvas.draw_line(transform.point(start), transform.point(end), &paint);
        return;
    }

    let (dash, gap) = if style == "dashed" {
        (stroke_width * 4.5, stroke_width * 3.)
    } else {
        (stroke_width.max(1.5), stroke_width * 2.5)
    };
    let unit = [dx / length, dy / length];
    let mut offset = 0.;
    while offset < length {
        let piece_end = (offset + dash).min(length);
        let from = [start[0] + unit[0] * offset, start[1] + unit[1] * offset];
        let to = [
            start[0] + unit[0] * piece_end,
            start[1] + unit[1] * piece_end,
        ];
        canvas.draw_line(transform.point(from), transform.point(to), &paint);
        offset += dash + gap;
    }
}

fn draw_arrowhead(
    canvas: &freya_skia_safe::Canvas,
    tip: [f32; 2],
    neighbor: [f32; 2],
    transform: ExportTransform,
    stroke: Color,
    stroke_width: f32,
) {
    let base_angle = (tip[1] - neighbor[1]).atan2(tip[0] - neighbor[0]);
    let length = 12.;
    let mut paint = Paint::default();
    paint
        .set_anti_alias(true)
        .set_style(PaintStyle::Stroke)
        .set_stroke_width(transform.scalar(stroke_width).max(0.5))
        .set_color(stroke);
    for branch_angle in [
        base_angle + 150_f32.to_radians(),
        base_angle - 150_f32.to_radians(),
    ] {
        let end = [
            tip[0] + length * branch_angle.cos(),
            tip[1] + length * branch_angle.sin(),
        ];
        canvas.draw_line(transform.point(tip), transform.point(end), &paint);
    }
}

fn draw_text(
    canvas: &freya_skia_safe::Canvas,
    element: &Value,
    transform: ExportTransform,
    color: Color,
) {
    let text = string_value(element, "text", "");
    if text.is_empty() {
        return;
    }
    let x = number(element, "x", 0.);
    let y = number(element, "y", 0.);
    let width = number(element, "width", 1.).max(1.);
    let height = number(element, "height", 1.).max(1.);
    let font_size = number(element, "fontSize", 20.).max(1.);
    let angle = number(element, "angle", 0.);
    let center = transform.point([x + width / 2., y + height / 2.]);
    let origin = transform.point([x, y + font_size]);
    let mut font = Font::default();
    font.set_size(transform.scalar(font_size).max(1.));
    let mut paint = Paint::default();
    paint.set_anti_alias(true).set_style(PaintStyle::Fill).set_color(color);

    canvas.save();
    if angle.abs() > f32::EPSILON {
        canvas.rotate(angle.to_degrees(), Some(center));
    }
    canvas.draw_str(text, origin, &font, &paint);
    canvas.restore();
}

fn absolute_points(element: &Value) -> Vec<[f32; 2]> {
    let x = number(element, "x", 0.);
    let y = number(element, "y", 0.);
    element
        .get("points")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|point| {
            let pair = point.as_array()?;
            Some([
                x + pair.first()?.as_f64()? as f32,
                y + pair.get(1)?.as_f64()? as f32,
            ])
        })
        .collect()
}

fn rotate_points(mut points: Vec<[f32; 2]>, center: [f32; 2], angle: f32) -> Vec<[f32; 2]> {
    if angle.abs() <= f32::EPSILON {
        return points;
    }
    let sin = angle.sin();
    let cos = angle.cos();
    for point in &mut points {
        let x = point[0] - center[0];
        let y = point[1] - center[1];
        *point = [
            center[0] + x * cos - y * sin,
            center[1] + x * sin + y * cos,
        ];
    }
    points
}

fn parse_color(value: &str, opacity: f32) -> Color {
    if value.eq_ignore_ascii_case("transparent") {
        return Color::TRANSPARENT;
    }
    let hex = value.trim().trim_start_matches('#');
    let parsed = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0) * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0) * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0) * 17;
            (r, g, b, 255)
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = if hex.len() == 8 {
                u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
            } else {
                255
            };
            (r, g, b, a)
        }
        _ => (27, 27, 31, 255),
    };
    let alpha = ((parsed.3 as f32) * opacity.clamp(0., 100.) / 100.).round() as u8;
    Color::from_argb(alpha, parsed.0, parsed.1, parsed.2)
}

fn number(value: &Value, key: &str, fallback: f32) -> f32 {
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(fallback)
}

fn bool_value(value: &Value, key: &str, fallback: bool) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn string_value<'a>(value: &'a Value, key: &str, fallback: &'a str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or(fallback)
}

fn embed_scene_metadata(png: &[u8], scene: &Value) -> Result<Vec<u8>, String> {
    if png.len() < PNG_SIGNATURE.len() || &png[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err("Unable to embed Excalidraw scene: Skia returned invalid PNG data".to_owned());
    }
    let metadata = serde_json::to_string(scene)
        .map_err(|error| format!("Unable to encode Excalidraw PNG metadata: {error}"))?;
    let ascii_metadata = ascii_json(&metadata);
    let mut text = Vec::with_capacity(EXCALIDRAW_PNG_KEYWORD.len() + 1 + ascii_metadata.len());
    text.extend_from_slice(EXCALIDRAW_PNG_KEYWORD.as_bytes());
    text.push(0);
    text.extend_from_slice(ascii_metadata.as_bytes());
    let chunk = png_chunk(*b"tEXt", &text)?;

    let mut cursor = PNG_SIGNATURE.len();
    while cursor + 12 <= png.len() {
        let length = u32::from_be_bytes(
            png[cursor..cursor + 4]
                .try_into()
                .map_err(|_| "Unable to inspect PNG chunk length".to_owned())?,
        ) as usize;
        let end = cursor
            .checked_add(12)
            .and_then(|value| value.checked_add(length))
            .ok_or_else(|| "Unable to inspect PNG: chunk length overflow".to_owned())?;
        if end > png.len() {
            return Err("Unable to inspect PNG: truncated chunk".to_owned());
        }
        if &png[cursor + 4..cursor + 8] == b"IEND" {
            let mut output = Vec::with_capacity(png.len() + chunk.len());
            output.extend_from_slice(&png[..cursor]);
            output.extend_from_slice(&chunk);
            output.extend_from_slice(&png[cursor..]);
            return Ok(output);
        }
        cursor = end;
    }
    Err("Unable to embed Excalidraw scene: PNG IEND chunk missing".to_owned())
}

fn ascii_json(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii() {
            output.push(character);
        } else {
            for unit in character.encode_utf16(&mut [0; 2]).iter() {
                output.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    output
}

fn png_chunk(kind: [u8; 4], data: &[u8]) -> Result<Vec<u8>, String> {
    let length = u32::try_from(data.len())
        .map_err(|_| "Unable to embed Excalidraw scene: metadata too large".to_owned())?;
    let mut output = Vec::with_capacity(data.len() + 12);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(&kind);
    output.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(kind.len() + data.len());
    crc_input.extend_from_slice(&kind);
    crc_input.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    Ok(output)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn metadata_from_png(png: &[u8]) -> Option<String> {
        let mut cursor = PNG_SIGNATURE.len();
        while cursor + 12 <= png.len() {
            let length = u32::from_be_bytes(png[cursor..cursor + 4].try_into().ok()?) as usize;
            let end = cursor.checked_add(12)?.checked_add(length)?;
            if end > png.len() {
                return None;
            }
            if &png[cursor + 4..cursor + 8] == b"tEXt" {
                let data = &png[cursor + 8..cursor + 8 + length];
                let zero = data.iter().position(|byte| *byte == 0)?;
                if &data[..zero] == EXCALIDRAW_PNG_KEYWORD.as_bytes() {
                    return String::from_utf8(data[zero + 1..].to_vec()).ok();
                }
            }
            cursor = end;
        }
        None
    }

    #[test]
    fn native_export_is_png_and_embeds_recoverable_excalidraw_json() {
        let scene = json!({
            "type":"excalidraw",
            "version":2,
            "title":"éléphant 🐘",
            "elements":[
                {"id":"r","type":"rectangle","x":10,"y":20,"width":80,"height":40,"strokeColor":"#111111","backgroundColor":"#ffe066","strokeWidth":2,"opacity":100},
                {"id":"e","type":"ellipse","x":110,"y":20,"width":60,"height":40,"angle":0.2,"strokeColor":"#6965db","backgroundColor":"transparent","strokeWidth":3,"opacity":90},
                {"id":"a","type":"arrow","x":20,"y":100,"width":120,"height":40,"points":[[0,0],[120,40]],"strokeColor":"#e03131","strokeWidth":2,"strokeStyle":"dashed","endArrowhead":"arrow","opacity":100},
                {"id":"t","type":"text","x":20,"y":160,"width":180,"height":30,"text":"Café 🐘","fontSize":20,"strokeColor":"#1b1b1f","opacity":100}
            ],
            "appState":{"viewBackgroundColor":"#ffffff","exportBackground":true,"exportEmbedScene":true},
            "files":{}
        });
        let raw = serde_json::to_string(&scene).unwrap();
        let png = render_png(&raw).unwrap();
        assert_eq!(&png[..PNG_SIGNATURE.len()], PNG_SIGNATURE);
        let metadata = metadata_from_png(&png).expect("embedded Excalidraw metadata");
        assert!(metadata.is_ascii());
        let decoded: Value = serde_json::from_str(&metadata).unwrap();
        assert_eq!(decoded["type"], "excalidraw");
        assert_eq!(decoded["title"], "éléphant 🐘");
        assert_eq!(decoded["elements"][3]["text"], "Café 🐘");
    }

    #[test]
    fn empty_scene_still_exports_a_valid_preview() {
        let png = render_png(r#"{"type":"excalidraw","elements":[],"appState":{"viewBackgroundColor":"#ffffff"},"files":{}}"#).unwrap();
        assert_eq!(&png[..PNG_SIGNATURE.len()], PNG_SIGNATURE);
        assert!(png.windows(4).any(|chunk| chunk == b"IEND"));
    }

    #[test]
    fn invalid_scene_is_rejected_before_rasterization() {
        assert!(render_png(r#"{"type":"other","elements":[]}"#).is_err());
        assert!(render_png("not-json").is_err());
    }
}
