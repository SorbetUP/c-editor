use crate::{crop_source_rect, svg, DrawingElement, DrawingScene};
use std::fmt::Write;

pub use crate::svg::SvgRenderOptions;

/// Render through the deterministic structural renderer, then upgrade image
/// placeholders when the Excalidraw `files` table contains the referenced
/// binary `dataURL`. Missing/corrupt assets deliberately keep the old visual
/// placeholder so exports stay deterministic and non-destructive.
pub fn render_scene_svg(scene: &DrawingScene, options: &SvgRenderOptions) -> String {
    let mut output = svg::render_scene_svg(scene, options);
    for element in scene
        .visible_elements()
        .filter(|element| element.kind == "image")
    {
        let Some(data_url) = image_data_url(scene, element) else {
            continue;
        };
        replace_image_placeholder(&mut output, element, data_url);
    }
    output
}

pub fn image_data_url<'a>(scene: &'a DrawingScene, element: &DrawingElement) -> Option<&'a str> {
    if element.kind != "image" || element.is_deleted {
        return None;
    }
    let file_id = element.extra.get("fileId")?.as_str()?;
    let file = scene.files.as_object()?.get(file_id)?;
    let data_url = file.get("dataURL")?.as_str()?;
    data_url.starts_with("data:").then_some(data_url)
}

fn replace_image_placeholder(output: &mut String, element: &DrawingElement, data_url: &str) {
    let escaped_id = escape(&element.id);
    let start_marker = format!("<g id=\"{escaped_id}\"");
    let Some(start) = output.find(&start_marker) else {
        return;
    };
    let Some(relative_end) = output[start..].find("</g>") else {
        return;
    };
    let end = start + relative_end + "</g>".len();
    let mut replacement = String::new();
    if let Some(crop) = crop_source_rect(element) {
        let _ = write!(
            replacement,
            "<g id=\"{}\"{}><svg x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\" overflow=\"hidden\"><image href=\"{}\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" opacity=\"{}\" preserveAspectRatio=\"none\"/></svg></g>",
            escaped_id,
            transform(element),
            number(element.x),
            number(element.y),
            number(element.width.abs()),
            number(element.height.abs()),
            number(crop.x),
            number(crop.y),
            number(crop.width),
            number(crop.height),
            escape(data_url),
            number(crop.natural_width),
            number(crop.natural_height),
            number((element.opacity / 100.0).clamp(0.0, 1.0)),
        );
    } else {
        let _ = write!(
            replacement,
            "<g id=\"{}\"{}><image href=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" opacity=\"{}\" preserveAspectRatio=\"none\"/></g>",
            escaped_id,
            transform(element),
            escape(data_url),
            number(element.x),
            number(element.y),
            number(element.width.abs()),
            number(element.height.abs()),
            number((element.opacity / 100.0).clamp(0.0, 1.0)),
        );
    }
    output.replace_range(start..end, &replacement);
}

fn transform(element: &DrawingElement) -> String {
    if element.angle.abs() <= f32::EPSILON {
        return String::new();
    }
    let (x, y, width, height) = element.bounds();
    format!(
        " transform=\"rotate({} {} {})\"",
        number(element.angle.to_degrees()),
        number(x + width / 2.0),
        number(y + height / 2.0),
    )
}

fn number(value: f32) -> String {
    if !value.is_finite() {
        return "0".to_owned();
    }
    let mut value = format!("{value:.3}");
    while value.contains('.') && value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    if value == "-0" {
        value = "0".to_owned();
    }
    value
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
