use crate::edit::EditError;

use super::{ProtocolError, ProtocolErrorCode};

impl From<EditError> for ProtocolError {
  fn from(error: EditError) -> Self {
    let code = match error {
      EditError::NodeNotFound(_) => ProtocolErrorCode::NodeNotFound,
      EditError::NodeAlreadyExists(_) => ProtocolErrorCode::NodeAlreadyExists,
      EditError::NodeHasChildren(_) => ProtocolErrorCode::NodeHasChildren,
      EditError::NotTextNode(_) => ProtocolErrorCode::NotTextNode,
      EditError::NonCollapsedSelection => ProtocolErrorCode::NonCollapsedSelection,
      EditError::CrossNodeSelection => ProtocolErrorCode::CrossNodeSelection,
      EditError::InvalidHeadingLevel(_) => ProtocolErrorCode::InvalidHeadingLevel,
      EditError::UnsupportedStructure(_) => ProtocolErrorCode::UnsupportedStructure,
      EditError::InvalidChildIndex { .. } => ProtocolErrorCode::InvalidChildIndex,
      EditError::InvalidUtf16Boundary { .. } => ProtocolErrorCode::InvalidUtf16Boundary,
      EditError::RangeOutOfBounds { .. } => ProtocolErrorCode::RangeOutOfBounds,
      EditError::RevisionMismatch { .. } => ProtocolErrorCode::RevisionMismatch,
    };
    Self {
      code,
      message: error.to_string(),
    }
  }
}
