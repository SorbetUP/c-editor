use crate::{DrawingElement, DrawingScene};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct SvgRenderOptions {
    pub width: u32,
    pub height: u32,
    pub background: String,
}

impl Default for SvgRenderOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            background: "#ffffff".to_owned(),
        }
    }
}

pub fn render_scene_svg(scene: &DrawingScene, options: &SvgRenderOptions) -> String {
    let mut output = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">",
        options.width, options.height, options.width, options.height
    );
    if !options.background.eq_ignore_ascii_case("transparent") && !options.background.is_empty() {
        let _ = write!(
            output,
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>",
            escape(&options.background)
        );
    }
    for element in scene.visible_elements() {
        render_element(&mut output, element);
    }
    output.push_str("</svg>");
    output
}

fn render_element(output: &mut String, element: &DrawingElement) {
    let transform = transform(element);
    let style = style(element);
    match element.kind.as_str() {
        "rectangle" => {
            let _ = write!(
                output,
                "<rect id=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{} {}/>",
                escape(&element.id),
                number(element.x),
                number(element.y),
                number(element.width.abs()),
                number(element.height.abs()),
                transform,
                style
            );
        }
        "frame" | "magicframe" | "embeddable" | "iframe" => {
            let _ = write!(
                output,
                "<rect id=\"{}\" data-excalidraw-type=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{} {}/>",
                escape(&element.id),
                escape(&element.kind),
                number(element.x),
                number(element.y),
                number(element.width.abs()),
                number(element.height.abs()),
                transform,
                style
            );
        }
        "ellipse" => {
            let _ = write!(
                output,
                "<ellipse id=\"{}\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"{} {}/>",
                escape(&element.id),
                number(element.x + element.width / 2.0),
                number(element.y + element.height / 2.0),
                number(element.width.abs() / 2.0),
                number(element.height.abs() / 2.0),
                transform,
                style
            );
        }
        "diamond" => {
            let x = element.x;
            let y = element.y;
            let width = element.width;
            let height = element.height;
            let points = format!(
                "{},{} {},{} {},{} {},{}",
                number(x + width / 2.0),
                number(y),
                number(x + width),
                number(y + height / 2.0),
                number(x + width / 2.0),
                number(y + height),
                number(x),
                number(y + height / 2.0)
            );
            let _ = write!(
                output,
                "<polygon id=\"{}\" points=\"{}\"{} {}/>",
                escape(&element.id),
                points,
                transform,
                style
            );
        }
        "line" | "arrow" | "freedraw" => render_linear(output, element, &transform),
        "text" => {
            let _ = write!(
                output,
                "<text id=\"{}\" x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" fill=\"{}\" opacity=\"{}\"{}>{}</text>",
                escape(&element.id),
                number(element.x),
                number(element.y + element.font_size.max(1.0)),
                number(element.font_size.max(1.0)),
                escape(&element.stroke_color),
                number((element.opacity / 100.0).clamp(0.0, 1.0)),
                transform,
                escape(&element.text)
            );
        }
        "image" => {
            let _ = write!(
                output,
                "<g id=\"{}\"{}><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#e9ecef\" stroke=\"#868e96\"/><path d=\"M {} {} L {} {} L {} {}\" fill=\"none\" stroke=\"#868e96\"/></g>",
                escape(&element.id),
                transform,
                number(element.x),
                number(element.y),
                number(element.width.abs()),
                number(element.height.abs()),
                number(element.x),
                number(element.y + element.height),
                number(element.x + element.width * 0.4),
                number(element.y + element.height * 0.55),
                number(element.x + element.width),
                number(element.y + element.height)
            );
        }
        _ => {}
    }
}

fn render_linear(output: &mut String, element: &DrawingElement, transform: &str) {
    let points = if element.points.is_empty() {
        vec![[0.0, 0.0], [element.width, element.height]]
    } else {
        element.points.clone()
    };
    let mut path = String::new();
    for (index, [x, y]) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(
            path,
            "{} {} {}",
            command,
            number(element.x + x),
            number(element.y + y)
        );
        if index + 1 < points.len() {
            path.push(' ');
        }
    }
    let _ = write!(
        output,
        "<path id=\"{}\" d=\"{}\"{} {}/>",
        escape(&element.id),
        path,
        transform,
        linear_style(element)
    );
    if element.kind == "arrow" {
        if let Some(kind) = element
            .start_arrowhead
            .as_deref()
            .filter(|kind| *kind != "none")
        {
            if let Some(window) = points.windows(2).next() {
                render_arrowhead(output, element, kind, window[1], window[0], transform);
            }
        }
        if let Some(kind) = element
            .end_arrowhead
            .as_deref()
            .filter(|kind| *kind != "none")
        {
            if let Some(window) = points.windows(2).last() {
                render_arrowhead(output, element, kind, window[0], window[1], transform);
            }
        }
    }
}

fn render_arrowhead(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    previous: [f32; 2],
    end: [f32; 2],
    transform: &str,
) {
    let dx = end[0] - previous[0];
    let dy = end[1] - previous[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let ux = dx / length;
    let uy = dy / length;
    let end = [element.x + end[0], element.y + end[1]];
    let perpendicular = [-uy, ux];

    match kind {
        "bar" | "cardinality_one" => render_bar(output, element, kind, end, ux, uy, perpendicular, 2.0, transform),
        "cardinality_exactly_one" => {
            render_bar(output, element, kind, end, ux, uy, perpendicular, 2.0, transform);
            render_bar(output, element, kind, end, ux, uy, perpendicular, 8.0, transform);
        }
        "circle" | "dot" => render_circle(output, element, kind, end, ux, uy, true, transform),
        "circle_outline" => render_circle(output, element, kind, end, ux, uy, false, transform),
        "triangle" => render_polygon_head(output, element, kind, end, ux, uy, perpendicular, HeadPolygon::Triangle, true, transform),
        "triangle_outline" => render_polygon_head(output, element, kind, end, ux, uy, perpendicular, HeadPolygon::Triangle, false, transform),
        "diamond" => render_polygon_head(output, element, kind, end, ux, uy, perpendicular, HeadPolygon::Diamond, true, transform),
        "diamond_outline" => render_polygon_head(output, element, kind, end, ux, uy, perpendicular, HeadPolygon::Diamond, false, transform),
        "cardinality_many" | "crowfoot_many" => {
            render_crowfoot(output, element, kind, end, ux, uy, perpendicular, 2.0, transform);
        }
        "cardinality_one_or_many" | "crowfoot_one_or_many" => {
            render_crowfoot(output, element, kind, end, ux, uy, perpendicular, 2.0, transform);
            render_bar(output, element, kind, end, ux, uy, perpendicular, 14.0, transform);
        }
        "cardinality_zero_or_one" => {
            render_circle_at(output, element, kind, end, ux, uy, 8.0, false, transform);
            render_bar(output, element, kind, end, ux, uy, perpendicular, 16.0, transform);
        }
        "cardinality_zero_or_many" => {
            render_circle_at(output, element, kind, end, ux, uy, 8.0, false, transform);
            render_crowfoot(output, element, kind, end, ux, uy, perpendicular, 18.0, transform);
        }
        "crowfoot_one" => {
            render_bar(output, element, kind, end, ux, uy, perpendicular, 2.0, transform);
        }
        _ => render_open_arrow(output, element, kind, end, ux, uy, perpendicular, transform),
    }
}

fn render_open_arrow(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    perpendicular: [f32; 2],
    transform: &str,
) {
    let base = [end[0] - ux * 12.0, end[1] - uy * 12.0];
    let wing = [perpendicular[0] * 5.0, perpendicular[1] * 5.0];
    let _ = write!(
        output,
        "<path data-excalidraw-arrowhead=\"{}\" d=\"M {} {} L {} {} M {} {} L {} {}\"{} {}/>",
        escape(kind),
        number(end[0]),
        number(end[1]),
        number(base[0] + wing[0]),
        number(base[1] + wing[1]),
        number(end[0]),
        number(end[1]),
        number(base[0] - wing[0]),
        number(base[1] - wing[1]),
        transform,
        linear_style(element)
    );
}

fn render_bar(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    perpendicular: [f32; 2],
    offset: f32,
    transform: &str,
) {
    let center = [end[0] - ux * offset, end[1] - uy * offset];
    let wing = [perpendicular[0] * 6.0, perpendicular[1] * 6.0];
    let _ = write!(
        output,
        "<path data-excalidraw-arrowhead=\"{}\" d=\"M {} {} L {} {}\"{} {}/>",
        escape(kind),
        number(center[0] + wing[0]),
        number(center[1] + wing[1]),
        number(center[0] - wing[0]),
        number(center[1] - wing[1]),
        transform,
        linear_style(element)
    );
}

fn render_circle(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    filled: bool,
    transform: &str,
) {
    render_circle_at(output, element, kind, end, ux, uy, 6.0, filled, transform);
}

#[allow(clippy::too_many_arguments)]
fn render_circle_at(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    offset: f32,
    filled: bool,
    transform: &str,
) {
    let center = [end[0] - ux * offset, end[1] - uy * offset];
    let fill = if filled {
        escape(&element.stroke_color)
    } else {
        "none".to_owned()
    };
    let _ = write!(
        output,
        "<circle data-excalidraw-arrowhead=\"{}\" cx=\"{}\" cy=\"{}\" r=\"5\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" opacity=\"{}\"{}/>",
        escape(kind),
        number(center[0]),
        number(center[1]),
        fill,
        escape(&element.stroke_color),
        number(element.stroke_width.max(0.5)),
        number((element.opacity / 100.0).clamp(0.0, 1.0)),
        transform
    );
}

enum HeadPolygon {
    Triangle,
    Diamond,
}

#[allow(clippy::too_many_arguments)]
fn render_polygon_head(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    perpendicular: [f32; 2],
    shape: HeadPolygon,
    filled: bool,
    transform: &str,
) {
    let back = [end[0] - ux * 12.0, end[1] - uy * 12.0];
    let wing = [perpendicular[0] * 6.0, perpendicular[1] * 6.0];
    let points = match shape {
        HeadPolygon::Triangle => format!(
            "{},{} {},{} {},{}",
            number(end[0]),
            number(end[1]),
            number(back[0] + wing[0]),
            number(back[1] + wing[1]),
            number(back[0] - wing[0]),
            number(back[1] - wing[1])
        ),
        HeadPolygon::Diamond => {
            let middle = [end[0] - ux * 6.0, end[1] - uy * 6.0];
            format!(
                "{},{} {},{} {},{} {},{}",
                number(end[0]),
                number(end[1]),
                number(middle[0] + wing[0]),
                number(middle[1] + wing[1]),
                number(back[0]),
                number(back[1]),
                number(middle[0] - wing[0]),
                number(middle[1] - wing[1])
            )
        }
    };
    let fill = if filled {
        escape(&element.stroke_color)
    } else {
        "none".to_owned()
    };
    let _ = write!(
        output,
        "<polygon data-excalidraw-arrowhead=\"{}\" points=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" opacity=\"{}\"{}/>",
        escape(kind),
        points,
        fill,
        escape(&element.stroke_color),
        number(element.stroke_width.max(0.5)),
        number((element.opacity / 100.0).clamp(0.0, 1.0)),
        transform
    );
}

#[allow(clippy::too_many_arguments)]
fn render_crowfoot(
    output: &mut String,
    element: &DrawingElement,
    kind: &str,
    end: [f32; 2],
    ux: f32,
    uy: f32,
    perpendicular: [f32; 2],
    offset: f32,
    transform: &str,
) {
    let junction = [end[0] - ux * offset, end[1] - uy * offset];
    let back = [junction[0] - ux * 10.0, junction[1] - uy * 10.0];
    let wing = [perpendicular[0] * 7.0, perpendicular[1] * 7.0];
    let _ = write!(
        output,
        "<path data-excalidraw-arrowhead=\"{}\" d=\"M {} {} L {} {} M {} {} L {} {} M {} {} L {} {}\"{} {}/>",
        escape(kind),
        number(junction[0]),
        number(junction[1]),
        number(back[0]),
        number(back[1]),
        number(junction[0]),
        number(junction[1]),
        number(back[0] + wing[0]),
        number(back[1] + wing[1]),
        number(junction[0]),
        number(junction[1]),
        number(back[0] - wing[0]),
        number(back[1] - wing[1]),
        transform,
        linear_style(element)
    );
}

fn style(element: &DrawingElement) -> String {
    format!(
        "fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.background_color),
        escape(&element.stroke_color),
        number(element.stroke_width.max(0.0)),
        dash_attribute(&element.stroke_style),
        number((element.opacity / 100.0).clamp(0.0, 1.0))
    )
}

fn linear_style(element: &DrawingElement) -> String {
    format!(
        "fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{} opacity=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
        escape(&element.stroke_color),
        number(element.stroke_width.max(0.0)),
        dash_attribute(&element.stroke_style),
        number((element.opacity / 100.0).clamp(0.0, 1.0))
    )
}

fn dash_attribute(stroke_style: &str) -> &'static str {
    match stroke_style {
        "dashed" => " stroke-dasharray=\"8 6\"",
        "dotted" => " stroke-dasharray=\"2 5\"",
        _ => "",
    }
}

fn transform(element: &DrawingElement) -> String {
    if element.angle.abs() <= f32::EPSILON {
        return String::new();
    }
    let (x, y, width, height) = element.bounds();
    let degrees = element.angle.to_degrees();
    format!(
        " transform=\"rotate({} {} {})\"",
        number(degrees),
        number(x + width / 2.0),
        number(y + height / 2.0)
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
