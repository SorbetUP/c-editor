//! Freya-facing adapter for Elephant's native Markdown editor.
//!
//! `NoteEditorHost.vue` owns the note identity and persistence policy while
//! `runtimeEditor.vue` owns the editing surface. This module keeps that same
//! boundary: the Freya view owns rendering/input events, and this adapter owns
//! the real Muya document session, revision checks, history, and Markdown
//! serialization. It deliberately contains no display-only text fallback.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use muya_core::{
    Command, EditError, EditorSession as MuyaEditorSession, Selection, SessionCommand,
    SessionSnapshot, SessionUpdate, ViewPatch,
};

/// Actions emitted by a Freya editor surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorAction {
    InsertText(String),
    Undo,
    Redo,
}

/// Errors at the file/session boundary of the editor adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorError {
    Io { path: PathBuf, message: String },
    MissingPath,
    Edit(EditError),
}

impl fmt::Display for EditorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(
                formatter,
                "unable to read or write {}: {message}",
                path.display()
            ),
            Self::MissingPath => formatter.write_str("the editor document has no file path"),
            Self::Edit(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EditorError {}

impl From<EditError> for EditorError {
    fn from(error: EditError) -> Self {
        Self::Edit(error)
    }
}

/// The complete state a Freya view needs after a session operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorUpdate {
    pub markdown: String,
    pub revision: u64,
    pub selection: Selection,
    pub patches: Vec<ViewPatch>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub composition_active: bool,
}

impl EditorUpdate {
    fn from_session(markdown: String, update: SessionUpdate) -> Self {
        Self {
            markdown,
            revision: update.revision,
            selection: update.selection,
            patches: update.patches,
            can_undo: update.can_undo,
            can_redo: update.can_redo,
            composition_active: update.composition_active,
        }
    }

    fn from_snapshot(snapshot: SessionSnapshot) -> Self {
        Self {
            markdown: snapshot.markdown,
            revision: snapshot.revision,
            selection: snapshot.selection,
            patches: Vec::new(),
            can_undo: snapshot.can_undo,
            can_redo: snapshot.can_redo,
            composition_active: snapshot.composition_active,
        }
    }
}

/// Revisioned Muya session exposed to Freya event handlers.
#[derive(Clone, Debug)]
pub struct EditorSession {
    inner: MuyaEditorSession,
}

impl EditorSession {
    pub fn from_markdown(markdown: &str) -> Self {
        Self {
            inner: MuyaEditorSession::from_markdown(markdown),
        }
    }

    pub fn snapshot(&self) -> EditorUpdate {
        EditorUpdate::from_snapshot(self.inner.snapshot())
    }

    pub fn markdown(&self) -> String {
        self.inner.snapshot().markdown
    }

    pub fn document(&self) -> &muya_core::Document {
        self.inner.document()
    }

    pub fn revision(&self) -> u64 {
        self.inner.snapshot().revision
    }

    pub fn set_selection(
        &mut self,
        expected_revision: u64,
        selection: Selection,
    ) -> Result<EditorUpdate, EditorError> {
        let update = self.inner.set_selection(expected_revision, selection)?;
        Ok(EditorUpdate::from_session(self.markdown(), update))
    }

    /// Dispatches an action against an explicit revision, matching the
    /// `expected_revision` guard used by the existing Rust/Web editor bridge.
    pub fn dispatch(
        &mut self,
        expected_revision: u64,
        action: EditorAction,
    ) -> Result<EditorUpdate, EditorError> {
        let command = match action {
            EditorAction::InsertText(text) => SessionCommand::Core(Command::InsertText(text)),
            EditorAction::Undo => SessionCommand::Undo,
            EditorAction::Redo => SessionCommand::Redo,
        };
        let update = self.inner.dispatch(expected_revision, command)?;
        Ok(EditorUpdate::from_session(self.markdown(), update))
    }

    pub fn dispatch_current(&mut self, action: EditorAction) -> Result<EditorUpdate, EditorError> {
        self.dispatch(self.revision(), action)
    }

    pub fn dispatch_text(&mut self, text: impl Into<String>) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::InsertText(text.into()))
    }

    pub fn undo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::Undo)
    }

    pub fn redo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::Redo)
    }
}

/// A note document loaded from the vault and backed by a real Muya session.
#[derive(Clone, Debug)]
pub struct EditorDocument {
    path: Option<PathBuf>,
    session: EditorSession,
}

impl EditorDocument {
    pub fn from_markdown(markdown: &str) -> Self {
        Self {
            path: None,
            session: EditorSession::from_markdown(markdown),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, EditorError> {
        let path = path.as_ref().to_path_buf();
        let markdown = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        Ok(Self {
            path: Some(path),
            session: EditorSession::from_markdown(&markdown),
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn session(&self) -> &EditorSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut EditorSession {
        &mut self.session
    }

    pub fn snapshot(&self) -> EditorUpdate {
        self.session.snapshot()
    }

    /// Serializes the current Muya document; this is the Markdown payload used
    /// by NoteEditorHost's save path, not a cached copy of the input file.
    pub fn serialize(&self) -> String {
        self.session.markdown()
    }

    pub fn dispatch(&mut self, action: EditorAction) -> Result<EditorUpdate, EditorError> {
        self.session.dispatch_current(action)
    }

    pub fn dispatch_text(&mut self, text: impl Into<String>) -> Result<EditorUpdate, EditorError> {
        self.session.dispatch_text(text)
    }

    pub fn set_selection(&mut self, selection: Selection) -> Result<EditorUpdate, EditorError> {
        let revision = self.session.revision();
        self.session.set_selection(revision, selection)
    }

    pub fn undo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.session.undo()
    }

    pub fn redo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.session.redo()
    }

    pub fn save(&self) -> Result<(), EditorError> {
        let path = self.path.as_ref().ok_or(EditorError::MissingPath)?;
        fs::write(path, self.serialize()).map_err(|error| io_error(path, error))
    }
}

fn io_error(path: &Path, error: io::Error) -> EditorError {
    EditorError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must be after the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("elephant-freya-editor-{label}-{nonce}.md"))
    }

    #[test]
    fn dispatches_text_undo_redo_and_serializes_the_real_document() {
        let mut document = EditorDocument::from_markdown("alpha");

        let inserted = document
            .dispatch_text("X")
            .expect("text dispatch must succeed");
        assert_eq!(inserted.markdown, "Xalpha");
        assert!(inserted.can_undo);

        let undone = document.undo().expect("undo must succeed");
        assert_eq!(undone.markdown, "alpha");
        assert!(undone.can_redo);

        let redone = document.redo().expect("redo must succeed");
        assert_eq!(redone.markdown, "Xalpha");
        assert_eq!(document.serialize(), "Xalpha");
    }

    #[test]
    fn loads_and_saves_markdown_through_the_document_path() {
        let path = test_path("load-save");
        fs::write(&path, "# Title\n\nBody").expect("fixture must be writable");

        let mut document = EditorDocument::load(&path).expect("Markdown must load from disk");
        assert_eq!(document.path(), Some(path.as_path()));
        assert_eq!(document.serialize(), "# Title\n\nBody");
        document
            .dispatch_text("!")
            .expect("text dispatch must succeed");
        document
            .save()
            .expect("serialized Markdown must save to disk");
        assert_eq!(fs::read_to_string(&path).unwrap(), "# !Title\n\nBody");

        fs::remove_file(path).expect("test fixture must be removed");
    }

    #[test]
    fn rejects_stale_revision_instead_of_mutating_the_document() {
        let mut session = EditorSession::from_markdown("alpha");
        let error = session
            .dispatch(42, EditorAction::InsertText("X".into()))
            .expect_err("stale revision must be rejected");

        assert_eq!(
            error,
            EditorError::Edit(EditError::RevisionMismatch {
                expected: 42,
                actual: 0,
            })
        );
        assert_eq!(session.markdown(), "alpha");
    }
}
