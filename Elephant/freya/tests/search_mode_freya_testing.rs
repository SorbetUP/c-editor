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

fn click_label_containing(runner: &mut TestingRunner, fragment: &str) {
    let node = labeled_nodes_containing(runner, fragment)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has a label containing {fragment:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

#[test]
fn search_modes_use_literal_exact_smart_fallback_and_explicit_semantic_boundary() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Search");
    runner.sync_and_update();
    click_label(&mut runner, "Search input");
    runner.write_text("alpha");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Open note Alphabet").len(),
        1,
        "Exact mode must match the literal query inside a real note title"
    );

    click_label_containing(&mut runner, "exact");
    runner.sync_and_update();
    assert!(
        labeled_nodes_containing(&runner, "Semantic search unavailable in Freya").len() == 1,
        "Semantic mode must not silently execute an exact/FTS query without an embedding index"
    );
    assert!(labeled_nodes(&runner, "Open note Alphabet").is_empty());

    click_label_containing(&mut runner, "semantic");
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Open note Alphabet").len(),
        1,
        "Smart mode must use the real exact fallback while no semantic index is connected"
    );
}
