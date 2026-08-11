use elephant_freya::app::app_with_vault;
use freya::prelude::*;
use freya_testing::{
    prelude::{KeyboardEventName, PlatformEvent},
    TestingNode, TestingRunner,
};
use std::{
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
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-keyboard-{stamp}"));
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
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn move_caret_to(runner: &mut TestingRunner, direction: NamedKey) {
    for _ in 0..32 {
        runner.press_key(Key::Named(direction));
    }
}

fn paragraph_texts(runner: &TestingRunner) -> Vec<String> {
    labeled_nodes(runner, "Paragraph")
        .into_iter()
        .map(|node| {
            let paragraph = Paragraph::try_downcast(node.element().as_ref())
                .expect("Paragraph accessibility target must be a real paragraph");
            paragraph
                .spans
                .iter()
                .map(|span| span.text.as_ref())
                .collect::<String>()
        })
        .collect()
}

fn open_note(runner: &mut TestingRunner) {
    click_label(runner, "Alpha");
    runner.sync_and_update();
}

fn press_modified_key(runner: &mut TestingRunner, key: Key, modifiers: Modifiers) {
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key,
        code: Code::Unidentified,
        modifiers,
    });
    runner.sync_and_update();
}

#[test]
fn editor_exposes_stable_production_accessibility_targets() {
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);

    assert_eq!(labeled_nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Editor scroll").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Paragraph").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Close note").len(), 1);
}

#[test]
fn enter_splits_a_real_muya_paragraph_through_the_editable_node() {
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    runner.press_key(Key::Named(NamedKey::Enter));

    let paragraphs = paragraph_texts(&runner);
    assert_eq!(paragraphs.len(), 2, "Enter must create two rendered blocks");
    assert_eq!(paragraphs.concat(), "alpha");
}

#[test]
fn backspace_and_delete_merge_real_muya_block_boundaries() {
    let fixture = FixtureVault::new("alpha\n\nbeta");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    assert_eq!(paragraph_texts(&runner), ["alpha", "beta"]);

    let paragraphs = labeled_nodes(&runner, "Paragraph");
    let second = paragraphs.get(1).expect("second paragraph must exist");
    runner.click_cursor(second.layout().area.center().to_f64());
    move_caret_to(&mut runner, NamedKey::ArrowLeft);
    runner.press_key(Key::Named(NamedKey::Backspace));
    assert_eq!(paragraph_texts(&runner), ["alphabeta"]);

    let fixture = FixtureVault::new("alpha\n\nbeta");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    move_caret_to(&mut runner, NamedKey::ArrowRight);
    runner.press_key(Key::Named(NamedKey::Delete));
    assert_eq!(paragraph_texts(&runner), ["alphabeta"]);
}

#[test]
fn command_shortcuts_use_muya_history_and_save_the_real_file() {
    let fixture = FixtureVault::new("alpha");
    let root = fixture.path().to_path_buf();
    let note_path = fixture.path().join("Alpha.md");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    runner.write_text("X");
    assert!(paragraph_texts(&runner).concat().contains('X'));

    press_modified_key(
        &mut runner,
        Key::Character("z".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    assert_eq!(paragraph_texts(&runner), ["alpha"]);

    press_modified_key(
        &mut runner,
        Key::Character("z".to_string()),
        Modifiers::ctrl_or_meta() | Modifiers::SHIFT,
    );
    assert!(paragraph_texts(&runner).concat().contains('X'));

    press_modified_key(
        &mut runner,
        Key::Character("s".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    assert!(fs::read_to_string(note_path)
        .expect("save shortcut must write the fixture note")
        .contains('X'));
}

#[test]
fn backspace_and_delete_remove_an_astral_character_as_one_utf16_grapheme() {
    let fixture = FixtureVault::new("A😀");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    move_caret_to(&mut runner, NamedKey::ArrowRight);
    runner.press_key(Key::Named(NamedKey::Backspace));
    assert_eq!(paragraph_texts(&runner), ["A"]);
    runner.press_key(Key::Named(NamedKey::Backspace));
    assert_eq!(paragraph_texts(&runner), [""]);

    let fixture = FixtureVault::new("😀B");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    move_caret_to(&mut runner, NamedKey::ArrowLeft);
    runner.press_key(Key::Named(NamedKey::Delete));
    assert_eq!(paragraph_texts(&runner), ["B"]);
}

#[test]
fn shift_selection_can_cross_inline_nodes_without_a_stale_muya_revision() {
    let fixture = FixtureVault::new("alpha **bold** omega");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    move_caret_to(&mut runner, NamedKey::ArrowLeft);
    for _ in 0..8 {
        press_modified_key(
            &mut runner,
            Key::Named(NamedKey::ArrowRight),
            Modifiers::SHIFT,
        );
    }

    let paragraph = require_label(&runner, "Paragraph");
    let rendered = Paragraph::try_downcast(paragraph.element().as_ref())
        .expect("selection target must remain a real editable paragraph");
    assert!(
        rendered.highlights.iter().any(|(start, end)| end > start),
        "shift+arrow must render a non-empty cross-inline selection"
    );

    runner.press_key(Key::Named(NamedKey::ArrowLeft));
    runner.write_text("!");
    assert!(
        paragraph_texts(&runner).concat().contains('!'),
        "a selection update must leave the current Muya revision usable by the next edit"
    );
}
