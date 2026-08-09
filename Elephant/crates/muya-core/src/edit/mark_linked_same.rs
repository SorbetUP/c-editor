use crate::model::{Document, InlineKind, InlineMarkKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::{EditError, Transaction};

pub(crate) fn build_same_mark_linked_toggle(
  document: &Document,
  selection: Selection,
  mark: InlineMarkKind,
) -> Result<Option<Transaction>, EditError> {
  let anchor = group_at_point(document, selection.anchor, mark)?;
  let focus = group_at_point(document, selection.focus, mark)?;

  match (mark, anchor, focus) {
    (InlineMarkKind::Strike, Some(left), Some(right)) if left == right => Ok(Some(noop(selection))),
    (InlineMarkKind::Emphasis, Some(left), Some(right)) if left == right => Ok(Some(
      super::mark_fragment_toggle::build_unwrap_group(document, selection, left)?,
    )),
    (InlineMarkKind::Emphasis, Some(group), None)
    | (InlineMarkKind::Emphasis, None, Some(group)) => {
      Ok(Some(retarget_emphasis(document, selection, group)?))
    }
    _ => Ok(None),
  }
}

fn retarget_emphasis(
  document: &Document,
  selection: Selection,
  group: u64,
) -> Result<Transaction, EditError> {
  let removal = super::mark_fragment_toggle::build_unwrap_group(document, selection, group)?;
  let mut candidate = document.clone();
  removal.apply(&mut candidate)?;
  let application =
    super::mark_dispatch::MarkCommand::ToggleEmphasis.build(&candidate, selection)?;

  let mut operations = removal.operations;
  operations.extend(application.operations);
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: application.selection_after,
  })
}

fn group_at_point(
  document: &Document,
  point: SelectionPoint,
  mark: InlineMarkKind,
) -> Result<Option<u64>, EditError> {
  let mut current = point.node;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if let NodeKind::Inline(InlineKind::MarkFragment {
      mark: candidate,
      group,
      ..
    }) = &node.kind
    {
      if *candidate == mark {
        return Ok(Some(*group));
      }
    }
    let Some(parent) = node.parent else {
      return Ok(None);
    };
    match document.node(parent).map(|node| &node.kind) {
      Some(NodeKind::Inline(_)) => current = parent,
      Some(NodeKind::Block(_)) | Some(NodeKind::Document) | None => return Ok(None),
    }
  }
}

fn noop(selection: Selection) -> Transaction {
  Transaction {
    operations: Vec::new(),
    selection_before: selection,
    selection_after: selection,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::edit::MarkCommand;
  use crate::model::{InlineKind, NodeKind};
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
  }

  #[test]
  fn removes_linked_emphasis_for_a_local_same_mark_toggle() {
    let mut document = parse_markdown("alpha **beta** gamma");
    create_linked_emphasis(&mut document);
    let local = between(&document, "pha ", 1, "pha ", 3);

    MarkCommand::ToggleEmphasis
      .build(&document, local)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "alpha **beta** gamma");
  }

  #[test]
  fn retargets_linked_emphasis_to_the_new_partial_selection_and_undoes() {
    let mut document = parse_markdown("alpha **beta** gamma");
    create_linked_emphasis(&mut document);
    let linked = to_markdown(&document);
    let selection = between(&document, "pha ", 1, "ta", 1);

    let inverse = MarkCommand::ToggleEmphasis
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "alp*ha **bet*a** gamma");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), linked);
  }

  #[test]
  fn keeps_a_local_linked_strike_toggle_as_a_noop() {
    let mut document = parse_markdown("**alpha** beta *gamma*");
    MarkCommand::ToggleStrike
      .build(&document, between(&document, "alpha", 2, "gamma", 3))
      .unwrap()
      .apply(&mut document)
      .unwrap();
    let linked = to_markdown(&document);
    let local = between(&document, "pha", 1, "pha", 3);

    let transaction = MarkCommand::ToggleStrike.build(&document, local).unwrap();
    assert!(transaction.operations.is_empty());
    transaction.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), linked);
  }
}
