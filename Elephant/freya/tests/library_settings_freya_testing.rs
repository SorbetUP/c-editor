use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey, Rect};
use freya_testing::{TestingNode, TestingRunner};
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
        let root = std::env::temp_dir().join(format!("elephant-freya-library-settings-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Broken.md"),
            "# Broken\n\nThis file will be removed after render.\n",
        )
        .expect("write fixture note");
        fs::write(root.join("Rename.md"), "# Rename\n\nRename me.\n")
            .expect("write rename fixture note");
        fs::write(root.join("Delete.md"), "# Delete\n\nDelete me.\n")
            .expect("write delete fixture note");
        fs::write(
            root.join("MissingAction.md"),
            "# MissingAction\n\nThis file will disappear before delete.\n",
        )
        .expect("write missing action fixture note");
        Self { root }
    }

    fn path(&self) -> &PathBuf {
        &self.root
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
        let root = std::env::temp_dir().join(format!("elephant-freya-settings-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create preferences profile");
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

fn ensure_label_visible(runner: &mut TestingRunner, label: &str) {
    let is_visible = |runner: &TestingRunner| {
        accessible_nodes(runner, label)
            .into_iter()
            .next()
            .map(|node| {
                let area = node.layout().visible_area();
                area.size.width > 1. && area.size.height > 1.
            })
            .unwrap_or(false)
    };

    if is_visible(runner) {
        return;
    }
    for _ in 0..8 {
        runner.scroll((900., 700.), (0., 500.));
        runner.sync_and_update();
        if is_visible(runner) {
            return;
        }
    }
    for _ in 0..16 {
        runner.scroll((900., 700.), (0., -500.));
        runner.sync_and_update();
        if is_visible(runner) {
            return;
        }
    }
    panic!("no visible Freya node has accessible label {label:?}");
}

fn click_library_card(runner: &mut TestingRunner, label: &str) {
    let node = library_card_node(runner, label);
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_smallest_label(runner: &mut TestingRunner, label: &str) {
    let nodes = accessible_nodes(runner, label);
    let node = nodes
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
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn library_card_node(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            let left_area = left.layout().area.size.area();
            let right_area = right.layout().area.size.area();
            left_area
                .partial_cmp(&right_area)
                .expect("library card areas must be ordered")
                .then_with(|| {
                    left.layout()
                        .area
                        .min_x()
                        .partial_cmp(&right.layout().area.min_x())
                        .expect("library card coordinates must be ordered")
                })
        })
        .unwrap_or_else(|| panic!("no Freya library card has accessible label {label:?}"))
}

fn hover_library_card(runner: &mut TestingRunner, label: &str) {
    let area = library_card_node(runner, label).layout().area;
    runner.move_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
    runner.sync_and_update();
}

#[test]
fn clicking_a_note_that_disappears_surfaces_the_real_open_error() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(accessible_nodes(&runner, "Broken").len() >= 1);
    fs::remove_file(fixture.path().join("Broken.md")).expect("remove note after render");

    click_library_card(&mut runner, "Broken");
    runner.sync_and_update();

    assert!(
        accessible_nodes(&runner, "Library error").len() >= 1,
        "the real card callback must surface the failed file load"
    );
}

#[test]
fn card_actions_are_overlayed_and_rename_and_delete_use_the_real_vault() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let before = library_card_node(&runner, "Rename").layout().area;
    hover_library_card(&mut runner, "Rename");
    assert_eq!(accessible_nodes(&runner, "Note actions").len(), 1);
    click_label(&mut runner, "Note actions");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Note actions").len() >= 1);
    assert_eq!(library_card_node(&runner, "Rename").layout().area, before);

    click_smallest_label(&mut runner, "Rename");
    runner.sync_and_update();
    let input = accessible_nodes(&runner, "Rename Rename")
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("the source rename input must be keyboard accessible"));
    let input_area = input.layout().area;
    runner.click_cursor((
        ((input_area.min_x() + input_area.max_x()) / 2.) as f64,
        ((input_area.min_y() + input_area.max_y()) / 2.) as f64,
    ));
    runner.write_text("Renamed");
    assert!(
        accessible_nodes(&runner, "Renamed").len() >= 1,
        "the focused source rename input must receive typed text"
    );
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert!(fixture.path().join("Renamed.md").is_file());
    assert!(!fixture.path().join("Rename.md").exists());

    hover_library_card(&mut runner, "Delete");
    click_label(&mut runner, "Note actions");
    runner.sync_and_update();
    click_smallest_label(&mut runner, "Delete");
    runner.sync_and_update();
    assert!(!fixture.path().join("Delete.md").exists());
    assert!(accessible_nodes(&runner, "Delete").is_empty());

    fs::remove_file(fixture.path().join("MissingAction.md")).expect("remove action target");
    hover_library_card(&mut runner, "MissingAction");
    click_label(&mut runner, "Note actions");
    runner.sync_and_update();
    click_smallest_label(&mut runner, "Delete");
    runner.sync_and_update();
    assert_eq!(
        accessible_nodes(&runner, "Library error").len(),
        1,
        "a real vault delete failure must remain visible in the library surface"
    );
}

#[test]
fn settings_control_persists_canonical_preferences_and_restores_on_restart() {
    let profile = ProfileOverride::new();
    fs::write(
        profile.preferences_path(),
        r#"{"autoSave":false,"unknownLeadKey":{"keep":true}}"#,
    )
    .expect("seed canonical preferences");
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    click_label(&mut runner, "Select Editor settings");
    runner.sync_and_update();
    ensure_label_visible(&mut runner, "Enable autosave");
    let before = Rect::try_downcast(
        accessible_nodes(&runner, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("autosave control must be a real rectangle")
    .style
    .background;
    click_label(&mut runner, "Enable autosave");
    runner.sync_and_update();
    let after = Rect::try_downcast(
        accessible_nodes(&runner, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("autosave control must remain a real rectangle")
    .style
    .background;
    assert_ne!(
        before, after,
        "the visible setting control must change on click"
    );

    let saved: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(profile.preferences_path()).expect("canonical preferences are saved"),
    )
    .expect("canonical preferences remain valid JSON");
    assert_eq!(saved["autoSave"], serde_json::Value::Bool(true));
    assert_eq!(
        saved["unknownLeadKey"]["keep"],
        serde_json::Value::Bool(true)
    );
    assert!(!fixture.path().join("preferences.json").exists());

    let restart_root = fixture.path().clone();
    let (mut restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    click_label(&mut restarted, "Settings");
    restarted.sync_and_update();
    click_label(&mut restarted, "Select Editor settings");
    restarted.sync_and_update();
    ensure_label_visible(&mut restarted, "Enable autosave");
    let restored = Rect::try_downcast(
        accessible_nodes(&restarted, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("restored autosave control must be a real rectangle")
    .style
    .background;
    assert_eq!(
        restored, after,
        "restart must restore the persisted visual state"
    );
}

#[test]
fn malformed_canonical_preferences_are_visible_through_the_settings_error_surface() {
    let profile = ProfileOverride::new();
    fs::write(profile.preferences_path(), "{ not valid preferences")
        .expect("write malformed canonical preferences");
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "Settings error").len(), 1);
    assert_eq!(
        accessible_nodes(&runner, "Settings could not be loaded").len(),
        0,
        "the visible error must be represented by the surface state, not a hidden fallback"
    );
    assert_eq!(
        fs::read_to_string(profile.preferences_path()).unwrap(),
        "{ not valid preferences"
    );
}
