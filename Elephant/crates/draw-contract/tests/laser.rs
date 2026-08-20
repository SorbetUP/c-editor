use elephant_draw_contract::{
    laser_ease_out, LaserTrails, DEFAULT_LASER_COLOR, LASER_DECAY_LENGTH,
    LASER_DECAY_TIME_MS, LASER_SIMPLIFY, LASER_SIZE, LASER_STREAMLINE,
};

#[test]
fn constants_match_current_excalidraw_laser_contract() {
    assert_eq!(DEFAULT_LASER_COLOR, "red");
    assert_eq!(LASER_DECAY_TIME_MS, 1_000.0);
    assert_eq!(LASER_DECAY_LENGTH, 50);
    assert_eq!(LASER_SIMPLIFY, 0.0);
    assert_eq!(LASER_STREAMLINE, 0.4);
    assert_eq!(LASER_SIZE, 2.0);
    assert!((laser_ease_out(0.5) - 0.9375).abs() < f32::EPSILON);
}

#[test]
fn trail_deduplicates_and_moves_to_past_on_end() {
    let mut trails = LaserTrails::default();
    trails.start_path(10.0, 20.0, 100.0);
    assert!(trails.has_current_trail());
    assert!(!trails.add_point_to_path(10.0, 20.0, 110.0));
    assert!(trails.add_point_to_path(30.0, 40.0, 120.0));
    assert_eq!(trails.current().unwrap().points.len(), 2);
    assert!(trails.end_path());
    assert!(!trails.has_current_trail());
    assert_eq!(trails.past().len(), 1);
    assert!(trails.past()[0].closed);
    assert!(!trails.past()[0].keep_head);
}

#[test]
fn stroke_outline_is_closed_and_finite() {
    let mut trails = LaserTrails::default();
    trails.start_path(10.0, 30.0, 100.0);
    for (index, point) in [[30.0, 32.0], [55.0, 24.0], [80.0, 42.0], [110.0, 28.0]]
        .into_iter()
        .enumerate()
    {
        trails.add_point_to_path(point[0], point[1], 110.0 + index as f64 * 10.0);
    }
    let outline = trails.current().unwrap().stroke_outline(145.0, Some(2.0));
    assert!(outline.len() > 20);
    assert!(outline.iter().flatten().all(|value| value.is_finite()));
    assert_eq!(outline.first(), outline.last());
}

#[test]
fn ended_trail_expires_after_one_second() {
    let mut trails = LaserTrails::default();
    trails.start_path(1.0, 2.0, 0.0);
    trails.add_point_to_path(3.0, 4.0, 10.0);
    trails.end_path();
    assert!(!trails.visible_samples(500.0).is_empty());
    assert!(trails.visible_samples(1_011.0).is_empty());
    trails.prune(1_011.0);
    assert!(trails.past().is_empty());
}
