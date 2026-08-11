//! Headless Freya acceptance for the converted Vue/Tauri shell.
//!
//! The click target is derived from the semantic accessibility label exposed
//! by the rendered node. This keeps the test tied to the real component
//! contract while avoiding brittle guessed coordinates.

use elephant_freya::app::app_with_vault;
use freya_testing::TestingRunner;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
    test.find(|node, element| {
        (element.accessibility().builder.label() == Some(wanted)).then(|| {
            let area = node.layout().visible_area();
            (
                f64::from(area.origin.x + area.size.width / 2.0),
                f64::from(area.origin.y + area.size.height / 2.0),
            )
        })
    })
}

fn has_label(test: &TestingRunner, wanted: &str) -> bool {
    center_of_label(test, wanted).is_some()
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
