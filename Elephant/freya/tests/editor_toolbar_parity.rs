use elephant_freya::editor::{EditorAction, EditorDocument};
use muya_core::{
    model::{InlineKind, ListKind, NodeKind},
    Selection, SelectionPoint,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureNote {
    root: PathBuf,
    path: PathBuf,
}

impl FixtureNote {
    fn new(markdown: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-toolbar-{stamp}"));
        fs::create_dir_all(&root).expect("create editor toolbar fixture");
        let path = root.join("Toolbar.md");
        fs::write(&path, markdown).expect("write editor toolbar fixture");
        Self { root, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FixtureNote {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn first_editable_inline(editor: &EditorDocument) -> (muya_core::NodeId, u32) {
    let document = editor.session().document();
    let block = document
        .children(document.root)
        .next()
        .expect("fixture must contain a block");
    let inline = document
        .children(block.id)
        .next()
        .expect("fixture must contain editable inline text");
    let value = match &inline.kind {
        NodeKind::Inline(InlineKind::Text { value }) => value,
        NodeKind::Inline(InlineKind::CodeSpan { code }) => code,
        kind => panic!("unexpected toolbar inline kind: {kind:?}"),
    };
    (inline.id, value.encode_utf16().count() as u32)
}

fn select_all(editor: &mut EditorDocument) {
    let (node, end) = first_editable_inline(editor);
    editor
        .set_selection(Selection {
            anchor: SelectionPoint {
                node,
                offset_utf16: 0,
            },
            focus: SelectionPoint {
                node,
                offset_utf16: end,
            },
        })
        .expect("select fixture text");
}

#[test]
fn heading_and_list_toolbar_actions_write_real_markdown_to_disk() {
    let heading_fixture = FixtureNote::new("alpha");
    let mut heading = EditorDocument::load(heading_fixture.path()).expect("load heading fixture");
    heading
        .dispatch(EditorAction::SetHeading(2))
        .expect("apply heading through Muya");
    heading.save().expect("save heading fixture");
    assert_eq!(
        fs::read_to_string(heading_fixture.path()).expect("read heading fixture"),
        "## alpha"
    );

    for (kind, expected) in [
        (ListKind::Unordered, "- alpha"),
        (ListKind::Ordered, "1. alpha"),
        (ListKind::Task, "- [ ] alpha"),
    ] {
        let fixture = FixtureNote::new("alpha");
        let mut editor = EditorDocument::load(fixture.path()).expect("load list fixture");
        editor
            .dispatch(EditorAction::SetListKind(kind))
            .expect("apply list through Muya");
        editor.save().expect("save list fixture");
        assert_eq!(
            fs::read_to_string(fixture.path()).expect("read list fixture"),
            expected
        );
    }
}

#[test]
fn inline_code_toolbar_action_stays_editable_and_persists() {
    let fixture = FixtureNote::new("alpha");
    let mut editor = EditorDocument::load(fixture.path()).expect("load inline-code fixture");
    select_all(&mut editor);
    editor
        .apply_inline_code()
        .expect("apply real inline-code transaction");
    assert_eq!(editor.serialize(), "`alpha`");

    let (code_node, end) = first_editable_inline(&editor);
    editor
        .set_selection(Selection::collapsed(SelectionPoint {
            node: code_node,
            offset_utf16: end,
        }))
        .expect("move caret into generated code span");
    editor
        .dispatch_text("!")
        .expect("generated code span must remain editable");
    editor.save().expect("save inline-code fixture");

    assert_eq!(
        fs::read_to_string(fixture.path()).expect("read inline-code fixture"),
        "`alpha!`"
    );
}

#[test]
fn link_toolbar_action_uses_the_real_selection_and_persists() {
    let fixture = FixtureNote::new("alpha");
    let mut editor = EditorDocument::load(fixture.path()).expect("load link fixture");
    select_all(&mut editor);
    editor
        .link_selection("https://example.com/docs")
        .expect("apply real link transaction");
    editor.save().expect("save link fixture");

    assert_eq!(
        fs::read_to_string(fixture.path()).expect("read link fixture"),
        "[alpha](https://example.com/docs)"
    );
}
