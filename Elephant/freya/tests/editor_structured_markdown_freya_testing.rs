use elephant_freya::app::app_with_vault;
use freya::prelude::*;
use freya_testing::TestingRunner;
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
        let root = std::env::temp_dir().join(format!("elephant-freya-structured-note-{stamp}"));
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

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = runner
        .find_many(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
}

fn has_label(runner: &TestingRunner, label: &str) -> bool {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .is_some()
}

fn has_placeholder_label(runner: &TestingRunner) -> bool {
    runner
        .find(|node, element| {
            let label = element.accessibility().builder.label().unwrap_or_default();
            let lower = label.to_ascii_lowercase();
            (lower.contains("not rendered")
                || lower.contains("non rendu")
                || lower.contains("non rendue"))
            .then_some(node)
        })
        .is_some()
}

#[test]
fn structured_muya_blocks_render_natively_and_save_real_markdown() {
    let markdown = r#"> # Quoted heading
>
> - quoted item
>   continuation
>
>   second paragraph
>   - nested

- root item
  continuation with **bold**

  > nested quote
- sibling

[^src]: first footnote paragraph

    second footnote paragraph

    - footnote nested item

[ref]: https://example.com "Example"

A [reference][ref] plus https://bare.example and <dev@example.com>.

<div>HTML source</div>

$$
x + y
$$

```mermaid
graph TD
  A-->B
```"#;
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

    assert!(has_label(&runner, "Block quote"));
    assert!(has_label(&runner, "Unordered list"));
    assert!(has_label(&runner, "Heading 1"));
    assert!(has_label(&runner, "Footnote src"));
    assert!(has_label(&runner, "Reference ref"));
    assert!(has_label(&runner, "HTML"));
    assert!(has_label(&runner, "Math"));
    assert!(has_label(&runner, "mermaid diagram"));
    assert!(
        !has_placeholder_label(&runner),
        "production Freya tree must not expose unsupported Markdown placeholders"
    );

    let evidence = std::env::temp_dir().join("elephant-freya-note-structured.png");
    runner.render_to_file(&evidence);
    assert!(
        fs::metadata(&evidence)
            .expect("structured note screenshot")
            .len()
            > 0
    );

    click_label(&mut runner, "Save");
    runner.sync_and_update();
    let saved = fs::read_to_string(note_path).expect("Save must persist the real fixture file");

    assert!(saved.contains("> # Quoted heading"), "saved Markdown: {saved:?}");
    assert!(saved.contains("- root item"), "saved Markdown: {saved:?}");
    assert!(saved.contains("[^src]: first footnote paragraph"), "saved Markdown: {saved:?}");
    assert!(saved.contains("- footnote nested item"), "saved Markdown: {saved:?}");
    assert!(saved.contains("[ref]: https://example.com"), "saved Markdown: {saved:?}");
    assert!(saved.contains("[reference][ref]"), "saved Markdown: {saved:?}");
    assert!(saved.contains("https://bare.example"), "saved Markdown: {saved:?}");
    assert!(saved.contains("<div>HTML source</div>"), "saved Markdown: {saved:?}");
    assert!(saved.contains("$$\nx + y\n$$"), "saved Markdown: {saved:?}");
    assert!(saved.contains("```mermaid"), "saved Markdown: {saved:?}");
}
