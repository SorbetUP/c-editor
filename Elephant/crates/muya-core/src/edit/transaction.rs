use crate::model::Document;
use crate::selection::Selection;
use crate::view::ViewPatch;

use super::{EditError, Operation};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
  pub operations: Vec<Operation>,
  pub selection_before: Selection,
  pub selection_after: Selection,
}

impl Transaction {
  pub fn apply(&self, document: &mut Document) -> Result<Self, EditError> {
    let mut candidate = document.clone();
    let mut inverse_operations = Vec::with_capacity(self.operations.len());

    for operation in &self.operations {
      inverse_operations.push(operation.apply(&mut candidate)?);
    }
    inverse_operations.reverse();
    candidate.revision = document.revision.saturating_add(1);
    *document = candidate;

    Ok(Self {
      operations: inverse_operations,
      selection_before: self.selection_after,
      selection_after: self.selection_before,
    })
  }

  pub fn view_patches(&self) -> Vec<ViewPatch> {
    self
      .operations
      .iter()
      .map(ViewPatch::from_operation)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::edit::Utf16Range;
  use crate::model::{InlineKind, NodeKind};
  use crate::selection::SelectionPoint;

  #[test]
  fn applies_atomically_and_builds_an_inverse_transaction() {
    let mut document = Document::new();
    let node = document.allocate(
      NodeKind::Inline(InlineKind::Text {
        value: "abc".to_string(),
      }),
      None,
    );
    let before = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 1,
    });
    let after = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 2,
    });
    let transaction = Transaction {
      operations: vec![Operation::ReplaceText {
        node,
        range: Utf16Range::new(1, 1),
        inserted: "X".to_string(),
      }],
      selection_before: before,
      selection_after: after,
    };

    let inverse = transaction.apply(&mut document).unwrap();
    assert_eq!(document.revision, 1);
    assert!(matches!(
      &document.node(node).unwrap().kind,
      NodeKind::Inline(InlineKind::Text { value }) if value == "aXbc"
    ));
    inverse.apply(&mut document).unwrap();
    assert_eq!(document.revision, 2);
    assert!(matches!(
      &document.node(node).unwrap().kind,
      NodeKind::Inline(InlineKind::Text { value }) if value == "abc"
    ));
  }

  #[test]
  fn exposes_ordered_logical_view_patches() {
    let point = SelectionPoint {
      node: crate::model::NodeId(4),
      offset_utf16: 0,
    };
    let transaction = Transaction {
      operations: vec![Operation::ReplaceText {
        node: point.node,
        range: Utf16Range::new(0, 0),
        inserted: "x".to_string(),
      }],
      selection_before: Selection::collapsed(point),
      selection_after: Selection::collapsed(SelectionPoint {
        node: point.node,
        offset_utf16: 1,
      }),
    };
    assert_eq!(
      transaction.view_patches(),
      vec![ViewPatch::ReplaceText {
        node: point.node,
        range: Utf16Range::new(0, 0),
        inserted: "x".to_string(),
      }]
    );
  }
}
