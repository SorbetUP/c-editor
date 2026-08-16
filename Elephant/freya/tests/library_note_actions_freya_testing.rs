//! Freya Testing proof for the note-card rename journey.
//!
//! The existing library tests cover the individual rename mechanics. This
//! test keeps the user journey intact: open the real note menu, rename through
//! its exposed action, observe the filesystem move, return to the library,
//! and reopen the renamed note.

use elephant_freya::app::app_with_vault;
use freya::prelude::{AccessibilityRole, Key, NamedKey};
use freya_testing::{TestingNode, TestingRunner};
use std::{
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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-note-actions-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nRename fixture note.\n")
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

fn library_entry(runner: &TestingRunner, label: &str) -> TestingNode {
    labeled_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("library card areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no library entry has accessible label {label:?}"))
}

fn click_node(runner: &mut TestingRunner, node: TestingNode) {
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    click_node(runner, node);
}

fn click_card_action(runner: &mut TestingRunner, card_label: &str, action_label: &str) {
    let card_area = library_entry(runner, card_label).layout().area;
    let node = labeled_nodes(runner, action_label)
        .into_iter()
        .filter(|node| {
            let area = node.layout().area;
            let center = area.center();
            center.x >= card_area.min_x()
                && center.x <= card_area.max_x()
                && center.y >= card_area.min_y()
                && center.y <= card_area.max_y()
        })
        .next()
        .unwrap_or_else(|| {
            panic!("no {action_label:?} control is exposed inside note card {card_label:?}")
        });
    click_node(runner, node);
}

fn replace_rename_input(runner: &mut TestingRunner, current_title: &str, next_title: &str) {
    let input = runner
        .find_many(|node, element| {
            (element.accessibility().builder.role() == AccessibilityRole::TextInput).then_some(node)
        })
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("rename TextInput is not visible for {current_title:?}"));
    click_node(runner, input);
    runner.press_key(Key::Named(NamedKey::End));
    for _ in current_title.chars() {
        runner.press_key(Key::Named(NamedKey::Backspace));
    }
    runner.write_text(next_title);
}

#[test]
fn note_context_rename_moves_the_real_file_and_reopens_from_the_library() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_card_action(&mut runner, "Alpha", "Note actions");
    runner.sync_and_update();
    assert_eq!(labeled_nodes(&runner, "Rename").len(), 1);

    click_label(&mut runner, "Rename");
    runner.sync_and_update();
    replace_rename_input(&mut runner, "Alpha", "Renamed Alpha");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();

    assert!(
        fixture.root.join("Renamed Alpha.md").is_file(),
        "Rename must move the note to the new filesystem path"
    );
    assert!(
        !fixture.root.join("Alpha.md").exists(),
        "Rename must remove the old filesystem path"
    );
    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 0);
    // Tauri renames the filesystem entry only; the note's H1/frontmatter title
    // remains Alpha until the editor title control is changed explicitly.
    let renamed_card = library_entry(&runner, "Alpha");
    assert!(
        renamed_card.layout().area.size.width > 0. && renamed_card.layout().area.size.height > 0.,
        "Rename must return to the visible library with the renamed card"
    );

    click_node(&mut runner, renamed_card);
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "NoteEditorHost").len(),
        1,
        "the renamed library entry must open through the real editor path"
    );
    assert_eq!(labeled_nodes(&runner, "Close note").len(), 1);
}
