use crate::model::{Document, Node, NodeId};
use crate::session::EditorSession;

use super::{ProtocolDocument, ProtocolSnapshot};

impl ProtocolDocument {
  pub fn from_document(document: &Document) -> Self {
    let mut nodes = Vec::with_capacity(document.nodes.len());
    append_preorder(document, document.root, &mut nodes);
    Self {
      root: document.root,
      nodes,
    }
  }
}

impl ProtocolSnapshot {
  pub fn from_session(session: &EditorSession) -> Self {
    let snapshot = session.snapshot();
    Self {
      markdown: snapshot.markdown,
      document: ProtocolDocument::from_document(session.document()),
      revision: snapshot.revision,
      selection: snapshot.selection,
      can_undo: snapshot.can_undo,
      can_redo: snapshot.can_redo,
      composition_active: snapshot.composition_active,
    }
  }
}

fn append_preorder(document: &Document, node_id: NodeId, output: &mut Vec<Node>) {
  let Some(node) = document.node(node_id) else {
    return;
  };
  output.push(node.clone());
  for child in &node.children {
    append_preorder(document, *child, output);
  }
}
