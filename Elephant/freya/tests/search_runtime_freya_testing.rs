use elephant_freya::{app::app_with_vault, vault_adapter::VaultAdapter};
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
        let root = std::env::temp_dir().join(format!("elephant-freya-search-runtime-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Alpha.md"),
            "# Alpha\n\nA searchable production-path fixture note.\n",
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

fn open_search_and_submit(runner: &mut TestingRunner, query: &str) {
    click_label(runner, "Search");
    runner.sync_and_update();
    click_label(runner, "Search input");
    runner.write_text(query);
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
}

#[test]
fn search_submit_uses_real_vault_result_and_keyboard_opens_single_editor_owner() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_search_and_submit(&mut runner, "searchable production");
    assert_eq!(labeled_nodes(&runner, "Open note Alpha").len(), 1);

    runner.press_key(Key::Named(NamedKey::ArrowDown));
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();

    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Paragraph").len(), 1);
}

#[test]
fn escape_closes_after_clear_and_missing_result_is_visible_as_error() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_search_and_submit(&mut runner, "alpha");
    assert_eq!(labeled_nodes(&runner, "Open note Alpha").len(), 1);

    fs::remove_file(fixture.path().join("Alpha.md")).expect("remove result after search");
    runner.press_key(Key::Named(NamedKey::ArrowDown));
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert!(
        runner
            .find_many(|node, element| {
                let accessibility = element.accessibility();
                let label = accessibility.builder.label()?;
                label.contains("Search failed for Alpha.md").then_some(node)
            })
            .len()
            == 1,
        "opening a vanished result must expose the production error"
    );

    runner.press_key(Key::Named(NamedKey::Escape));
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::Escape));
    runner.sync_and_update();
    assert!(labeled_nodes(&runner, "Search input").is_empty());
}

#[test]
fn search_rebuilds_production_fts_index_and_preserves_bm25_ranking() {
    let fixture = FixtureVault::new();
    fs::write(
        fixture.path().join("Dense.md"),
        "# Dense\n\nneedle needle needle needle\n",
    )
    .expect("write dense FTS fixture");
    fs::write(fixture.path().join("Sparse.md"), "# Sparse\n\nneedle\n")
        .expect("write sparse FTS fixture");

    let adapter = VaultAdapter::open(fixture.path()).expect("open production vault adapter");
    let refresh = adapter
        .rebuild_search_index()
        .expect("rebuild production FTS index");
    assert_eq!(refresh.status, "complete");
    assert!(
        fixture
            .path()
            .join(".elephantnote/index/notes.sqlite")
            .is_file(),
        "the real production index path must be created"
    );

    let hits = adapter
        .search_index("needle", 20)
        .expect("query production FTS index");
    assert!(hits.len() >= 2);
    assert_eq!(hits[0].path, "Dense.md");
    assert!(
        hits[0].score < hits[1].score,
        "the ordered BM25 scores must come from FTS ranking"
    );
}
