use crate::{DrawingElement, DrawingScene, DrawingTool, HistoryState, Viewport};
use serde_json::{json, Map, Value};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionState {
    pub index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawingStylePatch {
    pub stroke_color: Option<String>,
    pub background_color: Option<String>,
    pub stroke_width: Option<f32>,
    pub stroke_style: Option<String>,
    pub fill_style: Option<String>,
    pub opacity: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Gesture {
    Draw {
        index: usize,
        tool: DrawingTool,
        start: [f32; 2],
    },
    Move {
        index: usize,
        offset: [f32; 2],
        checkpointed: bool,
    },
    Pan {
        start: [f32; 2],
        pan: [f32; 2],
    },
    Erase {
        checkpointed: bool,
    },
}

pub struct DrawingEditor {
    pub scene: DrawingScene,
    pub viewport: Viewport,
    pub active_tool: DrawingTool,
    pub selection: SelectionState,
    pub history: HistoryState,
    gesture: Option<Gesture>,
    next_id: u64,
}

impl DrawingEditor {
    pub fn new(scene: DrawingScene) -> Self {
        let next_id = scene.elements.len() as u64;
        Self {
            scene,
            viewport: Viewport::default(),
            active_tool: DrawingTool::Selection,
            selection: SelectionState::default(),
            history: HistoryState::default(),
            gesture: None,
            next_id,
        }
    }

    pub fn set_tool(&mut self, tool: DrawingTool) {
        self.active_tool = tool;
        self.gesture = None;
    }

    pub fn pointer_down(&mut self, point: [f32; 2]) {
        let world = self.viewport.to_world(point);
        match self.active_tool {
            DrawingTool::Selection => {
                self.selection.index = self.hit_test(world);
                self.gesture = self.selection.index.map(|index| {
                    let element = &self.scene.elements[index];
                    Gesture::Move {
                        index,
                        offset: [world[0] - element.x, world[1] - element.y],
                        checkpointed: false,
                    }
                });
            }
            DrawingTool::Hand => {
                self.gesture = Some(Gesture::Pan {
                    start: point,
                    pan: self.viewport.pan,
                });
            }
            DrawingTool::Eraser => {
                let changed = self.erase_world(world, false);
                self.gesture = Some(Gesture::Erase {
                    checkpointed: changed,
                });
            }
            DrawingTool::Image => self.gesture = None,
            tool => {
                self.history.push(self.scene.clone());
                let index = self.scene.elements.len();
                let id = self.allocate_id();
                self.scene.elements.push(create_element(tool, world, id));
                self.selection.index = Some(index);
                self.gesture = Some(Gesture::Draw {
                    index,
                    tool,
                    start: world,
                });
            }
        }
    }

    pub fn pointer_move(&mut self, point: [f32; 2]) {
        let world = self.viewport.to_world(point);
        match self.gesture {
            Some(Gesture::Draw { index, tool, start }) => {
                let Some(element) = self.scene.elements.get_mut(index) else {
                    return;
                };
                match tool {
                    DrawingTool::Freehand => {
                        element
                            .points
                            .push([world[0] - start[0], world[1] - start[1]]);
                        update_linear_dimensions(element);
                    }
                    DrawingTool::Line | DrawingTool::Arrow => {
                        element.points =
                            vec![[0.0, 0.0], [world[0] - start[0], world[1] - start[1]]];
                        update_linear_dimensions(element);
                    }
                    DrawingTool::Text => {
                        element.width = (world[0] - start[0]).abs().max(1.0);
                    }
                    _ => {
                        element.x = start[0].min(world[0]);
                        element.y = start[1].min(world[1]);
                        element.width = (world[0] - start[0]).abs();
                        element.height = (world[1] - start[1]).abs();
                    }
                }
                mark_changed(element);
            }
            Some(Gesture::Move {
                index,
                offset,
                checkpointed,
            }) => {
                if !checkpointed {
                    self.history.push(self.scene.clone());
                    self.gesture = Some(Gesture::Move {
                        index,
                        offset,
                        checkpointed: true,
                    });
                }
                if let Some(element) = self.scene.elements.get_mut(index) {
                    element.x = world[0] - offset[0];
                    element.y = world[1] - offset[1];
                    mark_changed(element);
                }
            }
            Some(Gesture::Pan { start, pan }) => {
                self.viewport.pan = [pan[0] + point[0] - start[0], pan[1] + point[1] - start[1]];
            }
            Some(Gesture::Erase { checkpointed }) => {
                let changed = self.erase_world(world, checkpointed);
                if changed && !checkpointed {
                    self.gesture = Some(Gesture::Erase { checkpointed: true });
                }
            }
            None => {}
        }
    }

    pub fn pointer_up(&mut self) {
        self.gesture = None;
    }

    pub fn cancel_gesture(&mut self) {
        if matches!(
            self.gesture,
            Some(Gesture::Draw { .. })
                | Some(Gesture::Move {
                    checkpointed: true,
                    ..
                })
                | Some(Gesture::Erase { checkpointed: true })
        ) {
            if let Some(scene) = self.history.rollback() {
                self.scene = scene;
            }
        }
        self.gesture = None;
        self.selection.index = None;
    }

    pub fn undo(&mut self) {
        if let Some(scene) = self.history.undo(self.scene.clone()) {
            self.scene = scene;
            self.selection.index = None;
            self.gesture = None;
        }
    }

    pub fn redo(&mut self) {
        if let Some(scene) = self.history.redo(self.scene.clone()) {
            self.scene = scene;
            self.selection.index = None;
            self.gesture = None;
        }
    }

    pub fn selected(&self) -> Option<&DrawingElement> {
        self.selection
            .index
            .and_then(|index| self.scene.elements.get(index))
            .filter(|element| !element.is_deleted)
    }

    pub fn select_by_id(&mut self, id: &str) -> bool {
        self.selection.index = self
            .scene
            .elements
            .iter()
            .position(|element| element.id == id && !element.is_deleted);
        self.selection.index.is_some()
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        if self
            .scene
            .elements
            .get(index)
            .is_none_or(|element| element.is_deleted)
        {
            return false;
        }
        self.history.push(self.scene.clone());
        let element = &mut self.scene.elements[index];
        element.is_deleted = true;
        mark_changed(element);
        self.selection.index = None;
        true
    }

    pub fn duplicate_selection(&mut self, offset: [f32; 2]) -> Option<&DrawingElement> {
        let index = self.selection.index?;
        let mut duplicate = self.scene.elements.get(index)?.clone();
        if duplicate.is_deleted {
            return None;
        }
        self.history.push(self.scene.clone());
        duplicate.id = self.allocate_id();
        duplicate.x += offset[0];
        duplicate.y += offset[1];
        duplicate.is_deleted = false;
        reset_identity_metadata(&mut duplicate, self.next_id);
        self.scene.elements.push(duplicate);
        self.selection.index = Some(self.scene.elements.len() - 1);
        self.selected()
    }

    pub fn nudge_selection(&mut self, delta: [f32; 2]) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        if self
            .scene
            .elements
            .get(index)
            .is_none_or(|element| element.is_deleted)
        {
            return false;
        }
        self.history.push(self.scene.clone());
        let element = &mut self.scene.elements[index];
        element.x += delta[0];
        element.y += delta[1];
        mark_changed(element);
        true
    }

    pub fn set_selected_text(&mut self, text: impl Into<String>) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        if self
            .scene
            .elements
            .get(index)
            .is_none_or(|element| element.is_deleted || element.kind != "text")
        {
            return false;
        }
        self.history.push(self.scene.clone());
        let element = &mut self.scene.elements[index];
        element.text = text.into();
        element.extra.insert(
            "originalText".to_owned(),
            Value::String(element.text.clone()),
        );
        element.width = element
            .width
            .max(element.text.chars().count() as f32 * element.font_size.max(1.0) * 0.6);
        element.height = element.height.max(element.font_size.max(1.0));
        mark_changed(element);
        true
    }

    pub fn set_selected_style(&mut self, patch: DrawingStylePatch) -> bool {
        let Some(index) = self.selection.index else {
            return false;
        };
        if self
            .scene
            .elements
            .get(index)
            .is_none_or(|element| element.is_deleted)
        {
            return false;
        }
        self.history.push(self.scene.clone());
        let element = &mut self.scene.elements[index];
        if let Some(value) = patch.stroke_color {
            element.stroke_color = value;
        }
        if let Some(value) = patch.background_color {
            element.background_color = value;
        }
        if let Some(value) = patch.stroke_width {
            element.stroke_width = value.max(0.0);
        }
        if let Some(value) = patch.stroke_style {
            element.stroke_style = value;
        }
        if let Some(value) = patch.fill_style {
            element.fill_style = value;
        }
        if let Some(value) = patch.opacity {
            element.opacity = value.clamp(0.0, 100.0);
        }
        mark_changed(element);
        true
    }

    pub fn insert_image(
        &mut self,
        file_id: impl Into<String>,
        position: [f32; 2],
        size: [f32; 2],
    ) -> &DrawingElement {
        self.history.push(self.scene.clone());
        let id = self.allocate_id();
        let mut element = create_element(DrawingTool::Image, position, id);
        element.width = size[0].abs().max(1.0);
        element.height = size[1].abs().max(1.0);
        let file_id = file_id.into();
        element
            .extra
            .insert("fileId".to_owned(), Value::String(file_id));
        element
            .extra
            .insert("status".to_owned(), Value::String("saved".to_owned()));
        element.extra.insert("scale".to_owned(), json!([1, 1]));
        element.extra.insert("crop".to_owned(), Value::Null);
        self.scene.elements.push(element);
        self.selection.index = Some(self.scene.elements.len() - 1);
        self.selected().expect("inserted image must be selected")
    }

    pub fn zoom_at(&mut self, screen_point: [f32; 2], factor: f32) {
        self.viewport.zoom_at(screen_point, factor);
    }

    pub fn pan_by(&mut self, delta: [f32; 2]) {
        self.viewport.pan_by(delta);
    }

    fn hit_test(&self, point: [f32; 2]) -> Option<usize> {
        self.scene
            .elements
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, element)| element.hit_test(point).then_some(index))
    }

    fn erase_world(&mut self, world: [f32; 2], checkpointed: bool) -> bool {
        let Some(index) = self.hit_test(world) else {
            return false;
        };
        if !checkpointed {
            self.history.push(self.scene.clone());
        }
        let element = &mut self.scene.elements[index];
        element.is_deleted = true;
        mark_changed(element);
        if self.selection.index == Some(index) {
            self.selection.index = None;
        }
        true
    }

    fn allocate_id(&mut self) -> String {
        loop {
            let candidate = format!("draw-element-{}", self.next_id);
            self.next_id = self.next_id.wrapping_add(1);
            if !self
                .scene
                .elements
                .iter()
                .any(|element| element.id == candidate)
            {
                return candidate;
            }
        }
    }
}

pub fn create_element(tool: DrawingTool, start: [f32; 2], id: impl Into<String>) -> DrawingElement {
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
    if tool == DrawingTool::Freehand {
        extra.insert("pressures".to_owned(), json!([]));
        extra.insert("simulatePressure".to_owned(), json!(true));
        extra.insert(
            "strokeOptions".to_owned(),
            json!({"variability": "variable", "streamline": 0.5}),
        );
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
    if tool == DrawingTool::Frame {
        extra.insert("name".to_owned(), Value::Null);
        extra.insert("roughness".to_owned(), json!(0));
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
        stroke_color: match tool {
            DrawingTool::Image => "transparent",
            DrawingTool::Frame => "#bbb",
            _ => "#1e1e1e",
        }
        .to_owned(),
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

fn update_linear_dimensions(element: &mut DrawingElement) {
    if element.points.is_empty() {
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

fn mark_changed(element: &mut DrawingElement) {
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);
    let nonce = element
        .extra
        .get("versionNonce")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    element.extra.insert("version".to_owned(), json!(version));
    element
        .extra
        .insert("versionNonce".to_owned(), json!(nonce));
}

fn reset_identity_metadata(element: &mut DrawingElement, nonce_seed: u64) {
    element.extra.insert("version".to_owned(), json!(1));
    element.extra.insert(
        "versionNonce".to_owned(),
        json!(nonce_seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)),
    );
    element.extra.insert("index".to_owned(), Value::Null);
}
