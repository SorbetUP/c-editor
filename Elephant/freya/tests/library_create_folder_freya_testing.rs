use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-create-folder-{stamp}"));
        fs::create_dir_all(root.join("Folder")).expect("create fixture folder");
        Self { root }
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

fn click_node(runner: &mut TestingRunner, node: TestingNode) {
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
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

fn click_library_card(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("library card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya library card has accessible label {label:?}"));
    click_node(runner, node);
}

fn create_folder_through_ui(runner: &mut TestingRunner) {
    click_label(runner, "Create");
    runner.sync_and_update();
    assert_eq!(
        accessible_nodes(runner, "Folder").len(),
        1,
        "the Create popover must expose exactly one Folder action"
    );
    click_label(runner, "Folder");
    runner.sync_and_update();
}

#[test]
fn create_folder_targets_root_then_the_current_nested_directory() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let app_root = root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    create_folder_through_ui(&mut runner);
    assert!(
        root.join("New Folder").is_dir(),
        "Create → Folder at vault root must create <vault>/New Folder"
    );

    click_library_card(&mut runner, "Folder");
    runner.sync_and_update();
    create_folder_through_ui(&mut runner);
    assert!(
        root.join("Folder").join("New Folder").is_dir(),
        "Create → Folder inside Folder must create Folder/New Folder"
    );

    assert!(
        !root.join("Folder 2").exists(),
        "nested creation must not create a sibling by uniquifying the current folder path"
    );
}