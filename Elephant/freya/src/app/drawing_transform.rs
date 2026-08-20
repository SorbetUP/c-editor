use freya::prelude::*;

use super::drawing_scene::{DrawingCanvasState, DrawingScene, SelectionSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransformHandle {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    Rotation,
}

impl TransformHandle {
    fn label(self) -> &'static str {
        match self {
            Self::NorthWest => "Drawing resize handle nw",
            Self::North => "Drawing resize handle n",
            Self::NorthEast => "Drawing resize handle ne",
            Self::East => "Drawing resize handle e",
            Self::SouthEast => "Drawing resize handle se",
            Self::South => "Drawing resize handle s",
            Self::SouthWest => "Drawing resize handle sw",
            Self::West => "Drawing resize handle w",
            Self::Rotation => "Drawing rotation handle",
        }
    }
}

#[derive(Clone, Debug)]
struct TransformDrag {
    handle: TransformHandle,
    original: DrawingScene,
    selection: SelectionSet,
    bounds: (f32, f32, f32, f32),
    start_angle: f32,
}

#[derive(PartialEq)]
pub(super) struct TransformHandles {
    pub canvas: State<DrawingCanvasState>,
}

impl Component for TransformHandles {
    fn render(&self) -> impl IntoElement {
        let canvas = self.canvas;
        let snapshot = canvas.read();
        if !matches!(snapshot.active_tool.as_str(), "selection" | "lasso") {
            return rect().width(Size::px(0.0)).height(Size::px(0.0)).into_element();
        }
        let selection = SelectionSet::from_ids(
            snapshot
                .selected_element_ids()
                .into_iter()
                .map(str::to_owned),
        );
        let Some(bounds) = selection.bounds(&snapshot.document) else {
            return rect().width(Size::px(0.0)).height(Size::px(0.0)).into_element();
        };
        let viewport = snapshot.viewport;
        let can_rotate = selection.ids().any(|id| {
            snapshot.document.element_by_id(id).is_some_and(|element| {
                !matches!(element.kind.as_str(), "frame" | "magicframe")
                    && !element.is_deleted
                    && !element.is_locked()
            })
        });
        drop(snapshot);

        let drag = use_state(|| Option::<TransformDrag>::None);
        let (x, y, width, height) = bounds;
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let handles = [
            (TransformHandle::NorthWest, [x, y]),
            (TransformHandle::North, [cx, y]),
            (TransformHandle::NorthEast, [x + width, y]),
            (TransformHandle::East, [x + width, cy]),
            (TransformHandle::SouthEast, [x + width, y + height]),
            (TransformHandle::South, [cx, y + height]),
            (TransformHandle::SouthWest, [x, y + height]),
            (TransformHandle::West, [x, cy]),
        ];

        let mut children = handles
            .into_iter()
            .map(|(handle, position)| handle_node(canvas, drag, viewport, handle, position))
            .collect::<Vec<_>>();
        if can_rotate {
            children.push(handle_node(
                canvas,
                drag,
                viewport,
                TransformHandle::Rotation,
                [cx, y - 28.0 / viewport.zoom.max(f32::EPSILON)],
            ));
        }

        rect()
            .width(Size::px(0.0))
            .height(Size::px(0.0))
            .layer(Layer::OverlayLevel(40))
            .children(children)
            .into_element()
    }
}

fn handle_node(
    canvas: State<DrawingCanvasState>,
    drag: State<Option<TransformDrag>>,
    viewport: elephant_draw::Viewport,
    handle: TransformHandle,
    world_position: [f32; 2],
) -> Element {
    let screen = viewport.to_screen(world_position);
    let size = if handle == TransformHandle::Rotation { 10.0 } else { 9.0 };
    let mut down_canvas = canvas;
    let mut down_drag = drag;
    let mut move_canvas = canvas;
    let move_drag = drag;
    let mut up_drag = drag;

    rect()
        .position(
            Position::new_absolute()
                .left(screen[0] - size / 2.0)
                .top(screen[1] - size / 2.0),
        )
        .width(Size::px(size))
        .height(Size::px(size))
        .background(if handle == TransformHandle::Rotation {
            Color::from_rgb(105, 101, 219)
        } else {
            Color::WHITE
        })
        .border(
            Border::new()
                .fill(Color::from_rgb(105, 101, 219))
                .width(1.5),
        )
        .with_corner_radius(if handle == TransformHandle::Rotation {
            size / 2.0
        } else {
            2.0
        })
        .layer(Layer::OverlayLevel(42))
        .a11y_alt(handle.label())
        .on_pointer_down(move |event: Event<PointerEventData>| {
            if event.button() != Some(MouseButton::Left) || !event.is_primary() {
                return;
            }
            let screen = pointer_point(&event);
            let snapshot = down_canvas.read();
            let selection = SelectionSet::from_ids(
                snapshot
                    .selected_element_ids()
                    .into_iter()
                    .map(str::to_owned),
            );
            let Some(bounds) = selection.bounds(&snapshot.document) else {
                return;
            };
            let original = snapshot.document.clone();
            let world = snapshot.to_world(screen);
            let center = [bounds.0 + bounds.2 / 2.0, bounds.1 + bounds.3 / 2.0];
            let start_angle = (world[1] - center[1]).atan2(world[0] - center[0]);
            drop(snapshot);
            down_canvas.write().checkpoint();
            down_drag.set(Some(TransformDrag {
                handle,
                original,
                selection,
                bounds,
                start_angle,
            }));
            event.stop_propagation();
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            let Some(active) = move_drag.read().clone() else {
                return;
            };
            let mut state = move_canvas.write();
            let world = state.to_world(pointer_point(&event));
            state.document = active.original.clone();
            let outcome = if active.handle == TransformHandle::Rotation {
                let center = [
                    active.bounds.0 + active.bounds.2 / 2.0,
                    active.bounds.1 + active.bounds.3 / 2.0,
                ];
                let angle = (world[1] - center[1]).atan2(world[0] - center[0]);
                state
                    .document
                    .rotate_selection(&active.selection, angle - active.start_angle)
            } else {
                let (anchor, scale) = resize_transform(active.handle, active.bounds, world);
                state.document.scale_selection(&active.selection, anchor, scale)
            };
            if outcome.changed > 0 {
                state.revision = state.revision.wrapping_add(1);
            }
            event.stop_propagation();
        })
        .on_global_pointer_up(move |event: Event<PointerEventData>| {
            if up_drag.read().is_some() {
                up_drag.set(None);
                event.stop_propagation();
            }
        })
        .into_element()
}

fn resize_transform(
    handle: TransformHandle,
    bounds: (f32, f32, f32, f32),
    pointer: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let (x, y, width, height) = bounds;
    let right = x + width;
    let bottom = y + height;
    let safe_width = width.max(1.0);
    let safe_height = height.max(1.0);
    let clamp = |value: f32| value.max(0.01);

    match handle {
        TransformHandle::NorthWest => (
            [right, bottom],
            [
                clamp((right - pointer[0]) / safe_width),
                clamp((bottom - pointer[1]) / safe_height),
            ],
        ),
        TransformHandle::North => (
            [x, bottom],
            [1.0, clamp((bottom - pointer[1]) / safe_height)],
        ),
        TransformHandle::NorthEast => (
            [x, bottom],
            [
                clamp((pointer[0] - x) / safe_width),
                clamp((bottom - pointer[1]) / safe_height),
            ],
        ),
        TransformHandle::East => (
            [x, y],
            [clamp((pointer[0] - x) / safe_width), 1.0],
        ),
        TransformHandle::SouthEast => (
            [x, y],
            [
                clamp((pointer[0] - x) / safe_width),
                clamp((pointer[1] - y) / safe_height),
            ],
        ),
        TransformHandle::South => (
            [x, y],
            [1.0, clamp((pointer[1] - y) / safe_height)],
        ),
        TransformHandle::SouthWest => (
            [right, y],
            [
                clamp((right - pointer[0]) / safe_width),
                clamp((pointer[1] - y) / safe_height),
            ],
        ),
        TransformHandle::West => (
            [right, y],
            [clamp((right - pointer[0]) / safe_width), 1.0],
        ),
        TransformHandle::Rotation => ([x, y], [1.0, 1.0]),
    }
}

fn pointer_point(event: &Event<PointerEventData>) -> [f32; 2] {
    let point = event.global_location();
    [point.x as f32, point.y as f32]
}
