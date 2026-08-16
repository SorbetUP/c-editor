use elephant_freya::app::app_with_vault;
use freya::prelude::Paragraph;
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
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-links-{stamp}"));
        fs::create_dir_all(root.join("Notes")).expect("create notes directory");
        fs::write(
            root.join("Notes/Alpha.md"),
            "# Alpha\n\nOpen [Beta](Beta.md#target).\n",
        )
        .expect("write source note");
        fs::write(
            root.join("Notes/Beta.md"),
            "# Beta\n\n## Target\nTarget note.\n",
        )
        .expect("write target note");
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
        .unwrap_or_else(|| panic!("missing accessible Freya target {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_main_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .find(|node| node.layout().area.min_x() > 280.)
        .unwrap_or_else(|| panic!("missing main-content target {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        f64::from((area.min_x() + area.max_x()) / 2.),
        f64::from((area.min_y() + area.max_y()) / 2.),
    ));
}

fn click_inline_link_paragraph(runner: &mut TestingRunner) {
    let node = runner
        .find_many(|node, _| {
            let paragraph = Paragraph::try_downcast(node.element().as_ref())?;
            let text = paragraph
                .spans
                .iter()
                .map(|span| span.text.as_ref())
                .collect::<String>();
            (node.layout().area.min_x() > 280. && text.contains("Open ")).then_some(node)
        })
        .into_iter()
        .next()
        .expect("missing inline link paragraph");
    runner.click_cursor(node.layout().area.center().to_f64());
}

#[test]
fn activating_an_internal_markdown_link_opens_the_target_note() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_main_label(&mut runner, "Notes");
    runner.sync_and_update();
    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    click_label(&mut runner, "Open link Beta");
    runner.sync_and_update();

    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(
        labeled_nodes(&runner, "Open link Beta").len(),
        0,
        "the target note must be open through the real editor path"
    );
    assert_eq!(
        labeled_nodes(&runner, "Library error").len(),
        0,
        "a valid heading fragment must not surface an anchor error"
    );
}

#[test]
fn clicking_an_inline_markdown_link_opens_the_target_note() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_main_label(&mut runner, "Notes");
    runner.sync_and_update();
    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    click_inline_link_paragraph(&mut runner);
    runner.sync_and_update();

    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Open link Beta").len(), 0);
    assert_eq!(labeled_nodes(&runner, "Library error").len(), 0);
}
