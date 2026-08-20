use crate::DrawingElement;
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingCurrentStyle {
    pub stroke_color: String,
    pub background_color: String,
    pub stroke_width: f32,
    pub stroke_style: String,
    pub fill_style: String,
    pub opacity: f32,
    pub roughness: u64,
    pub roundness: Option<u64>,
    pub start_arrowhead: Option<String>,
    pub end_arrowhead: Option<String>,
}

impl Default for DrawingCurrentStyle {
    fn default() -> Self {
        Self {
            stroke_color: "#1e1e1e".to_owned(),
            background_color: "transparent".to_owned(),
            stroke_width: 2.0,
            stroke_style: "solid".to_owned(),
            fill_style: "hachure".to_owned(),
            opacity: 100.0,
            roughness: 1,
            roundness: None,
            start_arrowhead: None,
            end_arrowhead: Some("arrow".to_owned()),
        }
    }
}

impl DrawingCurrentStyle {
    pub fn apply_to(&self, element: &mut DrawingElement) {
        element.stroke_color = self.stroke_color.clone();
        element.background_color = self.background_color.clone();
        element.stroke_width = self.stroke_width.max(0.0);
        element.stroke_style = self.stroke_style.clone();
        element.fill_style = self.fill_style.clone();
        element.opacity = self.opacity.clamp(0.0, 100.0);
        element.extra.insert("roughness".to_owned(), json!(self.roughness));
        element.extra.insert(
            "roundness".to_owned(),
            self.roundness
                .map(|kind| json!({"type": kind}))
                .unwrap_or(Value::Null),
        );
        if element.kind == "arrow" {
            element.start_arrowhead = self.start_arrowhead.clone();
            element.end_arrowhead = self.end_arrowhead.clone();
        }
    }
}
