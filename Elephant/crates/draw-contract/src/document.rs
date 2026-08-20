use crate::geometry::{distance_to_segment, rotate_point};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;

fn default_stroke() -> String {
    "#1e1e1e".to_owned()
}
fn default_background() -> String {
    "transparent".to_owned()
}
fn default_stroke_width() -> f32 {
    2.0
}
fn default_style() -> String {
    "solid".to_owned()
}
fn default_opacity() -> f32 {
    100.0
}
fn default_font_size() -> f32 {
    20.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrawingScene {
    #[serde(rename = "type", default)]
    pub scene_type: String,
    #[serde(default)]
    pub elements: Vec<DrawingElement>,
    #[serde(rename = "appState", default)]
    pub app_state: Value,
    #[serde(default)]
    pub files: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrawingElement {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub width: f32,
    #[serde(default)]
    pub height: f32,
    #[serde(default)]
    pub points: Vec<[f32; 2]>,
    #[serde(default)]
    pub text: String,
    #[serde(rename = "strokeColor", default = "default_stroke")]
    pub stroke_color: String,
    #[serde(rename = "backgroundColor", default = "default_background")]
    pub background_color: String,
    #[serde(rename = "strokeWidth", default = "default_stroke_width")]
    pub stroke_width: f32,
    #[serde(rename = "strokeStyle", default = "default_style")]
    pub stroke_style: String,
    #[serde(rename = "fillStyle", default = "default_style")]
    pub fill_style: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(rename = "fontSize", default = "default_font_size")]
    pub font_size: f32,
    #[serde(rename = "endArrowhead", default)]
    pub end_arrowhead: Option<String>,
    #[serde(rename = "startArrowhead", default)]
    pub start_arrowhead: Option<String>,
    #[serde(rename = "isDeleted", default)]
    pub is_deleted: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl DrawingElement {
    pub fn is_linear(&self) -> bool {
        matches!(self.kind.as_str(), "line" | "arrow" | "freedraw")
    }

    pub fn is_locked(&self) -> bool {
        self.extra
            .get("locked")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if self.points.is_empty() {
            return (
                self.x.min(self.x + self.width),
                self.y.min(self.y + self.height),
                self.width.abs(),
                self.height.abs(),
            );
        }
        let mut min_x = self.x;
        let mut min_y = self.y;
        let mut max_x = self.x;
        let mut max_y = self.y;
        for [px, py] in &self.points {
            min_x = min_x.min(self.x + px);
            min_y = min_y.min(self.y + py);
            max_x = max_x.max(self.x + px);
            max_y = max_y.max(self.y + py);
        }
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn hit_test(&self, point: [f32; 2]) -> bool {
        self.hit_test_with_tolerance(point, 6.0)
    }

    pub fn hit_test_with_tolerance(&self, point: [f32; 2], tolerance: f32) -> bool {
        if self.is_deleted || self.is_locked() {
            return false;
        }
        let tolerance = tolerance.max(0.0);
        let (x, y, width, height) = self.bounds();
        let center = [x + width / 2.0, y + height / 2.0];
        let point = rotate_point(point, center, -self.angle);

        match self.kind.as_str() {
            "line" | "arrow" | "freedraw" => self.hit_test_linear(point, tolerance),
            "ellipse" => {
                let rx = width / 2.0 + tolerance;
                let ry = height / 2.0 + tolerance;
                if rx <= f32::EPSILON || ry <= f32::EPSILON {
                    return false;
                }
                let nx = (point[0] - center[0]) / rx;
                let ny = (point[1] - center[1]) / ry;
                nx * nx + ny * ny <= 1.0
            }
            "diamond" => {
                let rx = width / 2.0 + tolerance;
                let ry = height / 2.0 + tolerance;
                if rx <= f32::EPSILON || ry <= f32::EPSILON {
                    return false;
                }
                ((point[0] - center[0]).abs() / rx) + ((point[1] - center[1]).abs() / ry) <= 1.0
            }
            _ => {
                point[0] >= x - tolerance
                    && point[0] <= x + width + tolerance
                    && point[1] >= y - tolerance
                    && point[1] <= y + height + tolerance
            }
        }
    }

    fn hit_test_linear(&self, point: [f32; 2], tolerance: f32) -> bool {
        let absolute = self
            .points
            .iter()
            .map(|[px, py]| [self.x + px, self.y + py])
            .collect::<Vec<_>>();
        if absolute.len() < 2 {
            return absolute
                .first()
                .map(|only| distance_to_segment(point, *only, *only) <= tolerance)
                .unwrap_or(false);
        }
        absolute
            .windows(2)
            .any(|segment| distance_to_segment(point, segment[0], segment[1]) <= tolerance)
    }
}

#[derive(Debug)]
pub enum SceneError {
    InvalidJson(serde_json::Error),
    InvalidType,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid Excalidraw JSON: {error}"),
            Self::InvalidType => formatter.write_str("expected type=excalidraw"),
        }
    }
}

impl std::error::Error for SceneError {}

impl DrawingScene {
    pub fn empty() -> Self {
        Self {
            scene_type: "excalidraw".to_owned(),
            elements: Vec::new(),
            app_state: Value::Object(Map::new()),
            files: Value::Object(Map::new()),
            extra: Map::new(),
        }
    }

    pub fn from_json(raw: &str) -> Result<Self, SceneError> {
        let scene: Self = serde_json::from_str(raw).map_err(SceneError::InvalidJson)?;
        if scene.scene_type != "excalidraw" {
            return Err(SceneError::InvalidType);
        }
        Ok(scene)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn visible_elements(&self) -> impl DoubleEndedIterator<Item = &DrawingElement> {
        self.elements.iter().filter(|element| !element.is_deleted)
    }

    pub fn element_by_id(&self, id: &str) -> Option<&DrawingElement> {
        self.elements.iter().find(|element| element.id == id)
    }
}
