use elephant_draw::SelectionSet;
use freya::prelude::*;

use super::drawing_scene::{DrawingCanvasState, DrawingScene, Viewport};

const HANDLE_SIZE: f32 = 10.0;
const ROTATE_OFFSET: f32 = 28.0;
const MIN_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResizeHandle {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

impl ResizeHandle {
    const ALL: [Self; 8] = [
        Self::NorthWest,
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::NorthWest => "northwest",
            Self::North => "north",
            Self::NorthEast => "northeast",
            Self::East => "east",
            Self::SouthEast => "southeast",
            Self::South => "south",
            Self::SouthWest => "southwest",
            Self::West => "west",
        }
    }

    fn factors(self) -> [f32; 2] {
        match self {
            Self::NorthWest => [0.0, 0.0],
            Self::North => [0.5, 0.0],
            Self::NorthEast => [1.0, 0.0],
            Self::East => [1.0, 0.5],
            Self::SouthEast => [1.0, 1.0],
            Self::South => [0.5, 1.0],
            Self::SouthWest => [0.0, 1.0],
            Self::West => [0.0, 0.5],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransformMode {
    Resize(ResizeHandle),
    Rotate { start_angle: f32 },
}

#[derive(Clone, Debug, PartialEq)]
struct TransformGesture {
    mode: TransformMode,
    before: DrawingScene,
    selection: SelectionSet,
    bounds: (f32, f32, f32, f32),
    start_world: [f32; 2],
}

pub(super) fn selection_controls(canvas: State<DrawingCanvasState>) -> Element {
    let gesture_state = use_state(|| Option::<TransformGesture>::None);
    let snapshot = canvas.read();
    let viewport = snapshot.viewport;
    let bounds = snapshot.selection_bounds();
    let active = snapshot.active_tool == "selection";
    let has_rotatable = snapshot.selected_element_ids().into_iter().any(|id| {
        snapshot
            .document
            .element_by_id(id)
            .is_some_and(|element| !matches!(element.kind.as_str(), "frame" | "magicframe"))
    });
    drop(snapshot);

    let mut move_canvas = canvas;
    let move_gesture = gesture_state;
    let mut release_gesture = gesture_state;

    let mut root = rect()
        .position(
            Position::new_absolute()
                .left(0.0)
                .right(0.0)
                .top(0.0)
                .bottom(0.0),
        )
        .width(Size::fill())
        .height(Size::fill())
        .a11y_alt("Drawing selection controls")
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            let Some(gesture) = move_gesture.read().clone() else {
                return;
            };
            let world = move_canvas
                .read()
                .viewport
                .to_world(cursor_point(event.global_location()));
            apply_transform(&mut move_canvas, &gesture, world);
            event.stop_propagation();
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
            if release_gesture.read().is_some() {
                release_gesture.set(None);
                event.stop_propagation();
            }
        });

    let Some(bounds) = bounds.filter(|_| active) else {
        return root.into_element();
    };

    let children = ResizeHandle::ALL
        .into_iter()
        .map(|handle| resize_handle(canvas, gesture_state, viewport, bounds, handle))
        .collect::<Vec<_>>();
    root = root.children(children);

    if has_rotatable {
        root = root.child(rotate_handle(canvas, gesture_state, viewport, bounds));
    }

    root.into_element()
}

fn resize_handle(
    canvas: State<DrawingCanvasState>,
    gesture: State<Option<TransformGesture>>,
    viewport: Viewport,
    bounds: (f32, f32, f32, f32),
    handle: ResizeHandle,
) -> Element {
    let factors = handle.factors();
    let world = [
        bounds.0 + bounds.2 * factors[0],
        bounds.1 + bounds.3 * factors[1],
    ];
    let screen = to_screen(viewport, world);
    let mut canvas = canvas;
    let mut gesture = gesture;

    rect()
        .position(
            Position::new_absolute()
                .left(screen[0] - HANDLE_SIZE / 2.0)
                .top(screen[1] - HANDLE_SIZE / 2.0),
        )
        .width(Size::px(HANDLE_SIZE))
        .height(Size::px(HANDLE_SIZE))
        .background(Color::WHITE)
        .border(Border::new().fill(Color::from_rgb(105, 101, 219)).width(1.5))
        .with_corner_radius(2.0)
        .layer(Layer::OverlayLevel(42))
        .a11y_alt(format!("Drawing resize handle {}", handle.label()))
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.button() != Some(MouseButton::Left) || !event.is_primary() {
                return;
            }
            let snapshot = canvas.read();
            let Some(bounds) = snapshot.selection_bounds() else {
                return;
            };
            let selection = SelectionSet::from_ids(
                snapshot
                    .selected_element_ids()
                    .into_iter()
                    .map(str::to_owned),
            );
            if selection.is_empty() {
                return;
            }
            let start_world = snapshot
                .viewport
                .to_world(cursor_point(event.global_location()));
            let before = snapshot.document.clone();
            drop(snapshot);
            canvas.write().checkpoint();
            gesture.set(Some(TransformGesture {
                mode: TransformMode::Resize(handle),
                before,
                selection,
                bounds,
                start_world,
            }));
            event.stop_propagation();
        })
        .into_element()
}

fn rotate_handle(
    canvas: State<DrawingCanvasState>,
    gesture: State<Option<TransformGesture>>,
    viewport: Viewport,
    bounds: (f32, f32, f32, f32),
) -> Element {
    let world = [
        bounds.0 + bounds.2 / 2.0,
        bounds.1 - ROTATE_OFFSET / viewport.zoom.max(0.01),
    ];
    let screen = to_screen(viewport, world);
    let mut canvas = canvas;
    let mut gesture = gesture;

    rect()
        .position(
            Position::new_absolute()
                .left(screen[0] - HANDLE_SIZE / 2.0)
                .top(screen[1] - HANDLE_SIZE / 2.0),
        )
        .width(Size::px(HANDLE_SIZE))
        .height(Size::px(HANDLE_SIZE))
        .background(Color::WHITE)
        .border(Border::new().fill(Color::from_rgb(105, 101, 219)).width(1.5))
        .with_corner_radius(HANDLE_SIZE)
        .layer(Layer::OverlayLevel(42))
        .a11y_alt("Drawing rotate handle")
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.button() != Some(MouseButton::Left) || !event.is_primary() {
                return;
            }
            let snapshot = canvas.read();
            let Some(bounds) = snapshot.selection_bounds() else {
                return;
            };
            let selection = SelectionSet::from_ids(
                snapshot
                    .selected_element_ids()
                    .into_iter()
                    .map(str::to_owned),
            );
            if selection.is_empty() {
                return;
            }
            let start_world = snapshot
                .viewport
                .to_world(cursor_point(event.global_location()));
            let center = [bounds.0 + bounds.2 / 2.0, bounds.1 + bounds.3 / 2.0];
            let start_angle = (start_world[1] - center[1]).atan2(start_world[0] - center[0]);
            let before = snapshot.document.clone();
            drop(snapshot);
            canvas.write().checkpoint();
            gesture.set(Some(TransformGesture {
                mode: TransformMode::Rotate { start_angle },
                before,
                selection,
                bounds,
                start_world,
            }));
            event.stop_propagation();
        })
        .into_element()
}

fn apply_transform(
    canvas: &mut State<DrawingCanvasState>,
    gesture: &TransformGesture,
    world: [f32; 2],
) {
    let mut state = canvas.write();
    state.document = gesture.before.clone();
    let outcome = match gesture.mode {
        TransformMode::Resize(handle) => {
            let (anchor, scale) = resize_geometry(handle, gesture.bounds, gesture.start_world, world);
            state.document.scale_selection(&gesture.selection, anchor, scale)
        }
        TransformMode::Rotate { start_angle } => {
            let center = [
                gesture.bounds.0 + gesture.bounds.2 / 2.0,
                gesture.bounds.1 + gesture.bounds.3 / 2.0,
            ];
            let current_angle = (world[1] - center[1]).atan2(world[0] - center[0]);
            state
                .document
                .rotate_selection(&gesture.selection, current_angle - start_angle)
        }
    };
    if outcome.changed > 0 {
        state.revision = state.revision.wrapping_add(1);
    }
}

fn resize_geometry(
    handle: ResizeHandle,
    bounds: (f32, f32, f32, f32),
    start_world: [f32; 2],
    world: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let (x, y, width, height) = bounds;
    let (mut left, mut top, mut right, mut bottom) = (x, y, x + width, y + height);
    let delta = [world[0] - start_world[0], world[1] - start_world[1]];

    if matches!(
        handle,
        ResizeHandle::NorthWest | ResizeHandle::West | ResizeHandle::SouthWest
    ) {
        left = (left + delta[0]).min(right - MIN_SIZE);
    }
    if matches!(
        handle,
        ResizeHandle::NorthEast | ResizeHandle::East | ResizeHandle::SouthEast
    ) {
        right = (right + delta[0]).max(left + MIN_SIZE);
    }
    if matches!(
        handle,
        ResizeHandle::NorthWest | ResizeHandle::North | ResizeHandle::NorthEast
    ) {
        top = (top + delta[1]).min(bottom - MIN_SIZE);
    }
    if matches!(
        handle,
        ResizeHandle::SouthWest | ResizeHandle::South | ResizeHandle::SouthEast
    ) {
        bottom = (bottom + delta[1]).max(top + MIN_SIZE);
    }

    let affects_x = matches!(
        handle,
        ResizeHandle::NorthWest
            | ResizeHandle::NorthEast
            | ResizeHandle::East
            | ResizeHandle::SouthEast
            | ResizeHandle::SouthWest
            | ResizeHandle::West
    );
    let affects_y = matches!(
        handle,
        ResizeHandle::NorthWest
            | ResizeHandle::North
            | ResizeHandle::NorthEast
            | ResizeHandle::SouthEast
            | ResizeHandle::South
            | ResizeHandle::SouthWest
    );
    let scale = [
        if affects_x {
            ((right - left) / width.max(MIN_SIZE)).max(0.001)
        } else {
            1.0
        },
        if affects_y {
            ((bottom - top) / height.max(MIN_SIZE)).max(0.001)
        } else {
            1.0
        },
    ];
    let anchor = [
        if matches!(
            handle,
            ResizeHandle::NorthWest | ResizeHandle::West | ResizeHandle::SouthWest
        ) {
            x + width
        } else {
            x
        },
        if matches!(
            handle,
            ResizeHandle::NorthWest | ResizeHandle::North | ResizeHandle::NorthEast
        ) {
            y + height
        } else {
            y
        },
    ];
    (anchor, scale)
}

fn to_screen(viewport: Viewport, world: [f32; 2]) -> [f32; 2] {
    [
        viewport.pan[0] + world[0] * viewport.zoom,
        viewport.pan[1] + world[1] * viewport.zoom,
    ]
}

fn cursor_point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn southeast_resize_keeps_northwest_anchor() {
        let (anchor, scale) = resize_geometry(
            ResizeHandle::SouthEast,
            (10.0, 20.0, 100.0, 50.0),
            [110.0, 70.0],
            [160.0, 95.0],
        );
        assert_eq!(anchor, [10.0, 20.0]);
        assert_eq!(scale, [1.5, 1.5]);
    }

    #[test]
    fn west_resize_keeps_east_anchor_and_only_changes_x() {
        let (anchor, scale) = resize_geometry(
            ResizeHandle::West,
            (10.0, 20.0, 100.0, 50.0),
            [10.0, 45.0],
            [-15.0, 45.0],
        );
        assert_eq!(anchor, [110.0, 20.0]);
        assert_eq!(scale, [1.25, 1.0]);
    }
}
