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
        "    rgba, DrawingElement, DrawingScene, HistoryState, SelectionMode, SelectionSet, Viewport,\n",
        "    rgba, DrawingElement, DrawingScene, HistoryState, SelectionMode, SelectionSet, SnapGuide, Viewport,\n",
        "snap guide import",
    )
    text = replace_once(
        text,
        "    history: HistoryState,\n    pub revision: u64,",
        "    history: HistoryState,\n    snap_guides: Vec<SnapGuide>,\n    pub revision: u64,",
        "snap guide state",
    )
    text = replace_once(
        text,
        "            history: HistoryState::default(),\n            revision: 0,",
        "            history: HistoryState::default(),\n            snap_guides: Vec::new(),\n            revision: 0,",
        "snap guide initialization",
    )

    methods_anchor = '''    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }
'''
    methods = '''    pub fn snap_guides(&self) -> &[SnapGuide] {
        &self.snap_guides
    }

    pub fn duplicate_selection(&mut self) -> bool {
        let selection = if self.selection.is_empty() {
            let Some(element) = self.selected_element() else {
                return false;
            };
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let before = self.document.clone();
        let outcome = self.document.duplicate_selection_excalidraw(
            &selection,
            [10.0, 10.0],
            self.revision.wrapping_add(1),
        );
        if outcome.changed == 0 {
            return false;
        }
        self.history.push(before);
        self.selection = outcome.selection;
        self.sync_primary_from_selection();
        self.interaction = Interaction::None;
        self.snap_guides.clear();
        self.changed();
        true
    }

'''
    text = replace_once(text, methods_anchor, methods + methods_anchor, "duplicate and snap methods")

    move_element_old = '''                let target = [world[0] - offset[0], world[1] - offset[1]];

                if matches!(kind.as_str(), "frame" | "magicframe") {
                    let delta = [target[0] - origin[0], target[1] - origin[1]];
                    if self.document.translate_frame_with_children(&id, delta) > 0 {
                        self.changed();
                    }
                } else if let Some(element) = self.document.elements.get_mut(index) {
                    element.x = target[0];
                    element.y = target[1];
                    mark_changed(element);
                    self.changed();
                }
'''
    move_element_new = '''                let target = [world[0] - offset[0], world[1] - offset[1]];
                let requested = [target[0] - origin[0], target[1] - origin[1]];
                let selection = SelectionSet::from_ids(std::iter::once(id.clone()));
                let snap = self.document.snap_selection_delta(
                    &selection,
                    requested,
                    6.0 / self.viewport.zoom.max(0.01),
                );
                let delta = snap.delta;
                self.snap_guides = snap.guides;

                if matches!(kind.as_str(), "frame" | "magicframe") {
                    if self.document.translate_frame_with_children(&id, delta) > 0 {
                        self.changed();
                    }
                } else if self.document.translate_element(&id, delta) {
                    self.changed();
                }
'''
    text = replace_once(text, move_element_old, move_element_new, "single selection snapping")

    move_selection_old = '''                let selection = self.selection.clone();
                let changed = self.document.translate_selection(&selection, delta);
                self.interaction = Interaction::MoveSelection {
                    last_world: world,
                    checkpointed: checkpointed || changed > 0,
                };
'''
    move_selection_new = '''                let selection = self.selection.clone();
                let snap = self.document.snap_selection_delta(
                    &selection,
                    delta,
                    6.0 / self.viewport.zoom.max(0.01),
                );
                self.snap_guides = snap.guides;
                let changed = self.document.translate_selection(&selection, snap.delta);
                self.interaction = Interaction::MoveSelection {
                    last_world: world,
                    checkpointed: checkpointed || changed > 0,
                };
'''
    text = replace_once(text, move_selection_old, move_selection_new, "multi selection snapping")

    text = replace_once(
        text,
        "        self.interaction = Interaction::None;\n\n        let mut membership_changed = false;",
        "        self.interaction = Interaction::None;\n        self.snap_guides.clear();\n\n        let mut membership_changed = false;",
        "clear snap guides on pointer end",
    )
    text = replace_once(
        text,
        "        self.clear_selection();\n        self.interaction = Interaction::None;\n        self.set_active_tool_label(\"Selection\");",
        "        self.clear_selection();\n        self.interaction = Interaction::None;\n        self.snap_guides.clear();\n        self.set_active_tool_label(\"Selection\");",
        "clear snap guides on cancel",
    )
    path.write_text(text)


def sync_render() -> None:
    path = Path("Elephant/freya/src/app/drawing_render.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "use super::drawing_scene::{DrawingCanvasState, DrawingElement, Viewport};",
        "use super::drawing_scene::{DrawingCanvasState, DrawingElement, SnapGuide, Viewport};",
        "render snap import",
    )
    text = replace_once(
        text,
        "    output\n}\n\nfn render_element(",
        '''    for (index, guide) in state.snap_guides().iter().enumerate() {
        output.push(snap_guide(guide, state.viewport, index));
    }
    output
}

fn snap_guide(guide: &SnapGuide, viewport: Viewport, index: usize) -> Element {
    let (left, top, width, height, axis) = match guide.axis {
        elephant_draw::SnapAxis::X => (
            viewport.pan[0] + guide.position * viewport.zoom,
            viewport.pan[1] + guide.start * viewport.zoom,
            1.0,
            ((guide.end - guide.start).abs() * viewport.zoom).max(1.0),
            "x",
        ),
        elephant_draw::SnapAxis::Y => (
            viewport.pan[0] + guide.start * viewport.zoom,
            viewport.pan[1] + guide.position * viewport.zoom,
            ((guide.end - guide.start).abs() * viewport.zoom).max(1.0),
            1.0,
            "y",
        ),
    };
    rect()
        .key(("drawing-snap-guide", index, axis))
        .position(Position::new_absolute().left(left).top(top))
        .width(Size::px(width))
        .height(Size::px(height))
        .background(Color::from_rgb(105, 101, 219))
        .a11y_alt(format!("Drawing snap guide {axis}"))
        .into_element()
}

fn render_element(''',
        "render snap guides",
    )
    path.write_text(text)


def sync_keyboard() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    anchor = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("s")) {'''
    replacement = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("d")) {
                        keyboard_canvas.write().duplicate_selection();
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("s")) {'''
    text = replace_once(text, anchor, replacement, "duplicate keyboard shortcut")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_render()
    sync_keyboard()
