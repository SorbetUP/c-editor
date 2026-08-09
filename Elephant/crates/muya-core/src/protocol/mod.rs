mod dispatch;
mod document;
mod error;
mod response;
mod types;

pub use types::{
  EditorRequest, EditorResponse, ProtocolCommand, ProtocolDocument, ProtocolError,
  ProtocolErrorCode, ProtocolSnapshot, ProtocolUpdate,
};

pub const EDITOR_PROTOCOL_VERSION: u16 = 1;

#[cfg(test)]
mod tests;
