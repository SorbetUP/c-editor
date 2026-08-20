use elephant_draw_contract::{render_scene_svg, DrawingScene, SvgRenderOptions};
use serde_json::json;

#[test]
fn canonical_svg_export_embeds_real_cropped_images() {
    let raw = serde_json::to_string(&json!({
        "type": "excalidraw",
        "elements": [{
            "id": "image-1",
            "type": "image",
            "x": 40,
            "y": 30,
            "width": 120,
            "height": 80,
            "fileId": "file-1",
            "crop": {
                "x": 20,
                "y": 10,
                "width": 120,
                "height": 80,
                "naturalWidth": 200,
                "naturalHeight": 100
            },
            "opacity": 100
        }],
        "appState": {"viewBackgroundColor": "#ffffff"},
        "files": {
            "file-1": {
                "id": "file-1",
                "mimeType": "image/png",
                "dataURL": "data:image/png;base64,AAECAwQ=",
                "created": 1
            }
        }
    }))
    .unwrap();
    let scene = DrawingScene::from_json(&raw).unwrap();
    let svg = render_scene_svg(
        &scene,
        &SvgRenderOptions {
            width: 640,
            height: 480,
            background: "#ffffff".to_owned(),
        },
    );
    assert!(svg.contains("data:image/png;base64,AAECAwQ="));
    assert!(svg.contains("viewBox=\"20 10 120 80\""));
    assert!(svg.contains("width=\"200\" height=\"100\""));
    assert!(!svg.contains("fill=\"#e9ecef\""), "real image must replace placeholder");
}
