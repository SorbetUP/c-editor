//! Extended functional navigation proof derived from the Tauri reference.
//!
//! Contracts: TFR-NAV-001 and FH-001..FH-008 in
//! `docs/TAURI_FUNCTIONAL_REFERENCE.md`.
//!
//! These tests deliberately do not sleep between logical clicks. Two different
//! logical targets must stay different interactions even when a remount places
//! them at the same pointer coordinates.

use elephant_freya::app::app_with_vault;
use freya::prelude::Rect;
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
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-history-extended-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create Projects fixture directory");
        fs::write(root.join("Root.md"), "# Root\n\nRoot fixture note.\n")
            .expect("write Root fixture note");
        fs::write(root.join("Other.md"), "# Other\n\nOther fixture note.\n")
            .expect("write Other fixture note");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nNested fixture note whose bytes must survive navigation.\n",
        )
        .expect("write Plan fixture note");
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
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    click_node(runner, node);
}

fn click_sidebar_label(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .min_x()
                .partial_cmp(&right.layout().area.min_x())
                .expect("sidebar x positions must be ordered")
        })
        .unwrap_or_else(|| panic!("no sidebar node has accessible label {label:?}"));
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
                .expect("library card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no library card has accessible label {label:?}"));
    click_node(runner, node);
}

fn opacity(runner: &TestingRunner, label: &str) -> Option<f32> {
    Rect::try_downcast(
        nodes(runner, label)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("navigation control {label:?} is missing"))
            .element()
            .as_ref(),
    )
    .expect("navigation control must be a rectangle")
    .effect
    .and_then(|effect| effect.opacity)
}

fn runner_for(fixture: &FixtureVault) -> TestingRunner {
    let root = fixture.path().to_path_buf();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    runner
}

fn enter_projects(runner: &mut TestingRunner) {
    click_sidebar_label(runner, "All notes");
    runner.sync_and_update();
    click_library_card(runner, "Projects");
    runner.sync_and_update();
    assert!(
        !nodes(runner, "Plan").is_empty(),
        "FH-001: entering Projects must expose its real child note"
    );
    assert!(
        nodes(runner, "NoteEditorHost").is_empty(),
        "FH-001: entering a folder must not mount the note editor"
    );
}

fn open_plan(runner: &mut TestingRunner) {
    click_library_card(runner, "Plan");
    runner.sync_and_update();
    assert_eq!(
        nodes(runner, "NoteEditorHost").len(),
        1,
        "FH-002: the first Plan click after the folder remount must open the editor"
    );
    assert_eq!(
        nodes(runner, "Heading 1").len(),
        1,
        "the nested fixture must be rendered as real Markdown content"
    );
}

#[test]
fn folder_back_restores_root_and_forward_restores_folder() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    enter_projects(&mut runner);

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(
        !nodes(&runner, "Root").is_empty() && !nodes(&runner, "Other").is_empty(),
        "FH-003: Back from Projects must restore the root library"
    );
    assert!(nodes(&runner, "NoteEditorHost").is_empty());
    assert_eq!(opacity(&runner, "Avancer"), Some(1.0));

    click_label(&mut runner, "Avancer");
    runner.sync_and_update();
    assert!(
        !nodes(&runner, "Plan").is_empty(),
        "FH-004: Forward must restore the exact Projects directory state"
    );
    assert!(nodes(&runner, "Root").is_empty());
    assert!(nodes(&runner, "NoteEditorHost").is_empty());
}

#[test]
fn new_navigation_after_back_discards_the_stale_forward_branch() {
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);

    enter_projects(&mut runner);
    open_plan(&mut runner);

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!nodes(&runner, "Plan").is_empty());

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!nodes(&runner, "Other").is_empty());
    assert_eq!(opacity(&runner, "Avancer"), Some(1.0));

    click_library_card(&mut runner, "Other");
    runner.sync_and_update();
    assert_eq!(nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(nodes(&runner, "Heading 1").len(), 1);
    assert_eq!(
        opacity(&runner, "Avancer"),
        Some(0.3),
        "FH-005: a new navigation after Back must invalidate stale Forward history"
    );
}

#[test]
fn closing_nested_note_preserves_bytes_and_returns_to_the_folder_context() {
    let fixture = FixtureVault::new();
    let plan_path = fixture.path().join("Projects/Plan.md");
    let before = fs::read(&plan_path).expect("read Plan before opening");
    let mut runner = runner_for(&fixture);

    enter_projects(&mut runner);
    open_plan(&mut runner);

    click_label(&mut runner, "Close note");
    runner.sync_and_update();

    assert!(
        nodes(&runner, "NoteEditorHost").is_empty(),
        "FH-007: Close note must unmount the editor"
    );
    assert!(
        !nodes(&runner, "Plan").is_empty(),
        "FH-007: closing a nested note must return to its folder context"
    );
    assert_eq!(
        fs::read(&plan_path).expect("read Plan after closing"),
        before,
        "FH-007: opening and closing without edits must preserve file bytes"
    );
}

#[test]
fn close_then_immediate_reopen_loads_the_real_persisted_note() {
    let fixture = FixtureVault::new();
    let plan_path = fixture.path().join("Projects/Plan.md");
    let expected = fs::read_to_string(&plan_path).expect("read Plan fixture");
    let mut runner = runner_for(&fixture);

    enter_projects(&mut runner);
    open_plan(&mut runner);
    click_label(&mut runner, "Close note");
    runner.sync_and_update();
    assert!(!nodes(&runner, "Plan").is_empty());

    // No delay here: a close/remount followed by activation is a new logical
    // action and may not be swallowed by global-coordinate double-click state.
    click_library_card(&mut runner, "Plan");
    runner.sync_and_update();

    assert_eq!(
        nodes(&runner, "NoteEditorHost").len(),
        1,
        "FH-008: immediate reopen must mount the real editor"
    );
    assert!(
        expected.contains("Nested fixture note whose bytes must survive navigation."),
        "fixture itself must contain the persisted sentinel"
    );
    assert_eq!(
        fs::read_to_string(&plan_path).expect("read Plan after reopen"),
        expected,
        "FH-008: reopen must not replace or truncate persisted content"
    );
}
