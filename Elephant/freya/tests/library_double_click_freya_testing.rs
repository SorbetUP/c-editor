use elephant_freya::app::app_with_vault;
use freya::prelude::Label;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
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
        let root =
            std::env::temp_dir().join(format!("elephant-freya-library-double-click-{stamp}"));
        fs::create_dir_all(root.join("Folder")).expect("create fixture folder");
        fs::write(
            root.join("Folder/Inside.md"),
            "# Inside\n\nNested fixture note.\n",
        )
        .expect("write nested fixture note");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nRoot fixture note.\n")
            .expect("write root fixture note");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn library_card_center(runner: &TestingRunner, label: &str) -> (f64, f64) {
    accessible_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("library card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya library card has accessible label {label:?}"))
        .layout()
        .area
        .center()
        .to_f64()
        .into()
}

fn library_title_center(runner: &TestingRunner, label: &str) -> (f64, f64) {
    runner
        .find_many(|node, element| {
            Label::try_downcast(element)
                .filter(|candidate| candidate.text.as_ref() == label)
                .map(|_| node)
        })
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .min_x()
                .partial_cmp(&right.layout().area.min_x())
                .expect("library title coordinates must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya library title has accessible label {label:?}"))
        .layout()
        .area
        .center()
        .to_f64()
        .into()
}

fn double_click_library_card(runner: &mut TestingRunner, label: &str) {
    let center = library_card_center(runner, label);
    runner.click_cursor(center);
    runner.click_cursor(center);
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();
}

fn double_click_library_title(runner: &mut TestingRunner, label: &str) {
    let center = library_title_center(runner, label);
    runner.click_cursor(center);
    runner.click_cursor(center);
    runner.sync_and_update();
}

fn click_library_title(runner: &mut TestingRunner, label: &str) {
    runner.click_cursor(library_title_center(runner, label));
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();
}

fn runner_for(fixture: &FixtureVault) -> TestingRunner {
    let root = fixture.root().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    runner
}

#[test]
fn double_clicking_a_folder_card_opens_the_directory_once() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    double_click_library_card(&mut runner, "Folder");

    assert!(!accessible_nodes(&runner, "Inside").is_empty());
    assert!(!accessible_nodes(&runner, "Show notes as list").is_empty());
    assert!(
        accessible_nodes(&runner, "Close note").is_empty(),
        "the second click must not activate a child card mounted under the first click"
    );
}

#[test]
fn double_clicking_a_note_card_opens_the_real_editor() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    double_click_library_card(&mut runner, "Alpha");

    assert!(!accessible_nodes(&runner, "Close note").is_empty());
    assert!(!accessible_nodes(&runner, "Paragraph").is_empty());
}

#[test]
fn double_clicking_a_folder_title_begins_inline_rename_without_opening_it() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    double_click_library_title(&mut runner, "Folder");

    assert!(!accessible_nodes(&runner, "Rename Folder").is_empty());
    assert!(accessible_nodes(&runner, "Inside").is_empty());
}

#[test]
fn double_clicking_a_note_title_begins_inline_rename_without_opening_it() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    double_click_library_title(&mut runner, "Alpha");

    assert!(!accessible_nodes(&runner, "Rename Alpha").is_empty());
    assert!(accessible_nodes(&runner, "Close note").is_empty());
}

#[test]
fn single_clicking_a_folder_title_still_opens_the_directory() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    click_library_title(&mut runner, "Folder");

    assert!(!accessible_nodes(&runner, "Inside").is_empty());
    assert!(!accessible_nodes(&runner, "Show notes as list").is_empty());
}
