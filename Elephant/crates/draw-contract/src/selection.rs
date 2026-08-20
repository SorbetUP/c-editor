use crate::{rotate_point, DrawingElement, DrawingScene};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionMode {
    Touching,
    Contained,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionSet {
    ids: BTreeSet<String>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_ids(ids: impl IntoIterator<Item = String>) -> Self {
        Self {
            ids: ids.into_iter().collect(),
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.ids.iter().map(String::as_str)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }

    pub fn insert(&mut self, id: impl Into<String>) -> bool {
        self.ids.insert(id.into())
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.ids.remove(id)
    }

    pub fn toggle(&mut self, id: impl Into<String>) -> bool {
        let id = id.into();
        if self.ids.remove(&id) {
            false
        } else {
            self.ids.insert(id);
            true
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn bounds(&self, scene: &DrawingScene) -> Option<(f32, f32, f32, f32)> {
        let mut selected = scene
            .elements
            .iter()
            .filter(|element| self.contains(&element.id) && selectable(element));
        let first = selected.next()?;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = outline_bounds(&element_outline(first));
        for element in selected {
            let (x1, y1, x2, y2) = outline_bounds(&element_outline(element));
            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
        }
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

impl DrawingScene {
    pub fn select_in_rect(
        &self,
        start: [f32; 2],
        end: [f32; 2],
        mode: SelectionMode,
    ) -> SelectionSet {
        let rect = normalized_rect(start, end);
        SelectionSet::from_ids(
            self.elements
                .iter()
                .filter(|element| selectable(element) && rect_selects(element, rect, mode))
                .map(|element| element.id.clone()),
        )
    }

    pub fn select_in_lasso(&self, polygon: &[[f32; 2]], mode: SelectionMode) -> SelectionSet {
        if polygon.len() < 3 {
            return SelectionSet::new();
        }
        SelectionSet::from_ids(
            self.elements
                .iter()
                .filter(|element| selectable(element) && lasso_selects(element, polygon, mode))
                .map(|element| element.id.clone()),
        )
    }

    pub fn translate_selection(&mut self, selection: &SelectionSet, delta: [f32; 2]) -> usize {
        self.translate_selection_with_frame_children(selection, delta)
    }
}

fn selectable(element: &DrawingElement) -> bool {
    !element.is_deleted && !element.is_locked()
}

fn normalized_rect(start: [f32; 2], end: [f32; 2]) -> (f32, f32, f32, f32) {
    (
        start[0].min(end[0]),
        start[1].min(end[1]),
        start[0].max(end[0]),
        start[1].max(end[1]),
    )
}

fn rect_selects(element: &DrawingElement, rect: (f32, f32, f32, f32), mode: SelectionMode) -> bool {
    let outline = element_outline(element);
    match mode {
        SelectionMode::Contained => outline.iter().all(|point| point_in_rect(*point, rect)),
        SelectionMode::Touching => {
            outline.iter().any(|point| point_in_rect(*point, rect))
                || rect_corners(rect)
                    .iter()
                    .any(|point| element.hit_test_with_tolerance(*point, 0.0))
                || polyline_intersects_rect(&outline, !element.is_linear(), rect)
        }
    }
}

fn lasso_selects(element: &DrawingElement, polygon: &[[f32; 2]], mode: SelectionMode) -> bool {
    let outline = element_outline(element);
    match mode {
        SelectionMode::Contained => outline
            .iter()
            .all(|point| point_in_polygon(*point, polygon)),
        SelectionMode::Touching => {
            outline
                .iter()
                .any(|point| point_in_polygon(*point, polygon))
                || polygon
                    .iter()
                    .any(|point| element.hit_test_with_tolerance(*point, 0.0))
                || polylines_intersect(&outline, !element.is_linear(), polygon, true)
        }
    }
}

fn element_outline(element: &DrawingElement) -> Vec<[f32; 2]> {
    let (x, y, width, height) = element.bounds();
    let center = [x + width / 2.0, y + height / 2.0];
    if element.is_linear() && !element.points.is_empty() {
        return element
            .points
            .iter()
            .map(|[px, py]| rotate_point([element.x + px, element.y + py], center, element.angle))
            .collect();
    }
    [
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ]
    .into_iter()
    .map(|point| rotate_point(point, center, element.angle))
    .collect()
}

fn outline_bounds(outline: &[[f32; 2]]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for [x, y] in outline {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    (min_x, min_y, max_x, max_y)
}

fn rect_corners(rect: (f32, f32, f32, f32)) -> [[f32; 2]; 4] {
    [
        [rect.0, rect.1],
        [rect.2, rect.1],
        [rect.2, rect.3],
        [rect.0, rect.3],
    ]
}

fn point_in_rect(point: [f32; 2], rect: (f32, f32, f32, f32)) -> bool {
    point[0] >= rect.0 && point[0] <= rect.2 && point[1] >= rect.1 && point[1] <= rect.3
}

fn point_in_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if point_on_segment(point, previous, current) {
            return true;
        }
        let crosses = (current[1] > point[1]) != (previous[1] > point[1]);
        if crosses {
            let denominator = previous[1] - current[1];
            if denominator.abs() > f32::EPSILON {
                let intersection =
                    (previous[0] - current[0]) * (point[1] - current[1]) / denominator + current[0];
                if point[0] < intersection {
                    inside = !inside;
                }
            }
        }
        previous = current;
    }
    inside
}

fn polyline_intersects_rect(
    outline: &[[f32; 2]],
    closed: bool,
    rect: (f32, f32, f32, f32),
) -> bool {
    let corners = rect_corners(rect);
    polylines_intersect(outline, closed, &corners, true)
}

fn polylines_intersect(
    first: &[[f32; 2]],
    first_closed: bool,
    second: &[[f32; 2]],
    second_closed: bool,
) -> bool {
    for (a, b) in segments(first, first_closed) {
        for (c, d) in segments(second, second_closed) {
            if segments_intersect(a, b, c, d) {
                return true;
            }
        }
    }
    false
}

fn segments(points: &[[f32; 2]], closed: bool) -> Vec<([f32; 2], [f32; 2])> {
    let mut segments = points
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect::<Vec<_>>();
    if closed && points.len() > 2 {
        segments.push((points[points.len() - 1], points[0]));
    }
    segments
}

fn segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);

    if ab_c * ab_d < 0.0 && cd_a * cd_b < 0.0 {
        return true;
    }
    (ab_c.abs() <= f32::EPSILON && point_on_segment(c, a, b))
        || (ab_d.abs() <= f32::EPSILON && point_on_segment(d, a, b))
        || (cd_a.abs() <= f32::EPSILON && point_on_segment(a, c, d))
        || (cd_b.abs() <= f32::EPSILON && point_on_segment(b, c, d))
}

fn orientation(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn point_on_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> bool {
    orientation(start, end, point).abs() <= 0.0001
        && point[0] >= start[0].min(end[0]) - 0.0001
        && point[0] <= start[0].max(end[0]) + 0.0001
        && point[1] >= start[1].min(end[1]) - 0.0001
        && point[1] <= start[1].max(end[1]) + 0.0001
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
