#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

fn fixture_state() -> DrawingCanvasState {
    let svg = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyMDAiIGhlaWdodD0iMTIwIj48cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjEyMCIgZmlsbD0iI2ZmNmI2YiIvPjxyZWN0IHg9IjEwMCIgd2lkdGg9IjEwMCIgaGVpZ2h0PSIxMjAiIGZpbGw9IiM0ZDk2ZmYiLz48L3N2Zz4=";
    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "version": 2,
            "source": "drawing-excalidraw-styles-freya-testing",
            "elements": [
                {
                    "id":"hachure","type":"rectangle","x":30,"y":30,"width":120,"height":80,
                    "strokeColor":"#1e1e1e","backgroundColor":"#ff8787","strokeWidth":2,
                    "strokeStyle":"solid","fillStyle":"hachure","roughness":1,"seed":42,"opacity":100,
                    "roundness":{"type":3}
                },
                {
                    "id":"cross","type":"diamond","x":180,"y":30,"width":120,"height":80,
                    "strokeColor":"#1e1e1e","backgroundColor":"#4dabf7","strokeWidth":2,
                    "strokeStyle":"solid","fillStyle":"cross-hatch","roughness":2,"seed":77,"opacity":100
                },
                {
                    "id":"zigzag","type":"ellipse","x":330,"y":30,"width":120,"height":80,
                    "strokeColor":"#1e1e1e","backgroundColor":"#69db7c","strokeWidth":2,
                    "strokeStyle":"solid","fillStyle":"zigzag","roughness":1,"seed":93,"opacity":100
                },
                {
                    "id":"free","type":"freedraw","x":40,"y":160,"width":210,"height":65,
                    "points":[[0,20],[20,4],[42,40],[68,12],[95,54],[130,8],[165,48],[210,18]],
                    "pressures":[],"simulatePressure":true,"strokeOptions":{"variability":"variable","streamline":0.5},
                    "strokeColor":"#6741d9","backgroundColor":"transparent","strokeWidth":4,
                    "strokeStyle":"solid","fillStyle":"solid","roughness":1,"seed":123,"opacity":100
                },
                {
                    "id":"text","type":"text","x":310,"y":145,"width":250,"height":110,
                    "text":"Native Excalidraw\nmultiline text","originalText":"Native Excalidraw\nmultiline text",
                    "strokeColor":"#1e1e1e","backgroundColor":"transparent","strokeWidth":1,
                    "strokeStyle":"solid","fillStyle":"solid","opacity":100,"fontSize":24,"fontFamily":5,
                    "textAlign":"center","verticalAlign":"middle","lineHeight":1.25,"autoResize":false,
                    "containerId":null
                },
                {
                    "id":"image","type":"image","x":55,"y":285,"width":200,"height":120,
                    "strokeColor":"transparent","backgroundColor":"transparent","strokeWidth":2,
                    "strokeStyle":"solid","fillStyle":"solid","opacity":100,"fileId":"asset",
                    "status":"saved","scale":[-1,1],
                    "crop":{"x":100,"y":0,"width":100,"height":120,"naturalWidth":200,"naturalHeight":120}
                },
                {
                    "id":"frame","type":"frame","x":300,"y":285,"width":250,"height":120,
                    "strokeColor":"#868e96","backgroundColor":"transparent","strokeWidth":2,
                    "strokeStyle":"solid","fillStyle":"solid","roughness":0,"opacity":100,"name":"Frame title"
                }
            ],
            "appState": {"viewBackgroundColor":"#ffffff"},
            "files": {
                "asset": {"id":"asset","mimeType":"image/svg+xml","dataURL":svg,"created":1}
            }
        }))
        .expect("fixture JSON"),
    )
    .expect("style fixture must parse")
}

#[test]
fn native_canvas_renders_excalidraw_fill_text_freehand_crop_and_frame_contracts() {
    let state = fixture_state();
    let runner = TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    );

    for (id, kind) in [
        ("hachure", "rectangle"),
        ("cross", "diamond"),
        ("zigzag", "ellipse"),
        ("free", "freedraw"),
        ("text", "text"),
        ("image", "image"),
        ("frame", "frame"),
    ] {
        let label = format!("Drawing element {id} {kind}");
        assert!(runner
            .find(|node, element| {
                (element.accessibility().builder.label() == Some(label.as_str())).then_some(node)
            })
            .is_some(), "missing native render for {id}");
    }

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-excalidraw-styles.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(&evidence).expect("style screenshot").len() > 0);
}
