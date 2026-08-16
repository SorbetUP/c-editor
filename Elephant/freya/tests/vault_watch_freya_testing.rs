use elephant_freya::app::app_with_vault;
use freya::prelude::Paragraph;
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
        fs::write(root.join("Alpha.md"), "# Alpha\n\nOriginal body\n")
            .expect("write editable note");
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

fn paragraph_text(runner: &TestingRunner) -> String {
    runner
        .find_many(|node, _| Some(node))
        .into_iter()
        .filter_map(|node| Paragraph::try_downcast(node.element().as_ref()))
        .map(|paragraph| {
            paragraph
                .spans
                .iter()
                .map(|span| span.text.to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
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

#[test]
fn external_change_to_clean_open_note_reloads_the_real_editor_session() {
    let fixture = FixtureVault::new();
    let root = fixture.0.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let alpha = labeled(&runner, "Alpha")
        .into_iter()
        .next()
        .expect("Alpha card");
    runner.click_cursor(alpha.layout().area.center().to_f64());
    runner.sync_and_update();
    fs::write(fixture.0.join("Alpha.md"), "# Alpha\n\nExternal body\n")
        .expect("replace open note externally");
    runner.poll_n(Duration::from_millis(550), 3);

    let rendered = paragraph_text(&runner);
    assert!(
        rendered.contains("External body") && !rendered.contains("Original body"),
        "a clean open editor must consume the external file update; rendered paragraphs: {rendered:?}"
    );
}
