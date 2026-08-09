use crate::model::{Document, InlineMarkKind};
use crate::selection::Selection;

use super::{EditError, Transaction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkCommand {
  ToggleStrong,
  ToggleEmphasis,
  ToggleStrike,
}

impl MarkCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    if let Some(transaction) = super::mark_linked_same::build_same_mark_linked_toggle(
      document,
      selection,
      self.fragment_kind(),
    )? {
      return Ok(transaction);
    }
    if let Some(transaction) = super::mark_same_subtree::build_partial_same_top_level_toggle(
      document,
      selection,
      self.fragment_kind(),
    )? {
      return Ok(transaction);
    }
    self.compat().build(document, selection)
  }

  fn compat(self) -> super::mark_compat::MarkCommand {
    match self {
      Self::ToggleStrong => super::mark_compat::MarkCommand::ToggleStrong,
      Self::ToggleEmphasis => super::mark_compat::MarkCommand::ToggleEmphasis,
      Self::ToggleStrike => super::mark_compat::MarkCommand::ToggleStrike,
    }
  }

  fn fragment_kind(self) -> InlineMarkKind {
    match self {
      Self::ToggleStrong => InlineMarkKind::Strong,
      Self::ToggleEmphasis => InlineMarkKind::Emphasis,
      Self::ToggleStrike => InlineMarkKind::Strike,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{InlineKind, NodeId, NodeKind};
  use crate::selection::SelectionPoint;
  use crate::{parse_markdown, to_markdown};

  fn text(document: &Document, value: &str) -> NodeId {
    document
      .nodes
      .values()
      .find(|node| {
        matches!(
          &node.kind,
          NodeKind::Inline(InlineKind::Text { value: candidate }) if candidate == value
        )
      })
      .unwrap()
      .id
  }

  fn between(
    document: &Document,
    start_value: &str,
    start: u32,
    end_value: &str,
    end: u32,
  ) -> Selection {
    Selection {
      anchor: SelectionPoint {
        node: text(document, start_value),
        offset_utf16: start,
      },
      focus: SelectionPoint {
        node: text(document, end_value),
        offset_utf16: end,
      },
    }
  }

  fn create_linked_emphasis(document: &mut Document) {
    MarkCommand::ToggleEmphasis
      .build(document, between(document, "alpha ", 2, "beta", 2))
      .unwrap()
      .apply(document)
      .unwrap();
    assert_eq!(to_markdown(document), "al*pha **be*ta** gamma");
  }

  #[test]
  fn overlaps_a_linked_emphasis_group_with_strike_and_undoes() {
    let mut document = parse_markdown("alpha **beta** gamma");
    create_linked_emphasis(&mut document);

    let inverse = MarkCommand::ToggleStrike
      .build(&document, between(&document, "pha ", 1, "ta", 1))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "al*p~~ha **be*t~~a** gamma");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "al*pha **be*ta** gamma");
  }

  #[test]
  fn nests_strong_inside_the_start_fragment_and_undoes() {
    let mut document = parse_markdown("alpha **beta** gamma");
    create_linked_emphasis(&mut document);
    let start = text(&document, "pha ");
    let selection = Selection {
      anchor: SelectionPoint {
        node: start,
        offset_utf16: 0,
      },
      focus: SelectionPoint {
        node: start,
        offset_utf16: 3,
      },
    };

    let inverse = MarkCommand::ToggleStrong
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "al***pha** **be*ta** gamma");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "al*pha **be*ta** gamma");
  }
}
