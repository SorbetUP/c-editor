use elephant_freya::{app::app_with_vault, vault_adapter::VaultAdapter};
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

fn assert_horizontal_bounds(runner: &TestingRunner, label: &str, container: freya::prelude::Area) {
    let area = node_with_label(runner, label).layout().area;
    assert!(
        area.size.width > 1.,
        "{label} must keep a measurable width, got {:?}",
        area.size
    );
    assert!(
        area.origin.x >= container.origin.x - 0.5,
        "{label} starts outside its container: {:?} vs {:?}",
        area,
        container
    );
    assert!(
        area.origin.x + area.size.width <= container.origin.x + container.size.width + 0.5,
        "{label} overflows its container horizontally: {:?} vs {:?}",
        area,
        container
    );
}

fn areas_overlap(left: freya::prelude::Area, right: freya::prelude::Area) -> bool {
    left.origin.x < right.origin.x + right.size.width
        && left.origin.x + left.size.width > right.origin.x
        && left.origin.y < right.origin.y + right.size.height
        && left.origin.y + left.size.height > right.origin.y
}

#[test]
fn settings_is_a_full_viewport_modal_above_the_existing_workspace() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for_size(&fixture, (1280., 840.));
    open_settings(&mut runner);

    let backdrop = node_with_label(&runner, "Settings backdrop").layout().area;
    assert!(approximately(backdrop.origin.x, 0.));
    assert!(approximately(backdrop.origin.y, 0.));
    assert!(approximately(backdrop.size.width, 1280.));
    assert!(approximately(backdrop.size.height, 840.));

    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    assert!(approximately(panel.size.width, 1020.));
    assert!(approximately(panel.size.height, 780.));
    assert!(approximately(panel.origin.x, 130.));
    assert!(approximately(panel.origin.y, 30.));

    let workspace_note = node_with_label(&runner, "Welcome").layout().area;
    assert!(
        areas_overlap(panel, workspace_note),
        "the modal must be layered over the still-mounted workspace"
    );
}

#[test]
fn language_options_fill_the_dropdown_without_overflow() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for_size(&fixture, (800., 600.));
    open_settings(&mut runner);
    click_label(&mut runner, "Expand language options");

    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    let language = node_with_label(&runner, "Language").layout().area;
    for label in [
        "Selected language: System language · en",
        "Use Français · French language",
        "Use Deutsch · German language",
        "Use Português · Portuguese language",
        "Use Polski · Polish language",
        "Use Українська · Ukrainian language",
        "Use 日本語 · Japanese language",
        "Use 简体中文 · Simplified Chinese language",
        "Use العربية · Arabic language",
    ] {
        assert_horizontal_bounds(&runner, label, panel);
        let option = node_with_label(&runner, label).layout().area;
        assert!(
            option.size.width >= language.size.width * 0.8,
            "{label} must fill the bounded language dropdown: {:?} vs {:?}",
            option,
            language
        );
    }
}

#[test]
fn theme_cards_use_responsive_columns_without_horizontal_overflow() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for_size(&fixture, (800., 600.));
    open_settings(&mut runner);

    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    for label in [
        "Use Elephant theme",
        "Use Apple theme",
        "Use Graphite theme",
        "Use Nord theme",
        "Use Solar theme",
        "Use Forest theme",
        "Use Beige theme",
        "Use Pastel theme",
        "Use Gamer Violet theme",
    ] {
        assert_horizontal_bounds(&runner, label, panel);
    }
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
fn settings_section_accepts_keyboard_activation_after_focus() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    let editor_button = node_with_label(&runner, "Select Editor settings")
        .layout()
        .visible_area();
    let editor_center = (
        f64::from(editor_button.origin.x + editor_button.size.width / 2.0),
        f64::from(editor_button.origin.y + editor_button.size.height / 2.0),
    );
    runner.press_cursor(editor_center);
    runner.sync_and_update();
    runner.press_key(freya::prelude::Key::Named(freya::prelude::NamedKey::Enter));
    runner.sync_and_update();

    assert!(accessible_nodes(&runner, "Settings section editor").len() >= 1);
    assert!(accessible_nodes(&runner, "Enable autosave").len() >= 1);
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
fn settings_search_language_category_stays_bounded_and_horizontal() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    click_label(&mut runner, "Search all settings");
    runner.write_text("language");
    runner.sync_and_update();

    assert_eq!(accessible_nodes(&runner, "Open setting Language").len(), 1);
    let panel = node_with_label(&runner, "ElephantNote settings")
        .layout()
        .area;
    let card = node_with_label(&runner, "Open setting Language")
        .layout()
        .area;
    assert_horizontal_bounds(&runner, "Open setting Language", panel);

    let category = node_with_label(&runner, "Setting category Appearance")
        .layout()
        .area;
    assert!(
        category.size.width >= 40.,
        "category must retain intrinsic width: {:?}",
        category.size
    );
    assert!(
        category.size.height <= 20.,
        "category must remain a single horizontal label: {:?}",
        category.size
    );
    assert!(category.origin.x >= card.origin.x);
    assert!(category.origin.x + category.size.width <= card.origin.x + card.size.width);
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
fn language_settings_render_select_persist_reload_and_report_invalid_values() {
    let profile = ProfileOverride::new(r#"{"theme":"light","language":"system"}"#);
    let fixture = FixtureVault::new();
    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);

    assert_eq!(accessible_nodes(&runner, "Language").len(), 1);
    click_label(&mut runner, "Expand language options");
    assert_eq!(
        accessible_nodes(&runner, "Use Français · French language").len(),
        1
    );
    assert_eq!(
        accessible_nodes(&runner, "Selected language: System language · en").len(),
        1
    );

    let french_row = accessible_nodes(&runner, "Use Français · French language")
        .into_iter()
        .find(|node| {
            let area = node.layout().visible_area();
            area.size.width > 1.
                && area.size.height > 1.
                && area.origin.x >= 0.
                && area.origin.y >= 0.
                && area.origin.x + area.size.width <= 1280.
                && area.origin.y + area.size.height <= 840.
        })
        .expect("French language row must have a visible hit area");
    let french_area = french_row.layout().visible_area();
    let french_center = (
        f64::from(french_area.origin.x + french_area.size.width / 2.),
        f64::from(french_area.origin.y + french_area.size.height / 2.),
    );
    runner.press_cursor(french_center);
    runner.release_cursor(french_center);
    let persisted = profile.read_preferences();
    assert_eq!(persisted["language"], Value::String("fr".to_owned()));
    assert_eq!(
        persisted["elephantnote:tauri:language"],
        Value::String("fr".to_owned())
    );
    assert_eq!(
        accessible_nodes(&runner, "Current language: Français · French").len(),
        1
    );

    let restart_root = fixture.root.clone();
    let (mut restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    open_settings(&mut restarted);
    assert_eq!(
        accessible_nodes(&restarted, "Current language: Français · French").len(),
        1
    );

    drop(runner);
    drop(profile);
    let invalid_profile = ProfileOverride::new(r#"{"theme":"light","language":"xx"}"#);
    let invalid_fixture = FixtureVault::new();
    let mut invalid_runner = runner_for(&invalid_fixture);
    open_settings(&mut invalid_runner);
    assert_eq!(
        accessible_nodes(&invalid_runner, "Language preference error").len(),
        1
    );
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

#[test]
fn vault_settings_reads_restores_and_persists_real_trash_entries() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let adapter = VaultAdapter::open(&fixture.root).expect("open fixture vault");
    adapter.delete("Welcome.md").expect("move note to trash");

    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);
    click_label(&mut runner, "Select Vaults settings");
    click_label(&mut runner, "Refresh vault trash");

    assert_eq!(accessible_nodes(&runner, "Expand vault trash").len(), 1);
    click_label(&mut runner, "Expand vault trash");
    assert_eq!(accessible_nodes(&runner, "Restore Welcome.md").len(), 1);

    click_label(&mut runner, "Restore Welcome.md");
    assert_eq!(accessible_nodes(&runner, "Vault trash").len(), 1);
    assert!(fixture.root.join("Welcome.md").exists());
}

#[test]
fn addons_settings_reads_and_updates_the_real_vault_registry() {
    let _profile = ProfileOverride::new(r#"{"theme":"light"}"#);
    let fixture = FixtureVault::new();
    let addons = fixture.root.join(".elephantnote/addons");
    let package = addons.join("packages/example-addon");
    fs::create_dir_all(&package).expect("create addon package");
    fs::write(
        package.join("index.js"),
        r#"
self.elephantAddon = {
  activate(api) {
    api.log.info('settings fixture activated');
    return () => api.log.info('settings fixture disposed');
  },
  deactivate(api) { api.log.info('settings fixture deactivated'); }
};
"#,
    )
    .expect("write real addon worker entry");
    fs::write(
        addons.join("registry.json"),
        serde_json::json!({
            "version": 1,
            "addons": {
                "example-addon": {
                    "manifest": {
                        "id": "example-addon",
                        "name": "Example addon",
                        "version": "1.0.0",
                        "description": "Fixture addon",
                        "runtime": {"type": "javascript-worker", "entry": "index.js"}
                    },
                    "enabled": false,
                    "packageHash": "fixture",
                    "installedAt": "2026-08-15T00:00:00Z",
                    "source": "external"
                }
            }
        })
        .to_string(),
    )
    .expect("write addon registry");

    let mut runner = runner_for(&fixture);
    open_settings(&mut runner);
    click_label(&mut runner, "Select Addons settings");
    click_label(&mut runner, "Refresh addons");
    assert_eq!(
        accessible_nodes(&runner, "Enable Example addon addon").len(),
        1
    );

    click_label(&mut runner, "Enable Example addon addon");
    let registry: Value = serde_json::from_str(
        &fs::read_to_string(addons.join("registry.json")).expect("read addon registry"),
    )
    .expect("parse addon registry");
    assert_eq!(
        registry["addons"]["example-addon"]["enabled"],
        Value::Bool(true)
    );

    click_label(&mut runner, "Uninstall Example addon addon");
    let registry: Value = serde_json::from_str(
        &fs::read_to_string(addons.join("registry.json")).expect("read updated addon registry"),
    )
    .expect("parse updated addon registry");
    assert!(registry["addons"].get("example-addon").is_none());
    assert!(!addons.join("packages/example-addon").exists());
}
