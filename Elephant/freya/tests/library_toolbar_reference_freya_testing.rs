//! Functional parity tests for the current Tauri `LibraryToolbar.vue` contract.
//!
//! The reference cycles four sort modes in a fixed order and toggles grid/list.
//! These tests exercise the rendered Freya controls rather than the pure enum
//! helpers, so regressions in event wiring or accessibility state are caught.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
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
            .expect("clock must be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-toolbar-{stamp}"));
        fs::create_dir_all(&root).expect("create toolbar fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n").expect("write Alpha fixture");
        fs::write(root.join("Zulu.md"), "# Zulu\n").expect("write Zulu fixture");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing toolbar control {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
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
fn sort_control_cycles_in_the_exact_tauri_order() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    assert_eq!(nodes(&runner, "Sort: Updated newest").len(), 1);

    click(&mut runner, "Sort: Updated newest");
    assert_eq!(nodes(&runner, "Sort: Updated oldest").len(), 1);

    click(&mut runner, "Sort: Updated oldest");
    assert_eq!(nodes(&runner, "Sort: Title A-Z").len(), 1);

    click(&mut runner, "Sort: Title A-Z");
    assert_eq!(nodes(&runner, "Sort: Title Z-A").len(), 1);

    click(&mut runner, "Sort: Title Z-A");
    assert_eq!(nodes(&runner, "Sort: Updated newest").len(), 1);
}

#[test]
fn view_control_toggles_grid_and_list_bidirectionally() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    assert_eq!(
        nodes(&runner, "Show notes as list").len(),
        1,
        "Tauri starts the library in grid mode, so the action must offer list"
    );

    click(&mut runner, "Show notes as list");
    assert_eq!(nodes(&runner, "Show notes as grid").len(), 1);

    click(&mut runner, "Show notes as grid");
    assert_eq!(nodes(&runner, "Show notes as list").len(), 1);
}
