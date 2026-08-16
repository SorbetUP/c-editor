//! Freya Testing proof for the Settings modal lifecycle.
//!
//! This deliberately requires a real close contract. A Settings accessibility
//! label alone is not enough to claim that the user can return to the library.

use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey};
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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("elephant-freya-settings-modal-lifecycle-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nA fixture note.\n")
            .expect("write fixture note");
        Self { root }
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
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

#[test]
fn settings_modal_closes_and_returns_to_the_real_library() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(labeled_nodes(&runner, "ElephantNote settings").len(), 0);
    click_label(&mut runner, "Settings");
    runner.sync_and_update();

    assert_eq!(
        labeled_nodes(&runner, "ElephantNote settings").len(),
        1,
        "Settings must open the real panel, not only expose the rail label"
    );
    assert_eq!(labeled_nodes(&runner, "Settings sections").len(), 1);
    assert_eq!(
        labeled_nodes(&runner, "Settings section appearance").len(),
        1
    );

    if labeled_nodes(&runner, "Close settings").is_empty() {
        runner.press_key(Key::Named(NamedKey::Escape));
        runner.sync_and_update();
        assert_eq!(
            labeled_nodes(&runner, "ElephantNote settings").len(),
            0,
            "BLOCKED: no accessible `Close settings` control exists and Escape does not close the real Settings panel"
        );
    } else {
        click_label(&mut runner, "Close settings");
        runner.sync_and_update();
    }

    assert_eq!(
        labeled_nodes(&runner, "ElephantNote settings").len(),
        0,
        "closing Settings must unmount the real panel"
    );
    assert!(
        !labeled_nodes(&runner, "Sort: Updated newest").is_empty(),
        "closing Settings must return to the library toolbar"
    );
    assert!(
        !labeled_nodes(&runner, "Alpha").is_empty(),
        "closing Settings must return to the real library note card"
    );
}
