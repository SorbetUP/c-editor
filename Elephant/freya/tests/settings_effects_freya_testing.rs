//! Freya Testing proof for source-owned theme and navigation preferences.
//!
//! These tests intentionally drive the visible settings controls and inspect
//! the rendered shell plus the canonical profile after rebuilding the app.
//! They must fail if a control only changes a settings card or if the shell
//! falls back to its static palette/order.

use elephant_freya::{app::app_with_vault, theme};
use freya::prelude::{Fill, Rect};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct ProfileOverride {
    root: PathBuf,
    previous: Option<OsString>,
}

impl ProfileOverride {
    fn new(seed: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-effects-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create canonical profile");
        fs::write(root.join("preferences.json"), seed).expect("seed canonical preferences");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }

    fn preferences_path(&self) -> PathBuf {
        self.root.join("preferences.json")
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

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-effects-vault-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nA fixture note.\n")
            .expect("write fixture note");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
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
    let area = require_label(runner, label).layout().area;
    runner.click_cursor((
        f64::from(area.origin.x + area.size.width / 2.),
        f64::from(area.origin.y + area.size.height / 2.),
    ));
    runner.sync_and_update();
}

fn background(runner: &TestingRunner, label: &str) -> Fill {
    Rect::try_downcast(require_label(runner, label).element().as_ref())
        .expect("shell surface must be a real rect")
        .style
        .background
}

#[test]
fn source_theme_starts_light_toggles_the_shell_and_survives_restart() {
    let profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(
        background(&runner, "AppShell.vue"),
        Fill::Color(theme::color((247, 249, 252, 255))),
        "the source default is the active light cascade, not the first dark CSS block"
    );

    click_label(&mut runner, "Settings");
    click_label(&mut runner, "Dark");
    assert_eq!(
        background(&runner, "AppShell.vue"),
        Fill::Color(theme::color((15, 20, 29, 255))),
        "the real theme control must recolor the rendered shell"
    );

    let saved: Value = serde_json::from_str(
        &fs::read_to_string(profile.preferences_path()).expect("read canonical profile"),
    )
    .expect("canonical profile remains JSON");
    assert_eq!(saved["theme"], Value::String("dark".to_owned()));

    let restart_root = fixture.root.clone();
    let (restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    assert_eq!(
        background(&restarted, "AppShell.vue"),
        Fill::Color(theme::color((15, 20, 29, 255))),
        "the canonical theme must be consumed again after app reconstruction"
    );
}

#[test]
fn source_navigation_visibility_control_changes_real_rail_and_survives_restart() {
    let profile = ProfileOverride::new(
        r#"{"iconRailOrder":["search","sidebar-toggle"],"iconRailHidden":[]}"#,
    );
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(labeled_nodes(&runner, "Search").len(), 1);
    click_label(&mut runner, "Settings");
    click_label(&mut runner, "Hide Search in navigation");
    assert!(
        labeled_nodes(&runner, "Search").is_empty(),
        "the source visibility preference must remove the real rail action"
    );

    let saved: Value = serde_json::from_str(
        &fs::read_to_string(profile.preferences_path()).expect("read canonical profile"),
    )
    .expect("canonical profile remains JSON");
    assert_eq!(
        saved["iconRailHidden"][0],
        Value::String("search".to_owned())
    );

    let restart_root = fixture.root.clone();
    let (restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    assert!(
        labeled_nodes(&restarted, "Search").is_empty(),
        "navigation visibility must survive a canonical-profile restart"
    );
}
