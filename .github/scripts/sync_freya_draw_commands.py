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
    anchor = '''    pub fn delete_selection(&mut self) -> bool {'''
    methods = '''    pub fn select_all(&mut self) -> bool {
        let next = SelectionSet::from_ids(
            self.document
                .elements
                .iter()
                .filter(|element| !element.is_deleted && !is_locked(element))
                .map(|element| element.id.clone()),
        );
        if next == self.selection && self.selected.is_none() {
            return false;
        }
        self.selection = next;
        self.sync_primary_from_selection();
        self.interaction = Interaction::None;
        self.snap_guides.clear();
        self.changed();
        true
    }

    pub fn group_selection(&mut self) -> bool {
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let ids = self
            .document
            .elements
            .iter()
            .filter(|element| {
                selection.contains(&element.id) && !element.is_deleted && !is_locked(element)
            })
            .map(|element| element.id.clone())
            .collect::<Vec<_>>();
        if ids.len() < 2 {
            return false;
        }
        let refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
        let group_id = format!("freya-group-{:016x}", self.revision.wrapping_add(1));
        let before = self.document.clone();
        if self.document.group_elements(&refs, &group_id) == 0 {
            return false;
        }
        self.history.push(before);
        self.changed();
        true
    }

    pub fn ungroup_selection(&mut self) -> bool {
        let selected = self.selected_element_ids();
        if selected.is_empty() {
            return false;
        }
        let mut groups = Vec::<String>::new();
        for id in selected {
            let Some(element) = self.document.element_by_id(id) else {
                continue;
            };
            let Some(group_ids) = element.extra.get("groupIds").and_then(Value::as_array) else {
                continue;
            };
            for group_id in group_ids.iter().filter_map(Value::as_str) {
                if !groups.iter().any(|candidate| candidate == group_id) {
                    groups.push(group_id.to_owned());
                }
            }
        }
        if groups.is_empty() {
            return false;
        }
        let before = self.document.clone();
        let changed = groups
            .iter()
            .map(|group_id| self.document.ungroup_elements(group_id))
            .sum::<usize>();
        if changed == 0 {
            return false;
        }
        self.history.push(before);
        self.changed();
        true
    }

    pub fn nudge_selection(&mut self, delta: [f32; 2]) -> bool {
        if !delta[0].is_finite() || !delta[1].is_finite() || delta == [0.0, 0.0] {
            return false;
        }
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let moved_ids = selection.ids().map(str::to_owned).collect::<Vec<_>>();
        let before = self.document.clone();
        let moved = self.document.translate_selection(&selection, delta);
        if moved == 0 {
            return false;
        }
        for id in moved_ids {
            let is_frame = self
                .document
                .element_by_id(&id)
                .is_some_and(|element| matches!(element.kind.as_str(), "frame" | "magicframe"));
            if !is_frame {
                self.document.sync_element_frame_membership(&id);
            }
        }
        self.history.push(before);
        self.sync_primary_from_selection();
        self.snap_guides.clear();
        self.changed();
        true
    }

'''
    text = replace_once(text, anchor, methods + anchor, "selection commands")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    command_anchor = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("c")) {'''
    command_new = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("a")) {
                        keyboard_canvas.write().select_all();
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("g")) {
                        if event.modifiers.contains(Modifiers::SHIFT) {
                            keyboard_canvas.write().ungroup_selection();
                        } else {
                            keyboard_canvas.write().group_selection();
                        }
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("c")) {'''
    text = replace_once(text, command_anchor, command_new, "selection command shortcuts")

    match_anchor = '''                match &event.key {
                    Key::Named(NamedKey::Escape) => {'''
    match_new = '''                match &event.key {
                    Key::Named(NamedKey::ArrowLeft)
                    | Key::Named(NamedKey::ArrowRight)
                    | Key::Named(NamedKey::ArrowUp)
                    | Key::Named(NamedKey::ArrowDown)
                        if !command =>
                    {
                        let amount = if event.modifiers.contains(Modifiers::SHIFT) { 10.0 } else { 1.0 };
                        let delta = match event.key {
                            Key::Named(NamedKey::ArrowLeft) => [-amount, 0.0],
                            Key::Named(NamedKey::ArrowRight) => [amount, 0.0],
                            Key::Named(NamedKey::ArrowUp) => [0.0, -amount],
                            Key::Named(NamedKey::ArrowDown) => [0.0, amount],
                            _ => [0.0, 0.0],
                        };
                        keyboard_canvas.write().nudge_selection(delta);
                        event.stop_propagation();
                    }
                    Key::Named(NamedKey::Escape) => {'''
    text = replace_once(text, match_anchor, match_new, "keyboard nudge shortcuts")
    path.write_text(text)


def sync_test() -> None:
    path = Path("Elephant/freya/tests/drawing_commands_contract.rs")
    path.write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::{json, Value};

fn fixture() -> DrawingCanvasState {
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[
                {"id":"a","type":"rectangle","x":10,"y":10,"width":30,"height":30},
                {"id":"b","type":"ellipse","x":70,"y":10,"width":30,"height":30},
                {"id":"locked","type":"rectangle","x":130,"y":10,"width":30,"height":30,"locked":true}
            ],
            "appState":{},"files":{}
        }))
        .expect("fixture json"),
    )
    .expect("valid fixture")
}

#[test]
fn select_all_group_and_ungroup_match_native_selection_contract() {
    let mut state = fixture();
    assert!(state.select_all());
    assert_eq!(state.selected_element_ids(), vec!["a", "b"]);
    assert!(state.group_selection());
    for id in ["a", "b"] {
        assert_eq!(
            state
                .document
                .element_by_id(id)
                .and_then(|element| element.extra.get("groupIds"))
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1)
        );
    }
    assert!(state.ungroup_selection());
    for id in ["a", "b"] {
        assert!(state
            .document
            .element_by_id(id)
            .and_then(|element| element.extra.get("groupIds"))
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty));
    }
}

#[test]
fn keyboard_nudge_semantics_are_atomic_and_undoable() {
    let mut state = fixture();
    state.begin_pointer([20.0, 20.0]);
    state.end_pointer();
    assert!(state.nudge_selection([10.0, -1.0]));
    let element = state.document.element_by_id("a").expect("selected element");
    assert_eq!((element.x, element.y), (20.0, 9.0));
    assert!(state.undo());
    let element = state.document.element_by_id("a").expect("restored element");
    assert_eq!((element.x, element.y), (10.0, 10.0));
}
''')


def main() -> None:
    sync_scene()
    sync_view()
    sync_test()


if __name__ == "__main__":
    main()
