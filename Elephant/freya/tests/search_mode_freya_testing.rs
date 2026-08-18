use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey};
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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-search-mode-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Alphabet.md"),
            "# Alphabet\n\nThe real exact-match fixture contains alphabetic content.\n",
        )
        .expect("write fixture note");
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

fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn labeled_nodes_containing(runner: &TestingRunner, fragment: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        element
            .accessibility()
            .builder
            .label()
            .filter(|label| label.contains(fragment))
            .map(|_| node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

#[test]
fn search_overlay_matches_tauri_without_tabs_or_mode_controls() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Search notes");
    runner.poll(
        std::time::Duration::from_millis(10),
        std::time::Duration::from_millis(260),
    );
    runner.sync_and_update();
    click_label(&mut runner, "Search input");
    runner.write_text("alpha");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    runner.poll(
        std::time::Duration::from_millis(10),
        std::time::Duration::from_millis(260),
    );
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Open note Alphabet").len(),
        1,
        "Exact mode must match the literal query inside a real note title"
    );
    assert!(
        labeled_nodes_containing(&runner, "Search mode:").is_empty(),
        "SEARCH-MODE-001: the Tauri SearchModal has no mode control"
    );
    assert_eq!(labeled_nodes(&runner, "Graph workspace").len(), 0);
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "NoteEditorHost").len(),
        1,
        "SEARCH-MODE-002: Enter must open the exact-match note from the modal"
    );
}
