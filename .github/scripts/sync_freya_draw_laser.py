#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "    MagicFrame,\n    Embeddable,\n}",
        "    MagicFrame,\n    Embeddable,\n    Laser,\n}",
        "canvas laser enum",
    )
    text = replace_once(
        text,
        '            "embeddable" | "iframe" => Self::Embeddable,\n            _ => Self::Selection,',
        '            "embeddable" | "iframe" => Self::Embeddable,\n            "laser" => Self::Laser,\n            _ => Self::Selection,',
        "canvas laser from id",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "embeddable",\n        }',
        '            Self::Embeddable => "embeddable",\n            Self::Laser => "laser",\n        }',
        "canvas laser id",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "Embed",\n        }',
        '            Self::Embeddable => "Embed",\n            Self::Laser => "Laser",\n        }',
        "canvas laser label",
    )

    image_anchor = '''                DrawingTool::Image => {
                    pointer_gesture.set(None);
                    pointer_down.set(false);
                    match drawing_image::pick_image() {'''
    laser_branch = '''                DrawingTool::Laser => {
                    let mut canvas = pointer_state.write();
                    let world = canvas.to_world(screen);
                    let index = canvas.document.elements.len();
                    let mut element = new_element(DrawingTool::Freehand, world, index);
                    element.stroke_color = "#e03131".to_owned();
                    element.stroke_width = 3.0;
                    element.opacity = 90.0;
                    element.extra.insert(
                        "customData".to_owned(),
                        json!({"elephantTransient": "laser"}),
                    );
                    canvas.document.elements.push(element);
                    canvas.revision = canvas.revision.wrapping_add(1);
                    drop(canvas);
                    pointer_gesture.set(Some(Gesture {
                        tool: DrawingTool::Laser,
                        index,
                        start: world,
                    }));
                }
'''
    text = replace_once(text, image_anchor, laser_branch + image_anchor, "laser pointer branch")

    text = replace_once(
        text,
        '''        DrawingTool::Embeddable => elephant_draw::DrawingTool::Embeddable,
    };''',
        '''        DrawingTool::Embeddable => elephant_draw::DrawingTool::Embeddable,
        DrawingTool::Laser => elephant_draw::DrawingTool::Freehand,
    };''',
        "laser core mapping",
    )
    text = replace_once(
        text,
        '''            DrawingTool::Freehand => {
                element''',
        '''            DrawingTool::Freehand | DrawingTool::Laser => {
                element''',
        "laser freehand gesture",
    )

    finalize_anchor = '''fn finalize_gesture(canvas: &mut DrawingCanvasState, gesture: Gesture) {
    if gesture.tool != DrawingTool::Arrow {
        return;
    }'''
    finalize_new = '''fn finalize_gesture(canvas: &mut DrawingCanvasState, gesture: Gesture) {
    if gesture.tool == DrawingTool::Laser {
        if gesture.index < canvas.document.elements.len()
            && canvas.document.elements[gesture.index]
                .extra
                .get("customData")
                .and_then(Value::as_object)
                .and_then(|custom| custom.get("elephantTransient"))
                .and_then(Value::as_str)
                == Some("laser")
        {
            canvas.document.elements.remove(gesture.index);
            canvas.revision = canvas.revision.wrapping_add(1);
        }
        return;
    }
    if gesture.tool != DrawingTool::Arrow {
        return;
    }'''
    text = replace_once(text, finalize_anchor, finalize_new, "laser finalization")
    path.write_text(text)


def sync_scene() -> None:
    path = Path("Elephant/freya/src/app/drawing_scene.rs")
    text = path.read_text()
    text = replace_once(
        text,
        '            "Embed" => "embeddable",\n            _ => "selection",',
        '            "Embed" => "embeddable",\n            "Laser" => "laser",\n            _ => "selection",',
        "scene laser mapping",
    )
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "    MagicFrame,\n    Embeddable,\n    Eraser,",
        "    MagicFrame,\n    Embeddable,\n    Laser,\n    Eraser,",
        "view laser enum",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "Embed",\n            Self::Eraser => "Eraser",',
        '            Self::Embeddable => "Embed",\n            Self::Laser => "Laser",\n            Self::Eraser => "Eraser",',
        "view laser label",
    )
    text = replace_once(
        text,
        '            Self::MagicFrame | Self::Embeddable => "",\n            Self::Eraser => "0",',
        '            Self::MagicFrame | Self::Embeddable => "",\n            Self::Laser => "K",\n            Self::Eraser => "0",',
        "view laser shortcut",
    )
    text = replace_once(
        text,
        '            Self::MagicFrame | Self::Embeddable => false,\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        '            Self::MagicFrame | Self::Embeddable => false,\n            Self::Laser => value == "k",\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        "view laser shortcut matching",
    )
    icon_anchor = '''            Self::Embeddable => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m9 9-3 3 3 3"/><path d="m15 9 3 3-3 3"/></svg>"#,
            Self::Eraser =>'''
    icon_new = '''            Self::Embeddable => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m9 9-3 3 3 3"/><path d="m15 9 3 3-3 3"/></svg>"#,
            Self::Laser => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 21 14 10"/><path d="m12 4 2 2"/><path d="m18 3 .5 2.5L21 6l-2.5.5L18 9l-.5-2.5L15 6l2.5-.5Z"/></svg>"#,
            Self::Eraser =>'''
    text = replace_once(text, icon_anchor, icon_new, "view laser icon")

    embed_panel = '''        .child(tool_button(
            DrawingTool::Embeddable,
            *active.read(),
            active,
            canvas,
        ))'''
    laser_panel = embed_panel + '''
        .child(tool_button(
            DrawingTool::Laser,
            *active.read(),
            active,
            canvas,
        ))'''
    text = replace_once(text, embed_panel, laser_panel, "laser more-tools button")

    keyboard_anchor = '''                    Key::Character(value) if !command => {
                        if let Some(tool) = DrawingTool::ALL'''
    keyboard_new = '''                    Key::Character(value) if !command => {
                        if value.eq_ignore_ascii_case("k") {
                            keyboard_tool.set(DrawingTool::Laser);
                            keyboard_canvas.write().set_active_tool_label("Laser");
                            event.stop_propagation();
                            return;
                        }
                        if let Some(tool) = DrawingTool::ALL'''
    text = replace_once(text, keyboard_anchor, keyboard_new, "laser keyboard shortcut")
    path.write_text(text)


def sync_test() -> None:
    Path("Elephant/freya/tests/drawing_laser_freya_testing.rs").write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

#[test]
fn laser_is_visible_during_gesture_but_never_persisted() {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[],
            "appState":{"viewBackgroundColor":"#ffffff"},
            "files":{}
        })).unwrap(),
    ).unwrap();
    let mut runner = TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    );
    let mut state = runner.root().context::<State<DrawingCanvasState>>().unwrap();
    state.write().set_active_tool_label("Laser");

    runner.press_cursor((90., 130.));
    for point in [(130., 150.), (180., 120.), (230., 160.), (300., 125.)] {
        runner.move_cursor(point);
    }
    assert_eq!(state.peek().document.elements.len(), 1);
    let transient = &state.peek().document.elements[0];
    assert_eq!(transient.kind, "freedraw");
    assert_eq!(transient.extra["customData"]["elephantTransient"], "laser");

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-laser-during.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(&evidence).unwrap().len() > 0);

    runner.release_cursor((300., 125.));
    assert!(state.peek().document.elements.is_empty());
    let raw = state.peek().serialize_json().unwrap();
    assert!(!raw.contains("elephantTransient"));
    assert!(!raw.contains("\"type\": \"freedraw\""));
}
''')


if __name__ == "__main__":
    sync_canvas()
    sync_scene()
    sync_view()
    sync_test()
