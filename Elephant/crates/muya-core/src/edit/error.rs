use std::fmt;

use crate::model::NodeId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditError {
  NodeNotFound(NodeId),
  NodeAlreadyExists(NodeId),
  NodeHasChildren(NodeId),
  NotTextNode(NodeId),
  NonCollapsedSelection,
  CrossNodeSelection,
  InvalidHeadingLevel(u8),
  UnsupportedStructure(NodeId),
  InvalidChildIndex { parent: NodeId, index: usize },
  InvalidUtf16Boundary { node: NodeId, offset: u32 },
  RangeOutOfBounds { node: NodeId, start: u32, end: u32 },
  RevisionMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for EditError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NodeNotFound(node) => write!(formatter, "node {node:?} was not found"),
      Self::NodeAlreadyExists(node) => write!(formatter, "node {node:?} already exists"),
      Self::NodeHasChildren(node) => write!(formatter, "node {node:?} still has children"),
      Self::NotTextNode(node) => write!(formatter, "node {node:?} is not an editable text node"),
      Self::NonCollapsedSelection => {
        write!(formatter, "the command requires a collapsed selection")
      }
      Self::CrossNodeSelection => write!(
        formatter,
        "cross-node text selections are not supported yet"
      ),
      Self::InvalidHeadingLevel(level) => {
        write!(
          formatter,
          "heading level {level} is outside the supported range 1..=6"
        )
      }
      Self::UnsupportedStructure(node) => {
        write!(
          formatter,
          "node {node:?} is not in a supported editable structure"
        )
      }
      Self::InvalidChildIndex { parent, index } => {
        write!(
          formatter,
          "child index {index} is invalid for parent {parent:?}"
        )
      }
      Self::InvalidUtf16Boundary { node, offset } => {
        write!(
          formatter,
          "offset {offset} is not a UTF-16 boundary in node {node:?}"
        )
      }
      Self::RangeOutOfBounds { node, start, end } => {
        write!(formatter, "range {start}..{end} is outside node {node:?}")
      }
      Self::RevisionMismatch { expected, actual } => {
        write!(
          formatter,
          "document revision mismatch: expected {expected}, actual {actual}"
        )
      }
    }
  }
}

impl std::error::Error for EditError {}
