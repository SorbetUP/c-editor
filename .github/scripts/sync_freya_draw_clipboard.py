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
        "    rgba, DrawingElement, DrawingScene, HistoryState, SelectionMode, SelectionSet, SnapGuide, Viewport,\n",
        "    rgba, DrawingElement, DrawingScene, HistoryState, SceneFragment, SelectionMode, SelectionSet, SnapGuide, Viewport,\n",
        "clipboard fragment import",
    )

    anchor = '''    pub fn snap_guides(&self) -> &[SnapGuide] {
        &self.snap_guides
    }
'''
    methods = '''    pub fn copy_selection_fragment(&self) -> Option<SceneFragment> {
        let selection = if self.selection.is_empty() {
            let element = self.selected_element()?;
            SelectionSet::from_ids(std::iter::once(element.id.clone()))
        } else {
            self.selection.clone()
        };
        let fragment = self.document.copy_selection_fragment(&selection);
        (!fragment.elements.is_empty()).then_some(fragment)
    }

    pub fn paste_fragment(&mut self, fragment: &SceneFragment) -> bool {
        if fragment.elements.is_empty() {
            return false;
        }
        let before = self.document.clone();
        let namespace = format!("paste-{:016x}", self.revision.wrapping_add(1));
        let selection = self.document.paste_fragment(fragment, [10.0, 10.0], &namespace);
        if selection.is_empty() {
            return false;
        }
        self.history.push(before);
        self.selection = selection;
        self.sync_primary_from_selection();
        self.interaction = Interaction::None;
        self.snap_guides.clear();
        self.changed();
        true
    }

    pub fn paste_plain_text(&mut self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        let before = self.document.clone();
        let mut ordinal = self.document.elements.len();
        let id = loop {
            let candidate = format!("paste-text-{:016x}-{ordinal}", self.revision.wrapping_add(1));
            if self.document.element_by_id(&candidate).is_none() {
                break candidate;
            }
            ordinal = ordinal.saturating_add(1);
        };
        let position = self.viewport.to_world([160.0, 120.0]);
        let mut element = elephant_draw::create_element(elephant_draw::DrawingTool::Text, position, id.clone());
        element.text = text.to_owned();
        element.extra.insert("originalText".to_owned(), Value::String(text.to_owned()));
        let lines = text.lines().collect::<Vec<_>>();
        let longest = lines.iter().map(|line| line.chars().count()).max().unwrap_or(1) as f32;
        element.width = (longest * element.font_size.max(1.0) * 0.6).max(20.0);
        element.height = (lines.len().max(1) as f32 * element.font_size.max(1.0) * 1.25).max(20.0);
        self.document.elements.push(element);
        self.document.sync_fractional_indices();
        self.document.sync_element_frame_membership(&id);
        self.history.push(before);
        self.selection = SelectionSet::from_ids(std::iter::once(id));
        self.sync_primary_from_selection();
        self.interaction = Interaction::None;
        self.snap_guides.clear();
        self.changed();
        true
    }

'''
    text = replace_once(text, anchor, methods + anchor, "clipboard scene methods")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "use freya::prelude::*;\nuse serde_json::{Map, Value};",
        "use freya::{clipboard::Clipboard, prelude::*};\nuse serde_json::{json, Map, Value};",
        "clipboard imports",
    )

    helper_anchor = '''fn drawing_error_notice(error: &str) -> Element {'''
    helpers = '''fn copy_drawing_to_clipboard(canvas: State<DrawingCanvasState>) -> Result<(), String> {
    let fragment = canvas
        .read()
        .copy_selection_fragment()
        .ok_or_else(|| "Nothing selected to copy.".to_owned())?;
    let raw = serde_json::to_string(&json!({
        "type": "excalidraw/clipboard",
        "elements": fragment.elements,
        "files": fragment.files,
    }))
    .map_err(|error| error.to_string())?;
    Clipboard::set(raw).map_err(|error| error.to_string())
}

fn paste_drawing_from_clipboard(mut canvas: State<DrawingCanvasState>) -> Result<(), String> {
    let raw = Clipboard::get().map_err(|error| error.to_string())?;
    let parsed = serde_json::from_str::<Value>(&raw).ok();
    if let Some(value) = parsed.filter(|value| {
        value.get("type").and_then(Value::as_str) == Some("excalidraw/clipboard")
    }) {
        let elements = serde_json::from_value(
            value.get("elements").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
        )
        .map_err(|error| format!("Invalid Excalidraw clipboard elements: {error}"))?;
        let fragment = elephant_draw::SceneFragment {
            elements,
            files: value
                .get("files")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        };
        canvas.write().paste_fragment(&fragment);
        return Ok(());
    }
    canvas.write().paste_plain_text(&raw);
    Ok(())
}

'''
    text = replace_once(text, helper_anchor, helpers + helper_anchor, "clipboard helpers")

    command_anchor = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("d")) {'''
    command_new = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("c")) {
                        keyboard_error.set(copy_drawing_to_clipboard(keyboard_canvas).err());
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("x")) {
                        let result = copy_drawing_to_clipboard(keyboard_canvas);
                        if result.is_ok() {
                            keyboard_canvas.write().delete_selection();
                        }
                        keyboard_error.set(result.err());
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("v")) {
                        keyboard_error.set(paste_drawing_from_clipboard(keyboard_canvas).err());
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("d")) {'''
    text = replace_once(text, command_anchor, command_new, "clipboard keyboard shortcuts")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_view()
