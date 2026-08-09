use muya_core::{EditorRequest, EditorResponse, EditorSession, ProtocolSnapshot};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MuyaEditor {
    session: EditorSession,
}

#[wasm_bindgen]
impl MuyaEditor {
    #[wasm_bindgen(constructor)]
    pub fn new(markdown: &str) -> Self {
        Self {
            session: EditorSession::from_markdown(markdown),
        }
    }

    pub fn handle_json(&mut self, request_json: &str) -> Result<String, JsValue> {
        handle_json_inner(&mut self.session, request_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        let response = muya_compatible_response(EditorResponse::Snapshot(
            ProtocolSnapshot::from_session(&self.session),
        ));
        serde_json::to_string(&response).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn revision(&self) -> u64 {
        self.session.document().revision
    }
}

fn handle_json_inner(session: &mut EditorSession, request_json: &str) -> Result<String, String> {
    let request: EditorRequest = serde_json::from_str(request_json)
        .map_err(|error| format!("invalid editor request JSON: {error}"))?;
    let response = muya_compatible_response(session.handle_request(request));
    serde_json::to_string(&response)
        .map_err(|error| format!("failed to serialize editor response: {error}"))
}

fn muya_compatible_response(mut response: EditorResponse) -> EditorResponse {
    if let EditorResponse::Snapshot(snapshot) = &mut response {
        append_muya_block_terminator(&mut snapshot.markdown);
    }
    response
}

fn append_muya_block_terminator(markdown: &mut String) {
    markdown.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use muya_core::{ProtocolCommand, ProtocolErrorCode, EDITOR_PROTOCOL_VERSION};

    fn request(revision: u64, command: ProtocolCommand) -> String {
        serde_json::to_string(&EditorRequest {
            protocol_version: EDITOR_PROTOCOL_VERSION,
            expected_revision: revision,
            command,
        })
        .unwrap()
    }

    #[test]
    fn routes_json_requests_through_the_editor_session() {
        let mut session = EditorSession::from_markdown("abc");
        let json = handle_json_inner(
            &mut session,
            &request(0, ProtocolCommand::InsertText { text: "x".into() }),
        )
        .unwrap();
        let response: EditorResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(response, EditorResponse::Update(_)));
        assert_eq!(session.snapshot().markdown, "xabc");
    }

    #[test]
    fn snapshots_include_the_logical_document_tree_and_muya_newline() {
        let session = EditorSession::from_markdown("**bold**");
        let response = muya_compatible_response(EditorResponse::Snapshot(
            ProtocolSnapshot::from_session(&session),
        ));
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["type"], "snapshot");
        assert_eq!(json["payload"]["markdown"], "**bold**\n");
        assert!(
            json["payload"]["document"]["nodes"]
                .as_array()
                .unwrap()
                .len()
                >= 4
        );
    }

    #[test]
    fn empty_snapshots_match_muyas_single_terminal_newline() {
        let session = EditorSession::from_markdown("");
        let response = muya_compatible_response(EditorResponse::Snapshot(
            ProtocolSnapshot::from_session(&session),
        ));
        let EditorResponse::Snapshot(snapshot) = response else {
            panic!("expected snapshot");
        };
        assert_eq!(snapshot.markdown, "\n");
    }

    #[test]
    fn appends_a_block_terminator_after_existing_blank_structure() {
        let mut markdown = "- alpha\n\n".to_string();
        append_muya_block_terminator(&mut markdown);
        assert_eq!(markdown, "- alpha\n\n\n");
    }

    #[test]
    fn snapshot_requests_use_the_same_muya_compatibility_boundary() {
        let mut session = EditorSession::from_markdown("abc");
        let json = handle_json_inner(&mut session, &request(0, ProtocolCommand::Snapshot)).unwrap();
        let response: EditorResponse = serde_json::from_str(&json).unwrap();
        let EditorResponse::Snapshot(snapshot) = response else {
            panic!("expected snapshot");
        };
        assert_eq!(snapshot.markdown, "abc\n");
    }

    #[test]
    fn preserves_semantic_protocol_errors_as_json_responses() {
        let mut session = EditorSession::from_markdown("abc");
        let json = handle_json_inner(&mut session, &request(4, ProtocolCommand::Undo)).unwrap();
        let response: EditorResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(
          response,
          EditorResponse::Error(error)
            if error.code == ProtocolErrorCode::RevisionMismatch
        ));
    }

    #[test]
    fn rejects_malformed_json_before_mutating_the_session() {
        let mut session = EditorSession::from_markdown("abc");
        let error = handle_json_inner(&mut session, "{not-json").unwrap_err();
        assert!(error.starts_with("invalid editor request JSON:"));
        assert_eq!(session.snapshot().markdown, "abc");
    }
}
