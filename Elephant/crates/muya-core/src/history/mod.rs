mod linked_coalescing;

use crate::edit::{EditError, Operation, Transaction};
use crate::model::{Document, InlineKind, InlineMarkKind, NodeKind};
use crate::selection::Selection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryStep {
  pub selection: Selection,
  pub transaction: Transaction,
}

#[derive(Clone, Debug)]
pub struct History {
  undo: Vec<Transaction>,
  redo: Vec<Transaction>,
  max_entries: usize,
  group_active: bool,
  group_inverse: Option<Transaction>,
}

impl Default for History {
  fn default() -> Self {
    Self::new(500)
  }
}

impl History {
  pub fn new(max_entries: usize) -> Self {
    Self {
      undo: Vec::new(),
      redo: Vec::new(),
      max_entries: max_entries.max(1),
      group_active: false,
      group_inverse: None,
    }
  }

  pub fn begin_group(&mut self) {
    if !self.group_active {
      self.group_active = true;
      self.group_inverse = None;
    }
  }

  pub fn is_group_active(&self) -> bool {
    self.group_active
  }

  pub fn apply_grouped(
    &mut self,
    document: &mut Document,
    transaction: &Transaction,
  ) -> Result<Selection, EditError> {
    if !self.group_active {
      self.begin_group();
    }
    let inverse = transaction.apply(document)?;
    self.group_inverse = Some(match self.group_inverse.take() {
      Some(previous) => merge_inverse_transactions(inverse, previous),
      None => inverse,
    });
    self.redo.clear();
    Ok(transaction.selection_after)
  }

  pub fn commit_group(&mut self) {
    if !self.group_active {
      return;
    }
    self.group_active = false;
    if let Some(inverse) = self.group_inverse.take() {
      self.undo.push(inverse);
      self.trim_undo();
    }
  }

  pub fn cancel_group(&mut self, document: &mut Document) -> Result<Option<Selection>, EditError> {
    Ok(self.cancel_group_step(document)?.map(|step| step.selection))
  }

  pub fn cancel_group_step(
    &mut self,
    document: &mut Document,
  ) -> Result<Option<HistoryStep>, EditError> {
    if !self.group_active {
      return Ok(None);
    }
    self.group_active = false;
    let Some(inverse) = self.group_inverse.take() else {
      return Ok(None);
    };
    let selection = inverse.selection_after;
    let applied = inverse.clone();
    inverse.apply(document)?;
    Ok(Some(HistoryStep {
      selection,
      transaction: applied,
    }))
  }

  pub fn apply(
    &mut self,
    document: &mut Document,
    transaction: &Transaction,
  ) -> Result<Selection, EditError> {
    self.commit_group();
    let invalidate = invalidates_history(document, transaction);
    let coalesce = linked_coalescing::should_coalesce(document, transaction, self.undo.last());
    let inverse = transaction.apply(document)?;
    if invalidate {
      self.undo.clear();
      self.redo.clear();
    } else {
      if coalesce {
        if let Some(previous) = self.undo.pop() {
          self
            .undo
            .push(merge_inverse_transactions(inverse, previous));
        } else {
          self.undo.push(inverse);
        }
      } else {
        self.undo.push(inverse);
      }
      self.redo.clear();
      self.trim_undo();
    }
    Ok(transaction.selection_after)
  }

  pub fn undo(&mut self, document: &mut Document) -> Result<Option<Selection>, EditError> {
    Ok(self.undo_step(document)?.map(|step| step.selection))
  }

  pub fn undo_step(&mut self, document: &mut Document) -> Result<Option<HistoryStep>, EditError> {
    self.commit_group();
    let Some(transaction) = self.undo.pop() else {
      return Ok(None);
    };
    let selection = transaction.selection_after;
    let applied = transaction.clone();
    let redo = transaction.apply(document)?;
    self.redo.push(redo);
    Ok(Some(HistoryStep {
      selection,
      transaction: applied,
    }))
  }

  pub fn redo(&mut self, document: &mut Document) -> Result<Option<Selection>, EditError> {
    Ok(self.redo_step(document)?.map(|step| step.selection))
  }

  pub fn redo_step(&mut self, document: &mut Document) -> Result<Option<HistoryStep>, EditError> {
    self.commit_group();
    let Some(transaction) = self.redo.pop() else {
      return Ok(None);
    };
    let selection = transaction.selection_after;
    let applied = transaction.clone();
    let undo = transaction.apply(document)?;
    self.undo.push(undo);
    self.trim_undo();
    Ok(Some(HistoryStep {
      selection,
      transaction: applied,
    }))
  }

  pub fn can_undo(&self) -> bool {
    self.group_inverse.is_some() || !self.undo.is_empty()
  }

  pub fn can_redo(&self) -> bool {
    !self.redo.is_empty()
  }

  fn trim_undo(&mut self) {
    if self.undo.len() > self.max_entries {
      let excess = self.undo.len() - self.max_entries;
      self.undo.drain(..excess);
    }
  }
}

fn invalidates_history(document: &Document, transaction: &Transaction) -> bool {
  let mut group = None;
  let mut removed = 0usize;
  for operation in &transaction.operations {
    let Operation::RemoveNode { node } = operation else {
      continue;
    };
    let Some(candidate) = document.node(*node) else {
      continue;
    };
    let NodeKind::Inline(InlineKind::MarkFragment {
      mark: InlineMarkKind::Emphasis,
      group: candidate_group,
      ..
    }) = &candidate.kind
    else {
      continue;
    };
    if group.is_some_and(|current| current != *candidate_group) {
      return false;
    }
    group = Some(*candidate_group);
    removed += 1;
  }
  let Some(group) = group else {
    return false;
  };
  let total = document
    .nodes
    .values()
    .filter(|node| {
      matches!(
        &node.kind,
        NodeKind::Inline(InlineKind::MarkFragment {
          mark: InlineMarkKind::Emphasis,
          group: candidate,
          ..
        }) if *candidate == group
      )
    })
    .count();
  removed >= 2 && removed == total
}

fn merge_inverse_transactions(latest: Transaction, previous: Transaction) -> Transaction {
  let mut operations = latest.operations;
  operations.extend(previous.operations);
  Transaction {
    operations,
    selection_before: latest.selection_before,
    selection_after: previous.selection_after,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::edit::{Command, MarkCommand};
  use crate::model::{InlineKind, NodeId, NodeKind};
  use crate::selection::{Selection, SelectionPoint};
  use crate::{parse_markdown, to_markdown};

  fn initial_document() -> (Document, NodeId, Selection) {
    let document = parse_markdown("x");
    let paragraph = document.children(document.root).next().unwrap();
    let node = document.children(paragraph.id).next().unwrap().id;
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 1,
    });
    (document, node, selection)
  }

  #[test]
  fn applies_undoes_and_redoes_transactions() {
    let (mut document, node, selection) = initial_document();
    let transaction = Command::InsertText("X".to_string())
      .build(&document, selection)
      .unwrap();
    let mut history = History::default();

    history.apply(&mut document, &transaction).unwrap();
    assert!(history.can_undo());
    assert_text(&document, node, "xX");

    history.undo(&mut document).unwrap();
    assert!(history.can_redo());
    assert_text(&document, node, "x");

    history.redo(&mut document).unwrap();
    assert_text(&document, node, "xX");
  }

  #[test]
  fn undo_steps_expose_the_applied_transaction() {
    let (mut document, node, selection) = initial_document();
    let transaction = Command::InsertText("X".to_string())
      .build(&document, selection)
      .unwrap();
    let mut history = History::default();
    history.apply(&mut document, &transaction).unwrap();

    let step = history.undo_step(&mut document).unwrap().unwrap();
    assert_eq!(step.selection, selection);
    assert_eq!(step.transaction.selection_after, selection);
    assert_text(&document, node, "x");
  }

  #[test]
  fn groups_composition_updates_into_one_undo_entry() {
    let (mut document, node, mut selection) = initial_document();
    let mut history = History::default();
    history.begin_group();

    for inserted in ["に", "ほ", "ん"] {
      let transaction = Command::InsertText(inserted.to_string())
        .build(&document, selection)
        .unwrap();
      selection = history.apply_grouped(&mut document, &transaction).unwrap();
    }
    history.commit_group();
    assert_eq!(to_markdown(&document), "xにほん");

    let restored = history.undo(&mut document).unwrap().unwrap();
    assert_eq!(to_markdown(&document), "x");
    assert_eq!(
      restored,
      Selection::collapsed(SelectionPoint {
        node,
        offset_utf16: 1,
      })
    );
    assert!(!history.can_undo());
  }

  #[test]
  fn cancels_an_active_composition_without_creating_history() {
    let (mut document, _, selection) = initial_document();
    let mut history = History::default();
    history.begin_group();
    let transaction = Command::InsertText("未確定".to_string())
      .build(&document, selection)
      .unwrap();
    history.apply_grouped(&mut document, &transaction).unwrap();
    assert_eq!(to_markdown(&document), "x未確定");

    let restored = history.cancel_group(&mut document).unwrap().unwrap();
    assert_eq!(to_markdown(&document), "x");
    assert_eq!(restored, selection);
    assert!(!history.can_undo());
    assert!(!history.is_group_active());
  }

  #[test]
  fn commits_a_group_before_a_normal_edit() {
    let (mut document, _, selection) = initial_document();
    let mut history = History::default();
    history.begin_group();
    let first = Command::InsertText("a".to_string())
      .build(&document, selection)
      .unwrap();
    let after_first = history.apply_grouped(&mut document, &first).unwrap();
    let second = Command::InsertText("b".to_string())
      .build(&document, after_first)
      .unwrap();
    history.apply(&mut document, &second).unwrap();
    assert_eq!(to_markdown(&document), "xab");

    history.undo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "xa");
    history.undo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "x");
  }

  #[test]
  fn undoes_and_redoes_paragraph_splits_without_changing_ids() {
    let mut document = parse_markdown("hello");
    let paragraph = document.children(document.root).next().unwrap();
    let node = document.children(paragraph.id).next().unwrap().id;
    let new_block = document.next_available_id();
    let new_text = NodeId(new_block.0 + 1);
    let selection = Selection::collapsed(SelectionPoint {
      node,
      offset_utf16: 2,
    });
    let transaction = Command::InsertParagraph
      .build(&document, selection)
      .unwrap();
    let mut history = History::default();

    history.apply(&mut document, &transaction).unwrap();
    assert_eq!(to_markdown(&document), "he\n\nllo");
    assert!(document.node(new_block).is_some());
    assert!(document.node(new_text).is_some());

    history.undo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "hello");
    assert!(document.node(new_block).is_none());
    assert!(document.node(new_text).is_none());

    history.redo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "he\n\nllo");
    assert!(document.node(new_block).is_some());
    assert!(document.node(new_text).is_some());
  }

  #[test]
  fn coalesces_an_overlapping_linked_mark_with_the_group_it_extends() {
    let mut document = parse_markdown("alpha **beta** gamma");
    let mut history = History::default();
    let emphasis_selection = between(&document, "alpha ", 2, "beta", 2);
    let emphasis = MarkCommand::ToggleEmphasis
      .build(&document, emphasis_selection)
      .unwrap();
    history.apply(&mut document, &emphasis).unwrap();
    assert_eq!(to_markdown(&document), "al*pha **be*ta** gamma");

    let strike_selection = between(&document, "pha ", 1, "ta", 1);
    let strike = MarkCommand::ToggleStrike
      .build(&document, strike_selection)
      .unwrap();
    history.apply(&mut document, &strike).unwrap();
    assert_eq!(to_markdown(&document), "al*p~~ha **be*t~~a** gamma");

    history.undo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "alpha **beta** gamma");
    assert!(!history.can_undo());

    history.redo(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "al*p~~ha **be*t~~a** gamma");
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
        node: text_node(document, start_value),
        offset_utf16: start,
      },
      focus: SelectionPoint {
        node: text_node(document, end_value),
        offset_utf16: end,
      },
    }
  }

  fn text_node(document: &Document, expected: &str) -> NodeId {
    document
      .nodes
      .values()
      .find(|node| {
        matches!(
          &node.kind,
          NodeKind::Inline(InlineKind::Text { value }) if value == expected
        )
      })
      .unwrap()
      .id
  }

  fn assert_text(document: &Document, node: NodeId, expected: &str) {
    assert!(matches!(
      &document.node(node).unwrap().kind,
      NodeKind::Inline(InlineKind::Text { value }) if value == expected
    ));
  }
}
