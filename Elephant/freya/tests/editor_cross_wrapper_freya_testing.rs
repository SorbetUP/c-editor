use elephant_freya::editor::EditorDocument;
use muya_core::{
    model::{Document, InlineKind, NodeId, NodeKind},
    Selection, SelectionPoint,
};

fn text_node(document: &Document, value: &str) -> NodeId {
    document
        .nodes
        .values()
        .find(|node| {
            matches!(
                &node.kind,
                NodeKind::Inline(InlineKind::Text { value: candidate }) if candidate == value
            )
        })
        .unwrap_or_else(|| panic!("missing text node {value:?}"))
        .id
}

#[test]
fn editor_document_replaces_cross_wrapper_selection_in_one_muya_transaction() {
    let original = "**alpha** middle ~~gamma~~";
    let mut editor = EditorDocument::from_markdown(original);
    let alpha = text_node(editor.session().document(), "alpha");
    let gamma = text_node(editor.session().document(), "gamma");
    editor
        .set_selection(Selection {
            anchor: SelectionPoint {
                node: alpha,
                offset_utf16: 2,
            },
            focus: SelectionPoint {
                node: gamma,
                offset_utf16: 3,
            },
        })
        .expect("set cross-wrapper selection");

    let update = editor
        .paste_markdown("X")
        .expect("Muya must replace a cross-wrapper selection directly");
    assert_eq!(update.markdown, "**alX**~~ma~~");
    assert_eq!(editor.serialize(), "**alX**~~ma~~");
    assert!(update.selection.is_collapsed());

    editor.undo().expect("undo cross-wrapper replacement");
    assert_eq!(editor.serialize(), original);
}

#[test]
fn deleting_cross_wrapper_selection_does_not_emit_empty_markers() {
    let mut editor = EditorDocument::from_markdown("**alpha** beta");
    let alpha = text_node(editor.session().document(), "alpha");
    let beta = text_node(editor.session().document(), " beta");
    editor
        .set_selection(Selection {
            anchor: SelectionPoint {
                node: alpha,
                offset_utf16: 0,
            },
            focus: SelectionPoint {
                node: beta,
                offset_utf16: 2,
            },
        })
        .expect("set deletion selection");

    editor
        .dispatch_text("")
        .expect("cross-wrapper deletion must be native");
    assert_eq!(editor.serialize(), "eta");
}
