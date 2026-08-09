//! DOM-independent core for the Muya replacement.
//!
//! The crate owns the document model, Markdown syntax registry, parsing,
//! serialization, semantic transactions, feature editing, history, logical
//! selections and view patches. Structural transactions preserve stable IDs
//! while inserting, removing, restoring and moving subtrees. Browser and Tauri
//! integrations remain adapters around this crate.

pub mod edit;
pub mod features;
pub mod history;
pub mod model;
pub mod parser;
pub mod protocol;
pub mod selection;
pub mod serializer;
pub mod session;
pub mod syntax;
pub mod view;

pub use edit::{
  Command, EditError, GraphemeCommand, MarkCommand, Operation, ParagraphBoundaryCommand,
  Transaction, Utf16Range,
};
pub use features::{ListCommand, TableCommand, TableNavigationCommand};
pub use history::{History, HistoryStep};
pub use model::{DetachedSubtree, Document, Node, NodeId, NodeKind, SourceRange};
pub use parser::parse_markdown;
pub use protocol::{
  EditorRequest, EditorResponse, ProtocolCommand, ProtocolDocument, ProtocolError,
  ProtocolErrorCode, ProtocolSnapshot, ProtocolUpdate, EDITOR_PROTOCOL_VERSION,
};
pub use selection::{Selection, SelectionPoint};
pub use serializer::to_markdown;
pub use session::{EditorSession, SessionCommand, SessionSnapshot, SessionUpdate};
pub use view::ViewPatch;
