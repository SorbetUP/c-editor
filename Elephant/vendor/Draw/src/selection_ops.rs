use crate::{DrawingEditor, DrawingStylePatch, SelectionMode, SelectionSet};
use serde_json::{json, Value};

impl DrawingEditor {
    pub fn select_rect(
        &self,
        start: [f32; 2],
        end: [f32; 2],
        mode: SelectionMode,
    ) -> SelectionSet {
        self.scene.select_in_rect(start, end, mode)
    }

    pub fn select_lasso(&self, polygon: &[[f32; 2]], mode: SelectionMode) -> SelectionSet {
        self.scene.select_in_lasso(polygon, mode)
    }

    pub fn translate_selection_set(
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
        let before = self.scene.clone();
        let changed = self.scene.translate_selection(selection, delta);
        if changed == 0 {
            return 0;
        }
        self.history.push(before);
        self.select_first(selection);
        changed
    }

    pub fn delete_selection_set(&mut self, selection: &SelectionSet) -> usize {
        self.mutate_selection_set(selection, |element| {
            if element.is_deleted || is_locked(element) {
                return false;
            }
            element.is_deleted = true;
            true
        })
    }

    pub fn set_selection_set_locked(
        &mut self,
        selection: &SelectionSet,
        locked: bool,
    ) -> usize {
        let changed = self.mutate_selection_set(selection, |element| {
            if element.is_deleted || is_locked(element) == locked {
                return false;
            }
            element.extra.insert("locked".to_owned(), json!(locked));
            true
        });
        if locked && changed > 0 {
            self.selection.index = None;
        }
        changed
    }

    pub fn set_selection_set_style(
        &mut self,
        selection: &SelectionSet,
        patch: &DrawingStylePatch,
    ) -> usize {
        self.mutate_selection_set(selection, |element| {
            if element.is_deleted || is_locked(element) {
                return false;
            }
            let mut changed = false;
            if let Some(value) = &patch.stroke_color {
                if element.stroke_color != *value {
                    element.stroke_color = value.clone();
                    changed = true;
                }
            }
            if let Some(value) = &patch.background_color {
                if element.background_color != *value {
                    element.background_color = value.clone();
                    changed = true;
                }
            }
            if let Some(value) = patch.stroke_width {
                let value = value.max(0.0);
                if (element.stroke_width - value).abs() > f32::EPSILON {
                    element.stroke_width = value;
                    changed = true;
                }
            }
            if let Some(value) = &patch.stroke_style {
                if element.stroke_style != *value {
                    element.stroke_style = value.clone();
                    changed = true;
                }
            }
            if let Some(value) = &patch.fill_style {
                if element.fill_style != *value {
                    element.fill_style = value.clone();
                    changed = true;
                }
            }
            if let Some(value) = patch.opacity {
                let value = value.clamp(0.0, 100.0);
                if (element.opacity - value).abs() > f32::EPSILON {
                    element.opacity = value;
                    changed = true;
                }
            }
            changed
        })
    }

    pub fn group_selection_set(&mut self, selection: &SelectionSet, group_id: &str) -> usize {
        if group_id.is_empty() {
            return 0;
        }
        let ids = selection
            .ids()
            .filter(|id| !self.scene.is_element_locked(id))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if ids.len() < 2 {
            return 0;
        }
        let refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
        let before = self.scene.clone();
        let changed = self.scene.group_elements(&refs, group_id);
        if changed == 0 {
            return 0;
        }
        self.history.push(before);
        self.select_first(selection);
        changed
    }

    pub fn ungroup_selection_set(&mut self, selection: &SelectionSet, group_id: &str) -> usize {
        if group_id.is_empty() || selection.is_empty() {
            return 0;
        }
        let before = self.scene.clone();
        let mut changed = 0;
        for element in &mut self.scene.elements {
            if !selection.contains(&element.id) || element.is_deleted || is_locked(element) {
                continue;
            }
            let Some(groups) = element
                .extra
                .get_mut("groupIds")
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            let old_len = groups.len();
            groups.retain(|value| value.as_str() != Some(group_id));
            if groups.len() != old_len {
                mark_changed(element);
                changed += 1;
            }
        }
        if changed == 0 {
            return 0;
        }
        self.history.push(before);
        self.select_first(selection);
        changed
    }

    fn mutate_selection_set(
        &mut self,
        selection: &SelectionSet,
        mut mutation: impl FnMut(&mut crate::DrawingElement) -> bool,
    ) -> usize {
        if selection.is_empty() {
            return 0;
        }
        let before = self.scene.clone();
        let mut changed = 0;
        for element in &mut self.scene.elements {
            if !selection.contains(&element.id) || !mutation(element) {
                continue;
            }
            mark_changed(element);
            changed += 1;
        }
        if changed == 0 {
            return 0;
        }
        self.history.push(before);
        self.select_first(selection);
        changed
    }

    fn select_first(&mut self, selection: &SelectionSet) {
        self.selection.index = selection
            .ids()
            .find_map(|id| self.scene.element_index(id));
    }
}

fn is_locked(element: &crate::DrawingElement) -> bool {
    element
        .extra
        .get("locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
