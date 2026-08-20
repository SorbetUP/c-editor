use elephant_freya::app::app_with_vault;
use freya::prelude::*;
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
    fn new(markdown: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-atomic-inline-{stamp}"));
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

fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find_many(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"))
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = require_label(runner, label);
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn click_paragraph_edge(runner: &mut TestingRunner, start: bool) {
    let paragraph = require_label(runner, "Paragraph");
    let area = paragraph.layout().area;
    let x = if start {
        area.min_x() + 1.
    } else {
        area.max_x() - 1.
    };
    runner.click_cursor((x as f64, ((area.min_y() + area.max_y()) / 2.) as f64));
}

fn paragraph_text(runner: &TestingRunner) -> String {
    let node = require_label(runner, "Paragraph");
    let paragraph = Paragraph::try_downcast(node.element().as_ref())
        .expect("Paragraph accessibility target must be a real paragraph");
    paragraph
        .spans
        .iter()
        .map(|span| span.text.as_ref())
        .collect::<String>()
}

#[test]
fn atomic_muya_nodes_do_not_break_freya_text_mapping_or_save() {
    let markdown = "before \\* :grinning: <https://example.com> <kbd>Ctrl</kbd> $x+y$ ^2^ ~n~ [^src] ![alt](img.png) after";
    let fixture = FixtureVault::new(markdown);
    let root = fixture.path().to_path_buf();
    let note_path = fixture.path().join("Alpha.md");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1400., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Alpha");
    runner.sync_and_update();

    let rendered = paragraph_text(&runner);
    for forbidden in ["non rendu", "not rendered", "footnote:", "Image non"] {
        assert!(
            !rendered.contains(forbidden),
            "production paragraph leaked placeholder {forbidden:?}: {rendered:?}"
        );
    }
    assert!(rendered.contains('😀'));
    assert!(rendered.contains("https://example.com"));
    assert!(rendered.contains("<kbd>Ctrl</kbd>"));
    assert!(rendered.contains("x+y"));
    assert!(rendered.contains("src"));
    assert!(rendered.contains("alt"));

    click_paragraph_edge(&mut runner, false);
    runner.write_text("!");
    runner.sync_and_update();
    assert!(paragraph_text(&runner).ends_with("after!"));

    click_paragraph_edge(&mut runner, true);
    runner.write_text("START ");
    runner.sync_and_update();
    assert!(paragraph_text(&runner).starts_with("START before"));

    click_label(&mut runner, "Save");
    runner.sync_and_update();
    let saved = fs::read_to_string(note_path).expect("Save must write real fixture file");
    assert!(saved.starts_with("START before"), "saved Markdown: {saved:?}");
    assert!(saved.ends_with("after!"), "saved Markdown: {saved:?}");
    assert!(saved.contains(":grinning:"), "emoji shortcode must survive save: {saved:?}");
    assert!(saved.contains("$x+y$"), "inline math must survive save: {saved:?}");
    assert!(saved.contains("<kbd>Ctrl</kbd>"), "inline HTML must survive save: {saved:?}");
    assert!(saved.contains("[^src]"), "footnote reference must survive save: {saved:?}");
    assert!(saved.contains("![alt](img.png)"), "image must survive save: {saved:?}");
}