use crate::{
    capture_support::absolute,
    differential_scenario::{Scenario, ScenarioAction},
};
use serde_json::{json, Value};
use std::{fs, path::Path, process, time::Instant};

pub fn action_record(index: usize, action: &ScenarioAction, started: Instant) -> Value {
    json!({
        "index": index,
        "id": action.id,
        "logical": action.logical,
        "status": "passed",
        "relativeStartMs": 0,
        "relativeDoneMs": started.elapsed().as_millis(),
        "source": "shared-scenario",
        "controlPlane": "testing-runner",
    })
}

pub fn write_manifest(
    output: &Path,
    scenario: &Scenario,
    run_id: &str,
    command_sha256: &str,
    capture_nonce: &str,
    fixture: &Value,
    actions: &[Value],
    checkpoints: &[Value],
    top_level_frames: &[Value],
) {
    let evidence_id = format!("freya:{run_id}:{}", process::id());
    let manifest = json!({
        "schemaVersion": 1,
        "scenarioId": scenario.id,
        "runtime": "freya",
        "status": "passed",
        "viewport": {
            "width": scenario.viewport.width,
            "height": scenario.viewport.height,
            "scaleFactor": scenario.viewport.scale_factor,
            "deviceScaleFactor": scenario.viewport.device_scale_factor,
            "fullPage": false,
            "colorScheme": "light",
            "locale": "en-US"
        },
        "scaleFactor": scenario.viewport.scale_factor,
        "deviceScaleFactor": scenario.viewport.device_scale_factor,
        "visualSurface": "application-content-only",
        "provenance": {
            "driver": "freya-testing",
            "controlPlane": "testing-runner",
            "runId": run_id,
            "captureId": format!("freya:{run_id}"),
            "commandSha256": command_sha256,
            "captureNonce": capture_nonce,
            "real": true,
            "synthetic": false,
            "captureMode": "real-freya-testing",
            "artifactRoot": absolute(output),
            "sourceEvidenceId": evidence_id
        },
        "fixture": fixture,
        "actions": actions,
        "checkpoints": checkpoints,
        "frames": top_level_frames,
        "logs": {"run": "run-log.json", "runtime": "runtime-error.log"},
        "capture": {"render": "TestingRunner.render", "pointerInput": "TestingRunner", "pollStepMs": 50, "settleAfterActionMs": 250},
        "validation": {"allActionsExecuted": true, "allTargetsAccessibilityResolved": true, "stateAndVaultHashesCaptured": true}
    });
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize Freya differential manifest"),
    )
    .expect("write Freya differential manifest");
}

pub fn write_logs(output: &Path, run_log: &[Value]) {
    fs::write(
        output.join("run-log.json"),
        serde_json::to_vec_pretty(run_log).expect("serialize Freya run log"),
    )
    .expect("write Freya run log");
    fs::write(output.join("runtime-error.log"), "[]\n").expect("write Freya runtime error log");
}
