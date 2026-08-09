use serde::{Deserialize, Serialize};

use crate::model::NodeId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SelectionPoint {
  pub node: NodeId,
  pub offset_utf16: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Selection {
  pub anchor: SelectionPoint,
  pub focus: SelectionPoint,
}

impl Selection {
  pub fn collapsed(point: SelectionPoint) -> Self {
    Self {
      anchor: point,
      focus: point,
    }
  }

  pub fn is_collapsed(self) -> bool {
    self.anchor == self.focus
  }

  pub fn caret(self) -> Option<SelectionPoint> {
    self.is_collapsed().then_some(self.focus)
  }

  pub fn ordered_same_node(self) -> Option<(NodeId, u32, u32)> {
    if self.anchor.node != self.focus.node {
      return None;
    }
    Some((
      self.anchor.node,
      self.anchor.offset_utf16.min(self.focus.offset_utf16),
      self.anchor.offset_utf16.max(self.focus.offset_utf16),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn exposes_collapsed_carets() {
    let point = SelectionPoint {
      node: NodeId(4),
      offset_utf16: 3,
    };
    let selection = Selection::collapsed(point);
    assert!(selection.is_collapsed());
    assert_eq!(selection.caret(), Some(point));
  }

  #[test]
  fn orders_forward_and_backward_same_node_selections() {
    let selection = Selection {
      anchor: SelectionPoint {
        node: NodeId(4),
        offset_utf16: 8,
      },
      focus: SelectionPoint {
        node: NodeId(4),
        offset_utf16: 2,
      },
    };
    assert_eq!(selection.ordered_same_node(), Some((NodeId(4), 2, 8)));
  }
}
