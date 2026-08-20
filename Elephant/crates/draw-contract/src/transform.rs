use crate::{rotate_point, DrawingElement, DrawingScene, SelectionSet};
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransformOutcome {
    pub changed: usize,
    pub bounds: Option<(f32, f32, f32, f32)>,
}

impl DrawingScene {
    /// Scale exactly the selected elements around a world-space anchor.
    /// Excalidraw frame children are not implicitly resized when a frame alone
    /// is resized; they remain independent scene elements clipped by the frame.
    pub fn scale_selection(
        &mut self,
        selection: &SelectionSet,
        anchor: [f32; 2],
        scale: [f32; 2],
    ) -> TransformOutcome {
        if selection.is_empty()
            || !anchor.into_iter().all(f32::is_finite)
            || !scale.into_iter().all(f32::is_finite)
            || scale[0] <= 0.0
            || scale[1] <= 0.0
        {
            return TransformOutcome::default();
        }
        let ids = selected_ids(selection);
        if ids.is_empty() {
            return TransformOutcome::default();
        }
        let mut changed = 0;
        for element in &mut self.elements {
            if !ids.contains(&element.id) || element.is_deleted || element.is_locked() {
                continue;
            }
            scale_element(element, anchor, scale);
            mark_changed(element);
            changed += 1;
        }
        TransformOutcome {
            changed,
            bounds: (changed > 0).then(|| bounds_for_ids(self, &ids)).flatten(),
        }
    }

    /// Rotate selected non-frame elements around their common selection center.
    /// Current Excalidraw frames and magic frames never rotate.
    pub fn rotate_selection(
        &mut self,
        selection: &SelectionSet,
        angle_delta: f32,
    ) -> TransformOutcome {
        if selection.is_empty() || !angle_delta.is_finite() || angle_delta.abs() <= f32::EPSILON {
            return TransformOutcome::default();
        }
        let ids = selected_ids(selection);
        let Some((x, y, width, height)) = bounds_for_ids(self, &ids) else {
            return TransformOutcome::default();
        };
        let center = [x + width / 2.0, y + height / 2.0];
        let mut changed = 0;
        for element in &mut self.elements {
            if !ids.contains(&element.id)
                || element.is_deleted
                || element.is_locked()
                || matches!(element.kind.as_str(), "frame" | "magicframe")
            {
                continue;
            }
            let (ex, ey, ew, eh) = element.bounds();
            let old_center = [ex + ew / 2.0, ey + eh / 2.0];
            let new_center = rotate_point(old_center, center, angle_delta);
            element.x += new_center[0] - old_center[0];
            element.y += new_center[1] - old_center[1];
            element.angle = normalize_angle(element.angle + angle_delta);
            mark_changed(element);
            changed += 1;
        }
        TransformOutcome {
            changed,
            bounds: (changed > 0).then(|| bounds_for_ids(self, &ids)).flatten(),
        }
    }
}

fn selected_ids(selection: &SelectionSet) -> HashSet<String> {
    selection.ids().map(str::to_owned).collect()
}

fn scale_element(element: &mut DrawingElement, anchor: [f32; 2], scale: [f32; 2]) {
    element.x = anchor[0] + (element.x - anchor[0]) * scale[0];
    element.y = anchor[1] + (element.y - anchor[1]) * scale[1];
    if element.is_linear() && !element.points.is_empty() {
        for point in &mut element.points {
            point[0] *= scale[0];
            point[1] *= scale[1];
        }
    }
    element.width *= scale[0];
    element.height *= scale[1];
    if element.kind == "text" {
        let uniform = ((scale[0] + scale[1]) / 2.0).max(0.01);
        element.font_size = (element.font_size * uniform).max(1.0);
    }
}

fn bounds_for_ids(scene: &DrawingScene, ids: &HashSet<String>) -> Option<(f32, f32, f32, f32)> {
    let mut elements = scene
        .elements
        .iter()
        .filter(|element| ids.contains(&element.id) && !element.is_deleted);
    let first = elements.next()?;
    let (x, y, width, height) = first.bounds();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (x, y, x + width, y + height);
    for element in elements {
        let (x, y, width, height) = element.bounds();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + width);
        max_y = max_y.max(y + height);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

fn normalize_angle(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::TAU)
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
    element.extra.insert("versionNonce".to_owned(), json!(nonce));
}
