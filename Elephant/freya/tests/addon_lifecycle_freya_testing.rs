//! Freya proof for the real add-on lifecycle surface.
//!
//! This test deliberately removes the persisted registry after the settings
//! list has loaded. The subsequent action must expose the host error instead
//! of clearing it during a best-effort refresh. It also proves that a
//! JavaScript add-on is identified as unavailable rather than treated as a
//! migrated worker.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-addon-lifecycle-{stamp}"));
        let addons = root.join(".elephantnote/addons");
        fs::create_dir_all(addons.join("packages/example-addon")).expect("create addon package");
        fs::write(root.join("Welcome.md"), "# Welcome\n\nLifecycle fixture.\n")
            .expect("write fixture note");
        fs::write(
            addons.join("registry.json"),
            json!({
                "version": 1,
                "addons": {
                    "example-addon": {
                        "manifest": {
                            "id": "example-addon",
                            "name": "Example addon",
                            "version": "1.0.0",
                            "runtime": {"type": "javascript-worker", "entry": "index.js"}
                        },
                        "enabled": false,
                        "packageHash": "fixture",
                        "installedAt": "2026-08-16T00:00:00Z",
                        "source": "external"
                    }
                }
            })
            .to_string(),
        )
        .expect("write addon registry");
        Self { root }
    }

    fn registry_path(&self) -> PathBuf {
        self.root.join(".elephantnote/addons/registry.json")
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"));
    let area = node.layout().visible_area();
    runner.click_cursor((
        f64::from(area.origin.x + area.size.width / 2.0),
        f64::from(area.origin.y + area.size.height / 2.0),
    ));
    runner.sync_and_update();
}

fn runner_for(fixture: &FixtureVault) -> TestingRunner {
    let root = fixture.root.clone();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    runner
}

#[test]
fn addon_action_error_and_disconnected_worker_status_are_explicit() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    click_label(&mut runner, "Settings");
    click_label(&mut runner, "Select Addons settings");
    click_label(&mut runner, "Refresh addons");

    assert_eq!(
        labeled_nodes(
            &runner,
            "Addon runtime status Example addon: JavaScript worker runtime is not connected in Freya",
        )
        .len(),
        1,
        "Freya must not claim that the JavaScript worker is migrated"
    );

    fs::remove_file(fixture.registry_path()).expect("remove registry to force host error");
    click_label(&mut runner, "Enable Example addon addon");

    let error_labels = runner.find_many(|node, element| {
        element
            .accessibility()
            .builder
            .label()
            .filter(|label| label.starts_with("Addon lifecycle error: "))
            .map(|_| node)
    });
    assert_eq!(
        error_labels.len(),
        1,
        "the failed host action must stay visible"
    );
    let error = error_labels[0]
        .element()
        .accessibility()
        .builder
        .label()
        .expect("error label")
        .to_owned();
    assert!(error.contains("Unknown addon"), "unexpected error: {error}");
}
