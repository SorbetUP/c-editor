use elephant_freya::{app::app_with_vault_view, navigation_contract::WorkspaceView};
use elephantnote_knowledge_core::{KnowledgeStore, WikiCitation, WikiDraft, WikiDraftStatus};
use freya_testing::{TestingNode, TestingRunner};
use std::{fs, path::PathBuf};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("elephant-freya-wiki-view-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create Wiki fixture");
        let store = KnowledgeStore::open(&root).expect("open knowledge store");
        store
            .save_wiki_draft(&WikiDraft {
                id: "wiki-iroh".to_owned(),
                topic: "Peer-to-peer technology".to_owned(),
                title: "Iroh".to_owned(),
                slug: "iroh".to_owned(),
                markdown: "# Iroh\n\nA cited Wiki draft.".to_owned(),
                citations: vec![WikiCitation {
                    key: "cite-1".to_owned(),
                    document_path: "Notes/Iroh.md".to_owned(),
                    document_title: "Iroh".to_owned(),
                    chunk_id: "chunk-1".to_owned(),
                    heading: "Iroh".to_owned(),
                    start_offset: 0,
                    end_offset: 10,
                }],
                source_paths: vec!["Notes/Iroh.md".to_owned()],
                source_hash: "fixture".to_owned(),
                model_id: "fixture-model".to_owned(),
                status: WikiDraftStatus::Proposed,
                created_at: 1,
                updated_at: 1,
            })
            .expect("persist Wiki draft");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing Freya label {label:?}"))
}

fn click(runner: &mut TestingRunner, label: &str) {
    let area = node(runner, label).layout().area;
    runner.click_cursor((
        f64::from((area.min_x() + area.max_x()) / 2.),
        f64::from((area.min_y() + area.max_y()) / 2.),
    ));
}

#[test]
fn wiki_route_reads_and_accepts_the_real_persisted_draft() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(root.clone(), WorkspaceView::Wiki),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(node(&runner, "Wiki workspace").layout().area.size.width > 0.);
    assert!(node(&runner, "Wiki draft Iroh").layout().area.size.width > 0.);
    click(&mut runner, "Accept Wiki Iroh");
    runner.sync_and_update();

    assert!(fixture.root.join(".elephantnote/wiki/iroh.md").is_file());
    assert!(node(&runner, "Open Wiki Iroh").layout().area.size.width > 0.);
}

#[test]
fn wiki_route_dismisses_a_proposed_draft_in_the_real_store() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(root.clone(), WorkspaceView::Wiki),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(node(&runner, "Dismiss Wiki Iroh").layout().area.size.width > 0.);
    click(&mut runner, "Dismiss Wiki Iroh");
    runner.sync_and_update();

    let store = KnowledgeStore::open(&fixture.root).expect("reopen Wiki store");
    assert_eq!(
        store
            .wiki_draft("wiki-iroh")
            .expect("read dismissed Wiki draft")
            .expect("dismissed Wiki draft")
            .status,
        WikiDraftStatus::Rejected
    );
    assert!(runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some("Dismiss Wiki Iroh"))
                .then_some(node)
        })
        .is_none());
}
