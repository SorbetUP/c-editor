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

    text = replace_once(
        text,
        '''    BoxSelect {
        start_world: [f32; 2],
        current_world: [f32; 2],
    },
    LassoSelect {
        points: Vec<[f32; 2]>,
    },''',
        '''    BoxSelect {
        start_world: [f32; 2],
        current_world: [f32; 2],
        base_selection: SelectionSet,
    },
    LassoSelect {
        points: Vec<[f32; 2]>,
        base_selection: SelectionSet,
    },''',
        "additive selection interaction state",
    )
    text = replace_once(
        text,
        '''        let Interaction::BoxSelect {
            start_world,
            current_world,
        } = self.interaction''',
        '''        let Interaction::BoxSelect {
            start_world,
            current_world,
            ..
        } = self.interaction''',
        "additive marquee destructuring",
    )
    text = replace_once(
        text,
        '''            Interaction::LassoSelect { points } => Some(points.as_slice()),''',
        '''            Interaction::LassoSelect { points, .. } => Some(points.as_slice()),''',
        "additive lasso accessor",
    )
    text = replace_once(
        text,
        '''                self.interaction = Interaction::LassoSelect {
                    points: vec![world],
                };''',
        '''                self.interaction = Interaction::LassoSelect {
                    points: vec![world],
                    base_selection: SelectionSet::new(),
                };''',
        "normal lasso base selection",
    )
    text = replace_once(
        text,
        '''                    self.interaction = Interaction::BoxSelect {
                        start_world: world,
                        current_world: world,
                    };''',
        '''                    self.interaction = Interaction::BoxSelect {
                        start_world: world,
                        current_world: world,
                        base_selection: SelectionSet::new(),
                    };''',
        "normal box base selection",
    )

    begin_anchor = '''    pub(crate) fn begin_pointer(&mut self, point: [f32; 2]) {
        let world = self.to_world(point);'''
    begin_new = '''    pub(crate) fn begin_pointer_with_additive(&mut self, point: [f32; 2], additive: bool) {
        if !additive || !matches!(self.active_tool.as_str(), "selection" | "lasso") {
            self.begin_pointer(point);
            return;
        }
        let world = self.to_world(point);
        match self.active_tool.as_str() {
            "lasso" => {
                self.interaction = Interaction::LassoSelect {
                    points: vec![world],
                    base_selection: self.selection.clone(),
                };
                self.changed();
            }
            "selection" => {
                if self.hit_transform_handle(world).is_some() {
                    self.begin_pointer(point);
                    return;
                }
                if let Some(index) = self.hit_test(world) {
                    let id = self.document.elements[index].id.clone();
                    let selected_after_toggle = self.selection.toggle(id);
                    self.sync_primary_from_selection();
                    self.interaction = if selected_after_toggle {
                        Interaction::MoveSelection {
                            last_world: world,
                            checkpointed: false,
                        }
                    } else {
                        Interaction::None
                    };
                    self.changed();
                } else {
                    self.interaction = Interaction::BoxSelect {
                        start_world: world,
                        current_world: world,
                        base_selection: self.selection.clone(),
                    };
                }
            }
            _ => self.begin_pointer(point),
        }
    }

    pub(crate) fn begin_pointer(&mut self, point: [f32; 2]) {
        let world = self.to_world(point);'''
    text = replace_once(text, begin_anchor, begin_new, "additive pointer entrypoint")

    box_old = '''            Interaction::BoxSelect {
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
            }'''
    box_new = '''            Interaction::BoxSelect {
                start_world,
                current_world: _,
                base_selection,
            } => {
                let world = self.to_world(point);
                let mut next = self
                    .document
                    .select_in_rect(start_world, world, SelectionMode::Contained);
                for id in base_selection.ids() {
                    next.insert(id.to_owned());
                }
                self.selection = next;
                self.selected = None;
                self.interaction = Interaction::BoxSelect {
                    start_world,
                    current_world: world,
                    base_selection,
                };
                self.changed();
            }'''
    text = replace_once(text, box_old, box_new, "additive box selection")

    lasso_old = '''            Interaction::LassoSelect { mut points } => {
                let world = self.to_world(point);
                let should_append = points.last().is_none_or(|last| {
                    let dx = world[0] - last[0];
                    let dy = world[1] - last[1];
                    dx * dx + dy * dy >= 4.0
                });
                if should_append {
                    points.push(world);
                }
                self.selection = self.document.select_in_lasso(&points, SelectionMode::Contained);
                self.selected = None;
                self.interaction = Interaction::LassoSelect { points };
                self.changed();
            }'''
    lasso_new = '''            Interaction::LassoSelect {
                mut points,
                base_selection,
            } => {
                let world = self.to_world(point);
                let should_append = points.last().is_none_or(|last| {
                    let dx = world[0] - last[0];
                    let dy = world[1] - last[1];
                    dx * dx + dy * dy >= 4.0
                });
                if should_append {
                    points.push(world);
                }
                let mut next = self
                    .document
                    .select_in_lasso(&points, SelectionMode::Contained);
                for id in base_selection.ids() {
                    next.insert(id.to_owned());
                }
                self.selection = next;
                self.selected = None;
                self.interaction = Interaction::LassoSelect {
                    points,
                    base_selection,
                };
                self.changed();
            }'''
    text = replace_once(text, lasso_old, lasso_new, "additive lasso selection")
    path.write_text(text)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    text = replace_once(
        text,
        '''    let pointer_down_state = use_state(|| false);
    let editing_text_state = use_state(|| Option::<usize>::None);''',
        '''    let pointer_down_state = use_state(|| false);
    let selection_modifiers = use_state(Modifiers::empty);
    let editing_text_state = use_state(|| Option::<usize>::None);''',
        "modifier state",
    )
    text = replace_once(
        text,
        '''    let mut pointer_state = state;
    let mut move_state = state;''',
        '''    let mut pointer_state = state;
    let pointer_modifiers = selection_modifiers;
    let mut key_modifiers_down = selection_modifiers;
    let mut key_modifiers_up = selection_modifiers;
    let mut move_state = state;''',
        "modifier captures",
    )
    pointer_old = '''                DrawingTool::Selection
                | DrawingTool::Lasso
                | DrawingTool::Hand
                | DrawingTool::Eraser => {
                    pointer_state.write().begin_pointer(screen);
                    pointer_gesture.set(None);
                }'''
    pointer_new = '''                DrawingTool::Selection | DrawingTool::Lasso => {
                    let modifiers = *pointer_modifiers.read();
                    let additive = modifiers.contains(Modifiers::SHIFT)
                        || modifiers.contains(Modifiers::ctrl_or_meta());
                    pointer_state
                        .write()
                        .begin_pointer_with_additive(screen, additive);
                    pointer_gesture.set(None);
                }
                DrawingTool::Hand | DrawingTool::Eraser => {
                    pointer_state.write().begin_pointer(screen);
                    pointer_gesture.set(None);
                }'''
    text = replace_once(text, pointer_old, pointer_new, "modifier pointer dispatch")
    text = replace_once(
        text,
        '''        .on_wheel(move |event: Event<WheelEventData>| {
            wheel_state''',
        '''        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            key_modifiers_down.set(event.modifiers);
        })
        .on_global_key_up(move |event: Event<KeyboardEventData>| {
            key_modifiers_up.set(event.modifiers);
        })
        .on_wheel(move |event: Event<WheelEventData>| {
            wheel_state''',
        "modifier keyboard tracking",
    )
    path.write_text(text)


def sync_test() -> None:
    path = Path("Elephant/freya/tests/drawing_additive_selection_contract.rs")
    path.write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::json;

fn fixture() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[
                {"id":"a","type":"rectangle","x":10,"y":10,"width":40,"height":40},
                {"id":"b","type":"rectangle","x":80,"y":10,"width":40,"height":40},
                {"id":"c","type":"rectangle","x":150,"y":10,"width":40,"height":40}
            ],
            "appState":{},
            "files":{}
        }))
        .expect("fixture json"),
    )
    .expect("valid fixture")
}

fn ids(canvas: &DrawingCanvasState) -> Vec<String> {
    canvas
        .selected_element_ids()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn additive_click_toggles_membership() {
    let mut canvas = fixture();
    canvas.begin_pointer([25.0, 25.0]);
    canvas.end_pointer();
    assert_eq!(ids(&canvas), vec!["a"]);

    canvas.begin_pointer_with_additive([95.0, 25.0], true);
    canvas.end_pointer();
    assert_eq!(ids(&canvas), vec!["a", "b"]);

    canvas.begin_pointer_with_additive([95.0, 25.0], true);
    canvas.end_pointer();
    assert_eq!(ids(&canvas), vec!["a"]);
}

#[test]
fn additive_box_unions_with_existing_selection() {
    let mut canvas = fixture();
    canvas.begin_pointer([25.0, 25.0]);
    canvas.end_pointer();
    canvas.begin_pointer_with_additive([140.0, 0.0], true);
    canvas.move_pointer([205.0, 60.0]);
    canvas.end_pointer();
    assert_eq!(ids(&canvas), vec!["a", "c"]);
}

#[test]
fn additive_lasso_unions_with_existing_selection() {
    let mut canvas = fixture();
    canvas.begin_pointer([25.0, 25.0]);
    canvas.end_pointer();
    canvas.set_active_tool_label("Lasso");
    canvas.begin_pointer_with_additive([70.0, 0.0], true);
    for point in [[130.0, 0.0], [130.0, 60.0], [70.0, 60.0], [70.0, 0.0]] {
        canvas.move_pointer(point);
    }
    canvas.end_pointer();
    assert_eq!(ids(&canvas), vec!["a", "b"]);
}
''')


def main() -> None:
    sync_scene()
    sync_canvas()
    sync_test()


if __name__ == "__main__":
    main()
