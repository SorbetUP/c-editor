use elephant_freya::app::app_with_vault;
use freya::prelude::Paragraph;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    ffi::OsString,
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
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-muya-lifecycle-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nBefore lifecycle\n")
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
        let root = std::env::temp_dir().join(format!("elephant-freya-muya-profile-{stamp}"));
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

fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = require_label(runner, label);
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn paragraph_texts(runner: &TestingRunner) -> Vec<String> {
    labeled_nodes(runner, "Paragraph")
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

#[test]
fn alpha_edit_save_close_and_reopen_round_trips_real_muya_content() {
    const MARKER: &str = " persisted through lifecycle";

    let _profile = ProfileOverride::new();
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let note_path = fixture.path().join("Alpha.md");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);

    click_label(&mut runner, "Paragraph");
    runner.sync_and_update();
    runner.write_text(MARKER);
    runner.sync_and_update();
    assert!(
        paragraph_texts(&runner)
            .iter()
            .any(|text| text.contains(MARKER)),
        "Freya input must reach the rendered Muya paragraph"
    );

    click_label(&mut runner, "Save");
    runner.sync_and_update();
    let saved = fs::read_to_string(&note_path).expect("Save must write Alpha.md");
    assert!(
        saved.contains(MARKER),
        "Save must persist the text entered through the editable control"
    );

    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    assert!(
        labeled_nodes(&runner, "NoteEditorHost").is_empty(),
        "Close note must remove the editor surface"
    );
    assert!(
        !labeled_nodes(&runner, "Alpha").is_empty(),
        "closing must return to the library containing Alpha"
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert!(
        paragraph_texts(&runner)
            .iter()
            .any(|text| text.contains(MARKER)),
        "reopening Alpha must render the content read back from disk"
    );
    assert_eq!(
        fs::read_to_string(&note_path).expect("read reopened Alpha.md"),
        saved,
        "reopening must not alter the persisted note"
    );
}
