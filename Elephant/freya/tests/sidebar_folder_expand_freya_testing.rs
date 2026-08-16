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
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-sidebar-expand-{stamp}"));
        fs::create_dir_all(root.join("Projects/Nested")).expect("create fixture directories");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nA direct child of Projects.\n",
        )
        .expect("write direct child fixture note");
        fs::write(
            root.join("Projects/Nested/Deep.md"),
            "# Deep\n\nA nested descendant fixture note.\n",
        )
        .expect("write nested descendant fixture note");
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

fn require_node(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    runner.click_cursor(require_node(runner, label).layout().area.center().to_f64());
}

fn assert_visible(runner: &TestingRunner, label: &str) {
    let node = require_node(runner, label);
    assert!(
        node.layout().area.size.area() > 0.,
        "Freya label {label:?} must have a visible layout area"
    );
}

#[test]
fn existing_sidebar_folder_expands_shows_descendants_and_collapses() {
    let fixture = FixtureVault::new();
    assert!(fixture.root.join("Projects/Plan.md").is_file());
    assert!(fixture.root.join("Projects/Nested/Deep.md").is_file());

    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_visible(&runner, "Projects");
    assert_eq!(accessible_nodes(&runner, "Expand Projects").len(), 1);
    assert!(accessible_nodes(&runner, "Collapse Projects").is_empty());
    assert!(accessible_nodes(&runner, "Plan").is_empty());
    assert!(accessible_nodes(&runner, "Nested").is_empty());

    click_label(&mut runner, "Expand Projects");
    runner.sync_and_update();

    assert_visible(&runner, "Collapse Projects");
    assert!(accessible_nodes(&runner, "Expand Projects").is_empty());
    assert_visible(&runner, "Plan");
    assert_visible(&runner, "Nested");
    assert!(
        accessible_nodes(&runner, "Deep").is_empty(),
        "a nested folder descendant must remain lazy until that nested folder is expanded"
    );

    click_label(&mut runner, "Collapse Projects");
    runner.sync_and_update();

    assert_visible(&runner, "Expand Projects");
    assert!(accessible_nodes(&runner, "Collapse Projects").is_empty());
    assert!(accessible_nodes(&runner, "Plan").is_empty());
    assert!(accessible_nodes(&runner, "Nested").is_empty());
}

#[test]
fn sidebar_shows_an_error_when_the_vault_root_disappears() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("elephant-freya-sidebar-error-{stamp}"));
    fs::create_dir_all(root.join("Broken")).expect("create fixture folder");
    fs::write(root.join("Broken/Before.md"), "# Before\n").expect("write fixture note");

    let app_root = root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    fs::remove_dir_all(&root).expect("remove vault after initial listing");

    click_label(&mut runner, "Expand Broken");
    runner.sync_and_update();
    assert_visible(&runner, "Sidebar error in root");
}
