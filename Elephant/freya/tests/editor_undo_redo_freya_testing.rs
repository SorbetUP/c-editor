use elephant_freya::app::app_with_vault;
use freya::prelude::Paragraph;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MARKER: &str = " undo-redo-marker";

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-history-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nOriginal body\n").expect("write fixture note");
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

struct ProfileOverride {
    root: PathBuf,
    previous: Option<OsString>,
}

impl ProfileOverride {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("elephant-freya-editor-history-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create isolated Freya profile");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }
}

impl Drop for ProfileOverride {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => std::env::set_var("ELEPHANT_FREYA_PROFILE", previous),
            None => std::env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = require_label(runner, label);
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn paragraph_texts(runner: &TestingRunner) -> Vec<String> {
    accessible_nodes(runner, "Paragraph")
        .into_iter()
        .map(|node| {
            let paragraph = Paragraph::try_downcast(node.element().as_ref())
                .expect("Paragraph must remain a real Freya editable node");
            paragraph
                .spans
                .iter()
                .map(|span| span.text.as_ref())
                .collect::<String>()
        })
        .collect()
}

fn open_alpha(runner: &mut TestingRunner) {
    click_label(runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(runner, "NoteEditorHost").len(), 1);
}

fn new_runner(root: &Path) -> TestingRunner {
    let root = root.to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    runner
}

#[test]
fn editor_buttons_undo_then_redo_real_muya_content_and_reload_saved_file() {
    let _profile = ProfileOverride::new();
    let fixture = FixtureVault::new();
    let note_path = fixture.path().join("Alpha.md");
    let mut runner = new_runner(fixture.path());

    open_alpha(&mut runner);
    click_label(&mut runner, "Paragraph");
    runner.sync_and_update();
    runner.write_text(MARKER);
    runner.sync_and_update();

    assert!(
        paragraph_texts(&runner)
            .iter()
            .any(|text| text.contains(MARKER)),
        "the real Muya paragraph must contain the edit before history actions"
    );
    assert!(!accessible_nodes(&runner, "Undo").is_empty());

    click_label(&mut runner, "Undo");
    runner.sync_and_update();
    assert_eq!(paragraph_texts(&runner), vec!["Original body"]);
    assert!(!accessible_nodes(&runner, "Redo").is_empty());

    click_label(&mut runner, "Redo");
    runner.sync_and_update();
    assert!(
        paragraph_texts(&runner)
            .iter()
            .any(|text| text.contains(MARKER)),
        "Redo must restore the edited Muya content"
    );

    click_label(&mut runner, "Save");
    runner.sync_and_update();
    let saved = fs::read_to_string(&note_path).expect("Save must write Alpha.md");
    assert!(
        saved.contains(MARKER),
        "the redone Muya content must be persisted by the real Save button"
    );

    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "NoteEditorHost").is_empty());

    let mut reopened = new_runner(fixture.path());
    open_alpha(&mut reopened);
    assert!(
        paragraph_texts(&reopened)
            .iter()
            .any(|text| text.contains(MARKER)),
        "a fresh editor instance must read the saved redone content from disk"
    );
    assert_eq!(
        fs::read_to_string(&note_path).expect("read reloaded Alpha.md"),
        saved,
        "reloading the note must not alter the saved Muya content"
    );
}
