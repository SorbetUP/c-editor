//! Freya Testing proof for the visible Create -> Folder and Create -> Note paths.
//!
//! These tests require both the user-visible action and the real vault effect;
//! an accessibility label without a filesystem change or real opening is not
//! sufficient evidence.

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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-create-actions-{stamp}"));
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

fn click_node(runner: &mut TestingRunner, node: TestingNode) {
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("accessible node areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    click_node(runner, node);
}

fn click_create_menu_action(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .min_x()
                .partial_cmp(&right.layout().area.min_x())
                .expect("Create action positions must be ordered")
        })
        .unwrap_or_else(|| panic!("Create popover has no {label:?} action"));
    click_node(runner, node);
}

fn library_card(runner: &TestingRunner, label: &str) -> TestingNode {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .filter(|node| {
            let size = node.layout().area.size;
            size.width >= 200. && size.height >= 100.
        })
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("library card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya library card has accessible label {label:?}"));
    node
}

fn click_library_card(runner: &mut TestingRunner, label: &str) {
    let node = library_card(runner, label);
    click_node(runner, node);
}

fn open_create_menu(runner: &mut TestingRunner) {
    click_label(runner, "Create");
    runner.sync_and_update();
    assert_eq!(labeled_nodes(runner, "Note").len(), 1);
    assert_eq!(labeled_nodes(runner, "Folder").len(), 1);
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
fn create_folder_writes_the_vault_entry_and_opens_the_real_directory() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    open_create_menu(&mut runner);
    click_create_menu_action(&mut runner, "Folder");
    runner.sync_and_update();

    assert!(
        fixture.root.join("New Folder").is_dir(),
        "Create -> Folder must create the directory in the temporary vault"
    );
    let folder_card = library_card(&runner, "New Folder");
    assert!(
        folder_card.layout().area.size.width >= 200.
            && folder_card.layout().area.size.height >= 100.,
        "the created folder must be visible as a real library card"
    );

    click_library_card(&mut runner, "New Folder");
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Empty library").len(),
        1,
        "opening the created folder must switch the real library directory"
    );
}

#[test]
fn create_note_writes_an_entry_shows_it_and_opens_the_real_editor() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    open_create_menu(&mut runner);
    click_create_menu_action(&mut runner, "Note");
    runner.sync_and_update();

    let note_path = fixture.root.join("Untitled.md");
    assert!(
        note_path.is_file(),
        "Create -> Note must create Untitled.md in the temporary vault"
    );
    assert_eq!(
        labeled_nodes(&runner, "Close note").len(),
        1,
        "Create -> Note must open the real editor, not only create a file"
    );
    assert_eq!(
        labeled_nodes(&runner, "Note title").len(),
        1,
        "the newly created note must expose its real title control"
    );

    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    let note_card = library_card(&runner, "Untitled");
    assert!(
        note_card.layout().area.size.width >= 200. && note_card.layout().area.size.height >= 100.,
        "the created note must remain visible as a real library card after closing"
    );

    click_library_card(&mut runner, "Untitled");
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Close note").len(),
        1,
        "opening the created entry must mount the real editor again"
    );
    assert!(
        fs::read_to_string(note_path)
            .expect("read created note")
            .is_empty(),
        "the newly created note must use the production empty-note contract"
    );
}
