//! Hard regression for the historical risk of a silent 200-node graph cap.
//!
//! Contract GRAPH-003 in `docs/FUNCTIONAL_PARITY_MATRIX.md` requires a real
//! vault with more than 200 Markdown notes and explicit evidence around the
//! boundary, not a mocked graph payload.

use elephant_freya::{app::app_with_vault_view, navigation_contract::WorkspaceView};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-graph-205-{stamp}"));
        fs::create_dir_all(&root).expect("create graph fixture vault");

        for index in 0..205usize {
            let title = format!("Node{index:03}");
            let link = (index + 1 < 205).then(|| format!("[[Node{:03}]]", index + 1));
            let body = match link {
                Some(link) => format!("# {title}\n\nGraph boundary fixture {index}. {link}\n"),
                None => format!("# {title}\n\nGraph boundary fixture {index}.\n"),
            };
            fs::write(root.join(format!("{title}.md")), body)
                .unwrap_or_else(|error| panic!("write graph node {index}: {error}"));
        }

        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing accessible label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
}

#[test]
fn graph_rebuild_exposes_all_205_real_notes_and_boundary_nodes() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(root.clone(), WorkspaceView::Graph),
        (1440., 900.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Graph workspace");
    runner.sync_and_update();
    click_label(&mut runner, "Refresh graph");
    runner.sync_and_update();

    assert_eq!(
        nodes(&runner, "205 nœuds · 204 liens visibles").len(),
        1,
        "GRAPH-003: graph rebuild must report all 205 real notes and the 204 chain wikilinks"
    );
    for boundary in ["Node000", "Node199", "Node200", "Node204"] {
        assert_eq!(
            nodes(&runner, &format!("Select graph node {boundary}")).len(),
            1,
            "GRAPH-003: boundary node {boundary} must remain individually addressable"
        );
    }
    assert_eq!(
        nodes(&runner, "Graph edge Node199.md to Node200.md").len(),
        1,
        "GRAPH-003: the link crossing the 199→200 boundary must not disappear"
    );
    assert_eq!(
        nodes(&runner, "Graph edge Node203.md to Node204.md").len(),
        1,
        "GRAPH-003: data beyond 200 must include its real edges, not only node labels"
    );
}
