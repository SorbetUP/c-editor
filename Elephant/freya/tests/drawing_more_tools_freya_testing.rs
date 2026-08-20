#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

fn runner() -> (TestingRunner, State<DrawingCanvasState>) {
    let state = DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "version":2,
            "source":"drawing-more-tools-freya-testing",
            "elements":[],
            "appState":{"viewBackgroundColor":"#ffffff"},
            "files":{}
        }))
        .unwrap(),
    )
    .unwrap();
    TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    )
}

#[test]
fn magic_frame_and_embeddable_are_real_native_drawing_tools() {
    let (mut runner, mut state) = runner();

    state.write().set_active_tool_label("Magic Frame");
    runner.press_cursor((90., 110.));
    runner.move_cursor((230., 200.));
    runner.release_cursor((230., 200.));

    state.write().set_active_tool_label("Embed");
    runner.press_cursor((300., 120.));
    runner.move_cursor((500., 230.));
    runner.release_cursor((500., 230.));

    let snapshot = state.peek();
    assert_eq!(snapshot.document.elements.len(), 2);
    let magic = &snapshot.document.elements[0];
    assert_eq!(magic.kind, "magicframe");
    assert_eq!(magic.extra.get("name"), Some(&serde_json::Value::Null));
    assert!(magic.width > 100.0 && magic.height > 60.0);

    let embed = &snapshot.document.elements[1];
    assert_eq!(embed.kind, "embeddable");
    assert!(embed.width > 150.0 && embed.height > 80.0);

    let serialized = snapshot.serialize_json().unwrap();
    assert!(serialized.contains("\"type\": \"magicframe\""));
    assert!(serialized.contains("\"type\": \"embeddable\""));
    drop(snapshot);

    for (id, kind) in [("freya-element-0", "magicframe"), ("freya-element-1", "embeddable")] {
        let label = format!("Drawing element {id} {kind}");
        assert!(runner
            .find(|node, element| {
                (element.accessibility().builder.label() == Some(label.as_str())).then_some(node)
            })
            .is_some(), "missing rendered {kind}");
    }

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-more-tools.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(evidence).unwrap().len() > 0);
}
