pub use elephant_draw::{
    rgba, DrawingElement, DrawingScene, HistoryState, SelectionMode, SelectionSet, Viewport,
};
use serde_json::{json, Value};

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
    MoveSelection {
        last_world: [f32; 2],
        checkpointed: bool,
    },
    BoxSelect {
        start_world: [f32; 2],
        current_world: [f32; 2],
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
    selection: SelectionSet,
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
            selection: SelectionSet::new(),
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
            "Frame" => "frame",
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
        if self.selection.len() == 1 {
            return self.selection.ids().next();
        }
        if self.selection.len() > 1 {
            return None;
        }
        self.selected_element().map(|element| element.id.as_str())
    }

    pub fn selected_element_ids(&self) -> Vec<&str> {
        if self.selection.is_empty() {
            return self.selected_element_id().into_iter().collect();
        }
        self.selection.ids().collect()
    }

    pub fn is_element_selected(&self, id: &str) -> bool {
        if !self.selection.is_empty() {
            return self.selection.contains(id);
        }
        self.selected_element_id() == Some(id)
    }

    pub fn selected_element(&self) -> Option<&DrawingElement> {
        if self.selection.len() == 1 {
            let id = self.selection.ids().next()?;
            return self
                .document
                .element_by_id(id)
                .filter(|element| !is_locked(element));
        }
        if !self.selection.is_empty() {
            return None;
        }
        self.selected
            .and_then(|index| self.document.elements.get(index))
            .filter(|element| !element.is_deleted && !is_locked(element))
    }

    pub fn selection_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        if !self.selection.is_empty() {
            return self.selection.bounds(&self.document);
        }
        self.selected_element().map(DrawingElement::bounds)
    }

    pub fn selection_marquee_world(&self) -> Option<(f32, f32, f32, f32)> {
        let Interaction::BoxSelect {
            start_world,
            current_world,
        } = self.interaction
        else {
            return None;
        };
        Some(normalized_bounds(start_world, current_world))
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
        self.clear_selection();
        self.interaction = Interaction::None;
        self.changed();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(document) = self.history.redo(self.document.clone()) else {
            return false;
        };
        self.document = document;
        self.clear_selection();
        self.interaction = Interaction::None;
        self.changed();
        true
    }

    pub fn delete_selection(&mut self) -> bool {
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };

        let before = self.document.clone();
        let outcome = self.document.delete_selection_excalidraw(&selection);
        if outcome.changed == 0 {
            return false;
        }

        self.history.push(before);
        self.selection = outcome.selection;
        self.sync_primary_from_selection();
        self.interaction = Interaction::None;
        self.changed();
        true
    }

    pub fn set_selected_stroke(&mut self, color: impl Into<String>) -> bool {
        let color = color.into();
        self.mutate_selected(move |element| {
            if element.stroke_color == color {
                return false;
            }
            element.stroke_color = color.clone();
            true
        })
    }

    pub fn set_selected_background(&mut self, color: impl Into<String>) -> bool {
        let color = color.into();
        self.mutate_selected(move |element| {
            if element.background_color == color {
                return false;
            }
            element.background_color = color.clone();
            true
        })
    }

    pub fn set_selected_stroke_width(&mut self, width: f32) -> bool {
        if !width.is_finite() {
            return false;
        }
        let width = width.max(0.0);
        self.mutate_selected(move |element| {
            if (element.stroke_width - width).abs() <= f32::EPSILON {
                return false;
            }
            element.stroke_width = width;
            true
        })
    }

    pub fn set_selected_stroke_style(&mut self, style: impl Into<String>) -> bool {
        let style = style.into();
        self.mutate_selected(move |element| {
            if element.stroke_style == style {
                return false;
            }
            element.stroke_style = style.clone();
            true
        })
    }

    pub fn set_selected_opacity(&mut self, opacity: f32) -> bool {
        if !opacity.is_finite() {
            return false;
        }
        let opacity = opacity.clamp(0.0, 100.0);
        self.mutate_selected(move |element| {
            if (element.opacity - opacity).abs() <= f32::EPSILON {
                return false;
            }
            element.opacity = opacity;
            true
        })
    }

    pub fn set_selection_locked(&mut self, locked: bool) -> bool {
        if self.selection.len() > 1 {
            let selection = self.selection.clone();
            if !self.document.elements.iter().any(|element| {
                selection.contains(&element.id)
                    && !element.is_deleted
                    && is_locked(element) != locked
            }) {
                return false;
            }
            self.checkpoint();
            let mut changed = false;
            for element in &mut self.document.elements {
                if !selection.contains(&element.id)
                    || element.is_deleted
                    || is_locked(element) == locked
                {
                    continue;
                }
                element.extra.insert("locked".to_owned(), json!(locked));
                mark_changed(element);
                changed = true;
            }
            if changed {
                if locked {
                    self.clear_selection();
                }
                self.changed();
            }
            return changed;
        }

        let Some(index) = self.selected else {
            return false;
        };
        let Some(element) = self.document.elements.get(index) else {
            return false;
        };
        if element.is_deleted || is_locked(element) == locked {
            return false;
        }
        self.checkpoint();
        let element = &mut self.document.elements[index];
        element.extra.insert("locked".to_owned(), json!(locked));
        mark_changed(element);
        if locked {
            self.clear_selection();
        }
        self.changed();
        true
    }

    pub fn bring_selection_to_front(&mut self) -> bool {
        self.reorder_selection(Reorder::Front)
    }

    pub fn send_selection_to_back(&mut self) -> bool {
        self.reorder_selection(Reorder::Back)
    }

    pub fn bring_selection_forward(&mut self) -> bool {
        self.reorder_selection(Reorder::Forward)
    }

    pub fn send_selection_backward(&mut self) -> bool {
        self.reorder_selection(Reorder::Backward)
    }

    pub fn cancel_interaction(&mut self) {
        self.clear_selection();
        self.interaction = Interaction::None;
        self.set_active_tool_label("Selection");
        self.changed();
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        let zoom = zoom.clamp(elephant_draw::MIN_ZOOM, elephant_draw::MAX_ZOOM);
        if (self.viewport.zoom - zoom).abs() <= f32::EPSILON {
            return;
        }
        self.viewport.zoom = zoom;
        self.changed();
    }

    pub fn zoom_in(&mut self) {
        self.set_zoom(self.viewport.zoom * 1.1);
    }

    pub fn zoom_out(&mut self) {
        self.set_zoom(self.viewport.zoom / 1.1);
    }

    pub fn reset_zoom(&mut self) {
        self.set_zoom(1.0);
    }

    pub(crate) fn checkpoint(&mut self) {
        self.history.push(self.document.clone());
    }

    pub(crate) fn touch_element(&mut self, index: usize) {
        if let Some(element) = self.document.elements.get_mut(index) {
            mark_changed(element);
            self.changed();
        }
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
                if let Some(index) = self.hit_test(world) {
                    let id = self.document.elements[index].id.clone();
                    if self.selection.len() > 1 && self.selection.contains(&id) {
                        self.selected = None;
                        self.interaction = Interaction::MoveSelection {
                            last_world: world,
                            checkpointed: false,
                        };
                    } else {
                        self.selection = SelectionSet::from_ids(std::iter::once(id));
                        self.selected = Some(index);
                        let element = &self.document.elements[index];
                        self.interaction = Interaction::MoveElement {
                            index,
                            offset: [world[0] - element.x, world[1] - element.y],
                            checkpointed: false,
                        };
                    }
                } else {
                    self.clear_selection();
                    self.interaction = Interaction::BoxSelect {
                        start_world: world,
                        current_world: world,
                    };
                }
            }
            "eraser" => {
                let changed = self.erase_world(world, false);
                self.interaction = Interaction::Erase {
                    checkpointed: changed,
                };
            }
            _ => self.interaction = Interaction::None,
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

                let Some((id, kind, origin)) = self.document.elements.get(index).map(|element| {
                    (
                        element.id.clone(),
                        element.kind.clone(),
                        [element.x, element.y],
                    )
                }) else {
                    return;
                };
                let target = [world[0] - offset[0], world[1] - offset[1]];

                if matches!(kind.as_str(), "frame" | "magicframe") {
                    let delta = [target[0] - origin[0], target[1] - origin[1]];
                    if self.document.translate_frame_with_children(&id, delta) > 0 {
                        self.changed();
                    }
                } else if let Some(element) = self.document.elements.get_mut(index) {
                    element.x = target[0];
                    element.y = target[1];
                    mark_changed(element);
                    self.changed();
                }
            }
            Interaction::MoveSelection {
                last_world,
                checkpointed,
            } => {
                let world = self.to_world(point);
                let delta = [world[0] - last_world[0], world[1] - last_world[1]];
                if delta == [0.0, 0.0] {
                    return;
                }
                if !checkpointed {
                    self.checkpoint();
                }
                let selection = self.selection.clone();
                let changed = self.document.translate_selection(&selection, delta);
                self.interaction = Interaction::MoveSelection {
                    last_world: world,
                    checkpointed: checkpointed || changed > 0,
                };
                if changed > 0 {
                    self.changed();
                }
            }
            Interaction::BoxSelect {
                start_world,
                current_world: _,
            } => {
                let world = self.to_world(point);
                self.selection =
                    self.document
                        .select_in_rect(start_world, world, SelectionMode::Contained);
                self.selected = None;
                self.interaction = Interaction::BoxSelect {
                    start_world,
                    current_world: world,
                };
                self.changed();
            }
            Interaction::Pan {
                start_pointer,
                start_pan,
            } => {
                self.viewport.pan = [
                    start_pan[0] + point[0] - start_pointer[0],
                    start_pan[1] + point[1] - start_pointer[1],
                ];
                self.changed();
            }
            Interaction::Erase { checkpointed } => {
                let world = self.to_world(point);
                let changed = self.erase_world(world, checkpointed);
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
        let moved_ids = match &self.interaction {
            Interaction::MoveElement { index, .. } => self
                .document
                .elements
                .get(*index)
                .filter(|element| !matches!(element.kind.as_str(), "frame" | "magicframe"))
                .map(|element| vec![element.id.clone()])
                .unwrap_or_default(),
            Interaction::MoveSelection { .. } => self.selection.ids().map(str::to_owned).collect(),
            _ => Vec::new(),
        };

        if matches!(self.interaction, Interaction::BoxSelect { .. }) {
            self.sync_primary_from_selection();
        }
        self.interaction = Interaction::None;

        let mut membership_changed = false;
        for id in moved_ids {
            let is_frame = self
                .document
                .element_by_id(&id)
                .is_some_and(|element| matches!(element.kind.as_str(), "frame" | "magicframe"));
            if !is_frame && self.document.sync_element_frame_membership(&id) {
                membership_changed = true;
            }
        }
        if membership_changed {
            self.sync_primary_from_selection();
            self.changed();
        }
    }

    pub(crate) fn zoom_at(&mut self, point: [f32; 2], delta_y: f64) {
        let factor = if delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };
        self.viewport.zoom_at(point, factor);
        self.changed();
    }

    pub(crate) fn to_world(&self, point: [f32; 2]) -> [f32; 2] {
        self.viewport.to_world(point)
    }

    fn mutate_selected(
        &mut self,
        mut mutation: impl FnMut(&mut DrawingElement) -> bool,
    ) -> bool {
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let before = self.document.clone();
        let mut changed = false;
        for element in &mut self.document.elements {
            if !selection.contains(&element.id) || element.is_deleted || is_locked(element) {
                continue;
            }
            if mutation(element) {
                mark_changed(element);
                changed = true;
            }
        }
        if !changed {
            return false;
        }
        self.history.push(before);
        self.changed();
        true
    }

    fn reorder_selection(&mut self, direction: Reorder) -> bool {
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let is_selected = |element: &DrawingElement| {
            selection.contains(&element.id) && !element.is_deleted && !is_locked(element)
        };
        if !self.document.elements.iter().any(is_selected) {
            return false;
        }

        let before = self.document.clone();
        match direction {
            Reorder::Front => {
                self.document
                    .elements
                    .sort_by_key(|element| usize::from(is_selected(element)));
            }
            Reorder::Back => {
                self.document
                    .elements
                    .sort_by_key(|element| usize::from(!is_selected(element)));
            }
            Reorder::Forward => {
                for index in (0..self.document.elements.len().saturating_sub(1)).rev() {
                    if is_selected(&self.document.elements[index])
                        && !is_selected(&self.document.elements[index + 1])
                    {
                        self.document.elements.swap(index, index + 1);
                    }
                }
            }
            Reorder::Backward => {
                for index in 1..self.document.elements.len() {
                    if is_selected(&self.document.elements[index])
                        && !is_selected(&self.document.elements[index - 1])
                    {
                        self.document.elements.swap(index, index - 1);
                    }
                }
            }
        }

        let order_changed = self
            .document
            .elements
            .iter()
            .zip(&before.elements)
            .any(|(after, before)| after.id != before.id);
        if !order_changed {
            return false;
        }
        self.document.sync_fractional_indices();
        self.history.push(before);
        self.sync_primary_from_selection();
        self.changed();
        true
    }

    fn erase_world(&mut self, world: [f32; 2], checkpointed: bool) -> bool {
        let Some(index) = self.hit_test(world) else {
            return false;
        };
        if !checkpointed {
            self.checkpoint();
        }
        let id = self.document.elements[index].id.clone();
        let element = &mut self.document.elements[index];
        if element.is_deleted || is_locked(element) {
            return false;
        }
        element.is_deleted = true;
        mark_changed(element);
        self.selection.remove(&id);
        if self.selected == Some(index) {
            self.selected = None;
        }
        self.sync_primary_from_selection();
        self.changed();
        true
    }

    fn hit_test(&self, point: [f32; 2]) -> Option<usize> {
        self.document
            .elements
            .iter()
            .enumerate()
            .rev()
            .find(|(_, element)| {
                !element.is_deleted && !is_locked(element) && element.hit_test(point)
            })
            .map(|(index, _)| index)
    }

    fn clear_selection(&mut self) {
        self.selected = None;
        self.selection.clear();
    }

    fn sync_primary_from_selection(&mut self) {
        self.selected = if self.selection.len() == 1 {
            self.selection
                .ids()
                .next()
                .and_then(|id| self.document.element_index(id))
        } else {
            None
        };
    }

    fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

#[derive(Clone, Copy)]
enum Reorder {
    Front,
    Back,
    Forward,
    Backward,
}

fn normalized_bounds(start: [f32; 2], end: [f32; 2]) -> (f32, f32, f32, f32) {
    let min_x = start[0].min(end[0]);
    let min_y = start[1].min(end[1]);
    (
        min_x,
        min_y,
        start[0].max(end[0]) - min_x,
        start[1].max(end[1]) - min_y,
    )
}

fn is_locked(element: &DrawingElement) -> bool {
    element
        .extra
        .get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
