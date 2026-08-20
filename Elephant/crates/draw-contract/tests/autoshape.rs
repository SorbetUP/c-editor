use elephant_draw_contract::{recognize_shape, recognized_arrow_endpoint, RecognizedShape};

fn rectangle() -> Vec<[f32; 2]> {
    vec![
        [10.0, 10.0], [40.0, 9.0], [80.0, 11.0], [110.0, 10.0], [111.0, 35.0],
        [109.0, 60.0], [80.0, 61.0], [40.0, 59.0], [10.0, 60.0], [9.0, 35.0],
        [10.0, 10.0],
    ]
}

fn diamond() -> Vec<[f32; 2]> {
    vec![
        [60.0, 5.0], [85.0, 25.0], [110.0, 50.0], [85.0, 75.0], [60.0, 95.0],
        [35.0, 75.0], [10.0, 50.0], [35.0, 25.0], [60.0, 5.0],
    ]
}

fn ellipse() -> Vec<[f32; 2]> {
    (0..=96)
        .map(|index| {
            let angle = std::f32::consts::TAU * index as f32 / 96.0;
            [70.0 + 60.0 * angle.cos(), 55.0 + 35.0 * angle.sin()]
        })
        .collect()
}

#[test]
fn recognizes_closed_shape_family() {
    assert_eq!(recognize_shape(&rectangle(), 1.0, false).shape, RecognizedShape::Rectangle);
    assert_eq!(recognize_shape(&diamond(), 1.0, false).shape, RecognizedShape::Diamond);
    assert_eq!(recognize_shape(&ellipse(), 1.0, false).shape, RecognizedShape::Ellipse);
}

#[test]
fn recognizes_line_and_arrow_but_rejects_bent_scribble() {
    let line = (0..20)
        .map(|index| [index as f32 * 6.0, (index % 2) as f32 * 0.4])
        .collect::<Vec<_>>();
    assert_eq!(recognize_shape(&line, 1.0, false).shape, RecognizedShape::Line);

    let arrow = vec![
        [0.0, 0.0], [20.0, 0.5], [40.0, -0.5], [60.0, 0.0], [80.0, 0.0],
        [100.0, 0.0], [86.0, -13.0], [100.0, 0.0], [86.0, 13.0], [100.0, 0.0],
    ];
    assert_eq!(recognize_shape(&arrow, 1.0, false).shape, RecognizedShape::Arrow);

    let scribble = vec![
        [0.0, 0.0], [20.0, 18.0], [40.0, -18.0], [60.0, 22.0], [80.0, -20.0],
        [100.0, 0.0],
    ];
    assert_eq!(recognize_shape(&scribble, 1.0, false).shape, RecognizedShape::FreeDraw);
}

#[test]
fn tiny_on_screen_stroke_stays_freedraw() {
    let shape = rectangle()
        .into_iter()
        .map(|[x, y]| [x * 0.1, y * 0.1])
        .collect::<Vec<_>>();
    assert_eq!(recognize_shape(&shape, 1.0, false).shape, RecognizedShape::FreeDraw);
    assert_ne!(recognize_shape(&shape, 4.0, false).shape, RecognizedShape::FreeDraw);
}

#[test]
fn arrow_preview_does_not_flicker_into_another_shape() {
    let line = (0..20).map(|index| [index as f32 * 6.0, 0.0]).collect::<Vec<_>>();
    assert_eq!(recognize_shape(&line, 1.0, true).shape, RecognizedShape::FreeDraw);
}

#[test]
fn arrow_endpoint_uses_original_point_nearest_farthest_perimeter_tip() {
    let points = vec![[10.0, 10.0], [35.0, 20.0], [85.0, 43.0], [100.0, 50.0], [88.0, 30.0]];
    let bounds = [10.0, 10.0, 100.0, 50.0];
    assert_eq!(recognized_arrow_endpoint(&points, bounds), Some([100.0, 50.0]));
}
