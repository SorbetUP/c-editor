pub use elephant_draw::{rgba, DrawingElement, DrawingScene, HistoryState, Viewport};

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
        checkpointed: bool,
    },
    Pan {
        start_pointer: [f32; 2],
        start_pan: [f32; 2],
    },
    Erase {
        checkpointed: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingCanvasState {
    pub document: DrawingScene,
    pub viewport: Viewport,
    pub active_tool: String,
    selected: Option<usize>,
    interaction: Interaction,
    history: HistoryState,
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
            history: HistoryState::default(),
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
            "Image" => "image",
            "Eraser" => "eraser",
            "Hand" => "hand",
            _ => "selection",
        }
        .to_owned();
        if self.active_tool != next {
            self.active_tool = next;
            self.interaction = Interaction::None;
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub fn selected_element_id(&self) -> Option<&str> {
        self.selected
            .and_then(|index| self.document.elements.get(index))
            .filter(|element| !element.is_deleted)
            .map(|element| element.id.as_str())
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        let Some(document) = self.history.undo(self.document.clone()) else {
            return false;
        };
        self.document = document;
        self.selected = None;
        self.interaction = Interaction::None;
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(document) = self.history.redo(self.document.clone()) else {
            return false;
        };
        self.document = document;
        self.selected = None;
        self.interaction = Interaction::None;
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some(index) = self.selected else {
            return false;
        };
        if self
            .document
            .elements
            .get(index)
            .is_none_or(|element| element.is_deleted)
        {
            return false;
        }
        self.checkpoint();
        let element = &mut self.document.elements[index];
        element.is_deleted = true;
        self.selected = None;
        self.interaction = Interaction::None;
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn cancel_interaction(&mut self) {
        self.selected = None;
        self.interaction = Interaction::None;
        self.set_active_tool_label("Selection");
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        self.viewport.zoom = zoom.clamp(elephant_draw::MIN_ZOOM, elephant_draw::MAX_ZOOM);
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn checkpoint(&mut self) {
        self.history.push(self.document.clone());
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
        match self.active_tool.as_str() {
            "hand" => {
                self.interaction = Interaction::Pan {
                    start_pointer: point,
                    start_pan: self.viewport.pan,
                };
            }
            "selection" => {
                self.selected = self.hit_test(world);
                self.interaction = match self.selected {
                    Some(index) => {
                        let element = &self.document.elements[index];
                        Interaction::MoveElement {
                            index,
                            offset: [world[0] - element.x, world[1] - element.y],
                            checkpointed: false,
                        }
                    }
                    None => Interaction::None,
                };
            }
            "eraser" => {
                let changed = self.erase_world(world, false);
                self.interaction = Interaction::Erase {
                    checkpointed: changed,
                };
            }
            _ => {
                self.interaction = Interaction::None;
            }
        }
    }

    pub(crate) fn move_pointer(&mut self, point: [f32; 2]) {
        let interaction = self.interaction.clone();
        match interaction {
            Interaction::MoveElement {
                index,
                offset,
                checkpointed,
            } => {
                let world = self.to_world(point);
                if !checkpointed {
                    self.checkpoint();
                    self.interaction = Interaction::MoveElement {
                        index,
                        offset,
                        checkpointed: true,
                    };
                }
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
            Interaction::Erase { checkpointed } => {
                let changed = self.erase_world(self.to_world(point), checkpointed);
                if changed && !checkpointed {
                    self.interaction = Interaction::Erase {
                        checkpointed: true,
                    };
                }
            }
            Interaction::None => {}
        }
    }

    pub(crate) fn end_pointer(&mut self) {
        self.interaction = Interaction::None;
    }

    pub(crate) fn zoom_at(&mut self, point: [f32; 2], delta_y: f64) {
        let factor = if delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };
        self.viewport.zoom_at(point, factor);
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn to_world(&self, point: [f32; 2]) -> [f32; 2] {
        self.viewport.to_world(point)
    }

    fn erase_world(&mut self, world: [f32; 2], checkpointed: bool) -> bool {
        let Some(index) = self.hit_test(world) else {
            return false;
        };
        if !checkpointed {
            self.checkpoint();
        }
        let element = &mut self.document.elements[index];
        if element.is_deleted {
            return false;
        }
        element.is_deleted = true;
        if self.selected == Some(index) {
            self.selected = None;
        }
        self.revision = self.revision.wrapping_add(1);
        true
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
