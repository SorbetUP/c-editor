use serde::{Deserialize, Serialize};

use crate::edit::{Operation, Utf16Range};
use crate::model::{BlockKind, DetachedSubtree, Node, NodeId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewPatch {
  ReplaceText {
    node: NodeId,
    range: Utf16Range,
    inserted: String,
  },
  InsertNode {
    parent: NodeId,
    index: usize,
    node: Node,
  },
  InsertSubtree {
    parent: NodeId,
    index: usize,
    subtree: DetachedSubtree,
  },
  MoveNode {
    node: NodeId,
    new_parent: NodeId,
    new_index: usize,
  },
  RemoveNode {
    node: NodeId,
  },
  SetBlockKind {
    node: NodeId,
    kind: BlockKind,
  },
}

impl ViewPatch {
  pub fn from_operation(operation: &Operation) -> Self {
    match operation {
      Operation::ReplaceText {
        node,
        range,
        inserted,
      } => Self::ReplaceText {
        node: *node,
        range: *range,
        inserted: inserted.clone(),
      },
      Operation::InsertNode {
        parent,
        index,
        node,
      } => Self::InsertNode {
        parent: *parent,
        index: *index,
        node: node.clone(),
      },
      Operation::InsertSubtree {
        parent,
        index,
        subtree,
      } => Self::InsertSubtree {
        parent: *parent,
        index: *index,
        subtree: subtree.clone(),
      },
      Operation::MoveNode {
        node,
        new_parent,
        new_index,
      } => Self::MoveNode {
        node: *node,
        new_parent: *new_parent,
        new_index: *new_index,
      },
      Operation::RemoveNode { node } => Self::RemoveNode { node: *node },
      Operation::SetBlockKind { node, kind } => Self::SetBlockKind {
        node: *node,
        kind: kind.clone(),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::BlockKind;

  #[test]
  fn mirrors_operations_without_document_access() {
    let operation = Operation::SetBlockKind {
      node: NodeId(7),
      kind: BlockKind::Heading { level: 3 },
    };
    assert_eq!(
      ViewPatch::from_operation(&operation),
      ViewPatch::SetBlockKind {
        node: NodeId(7),
        kind: BlockKind::Heading { level: 3 },
      }
    );
  }

  #[test]
  fn mirrors_subtree_moves() {
    let operation = Operation::MoveNode {
      node: NodeId(9),
      new_parent: NodeId(3),
      new_index: 2,
    };
    assert_eq!(
      ViewPatch::from_operation(&operation),
      ViewPatch::MoveNode {
        node: NodeId(9),
        new_parent: NodeId(3),
        new_index: 2,
      }
    );
  }

  #[test]
  fn serializes_stable_patch_tags() {
    let patch = ViewPatch::ReplaceText {
      node: NodeId(4),
      range: Utf16Range::new(1, 3),
      inserted: "x".into(),
    };
    let value = serde_json::to_value(patch).unwrap();
    assert_eq!(value["type"], "replace_text");
    assert_eq!(value["range"]["start"], 1);
  }
}
