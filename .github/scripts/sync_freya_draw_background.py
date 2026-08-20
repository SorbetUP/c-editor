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
    anchor = '''    pub fn renderable_elements(&self) -> Vec<RenderableElement> {'''
    method = '''    pub fn background_rgba(&self) -> [u8; 4] {
        let color = self
            .document
            .app_state
            .get("viewBackgroundColor")
            .and_then(Value::as_str)
            .unwrap_or("#ffffff");
        rgba(color, 100.0)
    }

'''
    text = replace_once(text, anchor, method + anchor, "scene background accessor")
    path.write_text(text)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    anchor = '''    let primitives = drawing_render::render(&snapshot);
    let editing_index = *editing_text_state.read();

    rect()'''
    replacement = '''    let primitives = drawing_render::render(&snapshot);
    let editing_index = *editing_text_state.read();
    let [background_r, background_g, background_b, background_a] = snapshot.background_rgba();
    let canvas_background =
        Color::from_argb(background_a, background_r, background_g, background_b);

    rect()'''
    text = replace_once(text, anchor, replacement, "canvas background state")
    text = replace_once(
        text,
        '''        .background(Color::from_rgb(238, 238, 238))
        .overflow(Overflow::Clip)
        .a11y_alt("DrawingCanvas")''',
        '''        .background(canvas_background)
        .overflow(Overflow::Clip)
        .a11y_alt("DrawingCanvas")''',
        "canvas background paint",
    )
    path.write_text(text)


def sync_test() -> None:
    path = Path("Elephant/freya/tests/drawing_background_contract.rs")
    path.write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::json;

#[test]
fn canvas_background_comes_from_excalidraw_app_state() {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[],
            "appState":{"viewBackgroundColor":"#123456"},
            "files":{}
        }))
        .expect("fixture json"),
    )
    .expect("valid scene");
    assert_eq!(state.background_rgba(), [0x12, 0x34, 0x56, 0xff]);
}

#[test]
fn canvas_background_defaults_to_excalidraw_white() {
    let state = DrawingCanvasState::from_json(
        r#"{"type":"excalidraw","elements":[],"appState":{},"files":{}}"#,
    )
    .expect("valid scene");
    assert_eq!(state.background_rgba(), [255, 255, 255, 255]);
}
''')


def main() -> None:
    sync_scene()
    sync_canvas()
    sync_test()


if __name__ == "__main__":
    main()
