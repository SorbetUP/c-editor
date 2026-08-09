use crate::edit::{
  Command, EditError, GraphemeCommand, MarkCommand, ParagraphBoundaryCommand, PasteCommand,
};
use crate::features::{
  BlockCommand, BlockTypeCommand, CreateTable, ImageCommand, InsertHorizontalRule, InsertImage,
  ListCommand, TableCommand, TableNavigationCommand, TaskCommand,
};
use crate::session::{EditorSession, SessionCommand};

use super::response::response;
use super::{
  EditorRequest, EditorResponse, ProtocolCommand, ProtocolError, ProtocolErrorCode,
  ProtocolSnapshot, EDITOR_PROTOCOL_VERSION,
};

impl EditorSession {
  pub fn handle_request(&mut self, request: EditorRequest) -> EditorResponse {
    if request.protocol_version != EDITOR_PROTOCOL_VERSION {
      return EditorResponse::Error(ProtocolError {
        code: ProtocolErrorCode::UnsupportedProtocolVersion,
        message: format!(
          "unsupported editor protocol version {}; expected {}",
          request.protocol_version, EDITOR_PROTOCOL_VERSION
        ),
      });
    }

    let revision = request.expected_revision;
    match request.command {
      ProtocolCommand::Snapshot => EditorResponse::Snapshot(ProtocolSnapshot::from_session(self)),
      ProtocolCommand::SetSelection { selection } => {
        response(self.set_selection(revision, selection))
      }
      ProtocolCommand::InsertParagraph => {
        let result = self.dispatch(revision, SessionCommand::Core(Command::InsertParagraph));
        match result {
          Err(EditError::UnsupportedStructure(_)) => response(self.dispatch(
            revision,
            SessionCommand::ParagraphBoundary(ParagraphBoundaryCommand::InsertParagraph),
          )),
          other => response(other),
        }
      }
      command => response(self.dispatch(revision, to_session_command(command))),
    }
  }
}

fn to_session_command(command: ProtocolCommand) -> SessionCommand {
  match command {
    ProtocolCommand::InsertText { text } => SessionCommand::Core(Command::InsertText(text)),
    ProtocolCommand::PasteMarkdown { markdown } => {
      SessionCommand::Paste(PasteCommand::new(markdown))
    }
    ProtocolCommand::DeleteBackward => SessionCommand::Grapheme(GraphemeCommand::DeleteBackward),
    ProtocolCommand::SetParagraph => SessionCommand::Core(Command::SetParagraph),
    ProtocolCommand::SetHeading { level } => SessionCommand::Core(Command::SetHeading(level)),
    ProtocolCommand::ToggleStrong => SessionCommand::Mark(MarkCommand::ToggleStrong),
    ProtocolCommand::ToggleEmphasis => SessionCommand::Mark(MarkCommand::ToggleEmphasis),
    ProtocolCommand::ToggleStrike => SessionCommand::Mark(MarkCommand::ToggleStrike),
    ProtocolCommand::DuplicateBlock => SessionCommand::Block(BlockCommand::Duplicate),
    ProtocolCommand::DeleteBlock => SessionCommand::Block(BlockCommand::Delete),
    ProtocolCommand::InsertParagraphAfterBlock => {
      SessionCommand::Block(BlockCommand::InsertParagraphAfter)
    }
    ProtocolCommand::ToggleBlockQuote => {
      SessionCommand::BlockType(BlockTypeCommand::ToggleBlockQuote)
    }
    ProtocolCommand::ToggleCodeBlock => {
      SessionCommand::BlockType(BlockTypeCommand::ToggleCodeBlock)
    }
    ProtocolCommand::SetListKind { kind } => {
      SessionCommand::BlockType(BlockTypeCommand::SetListKind(kind))
    }
    ProtocolCommand::InsertHorizontalRule => SessionCommand::HorizontalRule(InsertHorizontalRule),
    ProtocolCommand::CreateTable { rows, columns } => {
      SessionCommand::CreateTable(CreateTable { rows, columns })
    }
    ProtocolCommand::InsertImage { source, alt, title } => {
      SessionCommand::InsertImage(InsertImage { source, alt, title })
    }
    ProtocolCommand::ReplaceImage {
      image,
      source,
      alt,
      title,
    } => SessionCommand::Image(ImageCommand::Replace {
      image,
      source,
      alt,
      title,
    }),
    ProtocolCommand::DeleteImage { image } => SessionCommand::Image(ImageCommand::Delete { image }),
    ProtocolCommand::IndentListItem => SessionCommand::List(ListCommand::IndentItem),
    ProtocolCommand::OutdentListItem => SessionCommand::List(ListCommand::OutdentItem),
    ProtocolCommand::SetTaskChecked {
      item,
      checked,
      auto_check,
    } => SessionCommand::Task(TaskCommand::SetChecked {
      item,
      checked,
      auto_check,
    }),
    ProtocolCommand::InsertTableRowAfter => SessionCommand::Table(TableCommand::InsertRowAfter),
    ProtocolCommand::DeleteTableRow => SessionCommand::Table(TableCommand::DeleteRow),
    ProtocolCommand::InsertTableColumnAfter => {
      SessionCommand::Table(TableCommand::InsertColumnAfter)
    }
    ProtocolCommand::DeleteTableColumn => SessionCommand::Table(TableCommand::DeleteColumn),
    ProtocolCommand::NextTableCell => {
      SessionCommand::TableNavigation(TableNavigationCommand::NextCell)
    }
    ProtocolCommand::PreviousTableCell => {
      SessionCommand::TableNavigation(TableNavigationCommand::PreviousCell)
    }
    ProtocolCommand::BeginComposition => SessionCommand::BeginComposition,
    ProtocolCommand::UpdateComposition { text } => SessionCommand::UpdateComposition(text),
    ProtocolCommand::CommitComposition => SessionCommand::CommitComposition,
    ProtocolCommand::CancelComposition => SessionCommand::CancelComposition,
    ProtocolCommand::Undo => SessionCommand::Undo,
    ProtocolCommand::Redo => SessionCommand::Redo,
    ProtocolCommand::Snapshot
    | ProtocolCommand::SetSelection { .. }
    | ProtocolCommand::InsertParagraph => {
      unreachable!("handled before command conversion")
    }
  }
}
