use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct FixtureVault(PathBuf);

impl FixtureVault {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-watch-integration-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create watched vault");
        fs::write(root.join("Welcome.md"), "# Welcome\n").expect("write initial note");
        Self(root)
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn labeled(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)
            && node.layout().area.min_x() > 280.)
            .then_some(node)
    })
}

#[test]
fn external_vault_note_appears_after_the_watcher_poll() {
    let fixture = FixtureVault::new();
    let root = fixture.0.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(labeled(&runner, "Welcome").len(), 1);
    runner.poll_n(Duration::from_millis(550), 2);
    fs::write(fixture.0.join("External.md"), "# External\n").expect("write external note");
    runner.poll_n(Duration::from_millis(550), 3);

    assert_eq!(
        labeled(&runner, "External").len(),
        1,
        "an externally-created note must reach the current Freya library page"
    );
}
