use crate::{
    create_element, recognize_shape, recognized_arrow_endpoint, DrawingCurrentStyle, DrawingElement,
    DrawingTool, RecognizedShape,
};
use serde_json::{json, Value};

pub fn convert_autoshape(
    points: &[[f32; 2]],
    zoom: f32,
    previous_was_arrow: bool,
    id: impl Into<String>,
    style: &DrawingCurrentStyle,
    frame_id: Option<&str>,
    group_ids: &[String],
) -> Option<DrawingElement> {
    let first = *points.first()?;
    let recognition = recognize_shape(points, zoom, previous_was_arrow);
    let id = id.into();

    let mut element = match recognition.shape {
        RecognizedShape::Rectangle | RecognizedShape::Diamond | RecognizedShape::Ellipse => {
            let tool = match recognition.shape {
                RecognizedShape::Rectangle => DrawingTool::Rectangle,
                RecognizedShape::Diamond => DrawingTool::Diamond,
                RecognizedShape::Ellipse => DrawingTool::Ellipse,
                _ => unreachable!(),
            };
            let [min_x, min_y, max_x, max_y] = recognition.bounds;
            let mut element = create_element(tool, [min_x, min_y], id);
            element.width = (max_x - min_x).max(0.0);
            element.height = (max_y - min_y).max(0.0);
            element
        }
        RecognizedShape::Line => {
            let end = *points.last()?;
            linear_element(DrawingTool::Line, first, end, id)
        }
        RecognizedShape::Arrow => {
            let end = recognized_arrow_endpoint(points, recognition.bounds)?;
            let tool = if distance(first, end) < 60.0 {
                DrawingTool::Line
            } else {
                DrawingTool::Arrow
            };
            linear_element(tool, first, end, id)
        }
        RecognizedShape::FreeDraw => {
            let mut element = create_element(DrawingTool::Freehand, first, id);
            element.points = points
                .iter()
                .map(|point| [point[0] - first[0], point[1] - first[1]])
                .collect();
            update_linear_size(&mut element);
            element
        }
    };

    style.apply_to(&mut element);
    if recognition.shape == RecognizedShape::FreeDraw {
        element.start_arrowhead = None;
        element.end_arrowhead = None;
    }
    if element.kind == "line" {
        element.start_arrowhead = None;
        element.end_arrowhead = None;
        element.extra.insert("polygon".to_owned(), json!(false));
    }
    element.extra.insert(
        "frameId".to_owned(),
        frame_id
            .map(|value| Value::String(value.to_owned()))
            .unwrap_or(Value::Null),
    );
    element.extra.insert("groupIds".to_owned(), json!(group_ids));
    Some(element)
}

fn linear_element(
    tool: DrawingTool,
    start: [f32; 2],
    end: [f32; 2],
    id: String,
) -> DrawingElement {
    let mut element = create_element(tool, start, id);
    element.points = vec![[0.0, 0.0], [end[0] - start[0], end[1] - start[1]]];
    update_linear_size(&mut element);
    element
}

fn update_linear_size(element: &mut DrawingElement) {
    if element.points.is_empty() {
        element.width = 0.0;
        element.height = 0.0;
        return;
    }
    let mut min_x = 0.0_f32;
    let mut min_y = 0.0_f32;
    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    for [x, y] in &element.points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    element.width = max_x - min_x;
    element.height = max_y - min_y;
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}
