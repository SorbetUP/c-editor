use crate::{generate_n_keys_between, DrawingScene};
use serde_json::{json, Value};

impl DrawingScene {
    pub fn element_index(&self, id: &str) -> Option<usize> {
        self.elements
            .iter()
            .position(|element| element.id == id && !element.is_deleted)
    }

    pub fn is_element_locked(&self, id: &str) -> bool {
        self.element_index(id)
            .and_then(|index| self.elements.get(index))
            .and_then(|element| element.extra.get("locked"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn set_element_locked(&mut self, id: &str, locked: bool) -> bool {
        let Some(index) = self.element_index(id) else {
            return false;
        };
        let element = &mut self.elements[index];
        if element
            .extra
            .get("locked")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            == locked
        {
            return false;
        }
        element.extra.insert("locked".to_owned(), json!(locked));
        mark_changed(element);
        true
    }

    pub fn toggle_element_locked(&mut self, id: &str) -> Option<bool> {
        let next = !self.is_element_locked(id);
        self.set_element_locked(id, next).then_some(next)
    }

    pub fn translate_element(&mut self, id: &str, delta: [f32; 2]) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if delta == [0.0, 0.0] {
            return false;
        }
        let element = &mut self.elements[index];
        element.x += delta[0];
        element.y += delta[1];
        mark_changed(element);
        true
    }

    pub fn set_element_size(&mut self, id: &str, size: [f32; 2]) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if !size[0].is_finite() || !size[1].is_finite() {
            return false;
        }
        let size = [size[0].max(0.0), size[1].max(0.0)];
        let element = &mut self.elements[index];
        if element.width == size[0] && element.height == size[1] {
            return false;
        }
        element.width = size[0];
        element.height = size[1];
        mark_changed(element);
        true
    }

    pub fn rotate_element(&mut self, id: &str, angle_radians: f32) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if !angle_radians.is_finite() {
            return false;
        }
        let normalized = normalize_angle(angle_radians);
        let element = &mut self.elements[index];
        if (element.angle - normalized).abs() <= f32::EPSILON {
            return false;
        }
        element.angle = normalized;
        mark_changed(element);
        true
    }

    /// Synchronize the persisted Excalidraw fractional `index` with the array
    /// order used by the native renderer. Deleted elements are included because
    /// Excalidraw reconciliation orders the complete scene, not only visible
    /// elements.
    pub fn sync_fractional_indices(&mut self) -> usize {
        let Some(indices) = generate_n_keys_between(None, None, self.elements.len()) else {
            return 0;
        };
        let mut changed = 0;
        for (element, index) in self.elements.iter_mut().zip(indices) {
            if element.extra.get("index").and_then(Value::as_str) == Some(index.as_str()) {
                continue;
            }
            element
                .extra
                .insert("index".to_owned(), Value::String(index));
            mark_changed(element);
            changed += 1;
        }
        changed
    }

    pub fn bring_element_to_front(&mut self, id: &str) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if index + 1 == self.elements.len() {
            return false;
        }
        let element = self.elements.remove(index);
        self.elements.push(element);
        self.sync_fractional_indices();
        true
    }

    pub fn send_element_to_back(&mut self, id: &str) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if index == 0 {
            return false;
        }
        let element = self.elements.remove(index);
        self.elements.insert(0, element);
        self.sync_fractional_indices();
        true
    }

    pub fn bring_element_forward(&mut self, id: &str) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if index + 1 >= self.elements.len() {
            return false;
        }
        self.elements.swap(index, index + 1);
        self.sync_fractional_indices();
        true
    }

    pub fn send_element_backward(&mut self, id: &str) -> bool {
        let Some(index) = self.mutable_index(id) else {
            return false;
        };
        if index == 0 {
            return false;
        }
        self.elements.swap(index, index - 1);
        self.sync_fractional_indices();
        true
    }

    pub fn group_elements(&mut self, ids: &[&str], group_id: &str) -> usize {
        if group_id.is_empty() || ids.len() < 2 {
            return 0;
        }
        let mut changed = 0;
        for element in &mut self.elements {
            if element.is_deleted || !ids.iter().any(|id| *id == element.id) {
                continue;
            }
            let groups = element
                .extra
                .entry("groupIds".to_owned())
                .or_insert_with(|| json!([]));
            let Some(groups) = groups.as_array_mut() else {
                continue;
            };
            if groups.iter().any(|value| value.as_str() == Some(group_id)) {
                continue;
            }
            groups.push(Value::String(group_id.to_owned()));
            mark_changed(element);
            changed += 1;
        }
        changed
    }

    pub fn ungroup_elements(&mut self, group_id: &str) -> usize {
        if group_id.is_empty() {
            return 0;
        }
        let mut changed = 0;
        for element in &mut self.elements {
            if element.is_deleted {
                continue;
            }
            let Some(groups) = element
                .extra
                .get_mut("groupIds")
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            let before = groups.len();
            groups.retain(|value| value.as_str() != Some(group_id));
            if groups.len() != before {
                mark_changed(element);
                changed += 1;
            }
        }
        changed
    }

    fn mutable_index(&self, id: &str) -> Option<usize> {
        let index = self.element_index(id)?;
        (!self.is_element_locked(id)).then_some(index)
    }
}

fn normalize_angle(angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    angle.rem_euclid(tau)
}

fn mark_changed(element: &mut crate::DrawingElement) {
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
