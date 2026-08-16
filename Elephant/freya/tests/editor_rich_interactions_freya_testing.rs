use elephant_freya::app::app_with_vault;
use freya::prelude::*;
use freya_clipboard::copypasta::ClipboardProvider;
use freya_testing::{
    prelude::{ImeEventName, KeyboardEventName, PlatformEvent},
    TestingNode, TestingRunner,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
struct MemoryClipboard(Arc<Mutex<String>>);

impl ClipboardProvider for MemoryClipboard {
    fn get_contents(&mut self) -> ClipboardResult<String> {
        self.0
            .lock()
            .map(|contents| contents.clone())
            .map_err(|_| boxed_clipboard_error("clipboard lock poisoned"))
    }

    fn set_contents(&mut self, contents: String) -> ClipboardResult<()> {
        self.0
            .lock()
            .map(|mut current| *current = contents)
            .map_err(|_| boxed_clipboard_error("clipboard lock poisoned"))
    }
}

type ClipboardResult<T> =
    std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn boxed_clipboard_error(message: &str) -> Box<dyn std::error::Error + Send + Sync + 'static> {
    Box::new(std::io::Error::other(message))
}

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new(markdown: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-editor-rich-{stamp}"));
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
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn open_note(runner: &mut TestingRunner) {
    click_label(runner, "Alpha");
    runner.sync_and_update();
}

fn send_modified_key(runner: &mut TestingRunner, key: Key, modifiers: Modifiers) {
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key,
        code: Code::Unidentified,
        modifiers,
    });
    runner.sync_and_update();
}

fn save_note(runner: &mut TestingRunner) {
    send_modified_key(
        runner,
        Key::Character("s".to_string()),
        Modifiers::ctrl_or_meta(),
    );
}

fn paragraph_texts(runner: &TestingRunner, label: &str) -> Vec<String> {
    labeled_nodes(runner, label)
        .into_iter()
        .map(|node| {
            let paragraph = Paragraph::try_downcast(node.element().as_ref())
                .expect("editable target must be a real Freya paragraph");
            paragraph
                .spans
                .iter()
                .map(|span| span.text.as_ref())
                .collect::<String>()
        })
        .collect()
}

#[test]
fn format_shortcuts_apply_real_muya_marks_and_save_markdown() {
    let fixture = FixtureVault::new("alpha beta");
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
    send_modified_key(
        &mut runner,
        Key::Character("a".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    send_modified_key(
        &mut runner,
        Key::Character("b".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    send_modified_key(
        &mut runner,
        Key::Character("s".to_string()),
        Modifiers::ctrl_or_meta(),
    );

    assert_eq!(
        fs::read_to_string(note_path).expect("format shortcut must save the real note"),
        "**alpha beta**"
    );
}

#[test]
fn task_marker_click_toggles_the_real_muya_task_and_persists() {
    let fixture = FixtureVault::new("- [ ] ship it");
    let root = fixture.path().to_path_buf();
    let note_path = fixture.path().join("Alpha.md");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Task unchecked");
    assert_eq!(paragraph_texts(&runner, "Paragraph"), vec!["ship it"]);
    click_label(&mut runner, "Close note");

    assert_eq!(
        fs::read_to_string(note_path).expect("task click must save the real note"),
        "- [x] ship it"
    );
}

#[test]
fn table_tab_moves_the_real_muya_selection_to_the_next_cell() {
    let fixture = FixtureVault::new("| A | B |\n| --- | --- |\n| C | D |");
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    open_note(&mut runner);
    let cells = labeled_nodes(&runner, "Table cell");
    assert_eq!(cells.len(), 2, "fixture must expose two body cells");
    runner.click_cursor(cells[0].layout().area.center().to_f64());
    runner.press_key(Key::Named(NamedKey::Tab));
    runner.sync_and_update();
    // Table navigation requests focus on the next native paragraph; give the
    // headless event loop one frame to apply that accessibility transition.
    runner.poll_n(Duration::from_millis(1), 2);
    runner.write_text("X");

    assert_eq!(paragraph_texts(&runner, "Table cell"), vec!["C", "XD"]);
}

#[test]
fn clipboard_paste_uses_freya_clipboard_and_preserves_muya_rich_markup() {
    let fixture = FixtureVault::new("**pasted**");
    let root = fixture.path().to_path_buf();
    let note_path = fixture.path().join("Alpha.md");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |runner| {
            let provider: Box<dyn ClipboardProvider> =
                Box::new(MemoryClipboard(Arc::new(Mutex::new(String::new()))));
            runner.provide_root_context(move || State::create(Some(provider)));
        },
        1.,
    );

    open_note(&mut runner);
    click_label(&mut runner, "Paragraph");
    send_modified_key(
        &mut runner,
        Key::Character("a".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    send_modified_key(
        &mut runner,
        Key::Character("c".to_string()),
        Modifiers::ctrl_or_meta(),
    );
    let paragraph = require_label(&runner, "Paragraph");
    let area = paragraph.layout().area;
    runner.click_cursor((
        (area.max_x() - 2.0) as f64,
        ((area.min_y() + area.max_y()) / 2.0) as f64,
    ));
    runner.sync_and_update();
    send_modified_key(
        &mut runner,
        Key::Character("v".to_string()),
        Modifiers::ctrl_or_meta(),
    );

    let paragraph = require_label(&runner, "Paragraph");
    let paragraph = Paragraph::try_downcast(paragraph.element().as_ref())
        .expect("paste target must remain a real paragraph");
    assert!(paragraph.spans.iter().any(|span| {
        span.text.contains("pastedpasted")
            && span.text_style_data.font_weight == Some(FontWeight::BOLD)
    }));
    save_note(&mut runner);
    assert_eq!(fs::read_to_string(note_path).unwrap(), "**pastedpasted**");
}

#[test]
fn ime_preedit_is_sent_through_the_real_muya_composition_group() {
    let fixture = FixtureVault::new("A");
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
    runner.send_event(PlatformEvent::ImePreedit {
        name: ImeEventName::Preedit,
        text: "e".to_string(),
        cursor: Some((1, 1)),
    });
    runner.sync_and_update();

    assert_eq!(paragraph_texts(&runner, "Paragraph"), vec!["Ae"]);

    runner.send_event(PlatformEvent::ImePreedit {
        name: ImeEventName::Preedit,
        text: "é".to_string(),
        cursor: Some((1, 1)),
    });
    runner.sync_and_update();
    assert_eq!(paragraph_texts(&runner, "Paragraph"), vec!["Aé"]);

    // freya-winit maps Ime::Commit to this real Keyboard Character event.
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("é".to_string()),
        code: Code::Unidentified,
        modifiers: Modifiers::default(),
    });
    runner.sync_and_update();
    assert_eq!(paragraph_texts(&runner, "Paragraph"), vec!["Aé"]);
    save_note(&mut runner);
    assert_eq!(fs::read_to_string(note_path).unwrap(), "Aé");

    let cancel_fixture = FixtureVault::new("A");
    let cancel_root = cancel_fixture.path().to_path_buf();
    let cancel_note_path = cancel_fixture.path().join("Alpha.md");
    let (mut cancel_runner, ()) = TestingRunner::new(
        move || app_with_vault(cancel_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    open_note(&mut cancel_runner);
    click_label(&mut cancel_runner, "Paragraph");
    cancel_runner.send_event(PlatformEvent::ImePreedit {
        name: ImeEventName::Preedit,
        text: "discard".to_string(),
        cursor: Some((7, 7)),
    });
    cancel_runner.sync_and_update();
    assert_eq!(
        paragraph_texts(&cancel_runner, "Paragraph"),
        vec!["Adiscard"]
    );
    cancel_runner.send_event(PlatformEvent::ImePreedit {
        name: ImeEventName::Preedit,
        text: String::new(),
        cursor: None,
    });
    cancel_runner.sync_and_update();
    assert_eq!(paragraph_texts(&cancel_runner, "Paragraph"), vec!["A"]);
    save_note(&mut cancel_runner);
    assert_eq!(fs::read_to_string(cancel_note_path).unwrap(), "A");
}
