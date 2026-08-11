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
fn search_input_escape_and_graph_refresh_expose_real_graph_boundary() {
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
        !labeled_nodes(&runner, "Graph service unavailable.").is_empty(),
        "refresh must expose the production Graph service error when no native adapter exists"
    );

    // The source Graph view never treats an unavailable service as an empty
    // successful graph.  Keep this assertion strict so a future adapter must
    // provide real nodes/edges before this scenario can become a data proof.
    assert!(labeled_nodes(&runner, "Nodes").is_empty());
    assert!(labeled_nodes(&runner, "Edges").is_empty());

    // These are still real Freya input/button actions. They must not turn the
    // explicit service error into fabricated filtered or recentered data.
    click_label(&mut runner, "Graph filter input");
    runner.write_text("alpha");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert!(!labeled_nodes(&runner, "Graph service unavailable.").is_empty());

    click_label(&mut runner, "Reset graph filter");
    runner.sync_and_update();
    assert!(!labeled_nodes(&runner, "Graph service unavailable.").is_empty());
}
