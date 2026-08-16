use elephant_freya::app::app_with_vault;
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
    fn with_note(name: &str, markdown: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-markdown-tags-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join(name), markdown).expect("write fixture note");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn click_smallest_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("accessible node areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
}

#[test]
fn crlf_inline_tags_normalize_for_real_freya_display_without_splitting_quoted_commas() {
    let fixture = FixtureVault::with_note(
        "Inline.md",
        "---\r\ntags: [\"### project   alpha\", \"Paris, France\"]\r\n---\r\n# Inline\r\n",
    );
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Inline");
    runner.sync_and_update();

    assert_eq!(accessible_nodes(&runner, "#project alpha").len(), 1);
    assert_eq!(accessible_nodes(&runner, "#Paris, France").len(), 1);
    assert!(accessible_nodes(&runner, "#France").is_empty());
}

#[test]
fn block_tags_normalize_and_round_trip_through_the_real_editor_writer() {
    let fixture = FixtureVault::with_note(
        "Block.md",
        "---\ntags:\n  - '### block   tag'\n  - \"Paris, France\"\ncreatedAt: '2026-08-16'\n---\n# Block\n",
    );
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Block");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "#block tag").len(), 1);
    assert_eq!(accessible_nodes(&runner, "#Paris, France").len(), 1);

    click_label(&mut runner, "Add tag");
    runner.write_text("\"### release   train\"");
    click_smallest_label(&mut runner, "Save");
    runner.sync_and_update();

    let saved = fs::read_to_string(fixture.root.join("Block.md")).expect("read saved note");
    assert!(
        saved.contains("tags: [\"block tag\", \"Paris, France\", \"release train\"]"),
        "saved tags must use normalized inline serialization: {saved}"
    );
    assert!(!saved.contains("  - '### block   tag'"), "{saved}");
    assert!(!saved.contains("  - \"Paris, France\""), "{saved}");
}
