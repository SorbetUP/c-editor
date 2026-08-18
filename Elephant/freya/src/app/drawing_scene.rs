pub use elephant_draw::{rgba, DrawingElement, DrawingScene, Viewport};

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
    pub active_tool: String,
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
            active_tool: "selection".to_owned(),
            selected: None,
            interaction: Interaction::None,
            revision: 0,
        }
    }

    pub fn serialize_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.document).map_err(|error| error.to_string())
    }

    pub fn set_active_tool_label(&mut self, label: &str) {
        let next = match label {
            "Freedraw" | "Freehand" | "Pencil" => "freedraw",
            "Rectangle" => "rectangle",
            "Diamond" => "diamond",
            "Ellipse" => "ellipse",
            "Arrow" => "arrow",
            "Line" => "line",
            "Text" => "text",
            "Eraser" => "eraser",
            "Hand" => "hand",
            _ => "selection",
        }
        .to_owned();
        if self.active_tool != next {
            self.active_tool = next;
            self.revision = self.revision.wrapping_add(1);
        }
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

    pub(crate) fn erase_at(&mut self, point: [f32; 2]) -> bool {
        let world = self.to_world(point);
        if let Some(index) = self.hit_test(world) {
            if let Some(element) = self.document.elements.get_mut(index) {
                if !element.is_deleted {
                    element.is_deleted = true;
                    self.revision = self.revision.wrapping_add(1);
                    return true;
                }
            }
        }
        false
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
