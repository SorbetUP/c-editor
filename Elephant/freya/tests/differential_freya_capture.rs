//! Real Freya Testing adapter for the shared fourteen-action differential run.
//!
//! The test remains deliberately boring: it validates the orchestrator
//! contract, creates the real application, dispatches each shared action to
//! the focused support modules, records the verified checkpoint, and writes
//! the manifest only after all actions complete.

#[path = "support/differential_capture.rs"]
mod capture_support;
#[path = "support/differential_actions.rs"]
mod differential_actions;
#[path = "support/differential_frames.rs"]
mod differential_frames;
#[path = "support/differential_manifest.rs"]
mod differential_manifest;
#[path = "support/differential_scenario.rs"]
mod differential_scenario;
#[path = "support/differential_snapshot.rs"]
mod differential_snapshot;
#[path = "support/differential_timelines.rs"]
mod differential_timelines;
#[path = "support/differential_ui.rs"]
mod differential_ui;

use differential_actions::run_action;
use differential_manifest::{action_record, write_logs, write_manifest};
use differential_scenario::{
    fixture_record, load_scenario, required_env, required_path_env, vault_root, ProfileGuard,
    Viewport, SCENARIO_PATH,
};
use differential_snapshot::checkpoint_record;
use differential_timelines::settle;
use elephant_freya::app::app_with_vault;
use freya_testing::TestingRunner;
use serde_json::json;
use std::{env, fs, path::PathBuf, time::Instant};

#[test]
fn capture_shared_fourteen_action_freya_journey() {
    let scenario_path = env::var_os("DIFFERENTIAL_SCENARIO_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(SCENARIO_PATH)
        });
    let scenario = load_scenario(&scenario_path);
    let output = required_path_env("DIFFERENTIAL_OUTPUT_DIR");
    let fixture_root = required_path_env("DIFFERENTIAL_FIXTURE_ROOT");
    let expected_viewport: Viewport =
        serde_json::from_str(&required_env("DIFFERENTIAL_EXPECTED_VIEWPORT_JSON"))
            .expect("parse orchestrator viewport contract");
    assert_eq!(expected_viewport.width, scenario.viewport.width);
    assert_eq!(expected_viewport.height, scenario.viewport.height);
    assert_eq!(
        expected_viewport.scale_factor,
        scenario.viewport.scale_factor
    );
    assert_eq!(
        expected_viewport.device_scale_factor,
        scenario.viewport.device_scale_factor
    );

    let run_id = required_env("DIFFERENTIAL_RUN_ID");
    let command_sha256 = required_env("DIFFERENTIAL_COMMAND_SHA256");
    let capture_nonce = required_env("DIFFERENTIAL_CAPTURE_NONCE");
    fs::create_dir_all(&output).expect("create Freya differential output");
    let vault_root = vault_root(&scenario, &fixture_root);
    let fixture = fixture_record(&scenario, &fixture_root);
    let _profile = ProfileGuard::new(&output);
    let app_vault = vault_root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_vault.clone()),
        (scenario.viewport.width, scenario.viewport.height).into(),
        |_| (),
        scenario.viewport.scale_factor.into(),
    );
    runner.animation_clock().set_speed(1.0);

    let mut actions = Vec::new();
    let mut checkpoints = Vec::new();
    let mut top_level_frames = Vec::new();
    let mut run_log = Vec::new();

    for (index, action) in scenario.actions.iter().enumerate() {
        let started = Instant::now();
        run_log.push(json!({
            "event": "action:start",
            "index": index,
            "id": action.id,
            "logical": action.logical
        }));
        let frames = run_action(&mut runner, &output, action, &vault_root);
        settle(&mut runner);
        let checkpoint = checkpoint_record(&runner, &vault_root, action, frames.clone());
        actions.push(action_record(index, action, started));
        top_level_frames.extend(frames.iter().cloned().map(|frame| {
            json!({
                "actionId": action.id,
                "checkpoint": action.checkpoint,
                "frame": frame
            })
        }));
        checkpoints.push(checkpoint);
        run_log.push(json!({
            "event": "action:done",
            "index": index,
            "id": action.id,
            "status": "passed",
            "durationMs": started.elapsed().as_millis()
        }));
    }

    write_manifest(
        &output,
        &scenario,
        &run_id,
        &command_sha256,
        &capture_nonce,
        &fixture,
        &actions,
        &checkpoints,
        &top_level_frames,
    );
    write_logs(&output, &run_log);
}
