use elephant_freya::app::app_with_vault;
use freya_testing::{
    prelude::{MouseButton, MouseEventName, PlatformEvent},
    TestingNode, TestingRunner,
};
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
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-tags-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Alpha.md"),
            "---\ntags: [old]\n---\n# Alpha\n\nTag edit fixture.\n",
        )
        .expect("write fixture note");
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

fn right_click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let cursor = node.layout().area.center().to_f64();
    for name in [MouseEventName::MouseDown, MouseEventName::MouseUp] {
        runner.send_event(PlatformEvent::Mouse {
            name,
            cursor,
            button: Some(MouseButton::Right),
        });
        runner.sync_and_update();
    }
}

#[test]
fn note_tags_are_edited_saved_and_reloaded_through_real_controls() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(
        accessible_nodes(&runner, "#old").len(),
        1,
        "the existing note tag must be observable after opening Alpha"
    );

    assert_eq!(
        accessible_nodes(&runner, "Add tag").len(),
        1,
        "missing real Freya tag control: expected accessible `Add tag` after opening Alpha; only passive tag chips are exposed"
    );

    click_label(&mut runner, "Add tag");
    runner.write_text("new-tag");
    click_smallest_label(&mut runner, "Save");
    runner.sync_and_update();
    click_label(&mut runner, "Save");
    runner.sync_and_update();
    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    let saved = fs::read_to_string(fixture.root.join("Alpha.md")).expect("read saved note");
    assert!(
        saved.contains("tags: [\"old\", \"new-tag\"]"),
        "saved tags must use the Tauri frontmatter serializer: {saved}"
    );
    assert!(fixture.root.join("Alpha.md").is_file());
    assert!(!fixture.root.join("new-tag.md").exists());

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(accessible_nodes(&runner, "#new-tag").len(), 1);
}

#[test]
fn tag_chip_click_edits_and_context_click_deletes_the_real_frontmatter() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    click_label(&mut runner, "#old");
    click_label(&mut runner, "Tag");
    runner.press_key(freya::prelude::Key::Named(freya::prelude::NamedKey::End));
    for _ in "old".chars() {
        runner.press_key(freya::prelude::Key::Named(
            freya::prelude::NamedKey::Backspace,
        ));
    }
    runner.write_text("edited");
    click_smallest_label(&mut runner, "Save");
    runner.sync_and_update();

    let edited = fs::read_to_string(fixture.root.join("Alpha.md")).expect("read edited note");
    assert!(edited.contains("tags: [\"edited\"]"), "{edited}");
    assert_eq!(accessible_nodes(&runner, "#edited").len(), 1);

    right_click_label(&mut runner, "#edited");
    runner.sync_and_update();
    let deleted = fs::read_to_string(fixture.root.join("Alpha.md")).expect("read deleted tag");
    assert!(deleted.contains("tags: []"), "{deleted}");
    assert!(accessible_nodes(&runner, "#edited").is_empty());
}
