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
    anchor = '''    pub fn selected_link(&self) -> Option<&str> {
        let id = self.selected_element_id()?;
        self.document.element_link(id)
    }
'''
    methods = '''
    pub fn selected_image_crop(&self) -> Option<elephant_draw::ImageCrop> {
        let id = self.selected_element_id()?;
        self.document.image_crop(id)
    }

    pub fn selected_image_id(&self) -> Option<&str> {
        let id = self.selected_element_id()?;
        self.document
            .element_by_id(id)
            .filter(|element| element.kind == "image" && !element.is_deleted && !element.is_locked())
            .map(|element| element.id.as_str())
    }

    pub fn adjust_selected_image_crop(
        &mut self,
        delta_left: f32,
        delta_top: f32,
        delta_right: f32,
        delta_bottom: f32,
    ) -> bool {
        let Some(id) = self.selected_image_id().map(str::to_owned) else {
            return false;
        };
        let Some(element) = self.document.element_by_id(&id) else {
            return false;
        };
        let crop = self.document.image_crop(&id);
        let natural = crop
            .map(|crop| [crop.natural_width, crop.natural_height])
            .unwrap_or_else(|| {
                [
                    element
                        .extra
                        .get("naturalWidth")
                        .and_then(Value::as_f64)
                        .map(|value| value as f32)
                        .unwrap_or(element.width.abs()),
                    element
                        .extra
                        .get("naturalHeight")
                        .and_then(Value::as_f64)
                        .map(|value| value as f32)
                        .unwrap_or(element.height.abs()),
                ]
            });
        if natural[0] < elephant_draw::MINIMAL_CROP_SIZE
            || natural[1] < elephant_draw::MINIMAL_CROP_SIZE
        {
            return false;
        }
        let before = self.document.clone();
        if !self.document.adjust_image_crop(
            &id,
            delta_left,
            delta_top,
            delta_right,
            delta_bottom,
            natural,
        ) {
            return false;
        }
        self.history.push(before);
        self.changed();
        true
    }
'''
    text = replace_once(text, anchor, anchor + methods, "crop state methods")
    path.write_text(text)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    anchor = '''    element.extra.insert("scale".to_owned(), json!([1, 1]));
    element.extra.insert("crop".to_owned(), Value::Null);
'''
    replacement = '''    element.extra.insert("scale".to_owned(), json!([1, 1]));
    element.extra.insert("crop".to_owned(), Value::Null);
    element
        .extra
        .insert("naturalWidth".to_owned(), json!(asset.width));
    element
        .extra
        .insert("naturalHeight".to_owned(), json!(asset.height));
'''
    text = replace_once(text, anchor, replacement, "persist image natural dimensions")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    anchor = '''        .child(label().font_size(15.0).color(text).text("Link"))
        .child(
            rect()
                .horizontal()
                .spacing(8.0)
                .child(action_button("Edit", "Edit selected element link", 62.0, move |_| {
                    edit_link_value.set(
                        link_canvas
                            .read()
                            .selected_link()
                            .unwrap_or_default()
                            .to_owned(),
                    );
                    edit_link_open.set(true);
                }))
                .child(action_button("Remove", "Remove selected element link", 70.0, move |_| {
                    remove_link.write().set_selected_link(None);
                })),
        )
        .into_element()
}'''
    replacement = '''        .child(label().font_size(15.0).color(text).text("Link"))
        .child(
            rect()
                .horizontal()
                .spacing(8.0)
                .child(action_button("Edit", "Edit selected element link", 62.0, move |_| {
                    edit_link_value.set(
                        link_canvas
                            .read()
                            .selected_link()
                            .unwrap_or_default()
                            .to_owned(),
                    );
                    edit_link_open.set(true);
                }))
                .child(action_button("Remove", "Remove selected element link", 70.0, move |_| {
                    remove_link.write().set_selected_link(None);
                })),
        )
        .maybe_child(canvas.read().selected_image_id().is_some().then(|| {
            let mut crop_left = canvas;
            let mut crop_top = canvas;
            let mut crop_right = canvas;
            let mut crop_bottom = canvas;
            let mut crop_reset = canvas;
            rect()
                .spacing(6.0)
                .child(label().font_size(15.0).color(text).text("Crop image"))
                .child(
                    rect()
                        .horizontal()
                        .spacing(6.0)
                        .child(action_button("Left", "Trim image left", 52.0, move |_| {
                            crop_left.write().adjust_selected_image_crop(10.0, 0.0, 0.0, 0.0);
                        }))
                        .child(action_button("Top", "Trim image top", 52.0, move |_| {
                            crop_top.write().adjust_selected_image_crop(0.0, 10.0, 0.0, 0.0);
                        }))
                        .child(action_button("Right", "Trim image right", 52.0, move |_| {
                            crop_right.write().adjust_selected_image_crop(0.0, 0.0, -10.0, 0.0);
                        }))
                        .child(action_button("Bottom", "Trim image bottom", 60.0, move |_| {
                            crop_bottom.write().adjust_selected_image_crop(0.0, 0.0, 0.0, -10.0);
                        })),
                )
                .child(action_button("Reset crop", "Reset image crop", 90.0, move |_| {
                    crop_reset.write().reset_selected_image_crop();
                }))
        }))
        .into_element()
}'''
    text = replace_once(text, anchor, replacement, "crop property controls")
    path.write_text(text)


def sync_test() -> None:
    path = Path("Elephant/freya/tests/drawing_crop_freya_testing.rs")
    path.write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

fn fixture() -> DrawingCanvasState {
    let raw = serde_json::to_string(&json!({
        "type": "excalidraw",
        "elements": [{
            "id": "image",
            "type": "image",
            "x": 80,
            "y": 70,
            "width": 160,
            "height": 100,
            "fileId": "file",
            "naturalWidth": 160,
            "naturalHeight": 100,
            "scale": [1, 1],
            "crop": null
        }],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {
            "file": {
                "id": "file",
                "mimeType": "image/svg+xml",
                "dataURL": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='100'%3E%3Crect width='80' height='100' fill='%23ff0000'/%3E%3Crect x='80' width='80' height='100' fill='%230000ff'/%3E%3C/svg%3E",
                "created": 1
            }
        }
    })).expect("fixture json");
    let mut state = DrawingCanvasState::from_json(&raw).expect("valid image fixture");
    state.begin_pointer([100.0, 90.0]);
    state.end_pointer();
    state
}

#[test]
fn selected_image_crop_uses_excalidraw_schema_and_is_undoable() {
    let mut state = fixture();
    assert!(state.adjust_selected_image_crop(30.0, 10.0, -20.0, -10.0));
    let crop = state.selected_image_crop().expect("crop");
    assert_eq!((crop.x, crop.y), (30.0, 10.0));
    assert_eq!((crop.width, crop.height), (110.0, 80.0));
    let raw = state.serialize_json().expect("serialize crop");
    assert!(raw.contains("\"naturalWidth\": 160.0"));
    assert!(raw.contains("\"naturalHeight\": 100.0"));
    assert!(state.undo());
    assert!(state.selected_image_crop().is_none());
}

#[test]
fn cropped_image_is_rendered_by_native_freya_canvas() {
    let mut state = fixture();
    assert!(state.adjust_selected_image_crop(70.0, 0.0, 0.0, 0.0));
    let output = std::env::temp_dir().join("elephant-freya-drawing-crop.png");
    let mut runner = TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    );
    runner.render_to_file(&output);
    let bytes = fs::read(&output).expect("crop screenshot");
    assert!(bytes.len() > 1_000, "crop screenshot must contain rendered pixels");
}
''')


def main() -> None:
    sync_scene()
    sync_canvas()
    sync_view()
    sync_test()


if __name__ == "__main__":
    main()
