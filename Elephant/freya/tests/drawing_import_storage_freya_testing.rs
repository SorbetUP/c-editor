#[path = "../src/app/drawing_scene.rs"]
mod drawing_scene;
#[path = "../src/app/drawing_storage.rs"]
mod drawing_storage;

use drawing_scene::DrawingCanvasState;
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("elephant-drawing-import-{label}-{stamp}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn imports_json_and_excalidraw_while_preserving_unknown_scene_data() {
    let scene = json!({
        "type": "excalidraw",
        "version": 2,
        "source": "external-excalidraw",
        "elements": [{
            "id": "future",
            "type": "future-shape",
            "x": 1,
            "y": 2,
            "width": 3,
            "height": 4,
            "unknownElementField": {"keep": [true, 1]}
        }],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {"asset": {"mimeType": "image/png", "data": "data:image/png;base64,AA=="}},
        "futureDocumentField": {"keep": true}
    });
    let raw = serde_json::to_string_pretty(&scene).unwrap();
    let state = DrawingCanvasState::from_json(&raw).expect("valid external Excalidraw JSON");
    let round_trip: serde_json::Value =
        serde_json::from_str(&state.serialize_json().unwrap()).unwrap();
    assert_eq!(
        round_trip["futureDocumentField"],
        scene["futureDocumentField"]
    );
    assert_eq!(
        round_trip["elements"][0]["unknownElementField"],
        scene["elements"][0]["unknownElementField"]
    );
    assert_eq!(round_trip["files"]["asset"], scene["files"]["asset"]);

    for extension in ["drawing.excalidraw", "drawing.json"] {
        let fixture = root(extension);
        fs::write(fixture.join(extension), &raw).unwrap();
        let loaded = drawing_storage::read_native_scene(&fixture, extension)
            .expect("storage must accept both Excalidraw extensions");
        assert_eq!(loaded.raw, raw);
        let _ = fs::remove_dir_all(fixture);
    }
}

#[test]
fn resolves_markdown_drawing_sidecars_without_a_preview_file() {
    let fixture = root("sidecar");
    fs::create_dir_all(fixture.join(".assets")).unwrap();
    let raw = serde_json::to_string(&json!({
        "type": "excalidraw", "elements": [], "appState": {}, "files": {}
    }))
    .unwrap();
    fs::write(fixture.join(".assets/board.excalidraw"), raw).unwrap();
    fs::write(
        fixture.join("Drawing.md"),
        "![Excalidraw](.assets/board.excalidraw.png)",
    )
    .unwrap();
    drawing_storage::read_native_scene(&fixture, "Drawing.md")
        .expect("markdown sidecar must resolve to the .excalidraw scene");
    let _ = fs::remove_dir_all(fixture);
}
