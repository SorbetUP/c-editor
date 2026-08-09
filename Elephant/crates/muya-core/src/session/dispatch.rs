use crate::edit::{
  build_code_span_delete_backward, build_code_span_insert_text, Command, EditError,
  GraphemeCommand, Transaction,
};
use crate::history::HistoryStep;
use crate::view::ViewPatch;

use super::helpers::validate_selection;
use super::{EditorSession, SessionCommand, SessionUpdate};

impl EditorSession {
  pub fn set_selection(
    &mut self,
    expected_revision: u64,
    selection: crate::selection::Selection,
  ) -> Result<SessionUpdate, EditError> {
    self.ensure_revision(expected_revision)?;
    validate_selection(&self.document, selection)?;
    self.selection = selection;
    Ok(self.update(Vec::new()))
  }

  pub fn dispatch(
    &mut self,
    expected_revision: u64,
    command: SessionCommand,
  ) -> Result<SessionUpdate, EditError> {
    self.ensure_revision(expected_revision)?;
    match command {
      SessionCommand::BeginComposition => {
        self.history.begin_group();
        Ok(self.update(Vec::new()))
      }
      SessionCommand::CommitComposition => {
        self.history.commit_group();
        Ok(self.update(Vec::new()))
      }
      SessionCommand::CancelComposition => {
        let step = self.history.cancel_group_step(&mut self.document)?;
        self.apply_history_step(step)
      }
      SessionCommand::Undo => {
        let step = self.history.undo_step(&mut self.document)?;
        self.apply_history_step(step)
      }
      SessionCommand::Redo => {
        let step = self.history.redo_step(&mut self.document)?;
        self.apply_history_step(step)
      }
      SessionCommand::UpdateComposition(inserted) => {
        let transaction =
          match build_code_span_insert_text(&self.document, self.selection, &inserted)? {
            Some(transaction) => transaction,
            None => Command::InsertText(inserted).build(&self.document, self.selection)?,
          };
        self.apply_transaction(transaction, true)
      }
      SessionCommand::Core(command) => {
        let transaction = match &command {
          Command::InsertText(inserted) => {
            match build_code_span_insert_text(&self.document, self.selection, inserted)? {
              Some(transaction) => transaction,
              None => command.build(&self.document, self.selection)?,
            }
          }
          _ => command.build(&self.document, self.selection)?,
        };
        self.apply_transaction(transaction, false)
      }
      SessionCommand::Grapheme(command) => {
        let transaction = match command {
          GraphemeCommand::DeleteBackward => {
            match build_code_span_delete_backward(&self.document, self.selection)? {
              Some(transaction) => transaction,
              None => command.build(&self.document, self.selection)?,
            }
          }
        };
        self.apply_transaction(transaction, false)
      }
      SessionCommand::Mark(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::Paste(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::ParagraphBoundary(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::Block(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::BlockType(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::HorizontalRule(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::CreateTable(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::InsertImage(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::Image(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::List(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::Table(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::TableNavigation(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
      SessionCommand::Task(command) => {
        self.apply_transaction(command.build(&self.document, self.selection)?, false)
      }
    }
  }

  fn ensure_revision(&self, expected: u64) -> Result<(), EditError> {
    let actual = self.document.revision;
    if expected != actual {
      return Err(EditError::RevisionMismatch { expected, actual });
    }
    Ok(())
  }

  fn apply_transaction(
    &mut self,
    transaction: Transaction,
    grouped: bool,
  ) -> Result<SessionUpdate, EditError> {
    if transaction.operations.is_empty() {
      self.selection = transaction.selection_after;
      return Ok(self.update(Vec::new()));
    }
    let patches = transaction.view_patches();
    self.selection = if grouped {
      self
        .history
        .apply_grouped(&mut self.document, &transaction)?
    } else {
      self.history.apply(&mut self.document, &transaction)?
    };
    Ok(self.update(patches))
  }

  fn apply_history_step(&mut self, step: Option<HistoryStep>) -> Result<SessionUpdate, EditError> {
    let Some(step) = step else {
      return Ok(self.update(Vec::new()));
    };
    self.selection = step.selection;
    Ok(self.update(step.transaction.view_patches()))
  }

  fn update(&self, patches: Vec<ViewPatch>) -> SessionUpdate {
    SessionUpdate {
      revision: self.document.revision,
      selection: self.selection,
      patches,
      can_undo: self.history.can_undo(),
      can_redo: self.history.can_redo(),
      composition_active: self.history.is_group_active(),
    }
  }
}
