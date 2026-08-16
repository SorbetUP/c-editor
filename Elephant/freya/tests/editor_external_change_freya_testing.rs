//! Functional proofs for filesystem changes arriving while a note is open.
//!
//! Contracts: EXT-001 / EXT-002 in `docs/TAURI_FUNCTIONAL_REFERENCE.md`.
//! These tests use the real vault watcher and physical Markdown file. A clean
//! editor must reload the new disk revision before the next edit, while a dirty
//! editor must surface a conflict and leave the external bytes untouched.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
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
        let root = std::env::temp_dir().join(format!("elephant-freya-external-change-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nOriginal body.\n")
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
    fn autosave_disabled() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-external-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create profile override");
        fs::write(
            root.join("preferences.json"),
            serde_json::json!({"autoSave": false, "autoSaveDelay": 5000}).to_string(),
        )
        .expect("write preferences");
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

fn nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_node(runner: &mut TestingRunner, node: TestingNode) {
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing accessible label {label:?}"));
    click_node(runner, node);
}

fn click_library_card(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("missing library card {label:?}"));
    click_node(runner, node);
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();
}

fn open_alpha(runner: &mut TestingRunner) {
    click_library_card(runner, "Alpha");
    assert_eq!(nodes(runner, "NoteEditorHost").len(), 1);
    click_label(runner, "Paragraph");
}

fn wait_for_vault_watcher(runner: &mut TestingRunner) {
    // Production watcher interval is 500 ms. Polling the normal event loop is
    // the observable wait for that runtime contract, not a replacement path.
    runner.poll(Duration::from_millis(20), Duration::from_millis(700));
    runner.sync_and_update();
}

#[test]
fn clean_external_revision_is_reloaded_before_the_next_local_edit() {
    let _profile = ProfileOverride::autosave_disabled();
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_alpha(&mut runner);
    let path = fixture.path().join("Alpha.md");
    fs::write(&path, "# Alpha\n\nExternal revision sentinel.\n")
        .expect("write external revision");
    wait_for_vault_watcher(&mut runner);

    // If the clean editor did not reload, this edit + close would serialize
    // the stale Original body and destroy the external sentinel.
    click_label(&mut runner, "Paragraph");
    runner.write_text(" Local edit after reload.");
    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    let saved = fs::read_to_string(&path).expect("read note after close");
    assert!(
        saved.contains("External revision sentinel."),
        "EXT-001: the next local edit must be based on the externally reloaded document"
    );
    assert!(
        saved.contains("Local edit after reload."),
        "EXT-001: a post-reload local edit must still persist normally"
    );
    assert!(
        !saved.contains("Original body."),
        "EXT-001: stale pre-refresh content must not be resurrected"
    );
}

#[test]
fn dirty_external_revision_surfaces_conflict_without_overwriting_external_bytes() {
    let _profile = ProfileOverride::autosave_disabled();
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_alpha(&mut runner);
    runner.write_text(" Unsaved local sentinel.");

    let path = fixture.path().join("Alpha.md");
    let external = "# Alpha\n\nExternal wins on disk until conflict is resolved.\n";
    fs::write(&path, external).expect("write conflicting external revision");
    wait_for_vault_watcher(&mut runner);

    assert_eq!(
        nodes(&runner, "Library error").len(),
        1,
        "EXT-002: a dirty/editor external collision must be visible instead of silently resolved"
    );
    assert_eq!(
        fs::read_to_string(&path).expect("read conflicting disk revision"),
        external,
        "EXT-002: watcher conflict detection must not overwrite the external bytes"
    );
    assert_eq!(
        nodes(&runner, "NoteEditorHost").len(),
        1,
        "EXT-002: the dirty editor must remain mounted so the user's local edit is not discarded"
    );
}
