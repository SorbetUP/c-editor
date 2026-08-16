use elephant_freya::app::app_with_vault;
use freya::prelude::{AccessibilityRole, Key, NamedKey};
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
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-note-title-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nTitle edit fixture.\n")
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

fn click_title_input(runner: &mut TestingRunner) {
    let node = runner
        .find(|node, element| {
            (element.accessibility().builder.role() == AccessibilityRole::TextInput)
                .then_some(node)
        })
        .expect("the real note title input must expose a text-input role");
    runner.click_cursor(node.layout().area.center().to_f64());
}

#[test]
fn note_title_control_edits_persists_and_reopens_the_real_note() {
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
        accessible_nodes(&runner, "Note title").len(),
        1,
        "missing real Freya title control: expected editable accessibility label `Note title` after opening Alpha; observed the editor shell without a title input"
    );

    click_title_input(&mut runner);
    runner.press_key(Key::Named(NamedKey::End));
    for _ in "Alpha".chars() {
        runner.press_key(Key::Named(NamedKey::Backspace));
    }
    runner.write_text("Renamed Alpha");
    click_label(&mut runner, "Save");
    runner.sync_and_update();
    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    assert_eq!(
        fs::read_to_string(fixture.root.join("Alpha.md")).expect("read saved Alpha"),
        "# Renamed Alpha\n\nTitle edit fixture."
    );
    assert!(fixture.root.join("Alpha.md").is_file());
    assert!(!fixture.root.join("Renamed Alpha.md").exists());
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 0);

    click_label(&mut runner, "Renamed Alpha");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(accessible_nodes(&runner, "Note title").len(), 1);
}
