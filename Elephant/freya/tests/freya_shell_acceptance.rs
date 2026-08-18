//! Headless Freya acceptance for the converted Vue/Tauri shell.
//!
//! Click targets are derived from semantic accessibility labels exposed by
//! rendered controls. When a label is repeated by a surrounding surface, the
//! smallest matching layout is the actionable control, matching the targeted
//! helpers used by the direct Library integration suite.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

fn fixture_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("elephant-freya-shell-{label}-{stamp}"));
    fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn center_of_label(test: &TestingRunner, wanted: &str) -> Option<(f64, f64)> {
    test.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(wanted)).then_some(node)
    })
    .into_iter()
    .min_by(|left, right| {
        left.layout()
            .area
            .size
            .area()
            .partial_cmp(&right.layout().area.size.area())
            .expect("accessible node areas must be ordered")
    })
    .map(|node| {
        let area = node.layout().area;
        (
            f64::from(area.origin.x + area.size.width / 2.0),
            f64::from(area.origin.y + area.size.height / 2.0),
        )
    })
}

fn has_label(test: &TestingRunner, wanted: &str) -> bool {
    center_of_label(test, wanted).is_some()
}

fn node_for_label(test: &TestingRunner, wanted: &str) -> TestingNode {
    test.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(wanted)).then_some(node)
    })
    .into_iter()
    .min_by(|left, right| {
        left.layout()
            .area
            .size
            .area()
            .partial_cmp(&right.layout().area.size.area())
            .expect("Create nodes must have ordered areas")
    })
    .unwrap_or_else(|| panic!("missing Freya label {wanted:?}"))
}

fn assert_create_geometry_and_menu(viewport: (f32, f32), expected: (f32, f32, f32)) {
    let root = fixture_root("create-geometry");
    let app_root = root.clone();
    let (mut test, _) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        viewport.into(),
        |_| {},
        1.0,
    );

    let create = node_for_label(&test, "Create");
    let area = create.layout().area;
    assert_eq!(area.size.width, expected.2);
    assert_eq!(area.size.height, expected.2);
    assert_eq!(area.origin.x, expected.0);
    assert_eq!(area.origin.y, expected.1);

    test.click_cursor((
        f64::from(area.origin.x + area.size.width / 2.),
        f64::from(area.origin.y + area.size.height / 2.),
    ));
    test.sync_and_update();
    assert!(has_label(&test, "Note"));
    assert!(has_label(&test, "Drawing"));
    assert!(has_label(&test, "Folder"));
    let note = node_for_label(&test, "Note").layout().area;
    assert!(
        note.max_y() <= area.origin.y,
        "Create menu items must be anchored above the FAB"
    );

    fs::remove_dir_all(root).expect("remove create geometry fixture");
}

#[test]
fn create_geometry_matches_desktop_toolbar_and_mobile_fab_contract() {
    assert_create_geometry_and_menu((1280., 840.), (1204., 764., 56.));
    assert_create_geometry_and_menu((420., 720.), (336., 636., 64.));
}

#[test]
fn converted_shell_exposes_source_labels_and_create_note_path() {
    let root = fixture_root("create-note");
    let app_root = root.clone();
    let (mut test, _) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (1280.0, 840.0).into(),
        |_| {},
        1.0,
    );

    assert!(has_label(&test, "All notes"));
    assert!(has_label(&test, "Create"));
    assert!(has_label(&test, "Show notes as list"));

    let create_center = center_of_label(&test, "Create").expect("Create action is rendered");
    test.click_cursor(create_center);
    test.sync_and_update();
    assert!(has_label(&test, "Note"));
    assert!(has_label(&test, "Drawing"));
    assert!(has_label(&test, "Folder"));

    let note_center = center_of_label(&test, "Note").expect("Note menu item is rendered");
    test.click_cursor(note_center);
    test.sync_and_update();

    let markdown_files = fs::read_dir(&root)
        .expect("read vault")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("md"))
        .count();
    assert_eq!(
        markdown_files, 1,
        "the real vault create path must create one note"
    );
    assert!(
        has_label(&test, "Untitled") || has_label(&test, "NoteEditorHost"),
        "Create → Note must also navigate to the newly created note"
    );

    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn invalid_vault_path_keeps_empty_picker_visible() {
    let missing = std::env::temp_dir().join("elephant-freya-missing-vault");
    let (test, _) = TestingRunner::new(
        move || app_with_vault(missing.clone()),
        (800.0, 600.0).into(),
        |_| {},
        1.0,
    );
    assert!(has_label(&test, "EmptyVaultPicker"));
}

#[test]
fn fixture_mount_does_not_persist_into_the_user_vault_registry() {
    let vault_root = fixture_root("registry-isolation-vault");
    let profile_root = fixture_root("registry-isolation-profile");
    let previous_profile = env::var_os("ELEPHANT_FREYA_PROFILE");
    env::set_var("ELEPHANT_FREYA_PROFILE", &profile_root);

    let app_root = vault_root.clone();
    let (test, _) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (800.0, 600.0).into(),
        |_| {},
        1.0,
    );
    let has_notes = has_label(&test, "All notes");
    let registry_persisted = profile_root.join("elephantnote.json").exists();
    drop(test);

    match previous_profile {
        Some(value) => env::set_var("ELEPHANT_FREYA_PROFILE", value),
        None => env::remove_var("ELEPHANT_FREYA_PROFILE"),
    }
    fs::remove_dir_all(vault_root).expect("remove fixture vault");
    fs::remove_dir_all(profile_root).expect("remove fixture profile");

    assert!(has_notes);
    assert!(
        !registry_persisted,
        "fixture mounts must not persist into the application registry"
    );
}

#[test]
fn narrow_window_exposes_the_mobile_navigation_shell() {
    let root = fixture_root("mobile-shell");
    let app_root = root.clone();
    let (test, _) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (420.0, 720.0).into(),
        |_| {},
        1.0,
    );

    assert!(has_label(&test, "Open navigation"));
    assert!(has_label(&test, "Search"));
    assert!(has_label(&test, "Settings"));
    fs::remove_dir_all(root).expect("remove mobile fixture");
}
