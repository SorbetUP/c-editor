use crate::DrawingElement;
use roughr::core::{FillStyle, Options, PathInfo};
use roughr::generator::Generator;
use roughr::Srgba;
use serde_json::Value;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct RoughPath {
    pub d: String,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub fill: Option<String>,
}

pub fn rough_shape_paths(element: &DrawingElement) -> Vec<RoughPath> {
    let roughness = element
        .extra
        .get("roughness")
        .and_then(Value::as_f64)
        .unwrap_or(1.0) as f32;
    if roughness <= f32::EPSILON || element.is_deleted {
        return Vec::new();
    }

    let mut options = Options::default();
    options.roughness = Some(roughness.clamp(0.0, 3.0));
    options.stroke = parse_color(&element.stroke_color);
    options.stroke_width = Some(element.stroke_width.max(0.5));
    options.seed = Some(
        element
            .extra
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(1),
    );
    options.fill = parse_color(&element.background_color);
    options.fill_style = Some(fill_style(&element.fill_style));
    options.stroke_line_dash = stroke_dash(&element.stroke_style);
    options.line_cap = Some(roughr::core::LineCap::Round);
    options.line_join = Some(roughr::core::LineJoin::Round);

    let generator = Generator::default();
    let options = Some(options);
    let width = element.width.abs().max(0.01);
    let height = element.height.abs().max(0.01);
    let paths = match element.kind.as_str() {
        "rectangle" => Generator::to_paths(generator.rectangle::<f32>(0.0, 0.0, width, height, &options)),
        "ellipse" => Generator::to_paths(generator.ellipse::<f32>(width / 2.0, height / 2.0, width, height, &options)),
        "diamond" => Generator::to_paths(generator.path::<f32>(
            format!(
                "M {} 0 L {} {} L {} {} L 0 {} Z",
                width / 2.0,
                width,
                height / 2.0,
                width / 2.0,
                height,
                height / 2.0
            ),
            &options,
        )),
        "line" | "arrow" => {
            let d = linear_path(element);
            if d.is_empty() {
                Vec::new()
            } else {
                Generator::to_paths(generator.path::<f32>(d, &options))
            }
        }
        _ => Vec::new(),
    };
    paths.into_iter().map(convert_path).collect()
}

fn convert_path(path: PathInfo) -> RoughPath {
    RoughPath {
        d: path.d,
        stroke: path.stroke.map(css_color),
        stroke_width: path.stroke_width.unwrap_or(0.0).max(0.0),
        fill: path.fill.map(css_color),
    }
}

fn fill_style(value: &str) -> FillStyle {
    match value {
        "solid" => FillStyle::Solid,
        "cross-hatch" => FillStyle::CrossHatch,
        "zigzag" => FillStyle::ZigZag,
        "dots" => FillStyle::Dots,
        "dashed" => FillStyle::Dashed,
        "zigzag-line" => FillStyle::ZigZagLine,
        _ => FillStyle::Hachure,
    }
}

fn stroke_dash(value: &str) -> Option<Vec<f64>> {
    match value {
        "dashed" => Some(vec![8.0, 6.0]),
        "dotted" => Some(vec![2.0, 5.0]),
        _ => None,
    }
}

fn linear_path(element: &DrawingElement) -> String {
    let points = if element.points.is_empty() {
        vec![[0.0, 0.0], [element.width, element.height]]
    } else {
        element.points.clone()
    };
    let mut output = String::new();
    for (index, [x, y]) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(output, "{command} {x} {y}");
        if index + 1 < points.len() {
            output.push(' ');
        }
    }
    output
}

fn parse_color(value: &str) -> Option<Srgba> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("transparent") {
        return None;
    }
    let hex = value.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        3 => {
            let mut digits = hex.chars();
            let r = digits.next()?.to_digit(16)? as u8 * 17;
            let g = digits.next()?.to_digit(16)? as u8 * 17;
            let b = digits.next()?.to_digit(16)? as u8 * 17;
            (r, g, b, 255)
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = if hex.len() == 8 {
                u8::from_str_radix(&hex[6..8], 16).ok()?
            } else {
                255
            };
            (r, g, b, a)
        }
        _ => return None,
    };
    Some(Srgba::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ))
}

fn css_color(color: Srgba) -> String {
    let r = (color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = color.alpha.clamp(0.0, 1.0);
    if (a - 1.0).abs() <= f32::EPSILON {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgba({r},{g},{b},{a:.3})")
    }
}
