use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-library-{name}-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        Self { root }
    }

    fn write_note(&self, relative_path: &str, title: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create note parent directory");
        }
        fs::write(path, format!("# {title}\n\nFixture note.\n")).expect("write fixture note");
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
fn create_button_opens_all_visible_actions_without_executing_one() {
    let fixture = FixtureVault::new("create-menu");
    fixture.write_note("Existing.md", "Existing");
    let mut runner = runner_for(&fixture);

    click_label(&mut runner, "Create");
    runner.sync_and_update();

    // `menu_open` is private shell state. The visible action labels are the
    // public headless signal that the Create popover is actually mounted.
    assert!(!accessible_nodes(&runner, "Note").is_empty());
    assert!(!accessible_nodes(&runner, "Drawing").is_empty());
    assert!(!accessible_nodes(&runner, "Folder").is_empty());
    assert!(
        !fixture.root().join("New Folder").exists(),
        "opening Create must not execute the Folder action"
    );
}

#[test]
fn library_view_control_toggles_between_grid_and_list() {
    let fixture = FixtureVault::new("view-toggle");
    fixture.write_note("Existing.md", "Existing");
    let mut runner = runner_for(&fixture);

    assert!(!accessible_nodes(&runner, "Show notes as list").is_empty());
    assert!(accessible_nodes(&runner, "Show notes as grid").is_empty());

    click_label(&mut runner, "Show notes as list");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Show notes as grid").is_empty());
    assert!(accessible_nodes(&runner, "Show notes as list").is_empty());

    click_label(&mut runner, "Show notes as grid");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Show notes as list").is_empty());
    assert!(accessible_nodes(&runner, "Show notes as grid").is_empty());
}

#[test]
fn library_sort_control_cycles_through_visible_sort_states() {
    let fixture = FixtureVault::new("sort-toggle");
    fixture.write_note("Existing.md", "Existing");
    let mut runner = runner_for(&fixture);

    let cycle = [
        ("Sort: Updated newest", "Sort: Updated oldest"),
        ("Sort: Updated oldest", "Sort: Title A-Z"),
        ("Sort: Title A-Z", "Sort: Title Z-A"),
        ("Sort: Title Z-A", "Sort: Updated newest"),
    ];
    for (current, next) in cycle {
        assert!(!accessible_nodes(&runner, current).is_empty());
        click_label(&mut runner, current);
        runner.sync_and_update();
        assert!(!accessible_nodes(&runner, next).is_empty());
        assert!(accessible_nodes(&runner, current).is_empty());
    }
}

#[test]
fn clicking_a_library_folder_opens_its_real_directory() {
    let fixture = FixtureVault::new("open-folder");
    fixture.write_note("Folder/Inside.md", "Inside");
    let mut runner = runner_for(&fixture);

    assert!(!accessible_nodes(&runner, "Folder").is_empty());
    click_library_card(&mut runner, "Folder");
    runner.sync_and_update();

    // The current relative path is internal shell state; the child entry and
    // the Back control are the observable result of opening the folder.
    assert!(!accessible_nodes(&runner, "Inside").is_empty());
    assert!(!accessible_nodes(&runner, "Retour").is_empty());
    assert!(fixture.root().join("Folder").join("Inside.md").is_file());
}
