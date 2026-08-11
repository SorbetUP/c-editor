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
        let root = std::env::temp_dir().join(format!("elephant-freya-search-graph-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Alpha.md"),
            "# Alpha\n\nA searchable fixture note.\n",
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
fn search_input_escape_and_graph_refresh_follow_source_states() {
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
    assert_eq!(labeled_nodes(&runner, "Search workspace").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Search input").len(), 1);

    click_label(&mut runner, "Search input");
    runner.write_text("alpha");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert!(
        !labeled_nodes(&runner, "Searching locally…").is_empty(),
        "the submitted query must expose the loading state"
    );

    runner.press_key(Key::Named(NamedKey::Escape));
    runner.sync_and_update();
    assert!(
        !labeled_nodes(&runner, "Search notes, paths, tags, or ideas…").is_empty(),
        "Escape must return the search surface to its empty state"
    );

    click_label(&mut runner, "Graph workspace");
    runner.sync_and_update();
    assert!(
        !labeled_nodes(&runner, "Graph not loaded").is_empty(),
        "the graph surface must expose its initial empty state"
    );
    assert_eq!(labeled_nodes(&runner, "Graph filter input").len(), 1);

    click_label(&mut runner, "Refresh graph");
    runner.sync_and_update();
    assert!(
        !labeled_nodes(&runner, "Building the semantic graph…").is_empty(),
        "refresh must expose the graph loading state"
    );
}
