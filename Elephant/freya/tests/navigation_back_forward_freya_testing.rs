use elephant_freya::app::app_with_vault;
use freya::prelude::Rect;
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
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-back-forward-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create fixture directory");
        fs::write(root.join("Root.md"), "# Root\n\nRoot fixture note.\n")
            .expect("write root fixture note");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nNested fixture note.\n",
        )
        .expect("write nested fixture note");
        Self { root }
    }

    fn path(&self) -> &Path {
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

fn control_opacity(runner: &TestingRunner, label: &str) -> Option<f32> {
    Rect::try_downcast(
        accessible_nodes(runner, label)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("navigation control {label:?} is not accessible"))
            .element()
            .as_ref(),
    )
    .expect("navigation control must be backed by a rectangle")
    .effect
    .and_then(|effect| effect.opacity)
}

#[test]
fn all_notes_folder_note_back_then_forward_preserves_observable_navigation() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(accessible_nodes(&runner, "All notes").len(), 1);
    click_sidebar_label(&mut runner, "All notes");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Projects").is_empty());
    assert!(!accessible_nodes(&runner, "Root").is_empty());

    click_library_card(&mut runner, "Projects");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Plan").is_empty());
    assert!(
        fixture.path().join("Projects").join("Plan.md").is_file(),
        "the observable Projects -> Plan fixture path must exist"
    );

    click_library_card(&mut runner, "Plan");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Heading 1").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Close note").len(), 1);

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Plan").is_empty());
    assert!(accessible_nodes(&runner, "NoteEditorHost").is_empty());
    assert_eq!(control_opacity(&runner, "Retour"), Some(1.0));
    assert!(
        !accessible_nodes(&runner, "Avancer").is_empty(),
        "Back must expose the Forward control as an observable continuation"
    );
    assert_eq!(control_opacity(&runner, "Avancer"), Some(1.0));

    // The UI exposes the folder and note labels, but not the private relative
    // path. The fixture path assertion above plus these visible transitions
    // are therefore the strongest path evidence available without state access.
    click_label(&mut runner, "Avancer");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Heading 1").len(), 1);
    assert_eq!(control_opacity(&runner, "Retour"), Some(1.0));
    assert_eq!(control_opacity(&runner, "Avancer"), Some(0.3));
}
