#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def sync_scene() -> None:
    path = Path("Elephant/freya/src/app/drawing_scene.rs")
    text = path.read_text()

    interaction_anchor = '''    MoveSelection {
        last_world: [f32; 2],
        checkpointed: bool,
    },
'''
    interaction_new = interaction_anchor + '''    ResizeSelection {
        handle: TransformHandle,
        checkpointed: bool,
    },
    RotateSelection {
        last_world: [f32; 2],
        checkpointed: bool,
    },
'''
    text = replace_once(text, interaction_anchor, interaction_new, "transform interactions")

    struct_anchor = '''#[derive(Clone, Debug, PartialEq)]
pub struct DrawingCanvasState {'''
    enum_code = '''#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransformHandle {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
    Rotation,
}

impl TransformHandle {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::North => "n",
            Self::South => "s",
            Self::West => "w",
            Self::East => "e",
            Self::NorthWest => "nw",
            Self::NorthEast => "ne",
            Self::SouthWest => "sw",
            Self::SouthEast => "se",
            Self::Rotation => "rotation",
        }
    }
}

'''
    text = replace_once(text, struct_anchor, enum_code + struct_anchor, "transform handle enum")

    bounds_anchor = '''    pub fn selection_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        if !self.selection.is_empty() {
            return self.selection.bounds(&self.document);
        }
        self.selected_element().map(DrawingElement::bounds)
    }
'''
    handle_methods = '''
    pub(crate) fn transform_handle_positions(&self) -> Vec<(TransformHandle, [f32; 2])> {
        if self.selected_element_ids().is_empty() {
            return Vec::new();
        }
        let Some((x, y, width, height)) = self.selection_bounds() else {
            return Vec::new();
        };
        let mut handles = vec![
            (TransformHandle::NorthWest, [x, y]),
            (TransformHandle::NorthEast, [x + width, y]),
            (TransformHandle::SouthWest, [x, y + height]),
            (TransformHandle::SouthEast, [x + width, y + height]),
        ];
        let minimum = 40.0 / self.viewport.zoom.max(0.01);
        if width.abs() > minimum {
            handles.push((TransformHandle::North, [x + width / 2.0, y]));
            handles.push((TransformHandle::South, [x + width / 2.0, y + height]));
        }
        if height.abs() > minimum {
            handles.push((TransformHandle::West, [x, y + height / 2.0]));
            handles.push((TransformHandle::East, [x + width, y + height / 2.0]));
        }
        let selected = self.selected_element_ids();
        let frame_only = selected.len() == 1
            && self
                .document
                .element_by_id(selected[0])
                .is_some_and(|element| matches!(element.kind.as_str(), "frame" | "magicframe"));
        if !frame_only {
            handles.push((
                TransformHandle::Rotation,
                [x + width / 2.0, y - 24.0 / self.viewport.zoom.max(0.01)],
            ));
        }
        handles
    }

    fn hit_transform_handle(&self, world: [f32; 2]) -> Option<TransformHandle> {
        let radius = 9.0 / self.viewport.zoom.max(0.01);
        self.transform_handle_positions()
            .into_iter()
            .find_map(|(handle, point)| {
                ((world[0] - point[0]).abs() <= radius
                    && (world[1] - point[1]).abs() <= radius)
                    .then_some(handle)
            })
    }
'''
    text = replace_once(text, bounds_anchor, bounds_anchor + handle_methods, "transform handle methods")

    selection_begin = '''            "selection" => {
                if let Some(index) = self.hit_test(world) {'''
    selection_begin_new = '''            "selection" => {
                if let Some(handle) = self.hit_transform_handle(world) {
                    self.interaction = if handle == TransformHandle::Rotation {
                        Interaction::RotateSelection {
                            last_world: world,
                            checkpointed: false,
                        }
                    } else {
                        Interaction::ResizeSelection {
                            handle,
                            checkpointed: false,
                        }
                    };
                    return;
                }
                if let Some(index) = self.hit_test(world) {'''
    text = replace_once(text, selection_begin, selection_begin_new, "transform begin")

    move_anchor = '''            Interaction::BoxSelect {
                start_world,
                current_world: _,
            } => {'''
    transform_arms = '''            Interaction::ResizeSelection {
                handle,
                checkpointed,
            } => {
                let world = self.to_world(point);
                let selection = self.selection.clone();
                let Some(bounds) = selection.bounds(&self.document) else {
                    return;
                };
                let Some((anchor, scale)) = resize_scale(handle, bounds, world) else {
                    return;
                };
                if !checkpointed {
                    self.checkpoint();
                }
                let outcome = self.document.scale_selection(&selection, anchor, scale);
                self.interaction = Interaction::ResizeSelection {
                    handle,
                    checkpointed: checkpointed || outcome.changed > 0,
                };
                if outcome.changed > 0 {
                    self.changed();
                }
            }
            Interaction::RotateSelection {
                last_world,
                checkpointed,
            } => {
                let world = self.to_world(point);
                let selection = self.selection.clone();
                let Some((x, y, width, height)) = selection.bounds(&self.document) else {
                    return;
                };
                let center = [x + width / 2.0, y + height / 2.0];
                let previous = (last_world[1] - center[1]).atan2(last_world[0] - center[0]);
                let current = (world[1] - center[1]).atan2(world[0] - center[0]);
                let delta = normalize_angle_delta(current - previous);
                if delta.abs() <= f32::EPSILON {
                    return;
                }
                if !checkpointed {
                    self.checkpoint();
                }
                let outcome = self.document.rotate_selection(&selection, delta);
                self.interaction = Interaction::RotateSelection {
                    last_world: world,
                    checkpointed: checkpointed || outcome.changed > 0,
                };
                if outcome.changed > 0 {
                    self.changed();
                }
            }
'''
    text = replace_once(text, move_anchor, transform_arms + move_anchor, "transform move")

    moved_ids_old = '''            Interaction::MoveSelection { .. } => self.selection.ids().map(str::to_owned).collect(),
            _ => Vec::new(),'''
    moved_ids_new = '''            Interaction::MoveSelection { .. } | Interaction::ResizeSelection { .. } => {
                self.selection.ids().map(str::to_owned).collect()
            }
            _ => Vec::new(),'''
    text = replace_once(text, moved_ids_old, moved_ids_new, "resize membership reconciliation")

    match_none = '''            Interaction::None => {}
        }
    }
'''
    text = replace_once(text, match_none, match_none, "interaction exhaustiveness sentinel")

    helper_anchor = '''fn normalized_bounds(start: [f32; 2], end: [f32; 2]) -> (f32, f32, f32, f32) {'''
    helpers = '''fn resize_scale(
    handle: TransformHandle,
    bounds: (f32, f32, f32, f32),
    pointer: [f32; 2],
) -> Option<([f32; 2], [f32; 2])> {
    if handle == TransformHandle::Rotation {
        return None;
    }
    let (x, y, width, height) = bounds;
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return None;
    }
    let right = x + width;
    let bottom = y + height;
    let minimum = 1.0;
    let (anchor, sx, sy) = match handle {
        TransformHandle::NorthWest => (
            [right, bottom],
            ((right - pointer[0]).max(minimum) / width),
            ((bottom - pointer[1]).max(minimum) / height),
        ),
        TransformHandle::NorthEast => (
            [x, bottom],
            ((pointer[0] - x).max(minimum) / width),
            ((bottom - pointer[1]).max(minimum) / height),
        ),
        TransformHandle::SouthWest => (
            [right, y],
            ((right - pointer[0]).max(minimum) / width),
            ((pointer[1] - y).max(minimum) / height),
        ),
        TransformHandle::SouthEast => (
            [x, y],
            ((pointer[0] - x).max(minimum) / width),
            ((pointer[1] - y).max(minimum) / height),
        ),
        TransformHandle::North => (
            [x, bottom],
            1.0,
            ((bottom - pointer[1]).max(minimum) / height),
        ),
        TransformHandle::South => (
            [x, y],
            1.0,
            ((pointer[1] - y).max(minimum) / height),
        ),
        TransformHandle::West => (
            [right, y],
            ((right - pointer[0]).max(minimum) / width),
            1.0,
        ),
        TransformHandle::East => (
            [x, y],
            ((pointer[0] - x).max(minimum) / width),
            1.0,
        ),
        TransformHandle::Rotation => return None,
    };
    Some((anchor, [sx.max(0.01), sy.max(0.01)]))
}

fn normalize_angle_delta(delta: f32) -> f32 {
    let mut delta = delta.rem_euclid(std::f32::consts::TAU);
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    delta
}

'''
    text = replace_once(text, helper_anchor, helpers + helper_anchor, "transform helpers")
    path.write_text(text)


def sync_render() -> None:
    path = Path("Elephant/freya/src/app/drawing_render.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "use super::drawing_scene::{DrawingCanvasState, DrawingElement, SnapGuide, Viewport};",
        "use super::drawing_scene::{DrawingCanvasState, DrawingElement, SnapGuide, TransformHandle, Viewport};",
        "render transform import",
    )
    text = replace_once(
        text,
        '''    for (index, guide) in state.snap_guides().iter().enumerate() {
        output.push(snap_guide(guide, state.viewport, index));
    }
    output
}''',
        '''    for (index, guide) in state.snap_guides().iter().enumerate() {
        output.push(snap_guide(guide, state.viewport, index));
    }
    for (handle, point) in state.transform_handle_positions() {
        output.push(transform_handle(handle, point, state.viewport));
    }
    output
}''',
        "render transform handles",
    )
    helper_anchor = '''fn snap_guide(guide: &SnapGuide, viewport: Viewport, index: usize) -> Element {'''
    helper = '''fn transform_handle(handle: TransformHandle, point: [f32; 2], viewport: Viewport) -> Element {
    let size = if handle == TransformHandle::Rotation { 10.0 } else { 8.0 };
    let left = viewport.pan[0] + point[0] * viewport.zoom - size / 2.0;
    let top = viewport.pan[1] + point[1] * viewport.zoom - size / 2.0;
    rect()
        .key(("drawing-transform-handle", handle.id()))
        .position(Position::new_absolute().left(left).top(top))
        .width(Size::px(size))
        .height(Size::px(size))
        .background(Color::WHITE)
        .border(Border::new().fill(Color::from_rgb(105, 101, 219)).width(1.5))
        .with_corner_radius(if handle == TransformHandle::Rotation { size / 2.0 } else { 2.0 })
        .a11y_alt(format!("Drawing transform handle {}", handle.id()))
        .into_element()
}

'''
    text = replace_once(text, helper_anchor, helper + helper_anchor, "transform handle renderer")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_render()
