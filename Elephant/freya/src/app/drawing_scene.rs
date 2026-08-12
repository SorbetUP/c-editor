use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

fn default_stroke_color() -> String {
    "#1b1b1f".to_owned()
}

fn default_background_color() -> String {
    "transparent".to_owned()
}

fn default_stroke_width() -> f32 {
    2.
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
    #[serde(rename = "strokeStyle", default)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingTool {
    Selection,
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Freedraw,
    Eraser,
}

impl DrawingTool {
    pub const ALL: [Self; 7] = [
        Self::Selection,
        Self::Rectangle,
        Self::Ellipse,
        Self::Line,
        Self::Arrow,
        Self::Freedraw,
        Self::Eraser,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Selection => "Select",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Line => "Line",
            Self::Arrow => "Arrow",
            Self::Freedraw => "Draw",
            Self::Eraser => "Eraser",
        }
    }
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
        before: Box<DrawingScene>,
    },
    DrawElement {
        index: usize,
        start: [f32; 2],
        before: Box<DrawingScene>,
    },
    Freedraw {
        index: usize,
        before: Box<DrawingScene>,
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
    active_tool: DrawingTool,
    selected: Option<usize>,
    interaction: Interaction,
    undo: Vec<DrawingScene>,
    redo: Vec<DrawingScene>,
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
            active_tool: DrawingTool::Selection,
            selected: None,
            interaction: Interaction::None,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
        }
    }

    pub fn serialize_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.document).map_err(|error| error.to_string())
    }

    pub fn active_tool(&self) -> DrawingTool {
        self.active_tool
    }

    pub fn set_tool(&mut self, tool: DrawingTool) {
        self.cancel_interaction();
        self.active_tool = tool;
        if tool != DrawingTool::Selection {
            self.selected = None;
        }
        self.bump_revision();
    }

    pub fn selected_element_id(&self) -> Option<&str> {
        self.selected
            .and_then(|index| self.document.elements.get(index))
            .map(|element| element.id.as_str())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn canvas_background(&self) -> &str {
        self.document
            .app_state
            .get("viewBackgroundColor")
            .and_then(Value::as_str)
            .unwrap_or("#ffffff")
    }

    pub fn is_dark_canvas(&self) -> bool {
        let [r, g, b, _] = rgba(self.canvas_background(), 100.);
        (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) < 128_000
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
        match self.active_tool {
            DrawingTool::Selection => self.begin_selection(point, world),
            DrawingTool::Eraser => self.erase_at(world),
            DrawingTool::Rectangle
            | DrawingTool::Ellipse
            | DrawingTool::Line
            | DrawingTool::Arrow => self.begin_shape(world),
            DrawingTool::Freedraw => self.begin_freedraw(world),
        }
    }

    fn begin_selection(&mut self, point: [f32; 2], world: [f32; 2]) {
        self.selected = self.hit_test(world);
        self.interaction = match self.selected {
            Some(index) => {
                let element = &self.document.elements[index];
                Interaction::MoveElement {
                    index,
                    offset: [world[0] - element.x, world[1] - element.y],
                    before: Box::new(self.document.clone()),
                }
            }
            None => Interaction::Pan {
                start_pointer: point,
                start_pan: self.viewport.pan,
            },
        };
        self.bump_revision();
    }

    fn begin_shape(&mut self, world: [f32; 2]) {
        let before = Box::new(self.document.clone());
        let kind = match self.active_tool {
            DrawingTool::Rectangle => "rectangle",
            DrawingTool::Ellipse => "ellipse",
            DrawingTool::Line => "line",
            DrawingTool::Arrow => "arrow",
            _ => return,
        };
        self.document
            .elements
            .push(new_element(kind, world, self.document.elements.len()));
        let index = self.document.elements.len() - 1;
        self.selected = Some(index);
        self.interaction = Interaction::DrawElement {
            index,
            start: world,
            before,
        };
        self.bump_revision();
    }

    fn begin_freedraw(&mut self, world: [f32; 2]) {
        let before = Box::new(self.document.clone());
        let mut element = new_element("freedraw", world, self.document.elements.len());
        element.points.push([0., 0.]);
        self.document.elements.push(element);
        let index = self.document.elements.len() - 1;
        self.selected = Some(index);
        self.interaction = Interaction::Freedraw { index, before };
        self.bump_revision();
    }

    fn erase_at(&mut self, world: [f32; 2]) {
        let Some(index) = self.hit_test(world) else {
            return;
        };
        let before = self.document.clone();
        if let Some(element) = self.document.elements.get_mut(index) {
            element.is_deleted = true;
            touch_element(element);
        }
        self.selected = None;
        self.commit_history(before);
    }

    pub(crate) fn move_pointer(&mut self, point: [f32; 2]) {
        let world = self.to_world(point);
        match self.interaction.clone() {
            Interaction::MoveElement { index, offset, .. } => {
                if let Some(element) = self.document.elements.get_mut(index) {
                    element.x = world[0] - offset[0];
                    element.y = world[1] - offset[1];
                    self.bump_revision();
                }
            }
            Interaction::DrawElement { index, start, .. } => {
                if let Some(element) = self.document.elements.get_mut(index) {
                    match element.kind.as_str() {
                        "rectangle" | "ellipse" => {
                            element.x = start[0].min(world[0]);
                            element.y = start[1].min(world[1]);
                            element.width = (world[0] - start[0]).abs();
                            element.height = (world[1] - start[1]).abs();
                        }
                        "line" | "arrow" => {
                            element.x = start[0];
                            element.y = start[1];
                            element.width = (world[0] - start[0]).abs();
                            element.height = (world[1] - start[1]).abs();
                            element.points =
                                vec![[0., 0.], [world[0] - start[0], world[1] - start[1]]];
                        }
                        _ => {}
                    }
                    self.bump_revision();
                }
            }
            Interaction::Freedraw { index, .. } => {
                if let Some(element) = self.document.elements.get_mut(index) {
                    let relative = [world[0] - element.x, world[1] - element.y];
                    let append = element
                        .points
                        .last()
                        .map(|last| distance(*last, relative) >= 0.75)
                        .unwrap_or(true);
                    if append {
                        element.points.push(relative);
                        let (_, _, width, height) = element.bounds();
                        element.width = width;
                        element.height = height;
                        self.bump_revision();
                    }
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
                self.bump_revision();
            }
            Interaction::None => {}
        }
    }

    pub(crate) fn end_pointer(&mut self) {
        let interaction = std::mem::replace(&mut self.interaction, Interaction::None);
        match interaction {
            Interaction::MoveElement { index, before, .. }
            | Interaction::DrawElement { index, before, .. }
            | Interaction::Freedraw { index, before } => {
                if before.as_ref() != &self.document {
                    if let Some(element) = self.document.elements.get_mut(index) {
                        touch_element(element);
                    }
                    self.undo.push(*before);
                    self.redo.clear();
                }
            }
            Interaction::Pan { .. } | Interaction::None => {}
        }
        self.bump_revision();
    }

    pub fn escape(&mut self) {
        self.cancel_interaction();
        self.selected = None;
        self.active_tool = DrawingTool::Selection;
        self.bump_revision();
    }

    fn cancel_interaction(&mut self) {
        match std::mem::replace(&mut self.interaction, Interaction::None) {
            Interaction::MoveElement { before, .. }
            | Interaction::DrawElement { before, .. }
            | Interaction::Freedraw { before, .. } => self.document = *before,
            Interaction::Pan { start_pan, .. } => self.viewport.pan = start_pan,
            Interaction::None => {}
        }
    }

    pub fn delete_selected(&mut self) -> bool {
        let Some(index) = self.selected else {
            return false;
        };
        if self
            .document
            .elements
            .get(index)
            .map(|element| element.is_deleted)
            .unwrap_or(true)
        {
            return false;
        }
        let before = self.document.clone();
        if let Some(element) = self.document.elements.get_mut(index) {
            element.is_deleted = true;
            touch_element(element);
        }
        self.selected = None;
        self.commit_history(before);
        true
    }

    pub fn undo(&mut self) -> bool {
        self.interaction = Interaction::None;
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.document, previous));
        self.selected = None;
        self.bump_revision();
        true
    }

    pub fn redo(&mut self) -> bool {
        self.interaction = Interaction::None;
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.document, next));
        self.selected = None;
        self.bump_revision();
        true
    }

    fn commit_history(&mut self, before: DrawingScene) {
        if before != self.document {
            self.undo.push(before);
            self.redo.clear();
            self.bump_revision();
        }
    }

    pub(crate) fn zoom_at(&mut self, point: [f32; 2], delta_y: f64) {
        let before = self.to_world(point);
        let factor = if delta_y < 0. { 1.1 } else { 1. / 1.1 };
        self.viewport.zoom = (self.viewport.zoom * factor as f32).clamp(0.2, 5.);
        self.viewport.pan = [
            point[0] - before[0] * self.viewport.zoom,
            point[1] - before[1] * self.viewport.zoom,
        ];
        self.bump_revision();
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

    fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn next_nonce(previous: u64) -> u32 {
    (previous
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223)
        & 0x7fff_ffff) as u32
}

fn touch_element(element: &mut DrawingElement) {
    let now = timestamp_millis();
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .saturating_add(1);
    let previous_nonce = element
        .extra
        .get("versionNonce")
        .and_then(Value::as_u64)
        .unwrap_or(now);
    element.extra.insert("version".into(), json!(version));
    element
        .extra
        .insert("versionNonce".into(), json!(next_nonce(previous_nonce)));
    element.extra.insert("updated".into(), json!(now));
}

fn new_element(kind: &str, origin: [f32; 2], ordinal: usize) -> DrawingElement {
    let now = timestamp_millis();
    let nonce = ((now ^ ordinal as u64) & 0x7fff_ffff) as u32;
    let mut extra = Map::new();
    extra.insert("roughness".into(), json!(1));
    extra.insert("seed".into(), json!(nonce));
    extra.insert("version".into(), json!(1));
    extra.insert("versionNonce".into(), json!(next_nonce(u64::from(nonce))));
    extra.insert("groupIds".into(), json!([]));
    extra.insert("frameId".into(), Value::Null);
    extra.insert("boundElements".into(), Value::Null);
    extra.insert("updated".into(), json!(now));
    extra.insert("link".into(), Value::Null);
    extra.insert("locked".into(), json!(false));

    DrawingElement {
        id: format!("freya-{now:x}-{ordinal:x}"),
        kind: kind.to_owned(),
        x: origin[0],
        y: origin[1],
        width: 0.,
        height: 0.,
        points: Vec::new(),
        text: String::new(),
        stroke_color: "#1b1b1f".to_owned(),
        background_color: "transparent".to_owned(),
        stroke_width: 2.,
        stroke_style: "solid".to_owned(),
        fill_style: "solid".to_owned(),
        opacity: 100.,
        angle: 0.,
        font_size: 20.,
        end_arrowhead: (kind == "arrow").then(|| "arrow".to_owned()),
        start_arrowhead: None,
        is_deleted: false,
        extra,
    }
}

impl DrawingElement {
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if matches!(self.kind.as_str(), "line" | "arrow" | "freedraw")
            && !self.points.is_empty()
        {
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
            return (
                self.x + min_x,
                self.y + min_y,
                max_x - min_x,
                max_y - min_y,
            );
        }
        let x2 = self.x + self.width;
        let y2 = self.y + self.height;
        (
            self.x.min(x2),
            self.y.min(y2),
            (x2 - self.x).abs(),
            (y2 - self.y).abs(),
        )
    }

    fn hit_test(&self, point: [f32; 2]) -> bool {
        let (x, y, width, height) = self.bounds();
        let padding = self.stroke_width.max(6.);
        if matches!(self.kind.as_str(), "line" | "arrow" | "freedraw")
            && self.points.len() >= 2
        {
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

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn distance_to_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let vector = [end[0] - start[0], end[1] - start[1]];
    let length_squared = vector[0] * vector[0] + vector[1] * vector[1];
    if length_squared <= f32::EPSILON {
        return distance(point, start);
    }
    let t = (((point[0] - start[0]) * vector[0] + (point[1] - start[1]) * vector[1])
        / length_squared)
        .clamp(0., 1.);
    let projected = [start[0] + t * vector[0], start[1] + t * vector[1]];
    distance(point, projected)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_state() -> DrawingCanvasState {
        DrawingCanvasState::from_json(
            r##"{
              "type":"excalidraw",
              "version":2,
              "source":"https://excalidraw.com",
              "elements":[],
              "appState":{"viewBackgroundColor":"#121212"},
              "files":{},
              "futureField":{"keep":true}
            }"##,
        )
        .unwrap()
    }

    #[test]
    fn scene_serialization_preserves_unknown_excalidraw_fields() {
        let mut state = empty_state();
        state.set_tool(DrawingTool::Rectangle);
        state.begin_pointer([20., 30.]);
        state.move_pointer([80., 90.]);
        state.end_pointer();
        let value: Value = serde_json::from_str(&state.serialize_json().unwrap()).unwrap();
        assert_eq!(value["type"], "excalidraw");
        assert_eq!(value["version"], 2);
        assert_eq!(value["futureField"]["keep"], true);
        assert_eq!(value["elements"][0]["type"], "rectangle");
        assert_eq!(value["elements"][0]["strokeStyle"], "solid");
        assert_eq!(value["elements"][0]["version"], 2);
        assert!(value["elements"][0]["updated"].as_u64().is_some());
    }

    #[test]
    fn reverse_drag_rectangle_normalizes_geometry() {
        let mut state = empty_state();
        state.set_tool(DrawingTool::Rectangle);
        state.begin_pointer([100., 90.]);
        state.move_pointer([20., 10.]);
        state.end_pointer();
        assert_eq!(state.document.elements[0].bounds(), (20., 10., 80., 80.));
    }

    #[test]
    fn selection_move_delete_undo_redo_roundtrip() {
        let mut state = empty_state();
        state.set_tool(DrawingTool::Rectangle);
        state.begin_pointer([10., 10.]);
        state.move_pointer([60., 50.]);
        state.end_pointer();
        let id = state.document.elements[0].id.clone();
        state.set_tool(DrawingTool::Selection);
        state.begin_pointer([20., 20.]);
        assert_eq!(state.selected_element_id(), Some(id.as_str()));
        state.move_pointer([40., 35.]);
        state.end_pointer();
        assert_eq!(state.document.elements[0].x, 30.);
        assert_eq!(state.document.elements[0].y, 25.);
        assert_eq!(state.document.elements[0].extra["version"], 3);
        assert!(state.delete_selected());
        assert!(state.document.elements[0].is_deleted);
        assert_eq!(state.document.elements[0].extra["version"], 4);
        assert!(state.undo());
        assert!(!state.document.elements[0].is_deleted);
        assert!(state.redo());
        assert!(state.document.elements[0].is_deleted);
    }

    #[test]
    fn tool_state_creates_line_arrow_freedraw_and_eraser() {
        let mut state = empty_state();
        for tool in [
            DrawingTool::Line,
            DrawingTool::Arrow,
            DrawingTool::Freedraw,
        ] {
            state.set_tool(tool);
            state.begin_pointer([0., 0.]);
            state.move_pointer([25., 15.]);
            state.move_pointer([40., 35.]);
            state.end_pointer();
        }
        assert_eq!(state.document.elements[0].kind, "line");
        assert_eq!(state.document.elements[1].kind, "arrow");
        assert_eq!(state.document.elements[2].kind, "freedraw");
        assert!(state.document.elements[2].points.len() >= 3);
        state.set_tool(DrawingTool::Eraser);
        state.begin_pointer([20., 17.]);
        assert!(state.document.elements[0].is_deleted);
    }

    #[test]
    fn geometry_handles_negative_coordinates_and_zoom_is_bounded() {
        let mut state = empty_state();
        state.set_tool(DrawingTool::Ellipse);
        state.begin_pointer([-120., -80.]);
        state.move_pointer([-20., -10.]);
        state.end_pointer();
        assert_eq!(
            state.document.elements[0].bounds(),
            (-120., -80., 100., 70.)
        );
        for _ in 0..200 {
            state.zoom_at([0., 0.], -1.);
        }
        assert_eq!(state.viewport.zoom, 5.);
        for _ in 0..400 {
            state.zoom_at([0., 0.], 1.);
        }
        assert_eq!(state.viewport.zoom, 0.2);
    }

    #[test]
    fn escape_cancels_in_progress_shape_without_corrupting_scene() {
        let mut state = empty_state();
        state.set_tool(DrawingTool::Rectangle);
        state.begin_pointer([10., 10.]);
        state.move_pointer([100., 100.]);
        state.escape();
        assert!(state.document.elements.is_empty());
        assert_eq!(state.active_tool(), DrawingTool::Selection);
    }
}
