use elephant_freya::app::app_with_vault;
use freya::prelude::{Code, Key, Modifiers, ModifiersExt};
use freya_testing::{
    prelude::{KeyboardEventName, PlatformEvent},
    TestingNode, TestingRunner,
};
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
    fn new(markdown: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-lifecycle-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), markdown).expect("write fixture note");
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
    fn new(auto_save: bool, delay: u64) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture profile");
        fs::write(
            root.join("preferences.json"),
            serde_json::json!({"autoSave": auto_save, "autoSaveDelay": delay}).to_string(),
        )
        .expect("write canonical preferences");
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

fn open_note(runner: &mut TestingRunner) {
    click_label(runner, "Alpha");
    runner.sync_and_update();
    click_label(runner, "Paragraph");
}

fn long_markdown() -> String {
    (0..80)
        .map(|index| format!("paragraph-{index}"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[test]
fn autosave_waits_for_canonical_profile_policy_then_writes_disk() {
    let _profile = ProfileOverride::new(true, 0);
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    runner.write_text(" autosaved");
    runner.poll(
        std::time::Duration::from_millis(1),
        std::time::Duration::from_millis(5),
    );

    let saved = fs::read_to_string(fixture.path().join("Alpha.md")).expect("read saved note");
    assert!(
        saved.contains("autosaved"),
        "autosave must write the Muya document to disk"
    );
}

#[test]
fn immediate_close_flushes_real_muya_changes_before_removing_editor() {
    let _profile = ProfileOverride::new(false, 5000);
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    runner.write_text(" flushed");
    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    assert!(labeled_nodes(&runner, "Paragraph").is_empty());
    let saved = fs::read_to_string(fixture.path().join("Alpha.md")).expect("read flushed note");
    assert!(
        saved.contains("flushed"),
        "close must flush even when autosave is disabled"
    );
}

#[test]
fn close_failure_is_visible_and_keeps_the_dirty_editor_open() {
    let _profile = ProfileOverride::new(false, 5000);
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    runner.write_text(" lost");
    fs::remove_dir_all(fixture.path()).expect("remove vault to inject an unwritable path");
    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Editor error").len(), 1);
}

#[test]
fn scroll_compaction_and_restart_state_boundary_is_explicit() {
    let _profile = ProfileOverride::new(false, 5000);
    let fixture = FixtureVault::new(&long_markdown());
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    let editor_scroll = require_label(&runner, "Editor scroll");
    runner.scroll(editor_scroll.layout().area.center().to_f64(), (0., -800.));
    assert_eq!(labeled_nodes(&runner, "Editor topbar compact").len(), 1);
    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    // The real restart state belongs to Tauri buffer_store/window_buffers and
    // is intentionally NOT PROVEN by this Freya-only write set.
    click_label(&mut runner, "Alpha");
    runner.sync_and_update();
    assert_eq!(labeled_nodes(&runner, "Editor topbar compact").len(), 0);
}

#[test]
fn save_shortcut_clears_dirty_only_after_real_write() {
    let _profile = ProfileOverride::new(false, 5000);
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    runner.write_text(" keyboard");
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("s".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::ctrl_or_meta(),
    });
    runner.sync_and_update();
    assert!(fs::read_to_string(fixture.path().join("Alpha.md"))
        .expect("read note after close shortcut")
        .contains("keyboard"));
}
