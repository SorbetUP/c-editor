use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey, Rect};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-navigation-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create fixture directories");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nA fixture note\n")
            .expect("write fixture note");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nA nested fixture note\n",
        )
        .expect("write nested fixture note");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn workspace_path(&self) -> PathBuf {
        self.root
            .join(".elephantnote")
            .join("config")
            .join("workspace.json")
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        let accessibility = element.accessibility();
        (accessibility.builder.label() == Some(label)).then_some(node)
    })
}

fn require_labeled_node(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"))
}

fn center(node: &TestingNode) -> (f64, f64) {
    let area = node.layout().area;
    (
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    )
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    runner.click_cursor(center(&require_labeled_node(runner, label)));
}

fn rect_opacity(runner: &TestingRunner, label: &str) -> Option<f32> {
    Rect::try_downcast(require_labeled_node(runner, label).element().as_ref())
        .expect("navigation control must be backed by a rectangle")
        .effect
        .and_then(|effect| effect.opacity)
}

fn read_shell_preferences(fixture: &FixtureVault) -> Value {
    serde_json::from_str(
        &fs::read_to_string(fixture.workspace_path()).expect("native workspace preferences"),
    )
    .expect("valid native workspace preferences")
}

#[test]
fn navigation_history_updates_visible_controls_and_round_trips_back_forward() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(rect_opacity(&runner, "Retour"), Some(0.3));
    assert_eq!(rect_opacity(&runner, "Avancer"), Some(0.3));

    click_label(&mut runner, "Projects");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Plan").len() >= 1);
    assert_eq!(rect_opacity(&runner, "Retour"), Some(1.0));

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Alpha").len() >= 1);
    assert!(accessible_nodes(&runner, "Plan").is_empty());
    assert_eq!(rect_opacity(&runner, "Avancer"), Some(1.0));

    click_label(&mut runner, "Avancer");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Plan").len() >= 1);
    assert!(accessible_nodes(&runner, "Alpha").is_empty());
}

#[test]
fn sidebar_resize_uses_pointer_and_keyboard_and_restores_from_native_workspace() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let resizer = require_labeled_node(&runner, "Resize sidebar");
    let before = resizer.layout().area;
    let start = center(&resizer);
    runner.press_cursor(start);
    runner.move_cursor((start.0 + 40., start.1));
    runner.sync_and_update();
    let moved = require_labeled_node(&runner, "Resize sidebar")
        .layout()
        .area;
    assert!(
        moved.min_x() > before.min_x(),
        "pointer resize must move the divider"
    );
    runner.release_cursor((start.0 + 40., start.1));
    runner.sync_and_update();

    let persisted = read_shell_preferences(&fixture);
    let persisted_width = persisted["freyaShell"]["sidebarWidth"]
        .as_u64()
        .expect("sidebar width persisted as an integer");
    assert!(persisted_width > 232);

    let resizer = require_labeled_node(&runner, "Resize sidebar");
    runner.click_cursor(center(&resizer));
    runner.press_key(Key::Named(NamedKey::ArrowLeft));
    runner.sync_and_update();
    let keyboard_width = read_shell_preferences(&fixture)["freyaShell"]["sidebarWidth"]
        .as_u64()
        .expect("keyboard resize persisted as an integer");
    assert_eq!(keyboard_width, persisted_width - 16);

    let restart_root = fixture.path().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    let restored = require_labeled_node(&runner, "Resize sidebar")
        .layout()
        .area;
    assert_eq!(restored.min_x(), moved.min_x() - 16.);
}

#[test]
fn rail_search_dragged_before_sidebar_toggle_persists_and_restores() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let search = require_labeled_node(&runner, "Search");
    let toggle = require_labeled_node(&runner, "Hide sidebar");
    assert!(search.layout().area.min_y() > toggle.layout().area.min_y());

    let search_center = center(&search);
    let toggle_center = center(&toggle);
    runner.press_cursor(search_center);
    runner.move_cursor(toggle_center);
    runner.sync_and_update();
    assert_ne!(
        Rect::try_downcast(
            require_labeled_node(&runner, "Hide sidebar")
                .element()
                .as_ref()
        )
        .expect("sidebar toggle must remain a rectangle")
        .style
        .background,
        Rect::try_downcast(require_labeled_node(&runner, "Search").element().as_ref())
            .expect("search must remain a rectangle")
            .style
            .background,
        "drop target must have a distinct visual state",
    );
    runner.release_cursor(toggle_center);
    runner.sync_and_update();

    let search = require_labeled_node(&runner, "Search");
    let toggle = require_labeled_node(&runner, "Hide sidebar");
    assert!(search.layout().area.min_y() < toggle.layout().area.min_y());
    let preferences = read_shell_preferences(&fixture);
    let order = preferences["freyaShell"]["railOrder"]
        .as_array()
        .expect("rail order persisted as an array")
        .iter()
        .map(|value| value.as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(order, ["search", "sidebar-toggle"]);

    let restart_root = fixture.path().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    assert!(
        require_labeled_node(&runner, "Search")
            .layout()
            .area
            .min_y()
            < require_labeled_node(&runner, "Hide sidebar")
                .layout()
                .area
                .min_y()
    );
}

#[test]
fn malformed_workspace_json_is_visible_and_never_replaced() {
    let fixture = FixtureVault::new();
    fs::create_dir_all(fixture.workspace_path().parent().expect("workspace parent"))
        .expect("create workspace metadata directory");
    fs::write(fixture.workspace_path(), "{ not valid json")
        .expect("write malformed workspace metadata");
    let original = fs::read_to_string(fixture.workspace_path()).expect("read malformed metadata");
    let root = fixture.path().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(accessible_nodes(&runner, "Library error").len(), 1);
    assert_eq!(
        fs::read_to_string(fixture.workspace_path()).unwrap(),
        original
    );
}

#[test]
fn non_object_workspace_json_is_visible_and_never_replaced() {
    let fixture = FixtureVault::new();
    fs::create_dir_all(fixture.workspace_path().parent().expect("workspace parent"))
        .expect("create workspace metadata directory");
    fs::write(fixture.workspace_path(), "[]").expect("write non-object workspace metadata");
    let original = fs::read_to_string(fixture.workspace_path()).expect("read non-object metadata");
    let root = fixture.path().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(accessible_nodes(&runner, "Library error").len(), 1);
    assert_eq!(
        fs::read_to_string(fixture.workspace_path()).unwrap(),
        original
    );
}
