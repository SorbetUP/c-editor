use crate::{
    differential_frames::{capture_frame, distinct_frame_count, wait_and_capture, FrameEvidence},
    differential_scenario::ScenarioAction,
    differential_ui::{background_for_label, require_label},
};
use freya::prelude::*;
use freya_testing::{
    prelude::{KeyboardEventName, PlatformEvent},
    TestingNode, TestingRunner,
};
use std::{path::Path, time::Duration};

pub fn press_modified_key(runner: &mut TestingRunner, key: Key, modifiers: Modifiers) {
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key,
        code: Code::Unidentified,
        modifiers,
    });
    runner.sync_and_update();
}

pub fn settle(runner: &mut TestingRunner) {
    runner.poll_n(Duration::from_millis(50), 5);
}

pub fn action_frames_after(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    dispatch: impl FnOnce(&mut TestingRunner),
) -> Vec<FrameEvidence> {
    let mut frames = vec![capture_frame(runner, output, action, 0)];
    dispatch(runner);
    let mut previous_ms = action.frames[0];
    for index in 1..action.frames.len() {
        wait_and_capture(runner, output, action, &mut frames, &mut previous_ms, index);
    }
    frames
}

fn interpolate(points: &[(f64, f64)], progress: f64) -> (f64, f64) {
    if points.len() < 2 {
        return points.first().copied().unwrap_or((0.0, 0.0));
    }
    let scaled = progress.clamp(0.0, 1.0) * (points.len() - 1) as f64;
    let segment = scaled.floor() as usize;
    if segment + 1 >= points.len() {
        return *points.last().expect("non-empty pointer path");
    }
    let fraction = scaled - segment as f64;
    let start = points[segment];
    let end = points[segment + 1];
    (
        start.0 + (end.0 - start.0) * fraction,
        start.1 + (end.1 - start.1) * fraction,
    )
}

fn pointer_target(runner: &TestingRunner, label: &str) -> TestingNode {
    if label == "Alpha note" {
        crate::differential_ui::require_note_card(runner, label)
    } else {
        require_label(runner, label)
    }
}

fn target_path(runner: &TestingRunner, label: &str, names: &[String]) -> Vec<(f64, f64)> {
    let area = pointer_target(runner, label).layout().area;
    let middle_point = area.center().to_f64();
    let middle = (middle_point.x, middle_point.y);
    names
        .iter()
        .map(|name| match name.as_str() {
            "center-plus-x-24" => (middle.0 + 24.0, middle.1),
            "center-plus-y-120" => (middle.0, middle.1 + 120.0),
            "center-plus-y-240" => (middle.0, middle.1 + 240.0),
            "left" => (
                area.origin.x as f64 + area.size.width as f64 * 0.2,
                middle.1,
            ),
            "right" => (
                area.origin.x as f64 + area.size.width as f64 * 0.8,
                middle.1,
            ),
            _ => middle,
        })
        .collect()
}

pub fn pointer_timeline(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    label: &str,
    path_names: &[String],
) -> Vec<FrameEvidence> {
    let before_background = Rect::try_downcast(pointer_target(runner, label).element().as_ref())
        .expect("pointer target must be a rendered Rect")
        .style
        .background;
    let points = target_path(runner, label, path_names);
    let mut frames = vec![capture_frame(runner, output, action, 0)];
    let mut previous_ms = action.frames[0];
    for index in 1..action.frames.len() {
        let progress =
            action.frames[index] as f64 / action.frames.last().copied().unwrap_or(1).max(1) as f64;
        runner.move_cursor(interpolate(&points, progress));
        wait_and_capture(runner, output, action, &mut frames, &mut previous_ms, index);
    }
    assert!(
        distinct_frame_count(&frames) > 1,
        "HARD_ISSUE {}: pointer movement produced no rendered frame change for visible target {label:?}",
        action.id
    );
    assert_ne!(
        before_background,
        Rect::try_downcast(pointer_target(runner, label).element().as_ref())
            .expect("pointer target must be a rendered Rect")
            .style
            .background,
        "HARD_ISSUE {}: visible target {label:?} did not expose a rendered hover/drop style transition",
        action.id
    );
    assert!(
        !crate::differential_ui::labeled_nodes(runner, label).is_empty(),
        "HARD_ISSUE {}: visible pointer target {label:?} disappeared after movement",
        action.id
    );
    frames
}

pub fn scroll_timeline(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
) -> Vec<FrameEvidence> {
    let path_names = action
        .pointer_path
        .clone()
        .unwrap_or_else(|| vec!["center".into()]);
    let points = target_path(runner, "Editor scroll", &path_names);
    let mut frames = vec![capture_frame(runner, output, action, 0)];
    let mut previous_ms = action.frames[0];
    for index in 1..action.frames.len() {
        let progress =
            action.frames[index] as f64 / action.frames.last().copied().unwrap_or(1).max(1) as f64;
        let point = interpolate(&points, progress);
        runner.move_cursor(point);
        if index + 1 == action.frames.len() {
            let delta = action.delta.as_ref().expect("scroll action delta");
            // Freya's TestingRunner wheel coordinate is content-relative:
            // negative Y moves the viewport down. The shared Tauri gesture
            // uses native wheel semantics where positive Y moves it down.
            runner.scroll(point, (delta.x, -delta.y));
        }
        wait_and_capture(runner, output, action, &mut frames, &mut previous_ms, index);
    }
    assert!(
        frames.first().map(|frame| &frame.sha256) != frames.last().map(|frame| &frame.sha256),
        "HARD_ISSUE Freya action scroll-alpha-note: real Editor scroll target did not change the rendered scroll state"
    );
    assert!(
        !crate::differential_ui::labeled_nodes(runner, "Editor scroll").is_empty(),
        "HARD_ISSUE Freya action scroll-alpha-note: Editor scroll accessibility target disappeared after action"
    );
    frames
}

pub fn drag_timeline(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    vault_root: &Path,
) -> Vec<FrameEvidence> {
    let source_before = require_label(runner, "Search").layout().area;
    let target_before = require_label(runner, "Hide sidebar").layout().area;
    let target_before_background = background_for_label(runner, "Hide sidebar");
    let source_point = source_before.center().to_f64();
    let target_point = target_before.center().to_f64();
    let source = (source_point.x, source_point.y);
    let target = (target_point.x, target_point.y);
    let points = [
        source,
        ((source.0 + target.0) / 2., (source.1 + target.1) / 2.),
        target,
    ];
    let mut frames = vec![capture_frame(runner, output, action, 0)];
    let mut previous_ms = action.frames[0];
    let mut drag_over_background = None;
    runner.move_cursor(source);
    runner.press_cursor(source);
    for index in 1..action.frames.len() {
        let progress =
            action.frames[index] as f64 / action.frames.last().copied().unwrap_or(1).max(1) as f64;
        let point = interpolate(&points, progress);
        runner.move_cursor(point);
        if index + 1 == action.frames.len() {
            runner.sync_and_update();
            drag_over_background = Some(background_for_label(runner, "Hide sidebar"));
            runner.release_cursor(point);
        }
        wait_and_capture(runner, output, action, &mut frames, &mut previous_ms, index);
    }
    let rail_order = crate::differential_ui::read_rail_order(vault_root);
    assert_eq!(
        rail_order.first().map(String::as_str),
        Some("search"),
        "HARD_ISSUE Freya action drag-search-rail-item: persisted rail order did not change after pointer drag"
    );
    assert!(
        distinct_frame_count(&frames) > 1,
        "HARD_ISSUE Freya action drag-search-rail-item: drag produced no rendered frame change"
    );
    assert_ne!(
        target_before_background,
        drag_over_background.expect("drag-over style must be captured before release"),
        "HARD_ISSUE Freya action drag-search-rail-item: drop target exposed no rendered drag-over transition"
    );
    let source_after = require_label(runner, "Search").layout().area;
    let target_after = require_label(runner, "Hide sidebar").layout().area;
    assert!(
        source_after.origin != source_before.origin || target_after.origin != target_before.origin,
        "HARD_ISSUE Freya action drag-search-rail-item: real rail target geometry did not change"
    );
    frames
}
