#!/usr/bin/env python3
from pathlib import Path


path = Path("Elephant/freya/src/app/drawing_render_svg.rs")
text = path.read_text()

old_import = "use elephant_draw::{font_family_css, layout_text, Arrowhead, ArrowheadPrimitive};"
new_import = "use elephant_draw::{rough_shape_paths, font_family_css, layout_text, Arrowhead, ArrowheadPrimitive};"
if new_import not in text:
    if old_import not in text:
        raise SystemExit("rough import anchor missing")
    text = text.replace(old_import, new_import, 1)

text = text.replace(
    "    append_rough_shape(&mut content, tag, &geometry, element);",
    "    append_rough_paths(&mut content, element);",
    1,
)

old_diamond = '''    append_rough_shape(
        &mut content,
        "polygon",
        &format!("points=\\\"{points}\\\""),
        element,
    );'''
if old_diamond in text:
    text = text.replace(old_diamond, "    append_rough_paths(&mut content, element);", 1)

text = text.replace(
    "    append_rough_path(&mut content, &path, element);",
    "    append_rough_paths(&mut content, element);",
    1,
)

old_shape_style = '''fn shape_style(element: &DrawingElement, fill: &str) -> String {
    format!(
        "fill=\\\"{fill}\\\" stroke=\\\"{}\\\" stroke-width=\\\"{}\\\"{} opacity=\\\"{}\\\" stroke-linecap=\\\"round\\\" stroke-linejoin=\\\"round\\\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}'''
new_shape_style = '''fn shape_style(element: &DrawingElement, fill: &str) -> String {
    if uses_shared_rough(element) {
        return format!("fill=\\\"none\\\" stroke=\\\"none\\\" opacity=\\\"{}\\\"", opacity(element));
    }
    format!(
        "fill=\\\"{fill}\\\" stroke=\\\"{}\\\" stroke-width=\\\"{}\\\"{} opacity=\\\"{}\\\" stroke-linecap=\\\"round\\\" stroke-linejoin=\\\"round\\\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}'''
if new_shape_style not in text:
    if old_shape_style not in text:
        raise SystemExit("shape_style anchor missing")
    text = text.replace(old_shape_style, new_shape_style, 1)

old_linear_style = '''fn linear_style(element: &DrawingElement) -> String {
    format!(
        "fill=\\\"none\\\" stroke=\\\"{}\\\" stroke-width=\\\"{}\\\"{} opacity=\\\"{}\\\" stroke-linecap=\\\"round\\\" stroke-linejoin=\\\"round\\\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}'''
new_linear_style = '''fn linear_style(element: &DrawingElement) -> String {
    if uses_shared_rough(element) {
        return format!("fill=\\\"none\\\" stroke=\\\"none\\\" opacity=\\\"{}\\\"", opacity(element));
    }
    format!(
        "fill=\\\"none\\\" stroke=\\\"{}\\\" stroke-width=\\\"{}\\\"{} opacity=\\\"{}\\\" stroke-linecap=\\\"round\\\" stroke-linejoin=\\\"round\\\"",
        escape(&element.stroke_color),
        element.stroke_width.max(0.5),
        dash_attribute(&element.stroke_style),
        opacity(element)
    )
}'''
if new_linear_style not in text:
    if old_linear_style not in text:
        raise SystemExit("linear_style anchor missing")
    text = text.replace(old_linear_style, new_linear_style, 1)

if "fn append_rough_shape(" in text:
    start = text.index("fn append_rough_shape(")
    end = text.index("fn roughness(", start)
    replacement = '''fn uses_shared_rough(element: &DrawingElement) -> bool {
    roughness(element) > f32::EPSILON
        && matches!(element.kind.as_str(), "rectangle" | "ellipse" | "diamond" | "line" | "arrow")
}

fn append_rough_paths(output: &mut String, element: &DrawingElement) {
    for path in rough_shape_paths(element) {
        let stroke = path.stroke.as_deref().unwrap_or("none");
        let fill = path.fill.as_deref().unwrap_or("none");
        let _ = write!(
            output,
            "<path data-excalidraw-rough=\\\"true\\\" d=\\\"{}\\\" fill=\\\"{}\\\" stroke=\\\"{}\\\" stroke-width=\\\"{}\\\" opacity=\\\"{}\\\" stroke-linecap=\\\"round\\\" stroke-linejoin=\\\"round\\\"/>",
            escape(&path.d),
            escape(fill),
            escape(stroke),
            path.stroke_width,
            opacity(element)
        );
    }
}

'''
    text = text[:start] + replacement + text[end:]

if "fn rough_offset(" in text:
    start = text.index("fn rough_offset(")
    end = text.index("fn rough_padding(", start)
    text = text[:start] + text[end:]

path.write_text(text)
