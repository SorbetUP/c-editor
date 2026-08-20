#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::{drawing_canvas, DrawingCanvasState};
use freya::prelude::State;
use freya_testing::TestingRunner;
use serde_json::json;
use std::fs;

const ARROWHEADS: [&str; 14] = [
    "arrow",
    "bar",
    "circle",
    "circle_outline",
    "triangle",
    "triangle_outline",
    "diamond",
    "diamond_outline",
    "cardinality_one",
    "cardinality_many",
    "cardinality_one_or_many",
    "cardinality_exactly_one",
    "cardinality_zero_or_one",
    "cardinality_zero_or_many",
];

fn fixture_state() -> DrawingCanvasState {
    let elements = ARROWHEADS
        .iter()
        .enumerate()
        .map(|(index, arrowhead)| {
            let column = index / 7;
            let row = index % 7;
            json!({
                "id": format!("arrow-{index}"),
                "type": "arrow",
                "x": 40 + column * 310,
                "y": 35 + row * 58,
                "width": 150,
                "height": 24,
                "points": [[0, 0], [150, 24]],
                "strokeColor": "#1e1e1e",
                "backgroundColor": "transparent",
                "strokeWidth": 2,
                "strokeStyle": "solid",
                "fillStyle": "solid",
                "opacity": 100,
                "startArrowhead": null,
                "endArrowhead": arrowhead,
                "elbowed": false
            })
        })
        .collect::<Vec<_>>();

    DrawingCanvasState::from_json(
        &serde_json::to_string(&json!({
            "type": "excalidraw",
            "version": 2,
            "source": "drawing-arrowheads-freya-testing",
            "elements": elements,
            "appState": {"viewBackgroundColor": "#ffffff"},
            "files": {}
        }))
        .expect("fixture JSON"),
    )
    .expect("arrowhead fixture must parse")
}

#[test]
fn every_current_arrowhead_renders_in_the_native_freya_canvas() {
    let state = fixture_state();
    let runner = TestingRunner::new(
        drawing_canvas,
        (640., 480.).into(),
        move |runner| runner.provide_root_context(|| State::create(state)),
        1.,
    );

    for index in 0..ARROWHEADS.len() {
        let label = format!("Drawing element arrow-{index} arrow");
        assert!(
            runner
                .find(|node, element| {
                    (element.accessibility().builder.label() == Some(label.as_str())).then_some(node)
                })
                .is_some(),
            "missing native arrow {index}: {}",
            ARROWHEADS[index]
        );
    }

    let evidence = std::env::temp_dir().join("elephant-freya-drawing-arrowheads.png");
    runner.render_to_file(&evidence);
    assert!(
        fs::metadata(&evidence)
            .expect("arrowhead screenshot")
            .len()
            > 0
    );
}
