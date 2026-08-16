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
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("elephant-freya-sidebar-folder-navigation-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create fixture folder");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nA nested navigation fixture.\n",
        )
        .expect("write nested fixture note");
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
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    click_node(runner, node);
}

fn click_sidebar_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .min_x()
                .partial_cmp(&right.layout().area.min_x())
                .expect("sidebar x positions must be ordered")
        })
        .unwrap_or_else(|| panic!("no sidebar node has accessible label {label:?}"));
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
        .unwrap_or_else(|| panic!("no library card has accessible label {label:?}"));
    click_node(runner, node);
}

#[test]
fn sidebar_folder_navigation_opens_nested_note_and_returns_to_library() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(!accessible_nodes(&runner, "Projects").is_empty());
    assert_eq!(accessible_nodes(&runner, "Expand Projects").len(), 1);
    assert!(accessible_nodes(&runner, "Plan").is_empty());

    click_label(&mut runner, "Expand Projects");
    runner.sync_and_update();

    assert_eq!(accessible_nodes(&runner, "Collapse Projects").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Plan").len(), 1);

    click_sidebar_label(&mut runner, "Projects");
    runner.sync_and_update();

    assert!(
        accessible_nodes(&runner, "Plan").len() >= 2,
        "opening Projects must expose Plan in both the expanded sidebar and the library"
    );
    assert_eq!(accessible_nodes(&runner, "Sort: Updated newest").len(), 1);

    click_library_card(&mut runner, "Plan");
    runner.sync_and_update();

    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Close note").len(), 1);

    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    assert!(accessible_nodes(&runner, "NoteEditorHost").is_empty());
    assert_eq!(accessible_nodes(&runner, "Sort: Updated newest").len(), 1);
    assert!(!accessible_nodes(&runner, "Create").is_empty());
}
