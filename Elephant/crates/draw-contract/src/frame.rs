use crate::{DrawingElement, DrawingScene, SelectionSet};
use serde_json::{json, Value};
use std::collections::HashSet;

impl DrawingScene {
    pub fn frame_children(&self, frame_id: &str) -> Vec<&DrawingElement> {
        self.elements
            .iter()
            .filter(|element| {
                !element.is_deleted
                    && element.extra.get("frameId").and_then(Value::as_str) == Some(frame_id)
            })
            .collect()
    }

    pub fn element_completely_in_frame(&self, element_id: &str, frame_id: &str) -> bool {
        let Some(element) = self.element_by_id(element_id) else {
            return false;
        };
        let Some(frame) = self.element_by_id(frame_id).filter(|frame| is_frame_like(frame)) else {
            return false;
        };
        if element.id == frame.id || is_frame_like(element) {
            return false;
        }
        bounds_contain(frame.bounds(), element.bounds())
    }

    pub fn add_elements_to_frame(&mut self, frame_id: &str, element_ids: &[&str]) -> usize {
        let Some(frame_index) = self.elements.iter().position(|element| {
            element.id == frame_id
                && !element.is_deleted
                && is_frame_like(element)
                && !is_locked(element)
        }) else {
            return 0;
        };
        let frame_id = self.elements[frame_index].id.clone();
        let requested = element_ids.iter().copied().collect::<HashSet<_>>();
        let mut candidates = Vec::<String>::new();

        for element in &self.elements {
            if !requested.contains(element.id.as_str())
                || element.id == frame_id
                || element.is_deleted
                || is_frame_like(element)
                || is_locked(element)
                || element
                    .extra
                    .get("frameId")
                    .and_then(Value::as_str)
                    .is_some_and(|owner| owner != frame_id)
            {
                continue;
            }
            candidates.push(element.id.clone());
            for bound_id in bound_text_ids(element) {
                if !candidates.iter().any(|id| id == &bound_id) {
                    candidates.push(bound_id);
                }
            }
        }

        let candidate_set = candidates.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut changed = 0;
        for element in &mut self.elements {
            if !candidate_set.contains(element.id.as_str()) || element.is_deleted {
                continue;
            }
            let current = element.extra.get("frameId").and_then(Value::as_str);
            if current == Some(frame_id.as_str()) {
                continue;
            }
            element
                .extra
                .insert("frameId".to_owned(), Value::String(frame_id.clone()));
            mark_changed(element);
            changed += 1;
        }

        if changed > 0 {
            self.reorder_frame_children(&frame_id, &candidate_set);
            self.sync_fractional_indices();
        }
        changed
    }

    pub fn remove_elements_from_frame(&mut self, element_ids: &[&str]) -> usize {
        let requested = element_ids.iter().copied().collect::<HashSet<_>>();
        let mut candidates = requested.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>();
        for element in &self.elements {
            if requested.contains(element.id.as_str()) {
                for bound_id in bound_text_ids(element) {
                    if !candidates.iter().any(|id| id == &bound_id) {
                        candidates.push(bound_id);
                    }
                }
            }
        }
        let candidates = candidates.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut changed = 0;
        for element in &mut self.elements {
            if !candidates.contains(element.id.as_str())
                || element.is_deleted
                || element.extra.get("frameId").is_none_or(Value::is_null)
            {
                continue;
            }
            element.extra.insert("frameId".to_owned(), Value::Null);
            mark_changed(element);
            changed += 1;
        }
        changed
    }

    pub fn remove_all_elements_from_frame(&mut self, frame_id: &str) -> usize {
        let ids = self
            .frame_children(frame_id)
            .into_iter()
            .map(|element| element.id.clone())
            .collect::<Vec<_>>();
        let refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
        self.remove_elements_from_frame(&refs)
    }

    pub fn sync_element_frame_membership(&mut self, element_id: &str) -> bool {
        let Some(element) = self.element_by_id(element_id) else {
            return false;
        };
        if is_frame_like(element) || is_locked(element) {
            return false;
        }
        let element_bounds = element.bounds();
        let current = element
            .extra
            .get("frameId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let target = self
            .elements
            .iter()
            .rev()
            .find(|frame| {
                !frame.is_deleted
                    && is_frame_like(frame)
                    && frame.id != element_id
                    && bounds_contain(frame.bounds(), element_bounds)
            })
            .map(|frame| frame.id.clone());

        if current == target {
            return false;
        }
        if let Some(frame_id) = target {
            self.add_elements_to_frame(&frame_id, &[element_id]) > 0
        } else {
            self.remove_elements_from_frame(&[element_id]) > 0
        }
    }

    pub fn translate_frame_with_children(&mut self, frame_id: &str, delta: [f32; 2]) -> usize {
        if !delta[0].is_finite()
            || !delta[1].is_finite()
            || delta == [0.0, 0.0]
            || self
                .element_by_id(frame_id)
                .is_none_or(|frame| !is_frame_like(frame) || is_locked(frame))
        {
            return 0;
        }

        let mut changed = 0;
        for element in &mut self.elements {
            let belongs_to_frame = element.id == frame_id
                || element.extra.get("frameId").and_then(Value::as_str) == Some(frame_id);
            if element.is_deleted || !belongs_to_frame {
                continue;
            }
            element.x += delta[0];
            element.y += delta[1];
            mark_changed(element);
            changed += 1;
        }
        changed
    }

    pub fn translate_selection_with_frame_children(
        &mut self,
        selection: &SelectionSet,
        delta: [f32; 2],
    ) -> usize {
        if selection.is_empty()
            || !delta[0].is_finite()
            || !delta[1].is_finite()
            || delta == [0.0, 0.0]
        {
            return 0;
        }

        let selected_frames = self
            .elements
            .iter()
            .filter(|element| {
                selection.contains(&element.id)
                    && !element.is_deleted
                    && is_frame_like(element)
                    && !is_locked(element)
            })
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();

        let mut changed = 0;
        for element in &mut self.elements {
            if element.is_deleted {
                continue;
            }
            let child_of_selected_frame = element
                .extra
                .get("frameId")
                .and_then(Value::as_str)
                .is_some_and(|frame_id| selected_frames.contains(frame_id));
            let selected_frame = selected_frames.contains(&element.id);
            let selected_regular = selection.contains(&element.id)
                && !is_frame_like(element)
                && !is_locked(element);
            if !selected_frame && !child_of_selected_frame && !selected_regular {
                continue;
            }
            element.x += delta[0];
            element.y += delta[1];
            mark_changed(element);
            changed += 1;
        }
        changed
    }

    fn reorder_frame_children(&mut self, frame_id: &str, candidates: &HashSet<&str>) {
        let mut moved = Vec::new();
        let mut remaining = Vec::with_capacity(self.elements.len());
        for element in self.elements.drain(..) {
            if candidates.contains(element.id.as_str()) {
                moved.push(element);
            } else {
                remaining.push(element);
            }
        }
        if moved.is_empty() {
            self.elements = remaining;
            return;
        }

        let insertion = remaining
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, element)| {
                if element.id == frame_id {
                    Some(index)
                } else if element.extra.get("frameId").and_then(Value::as_str) == Some(frame_id) {
                    Some(index + 1)
                } else {
                    None
                }
            })
            .unwrap_or(remaining.len());
        remaining.splice(insertion..insertion, moved);
        self.elements = remaining;
    }
}

fn is_frame_like(element: &DrawingElement) -> bool {
    matches!(element.kind.as_str(), "frame" | "magicframe")
}

fn is_locked(element: &DrawingElement) -> bool {
    element
        .extra
        .get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn bound_text_ids(element: &DrawingElement) -> Vec<String> {
    element
        .extra
        .get("boundElements")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|entry| entry.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn bounds_contain(container: (f32, f32, f32, f32), child: (f32, f32, f32, f32)) -> bool {
    let (cx, cy, cw, ch) = container;
    let (x, y, w, h) = child;
    x >= cx && y >= cy && x + w <= cx + cw && y + h <= cy + ch
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
