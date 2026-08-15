use elephant_freya::app::app_with_vault;
use freya::prelude::Rect;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    ffi::OsString,
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
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-settings-parity-vault-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Welcome.md"), "# Welcome\n\nSettings fixture.\n")
            .expect("write fixture note");
        Self { root }
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
    fn new(seed: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-settings-parity-profile-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create profile override");
        fs::write(root.join("preferences.json"), seed).expect("seed preferences");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }

    fn preferences_path(&self) -> PathBuf {
        self.root.join("preferences.json")
    }

    fn read_preferences(&self) -> Value {
        serde_json::from_str(
            &fs::read_to_string(self.preferences_path()).expect("read persisted preferences"),
        )
        .expect("persisted preferences stay valid JSON")
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

fn node_with_label(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"))
}

fn node_is_visible(node: &TestingNode) -> bool {
    let area = node.layout().visible_area();
    area.size.width > 1.
        && area.size.height > 1.
        && area.origin.x >= 0.
        && area.origin.y >= 0.
        && area.origin.x + area.size.width <= 1280.
        && area.origin.y + area.size.height <= 840.
}

fn ensure_label_visible(runner: &mut TestingRunner, label: &str) {
    if node_is_visible(&node_with_label(runner, label)) {
        return;
    }

    for _ in 0..8 {
        runner.scroll((900., 700.), (0., 500.));
        runner.sync_and_update();
        if node_is_visible(&node_with_label(runner, label)) {
            return;
        }
    }
    for _ in 0..16 {
        runner.scroll((900., 700.), (0., -500.));
        runner.sync_and_update();
        if node_is_visible(&node_with_label(runner, label)) {
            return;
        }
    }

    panic!("Freya node {label:?} exists but cannot be scrolled into view");
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    if label != "Settings" {
        ensure_label_visible(runner, label);
    }
    let node = node_with_label(runner, label);
    let area = node.layout().visible_area();
    runner.click_cursor((
        f64::from(area.origin.x + area.size.width / 2.0),
        f64::from(area.origin.y + area.size.height / 2.0),
    ));
    runner.sync_and_update();
}

fn open_settings(runner: &mut TestingRunner) {
    click_label(runner, "Settings");
    assert_eq!(accessible_nodes(runner, "ElephantNote settings").len(), 1);
}

fn runner_for_size(fixture: &FixtureVault, size: (f32, f32)) -> TestingRunner {
    let root = fixture.root.clone();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        size.into(),
        |_| (),
        1.,
    );
    runner
}

fn runner_for(fixture: &FixtureVault) -> TestingRunner {
    runner_for_size(fixture, (1280., 840.))
}

fn approximately(value: f32, expected: f32) -> bool {
    (value - expected).abs() < 0.5
}

#[test]
fn settings_panel_keeps_tauri_reference_geometry_when_space_is_available() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for_size(&fixture, (1600., 1000.));
    open_settings(&mut runner);
    let evidence_dir = std::env::temp_dir().join("freya-settings-parity");
    fs::create_dir_all(&evidence_dir).expect("create settings evidence directory");
    let evidence = evidence_dir.join("settings-panel.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(&evidence).expect("settings screenshot").len() > 0);

    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    assert!(approximately(panel.size.width, 1020.));
    assert!(approximately(panel.size.height, 780.));

    let search = node_with_label(&runner, "Search all settings")
        .layout()
        .area;
    assert!(approximately(search.size.width, 350.));
    assert!(approximately(search.size.height, 36.));

    let navigation = node_with_label(&runner, "Settings sections").layout().area;
    assert!(approximately(navigation.size.width, 196.));
}

#[test]
fn settings_panel_stays_inside_the_available_shell_viewport_on_small_windows() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for_size(&fixture, (800., 600.));
    open_settings(&mut runner);

    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    assert!(panel.size.width > 0. && panel.size.width <= 768.);
    assert!(panel.size.height > 0. && panel.size.height <= 568.);
    assert!(panel.origin.x >= 0. && panel.origin.y >= 0.);
    assert!(panel.origin.x + panel.size.width <= 800.);
    assert!(panel.origin.y + panel.size.height <= 600.);
}

#[test]
fn settings_search_is_live_and_opening_a_result_clears_search_mode() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    click_label(&mut runner, "Search all settings");
    runner.write_text("autosave");
    runner.sync_and_update();

    assert_eq!(
        accessible_nodes(&runner, "Settings search results").len(),
        1
    );
    assert_eq!(accessible_nodes(&runner, "Open setting Autosave").len(), 1);
    assert_eq!(
        accessible_nodes(&runner, "Open setting Autosave delay").len(),
        1
    );

    click_label(&mut runner, "Open setting Autosave");
    assert!(accessible_nodes(&runner, "Settings search results").is_empty());
    assert_eq!(
        accessible_nodes(&runner, "Settings section editor").len(),
        1
    );
    assert_eq!(accessible_nodes(&runner, "Enable autosave").len(), 1);
}

#[test]
fn color_mode_preserves_theme_family_and_round_trips_on_disk() {
    let profile =
        ProfileOverride::new(r#"{"theme":"nord-light","futurePreference":{"keep":true}}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    let before = Rect::try_downcast(
        node_with_label(&runner, "ElephantNote settings")
            .element()
            .as_ref(),
    )
    .expect("settings panel is a rectangle")
    .style
    .background;

    click_label(&mut runner, "Use Dark color mode");
    let dark = profile.read_preferences();
    assert_eq!(dark["theme"], Value::String("nord-dark".to_owned()));
    assert_eq!(dark["futurePreference"]["keep"], Value::Bool(true));

    let after = Rect::try_downcast(
        node_with_label(&runner, "ElephantNote settings")
            .element()
            .as_ref(),
    )
    .expect("settings panel remains a rectangle")
    .style
    .background;
    assert_ne!(
        before, after,
        "changing color mode must recolor the live panel"
    );

    click_label(&mut runner, "Use Light color mode");
    let light = profile.read_preferences();
    assert_eq!(light["theme"], Value::String("nord-light".to_owned()));

    let restart_root = fixture.root.clone();
    let (mut restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    open_settings(&mut restarted);
    assert_eq!(accessible_nodes(&restarted, "Use Dark color mode").len(), 1);
}

#[test]
fn theme_grid_collapses_and_expands_like_the_tauri_surface() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    assert_eq!(accessible_nodes(&runner, "Use Elephant theme").len(), 1);
    click_label(&mut runner, "Collapse themes");
    assert!(accessible_nodes(&runner, "Use Elephant theme").is_empty());
    assert_eq!(accessible_nodes(&runner, "Expand themes").len(), 1);

    click_label(&mut runner, "Expand themes");
    assert_eq!(accessible_nodes(&runner, "Use Elephant theme").len(), 1);
}

#[test]
fn icon_rail_visibility_uses_the_real_hidden_list_and_persists() {
    let profile = ProfileOverride::new(r#"{"theme":"light","iconRailHidden":[]}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    click_label(&mut runner, "Hide Search in navigation");
    let hidden = profile.read_preferences();
    assert_eq!(hidden["iconRailHidden"], serde_json::json!(["search"]));
    assert_eq!(
        accessible_nodes(&runner, "Show Search in navigation").len(),
        1
    );

    click_label(&mut runner, "Show Search in navigation");
    let visible = profile.read_preferences();
    assert_eq!(visible["iconRailHidden"], serde_json::json!([]));
    assert_eq!(
        accessible_nodes(&runner, "Hide Search in navigation").len(),
        1
    );
}

#[test]
fn editor_controls_persist_bounds_and_disabled_autosave_delay() {
    let profile = ProfileOverride::new(
        r#"{"theme":"light","autoSave":false,"autoSaveDelay":5000,"noteEditorMargin":44}"#,
    );
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);
    click_label(&mut runner, "Select Editor settings");

    click_label(&mut runner, "Increase Note margins");
    assert_eq!(
        profile.read_preferences()["noteEditorMargin"],
        Value::from(48)
    );
    click_label(&mut runner, "Increase Note margins");
    assert_eq!(
        profile.read_preferences()["noteEditorMargin"],
        Value::from(48),
        "upper bound must not be exceeded"
    );

    click_label(&mut runner, "Autosave delay");
    assert_eq!(
        profile.read_preferences()["autoSaveDelay"],
        Value::from(5000),
        "disabled delay control must not mutate persistence"
    );

    click_label(&mut runner, "Enable autosave");
    assert_eq!(profile.read_preferences()["autoSave"], Value::Bool(true));
    click_label(&mut runner, "Autosave delay");
    assert_eq!(
        profile.read_preferences()["autoSaveDelay"],
        Value::from(250)
    );

    click_label(&mut runner, "Decrease Note margins");
    assert_eq!(
        profile.read_preferences()["noteEditorMargin"],
        Value::from(44)
    );
}

#[test]
fn appearance_content_scrolls_to_controls_below_the_fold() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    let before = node_with_label(&runner, "Floating surfaces")
        .layout()
        .area
        .origin
        .y;
    ensure_label_visible(&mut runner, "Floating surfaces");
    let after = node_with_label(&runner, "Floating surfaces")
        .layout()
        .area
        .origin
        .y;

    assert!(
        (after - before).abs() >= 0.5,
        "Appearance must scroll so bottom controls are reachable"
    );
}
