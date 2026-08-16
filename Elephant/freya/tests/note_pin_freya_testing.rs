use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    ffi::OsString,
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
        let root = std::env::temp_dir().join(format!("elephant-freya-note-pin-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nPin fixture note\n")
            .expect("write fixture note");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn workspace_path(&self) -> PathBuf {
        self.root.join(".elephantnote/config/workspace.json")
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct ProfileOverride {
    root: PathBuf,
    previous: Option<OsString>,
}

impl ProfileOverride {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-note-pin-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create isolated Freya profile");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }
}

impl Drop for ProfileOverride {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => std::env::set_var("ELEPHANT_FREYA_PROFILE", previous),
            None => std::env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = require_label(runner, label);
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn pinned_paths(workspace_path: &Path) -> Option<Vec<String>> {
    let raw = fs::read_to_string(workspace_path).ok()?;
    let value: Value = serde_json::from_str(&raw).expect("workspace metadata must be valid JSON");
    Some(
        value["freyaShell"]["pinnedPaths"]
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
    )
}

#[test]
fn pin_and_unpin_open_alpha_restores_through_library_and_metadata() {
    let _profile = ProfileOverride::new();
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let workspace_path = fixture.workspace_path();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Pin note").is_empty());

    click_label(&mut runner, "Pin note");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Unpin note").is_empty());
    assert!(accessible_nodes(&runner, "Pin note").is_empty());

    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Alpha").is_empty());
    let pinned_after_pin = pinned_paths(&workspace_path);

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert!(
        !accessible_nodes(&runner, "Unpin note").is_empty(),
        "reopening Alpha must restore its pinned state"
    );

    click_label(&mut runner, "Unpin note");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Pin note").is_empty());

    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    let pinned_after_unpin = pinned_paths(&workspace_path);

    assert!(
        pinned_after_pin
            .as_ref()
            .is_some_and(|paths| paths.iter().any(|path| path == "Alpha.md")),
        "Pin note must persist Alpha.md in freyaShell.pinnedPaths ({})",
        workspace_path.display()
    );
    assert!(
        pinned_after_unpin
            .as_ref()
            .is_some_and(|paths| !paths.iter().any(|path| path == "Alpha.md")),
        "Unpin note must remove Alpha.md from freyaShell.pinnedPaths ({})",
        workspace_path.display()
    );
}
