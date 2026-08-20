use crate::{DrawingScene, SelectionSet};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapAxis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapGuide {
    pub axis: SnapAxis,
    pub position: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    pub delta: [f32; 2],
    pub guides: Vec<SnapGuide>,
}

impl DrawingScene {
    /// Resolve a requested translation against visible element edges and
    /// centers. The returned delta is in world coordinates and can be passed
    /// directly to `translate_selection`. Elements travelling with a selected
    /// frame are excluded as snap targets.
    pub fn snap_selection_delta(
        &self,
        selection: &SelectionSet,
        requested_delta: [f32; 2],
        threshold: f32,
    ) -> SnapResult {
        if selection.is_empty()
            || !requested_delta.into_iter().all(f32::is_finite)
            || !threshold.is_finite()
            || threshold < 0.0
        {
            return SnapResult {
                delta: requested_delta,
                guides: Vec::new(),
            };
        }
        let excluded = expanded_moving_ids(self, selection);
        let Some(source) = bounds_for_ids(self, &excluded) else {
            return SnapResult {
                delta: requested_delta,
                guides: Vec::new(),
            };
        };
        let moved = (
            source.0 + requested_delta[0],
            source.1 + requested_delta[1],
            source.2,
            source.3,
        );
        let source_x = anchors_x(moved);
        let source_y = anchors_y(moved);

        let mut best_x: Option<(f32, SnapGuide)> = None;
        let mut best_y: Option<(f32, SnapGuide)> = None;
        for element in self.elements.iter().filter(|element| {
            !element.is_deleted && !element.is_locked() && !excluded.contains(&element.id)
        }) {
            let target = element.bounds();
            for sx in source_x {
                for tx in anchors_x(target) {
                    let correction = tx - sx;
                    if correction.abs() > threshold {
                        continue;
                    }
                    let guide = SnapGuide {
                        axis: SnapAxis::X,
                        position: tx,
                        start: moved.1.min(target.1),
                        end: (moved.1 + moved.3).max(target.1 + target.3),
                    };
                    if best_x
                        .as_ref()
                        .is_none_or(|(best, _)| correction.abs() < best.abs())
                    {
                        best_x = Some((correction, guide));
                    }
                }
            }
            for sy in source_y {
                for ty in anchors_y(target) {
                    let correction = ty - sy;
                    if correction.abs() > threshold {
                        continue;
                    }
                    let guide = SnapGuide {
                        axis: SnapAxis::Y,
                        position: ty,
                        start: moved.0.min(target.0),
                        end: (moved.0 + moved.2).max(target.0 + target.2),
                    };
                    if best_y
                        .as_ref()
                        .is_none_or(|(best, _)| correction.abs() < best.abs())
                    {
                        best_y = Some((correction, guide));
                    }
                }
            }
        }

        let mut delta = requested_delta;
        let mut guides = Vec::new();
        if let Some((correction, guide)) = best_x {
            delta[0] += correction;
            guides.push(guide);
        }
        if let Some((correction, guide)) = best_y {
            delta[1] += correction;
            guides.push(guide);
        }
        SnapResult { delta, guides }
    }
}

fn expanded_moving_ids(scene: &DrawingScene, selection: &SelectionSet) -> HashSet<String> {
    let mut ids = selection.ids().map(str::to_owned).collect::<HashSet<_>>();
    let frames = scene
        .elements
        .iter()
        .filter(|element| {
            ids.contains(&element.id)
                && !element.is_deleted
                && matches!(element.kind.as_str(), "frame" | "magicframe")
        })
        .map(|element| element.id.clone())
        .collect::<HashSet<_>>();
    for element in &scene.elements {
        if element
            .extra
            .get("frameId")
            .and_then(Value::as_str)
            .is_some_and(|id| frames.contains(id))
        {
            ids.insert(element.id.clone());
        }
    }
    ids
}

fn bounds_for_ids(scene: &DrawingScene, ids: &HashSet<String>) -> Option<(f32, f32, f32, f32)> {
    let mut selected = scene
        .elements
        .iter()
        .filter(|element| ids.contains(&element.id) && !element.is_deleted);
    let first = selected.next()?;
    let (x, y, width, height) = first.bounds();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (x, y, x + width, y + height);
    for element in selected {
        let (x, y, width, height) = element.bounds();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + width);
        max_y = max_y.max(y + height);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

fn anchors_x(bounds: (f32, f32, f32, f32)) -> [f32; 3] {
    [bounds.0, bounds.0 + bounds.2 / 2.0, bounds.0 + bounds.2]
}

fn anchors_y(bounds: (f32, f32, f32, f32)) -> [f32; 3] {
    [bounds.1, bounds.1 + bounds.3 / 2.0, bounds.1 + bounds.3]
}
