use crate::edit::EditError;
use crate::session::SessionUpdate;

use super::{EditorResponse, ProtocolUpdate};

pub(super) fn response(result: Result<SessionUpdate, EditError>) -> EditorResponse {
  match result {
    Ok(update) => EditorResponse::Update(update.into()),
    Err(error) => EditorResponse::Error(error.into()),
  }
}

impl From<SessionUpdate> for ProtocolUpdate {
  fn from(update: SessionUpdate) -> Self {
    Self {
      revision: update.revision,
      selection: update.selection,
      patches: update.patches,
      can_undo: update.can_undo,
      can_redo: update.can_redo,
      composition_active: update.composition_active,
    }
  }
}
