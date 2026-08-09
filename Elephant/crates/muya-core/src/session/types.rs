use crate::edit::{Command, GraphemeCommand, MarkCommand, ParagraphBoundaryCommand, PasteCommand};
use crate::features::{
  BlockCommand, BlockTypeCommand, CreateTable, ImageCommand, InsertHorizontalRule, InsertImage,
  ListCommand, TableCommand, TableNavigationCommand, TaskCommand,
};
use crate::history::History;
use crate::model::Document;
use crate::selection::{Selection, SelectionPoint};
use crate::view::ViewPatch;
use crate::{parse_markdown, to_markdown};

use super::helpers::ensure_editable_text;

#[derive(Clone, Debug)]
pub enum SessionCommand {
  Core(Command),
  Grapheme(GraphemeCommand),
  Mark(MarkCommand),
  Paste(PasteCommand),
  ParagraphBoundary(ParagraphBoundaryCommand),
  Block(BlockCommand),
  BlockType(BlockTypeCommand),
  HorizontalRule(InsertHorizontalRule),
  CreateTable(CreateTable),
  InsertImage(InsertImage),
  Image(ImageCommand),
  List(ListCommand),
  Table(TableCommand),
  TableNavigation(TableNavigationCommand),
  Task(TaskCommand),
  BeginComposition,
  UpdateComposition(String),
  CommitComposition,
  CancelComposition,
  Undo,
  Redo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSnapshot {
  pub markdown: String,
  pub revision: u64,
  pub selection: Selection,
  pub can_undo: bool,
  pub can_redo: bool,
  pub composition_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionUpdate {
  pub revision: u64,
  pub selection: Selection,
  pub patches: Vec<ViewPatch>,
  pub can_undo: bool,
  pub can_redo: bool,
  pub composition_active: bool,
}

#[derive(Clone, Debug)]
pub struct EditorSession {
  pub(super) document: Document,
  pub(super) selection: Selection,
  pub(super) history: History,
}

impl EditorSession {
  pub fn from_markdown(markdown: &str) -> Self {
    Self::with_history_limit(markdown, 500)
  }

  pub fn with_history_limit(markdown: &str, max_history_entries: usize) -> Self {
    let mut document = parse_markdown(markdown);
    let text = ensure_editable_text(&mut document);
    Self {
      document,
      selection: Selection::collapsed(SelectionPoint {
        node: text,
        offset_utf16: 0,
      }),
      history: History::new(max_history_entries),
    }
  }

  pub fn document(&self) -> &Document {
    &self.document
  }

  pub fn selection(&self) -> Selection {
    self.selection
  }

  pub fn snapshot(&self) -> SessionSnapshot {
    SessionSnapshot {
      markdown: to_markdown(&self.document),
      revision: self.document.revision,
      selection: self.selection,
      can_undo: self.history.can_undo(),
      can_redo: self.history.can_redo(),
      composition_active: self.history.is_group_active(),
    }
  }
}
