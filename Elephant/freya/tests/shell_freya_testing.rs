use elephant_freya::{
    app::app_with_vault,
    source_contracts::{self, ComponentId},
};
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
        let root = std::env::temp_dir().join(format!("elephant-freya-shell-{stamp}"));
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
    let nodes = accessible_nodes(runner, label);
    nodes
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = require_labeled_node(runner, label);
    let area = node.layout().area;
    let center = (
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    );
    runner.click_cursor(center);
}

#[test]
fn converted_shell_exposes_vue_source_contracts_through_freya_accessibility() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    for component in ComponentId::ALL {
        let contract = source_contracts::contract(component)
            .unwrap_or_else(|| panic!("missing source contract for {component:?}"));
        assert_eq!(contract.id, component);
        assert!(contract
            .provenance
            .source_path
            .ends_with(component.source_name()));
        assert!(!contract.provenance.style_sources.is_empty());
        assert!(!contract.semantics.is_empty());
        assert!(!contract.dimensions.is_empty());
        assert!(!contract.states.is_empty());
        assert!(!contract.events.is_empty());
    }

    assert_eq!(accessible_nodes(&runner, "TopVaultBar").len(), 1);
    assert!(accessible_nodes(&runner, "Create").len() >= 1);
    assert!(accessible_nodes(&runner, "Sort: updated-newest").len() >= 1);
    assert!(accessible_nodes(&runner, "Show notes as list").len() >= 1);
    assert!(accessible_nodes(&runner, "Alpha").len() >= 1);
    assert!(accessible_nodes(&runner, "Projects").len() >= 1);
    assert_eq!(accessible_nodes(&runner, "All notes").len(), 1);

    click_label(&mut runner, "Create");
    assert_eq!(accessible_nodes(&runner, "Note").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Drawing").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Folder").len(), 1);

    click_label(&mut runner, "Note");
    runner.sync_and_update();

    assert!(fixture.path().join("Untitled.md").is_file());
    assert!(accessible_nodes(&runner, "Untitled").len() >= 1);

    click_label(&mut runner, "Projects");
    assert!(accessible_nodes(&runner, "Plan").len() >= 1);
}

#[test]
fn converted_settings_search_graph_and_editor_surfaces_are_reachable() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "ElephantNote settings").len() >= 1);
    assert!(accessible_nodes(&runner, "Settings sections").len() >= 1);
    assert!(accessible_nodes(&runner, "Settings section appearance").len() >= 1);
    click_label(&mut runner, "Select Editor settings");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Settings section editor").len() >= 1);

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    click_label(&mut runner, "Search");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Search workspace").len() >= 1);
    assert!(accessible_nodes(&runner, "Graph workspace").len() >= 1);
    click_label(&mut runner, "Graph workspace");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Graph not loaded").len() >= 1);
    assert!(accessible_nodes(&runner, "Refresh graph").len() >= 1);

    click_label(&mut runner, "Search");
    runner.sync_and_update();
    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Heading 1").len() >= 1);
    assert!(accessible_nodes(&runner, "Paragraph").len() >= 1);
}
