use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

fn default_stroke_color() -> String {
    "#000000".to_owned()
}
fn default_background_color() -> String {
    "transparent".to_owned()
}
fn default_stroke_width() -> f32 {
    1.
}
fn default_opacity() -> f32 {
    100.
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
    #[serde(rename = "strokeColor", default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(rename = "backgroundColor", default = "default_background_color")]
    pub background_color: String,
    #[serde(rename = "strokeWidth", default = "default_stroke_width")]
    pub stroke_width: f32,
    #[serde(default)]
    pub stroke_style: String,
    #[serde(rename = "fillStyle", default)]
    pub fill_style: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(rename = "fontSize", default)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub zoom: f32,
    pub pan: [f32; 2],
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            zoom: 1.,
            pan: [0., 0.],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderableElement {
    pub index: usize,
    pub kind: String,
    pub bounds: (f32, f32, f32, f32),
    pub stroke_rgba: [u8; 4],
    pub fill_rgba: [u8; 4],
}

#[derive(Clone, Debug, PartialEq)]
enum Interaction {
    None,
    MoveElement {
        index: usize,
        offset: [f32; 2],
    },
    Pan {
        start_pointer: [f32; 2],
        start_pan: [f32; 2],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingCanvasState {
    pub document: DrawingScene,
    pub viewport: Viewport,
    selected: Option<usize>,
    interaction: Interaction,
    pub revision: u64,
}

impl DrawingCanvasState {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let document: DrawingScene =
            serde_json::from_str(raw).map_err(|error| format!("Drawing scene invalid: {error}"))?;
        if document.scene_type != "excalidraw" {
            return Err("Drawing scene invalid: expected type=excalidraw".to_owned());
        }
        Ok(Self::new(document))
    }

    pub fn new(document: DrawingScene) -> Self {
        Self {
            document,
            viewport: Viewport::default(),
            selected: None,
            interaction: Interaction::None,
            revision: 0,
        }
    }

    pub fn serialize_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.document).map_err(|error| error.to_string())
    }

    pub fn selected_element_id(&self) -> Option<&str> {
        self.selected
            .and_then(|index| self.document.elements.get(index))
            .map(|element| element.id.as_str())
    }

    pub fn renderable_elements(&self) -> Vec<RenderableElement> {
        self.document
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !element.is_deleted)
            .map(|(index, element)| RenderableElement {
                index,
                kind: element.kind.clone(),
                bounds: element.bounds(),
                stroke_rgba: rgba(&element.stroke_color, element.opacity),
                fill_rgba: rgba(&element.background_color, element.opacity),
            })
            .collect()
    }

    pub(crate) fn begin_pointer(&mut self, point: [f32; 2]) {
        let world = self.to_world(point);
        self.selected = self.hit_test(world);
        self.interaction = match self.selected {
            Some(index) => {
                let element = &self.document.elements[index];
                Interaction::MoveElement {
                    index,
                    offset: [world[0] - element.x, world[1] - element.y],
                }
            }
            None => Interaction::Pan {
                start_pointer: point,
                start_pan: self.viewport.pan,
            },
        };
    }

    pub(crate) fn move_pointer(&mut self, point: [f32; 2]) {
        let interaction = self.interaction.clone();
        match interaction {
            Interaction::MoveElement { index, offset } => {
                let world = self.to_world(point);
                if let Some(element) = self.document.elements.get_mut(index) {
                    element.x = world[0] - offset[0];
                    element.y = world[1] - offset[1];
                    self.revision = self.revision.wrapping_add(1);
                }
            }
            Interaction::Pan {
                start_pointer,
                start_pan,
            } => {
                self.viewport.pan = [
                    start_pan[0] + point[0] - start_pointer[0],
                    start_pan[1] + point[1] - start_pointer[1],
                ];
                self.revision = self.revision.wrapping_add(1);
            }
            Interaction::None => {}
        }
    }

    pub(crate) fn end_pointer(&mut self) {
        self.interaction = Interaction::None;
    }

    pub(crate) fn zoom_at(&mut self, point: [f32; 2], delta_y: f64) {
        let before = self.to_world(point);
        let factor = if delta_y < 0. { 1.1 } else { 1. / 1.1 };
        self.viewport.zoom = (self.viewport.zoom * factor as f32).clamp(0.2, 5.);
        self.viewport.pan = [
            point[0] - before[0] * self.viewport.zoom,
            point[1] - before[1] * self.viewport.zoom,
        ];
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn to_world(&self, point: [f32; 2]) -> [f32; 2] {
        [
            (point[0] - self.viewport.pan[0]) / self.viewport.zoom,
            (point[1] - self.viewport.pan[1]) / self.viewport.zoom,
        ]
    }

    fn hit_test(&self, point: [f32; 2]) -> Option<usize> {
        self.document
            .elements
            .iter()
            .enumerate()
            .rev()
            .find(|(_, element)| !element.is_deleted && element.hit_test(point))
            .map(|(index, _)| index)
    }
}

impl DrawingElement {
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if matches!(self.kind.as_str(), "line" | "arrow" | "freedraw") && !self.points.is_empty() {
            let min_x = self
                .points
                .iter()
                .map(|point| point[0])
                .fold(f32::INFINITY, f32::min);
            let min_y = self
                .points
                .iter()
                .map(|point| point[1])
                .fold(f32::INFINITY, f32::min);
            let max_x = self
                .points
                .iter()
                .map(|point| point[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let max_y = self
                .points
                .iter()
                .map(|point| point[1])
                .fold(f32::NEG_INFINITY, f32::max);
            return (self.x + min_x, self.y + min_y, max_x - min_x, max_y - min_y);
        }
        (self.x, self.y, self.width, self.height)
    }

    fn hit_test(&self, point: [f32; 2]) -> bool {
        let (x, y, width, height) = self.bounds();
        let padding = self.stroke_width.max(6.);
        if matches!(self.kind.as_str(), "line" | "arrow" | "freedraw") && self.points.len() >= 2 {
            return self.points.windows(2).any(|segment| {
                distance_to_segment(
                    point,
                    [self.x + segment[0][0], self.y + segment[0][1]],
                    [self.x + segment[1][0], self.y + segment[1][1]],
                ) <= padding
            });
        }
        point[0] >= x - padding
            && point[0] <= x + width + padding
            && point[1] >= y - padding
            && point[1] <= y + height + padding
    }
}

fn distance_to_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let vector = [end[0] - start[0], end[1] - start[1]];
    let length_squared = vector[0] * vector[0] + vector[1] * vector[1];
    if length_squared <= f32::EPSILON {
        return ((point[0] - start[0]).powi(2) + (point[1] - start[1]).powi(2)).sqrt();
    }
    let t = (((point[0] - start[0]) * vector[0] + (point[1] - start[1]) * vector[1])
        / length_squared)
        .clamp(0., 1.);
    let projected = [start[0] + t * vector[0], start[1] + t * vector[1]];
    ((point[0] - projected[0]).powi(2) + (point[1] - projected[1]).powi(2)).sqrt()
}

pub(crate) fn rgba(value: &str, opacity: f32) -> [u8; 4] {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") || value.is_empty() {
        return [0, 0, 0, 0];
    }
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (hex, alpha) = match hex.len() {
        3 => {
            let expanded = hex.chars().flat_map(|c| [c, c]).collect::<String>();
            (expanded, 255)
        }
        6 => (hex.to_owned(), 255),
        8 => (
            hex[..6].to_owned(),
            u8::from_str_radix(&hex[6..], 16).unwrap_or(255),
        ),
        _ => return [0, 0, 0, 0],
    };
    [
        u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
        u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
        u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
        ((f32::from(alpha) * (opacity.clamp(0., 100.) / 100.)).round()) as u8,
    ]
}
