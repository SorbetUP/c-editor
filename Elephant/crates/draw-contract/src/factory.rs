use crate::{DrawingElement, DrawingTool};
use serde_json::{json, Map, Value};

pub fn create_element(
    tool: DrawingTool,
    start: [f32; 2],
    id: impl Into<String>,
) -> DrawingElement {
    let mut extra = Map::new();
    extra.insert("roundness".to_owned(), Value::Null);
    extra.insert("roughness".to_owned(), json!(1));
    extra.insert("seed".to_owned(), json!(1));
    extra.insert("version".to_owned(), json!(1));
    extra.insert("versionNonce".to_owned(), json!(1));
    extra.insert("index".to_owned(), Value::Null);
    extra.insert("groupIds".to_owned(), json!([]));
    extra.insert("frameId".to_owned(), Value::Null);
    extra.insert("boundElements".to_owned(), Value::Null);
    extra.insert("updated".to_owned(), json!(0));
    extra.insert("link".to_owned(), Value::Null);
    extra.insert("locked".to_owned(), json!(false));

    if tool == DrawingTool::Text {
        extra.insert("fontFamily".to_owned(), json!(5));
        extra.insert("textAlign".to_owned(), json!("left"));
        extra.insert("verticalAlign".to_owned(), json!("top"));
        extra.insert("containerId".to_owned(), Value::Null);
        extra.insert("originalText".to_owned(), json!(""));
        extra.insert("autoResize".to_owned(), json!(true));
        extra.insert("lineHeight".to_owned(), json!(1.25));
    }
    if tool == DrawingTool::Image {
        extra.insert("fileId".to_owned(), Value::Null);
        extra.insert("status".to_owned(), json!("pending"));
        extra.insert("scale".to_owned(), json!([1, 1]));
        extra.insert("crop".to_owned(), Value::Null);
    }
    if tool == DrawingTool::Line {
        extra.insert("polygon".to_owned(), json!(false));
        extra.insert("startBinding".to_owned(), Value::Null);
        extra.insert("endBinding".to_owned(), Value::Null);
    }
    if tool == DrawingTool::Arrow {
        extra.insert("elbowed".to_owned(), json!(false));
        extra.insert("startBinding".to_owned(), Value::Null);
        extra.insert("endBinding".to_owned(), Value::Null);
    }

    DrawingElement {
        id: id.into(),
        kind: tool.id().to_owned(),
        x: start[0],
        y: start[1],
        width: 1.0,
        height: 1.0,
        points: if matches!(
            tool,
            DrawingTool::Freehand | DrawingTool::Line | DrawingTool::Arrow
        ) {
            vec![[0.0, 0.0]]
        } else {
            Vec::new()
        },
        text: String::new(),
        stroke_color: "#000000".to_owned(),
        background_color: "transparent".to_owned(),
        stroke_width: 2.0,
        stroke_style: "solid".to_owned(),
        fill_style: "solid".to_owned(),
        opacity: 100.0,
        angle: 0.0,
        font_size: 20.0,
        end_arrowhead: (tool == DrawingTool::Arrow).then(|| "arrow".to_owned()),
        start_arrowhead: None,
        is_deleted: false,
        extra,
    }
}
