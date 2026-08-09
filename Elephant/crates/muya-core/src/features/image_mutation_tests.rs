use crate::features::ImageCommand;
use crate::model::{Document, InlineKind, NodeId, NodeKind};
use crate::to_markdown;

fn image(document: &Document) -> NodeId {
  document
    .nodes
    .values()
    .find_map(|node| match node.kind {
      NodeKind::Inline(InlineKind::Image { .. }) => Some(node.id),
      _ => None,
    })
    .unwrap()
}

#[test]
fn replaces_an_image_at_the_same_stable_node_id() {
  let mut session =
    crate::session::EditorSession::from_markdown("before ![old](old.png \"Old\") after");
  let image = image(session.document());
  let command = ImageCommand::Replace {
    image,
    source: "new image.png".into(),
    alt: "new alt".into(),
    title: Some("New title".into()),
  };
  let update = session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Image(command),
    )
    .unwrap();

  assert_eq!(
    session.snapshot().markdown,
    "before ![new alt](new%20image.png \"New title\") after"
  );
  assert_eq!(update.selection, session.selection());
  assert!(matches!(
    session.document().node(image).map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Image { alt, .. })) if alt == "new alt"
  ));

  session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Undo,
    )
    .unwrap();
  assert_eq!(
    session.snapshot().markdown,
    "before ![old](old.png \"Old\") after"
  );
  session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Redo,
    )
    .unwrap();
  assert_eq!(
    session.snapshot().markdown,
    "before ![new alt](new%20image.png \"New title\") after"
  );
}

#[test]
fn deletes_an_image_and_places_the_caret_on_preceding_text() {
  let mut session = crate::session::EditorSession::from_markdown("before ![old](old.png) after");
  let image = image(session.document());
  session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Image(ImageCommand::Delete { image }),
    )
    .unwrap();

  assert_eq!(session.snapshot().markdown, "before  after");
  let point = session.selection().focus;
  assert!(matches!(
    session.document().node(point.node).map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Text { value })) if value == "before "
  ));
  assert_eq!(point.offset_utf16, 7);
}

#[test]
fn deletes_an_image_only_document_without_creating_an_extra_paragraph() {
  let mut session = crate::session::EditorSession::from_markdown("![old](old.png)");
  assert_eq!(session.snapshot().markdown, "![old](old.png)");
  let image = image(session.document());

  session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Image(ImageCommand::Delete { image }),
    )
    .unwrap();

  assert_eq!(session.snapshot().markdown, "");
  let point = session.selection().focus;
  assert!(matches!(
    session.document().node(point.node).map(|node| &node.kind),
    Some(NodeKind::Inline(InlineKind::Text { value })) if value.is_empty()
  ));
  session
    .dispatch(
      session.document().revision,
      crate::session::SessionCommand::Undo,
    )
    .unwrap();
  assert_eq!(session.snapshot().markdown, "![old](old.png)");
}

#[test]
fn rejects_non_image_targets() {
  let session = crate::session::EditorSession::from_markdown("alpha");
  let text = session.selection().focus.node;
  let error = ImageCommand::Delete { image: text }
    .build(session.document(), session.selection())
    .unwrap_err();
  assert!(matches!(error, crate::edit::EditError::UnsupportedStructure(node) if node == text));
  assert_eq!(to_markdown(session.document()), "alpha");
}
