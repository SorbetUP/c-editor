use muya_core::model::InlineKind;
use muya_core::{
  EditorRequest, EditorResponse, EditorSession, NodeId, NodeKind, ProtocolCommand, Selection,
  SelectionPoint, EDITOR_PROTOCOL_VERSION,
};

fn request(command: ProtocolCommand) -> EditorRequest {
  EditorRequest {
    protocol_version: EDITOR_PROTOCOL_VERSION,
    expected_revision: 0,
    command,
  }
}

fn text_with_value(session: &EditorSession, expected: &str) -> NodeId {
  session
    .document()
    .nodes
    .values()
    .find(|node| {
      matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::Text { value }) if value == expected
      )
    })
    .unwrap()
    .id
}

fn select(session: &mut EditorSession, value: &str, start: u32, end: u32) {
  let node = text_with_value(session, value);
  let response = session.handle_request(request(ProtocolCommand::SetSelection {
    selection: Selection {
      anchor: SelectionPoint {
        node,
        offset_utf16: start,
      },
      focus: SelectionPoint {
        node,
        offset_utf16: end,
      },
    },
  }));
  assert!(matches!(response, EditorResponse::Update(_)));
}

#[test]
fn protocol_removes_a_whole_strong_mark_from_an_inner_selection() {
  let mut session = EditorSession::from_markdown("**alpha**");
  select(&mut session, "alpha", 1, 4);

  let response = session.handle_request(request(ProtocolCommand::ToggleStrong));
  assert!(matches!(response, EditorResponse::Update(_)));
  assert_eq!(session.snapshot().markdown, "alpha");
}

#[test]
fn protocol_keeps_nested_emphasis_inside_strong_as_a_muya_noop() {
  let mut session = EditorSession::from_markdown("***alpha***");
  select(&mut session, "alpha", 0, 5);

  let response = session.handle_request(request(ProtocolCommand::ToggleEmphasis));
  assert!(matches!(response, EditorResponse::Update(_)));
  assert_eq!(session.snapshot().markdown, "***alpha***");
}
