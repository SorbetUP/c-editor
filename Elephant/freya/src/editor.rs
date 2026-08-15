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
    time::{Duration, Instant},
};

use muya_core::{
    edit::PasteCommand,
    features::{BlockTypeCommand, TableNavigationCommand, TaskCommand},
    model::{InlineKind, InlineMarkKind, ListKind, NodeKind},
    Command, EditError, EditorSession as MuyaEditorSession, GraphemeCommand, MarkCommand,
    ParagraphBoundaryCommand, Selection, SelectionPoint, SessionCommand, SessionSnapshot,
    SessionUpdate, ViewPatch,
};

#[path = "app/editor_lifecycle.rs"]
mod editor_lifecycle;
pub(crate) use editor_lifecycle::Delay;
pub use editor_lifecycle::EditorPreferences;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct EditorViewState {
    pub scroll_top: i32,
    pub topbar_compact: bool,
}

/// Actions emitted by a Freya editor surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorAction {
    InsertText(String),
    PasteMarkdown(String),
    InsertParagraph,
    DeleteBackward,
    ToggleStrong,
    ToggleEmphasis,
    ToggleStrike,
    SetHeading(u8),
    SetListKind(ListKind),
    ToggleBlockQuote,
    ToggleCodeBlock,
    TableNavigation(TableNavigationCommand),
    SetTaskChecked {
        item: muya_core::NodeId,
        checked: bool,
        auto_check: bool,
    },
    BeginComposition,
    UpdateComposition(String),
    CommitComposition,
    CancelComposition,
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
            EditorAction::PasteMarkdown(markdown) => {
                SessionCommand::Paste(PasteCommand::new(markdown))
            }
            EditorAction::InsertParagraph => SessionCommand::Core(Command::InsertParagraph),
            EditorAction::DeleteBackward => {
                SessionCommand::Grapheme(GraphemeCommand::DeleteBackward)
            }
            EditorAction::ToggleStrong => SessionCommand::Mark(MarkCommand::ToggleStrong),
            EditorAction::ToggleEmphasis => SessionCommand::Mark(MarkCommand::ToggleEmphasis),
            EditorAction::ToggleStrike => SessionCommand::Mark(MarkCommand::ToggleStrike),
            EditorAction::SetHeading(level) => SessionCommand::Core(Command::SetHeading(level)),
            EditorAction::SetListKind(kind) => {
                SessionCommand::BlockType(BlockTypeCommand::SetListKind(kind))
            }
            EditorAction::ToggleBlockQuote => {
                SessionCommand::BlockType(BlockTypeCommand::ToggleBlockQuote)
            }
            EditorAction::ToggleCodeBlock => {
                SessionCommand::BlockType(BlockTypeCommand::ToggleCodeBlock)
            }
            EditorAction::TableNavigation(command) => SessionCommand::TableNavigation(command),
            EditorAction::SetTaskChecked {
                item,
                checked,
                auto_check,
            } => SessionCommand::Task(TaskCommand::SetChecked {
                item,
                checked,
                auto_check,
            }),
            EditorAction::BeginComposition => SessionCommand::BeginComposition,
            EditorAction::UpdateComposition(text) => SessionCommand::UpdateComposition(text),
            EditorAction::CommitComposition => SessionCommand::CommitComposition,
            EditorAction::CancelComposition => SessionCommand::CancelComposition,
            EditorAction::Undo => SessionCommand::Undo,
            EditorAction::Redo => SessionCommand::Redo,
        };
        self.dispatch_session_command(expected_revision, command)
    }

    pub fn dispatch_current(&mut self, action: EditorAction) -> Result<EditorUpdate, EditorError> {
        self.dispatch(self.revision(), action)
    }

    pub fn dispatch_text(&mut self, text: impl Into<String>) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::InsertText(text.into()))
    }

    pub fn paste_markdown(
        &mut self,
        markdown: impl Into<String>,
    ) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::PasteMarkdown(markdown.into()))
    }

    pub fn undo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::Undo)
    }

    pub fn redo(&mut self) -> Result<EditorUpdate, EditorError> {
        self.dispatch_current(EditorAction::Redo)
    }

    fn dispatch_session_command(
        &mut self,
        expected_revision: u64,
        command: SessionCommand,
    ) -> Result<EditorUpdate, EditorError> {
        let update = self.inner.dispatch(expected_revision, command)?;
        Ok(EditorUpdate::from_session(self.markdown(), update))
    }
}

/// A note document loaded from the vault and backed by a real Muya session.
#[derive(Clone, Debug)]
pub struct EditorDocument {
    path: Option<PathBuf>,
    session: EditorSession,
    saved_markdown: String,
    dirty: bool,
    last_edit_at: Option<Instant>,
    preferences: EditorPreferences,
    view_state: EditorViewState,
    focus_target: Option<muya_core::NodeId>,
    composition_selection: Option<Selection>,
    autosave_failure_revision: Option<u64>,
}

impl EditorDocument {
    pub fn from_markdown(markdown: &str) -> Self {
        let session = EditorSession::from_markdown(markdown);
        Self {
            path: None,
            saved_markdown: session.markdown(),
            session,
            dirty: false,
            last_edit_at: None,
            preferences: editor_lifecycle::load_preferences(),
            view_state: EditorViewState::default(),
            focus_target: None,
            composition_selection: None,
            autosave_failure_revision: None,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, EditorError> {
        let path = path.as_ref().to_path_buf();
        let markdown = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        let session = EditorSession::from_markdown(&markdown);
        Ok(Self {
            path: Some(path),
            saved_markdown: session.markdown(),
            session,
            dirty: false,
            last_edit_at: None,
            preferences: editor_lifecycle::load_preferences(),
            view_state: EditorViewState::default(),
            focus_target: None,
            composition_selection: None,
            autosave_failure_revision: None,
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

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn autosave_enabled(&self) -> bool {
        self.preferences.auto_save
    }

    pub fn autosave_delay(&self) -> Duration {
        Duration::from_millis(self.preferences.auto_save_delay_ms)
    }

    pub fn scroll_top(&self) -> i32 {
        self.view_state.scroll_top
    }

    pub fn topbar_compact(&self) -> bool {
        self.view_state.topbar_compact
    }

    pub fn focus_target(&self) -> Option<muya_core::NodeId> {
        self.focus_target
    }

    pub fn set_scroll_top(&mut self, scroll_top: i32) {
        self.view_state.scroll_top = scroll_top.max(0);
        self.view_state.topbar_compact = self.view_state.scroll_top > 24;
    }

    pub fn autosave_due(&self) -> bool {
        self.autosave_enabled()
            && self.dirty
            && self.autosave_failure_revision != Some(self.session.revision())
            && self
                .last_edit_at
                .is_some_and(|at| at.elapsed() >= self.autosave_delay())
    }

    /// Serializes the current Muya document; this is the Markdown payload used
    /// by NoteEditorHost's save path, not a cached copy of the input file.
    pub fn serialize(&self) -> String {
        self.session.markdown()
    }

    pub fn dispatch(&mut self, action: EditorAction) -> Result<EditorUpdate, EditorError> {
        let moves_table_focus = matches!(&action, EditorAction::TableNavigation(_));
        let begins_composition = matches!(&action, EditorAction::BeginComposition);
        let updates_composition = matches!(&action, EditorAction::UpdateComposition(_));
        let ends_composition = matches!(
            &action,
            EditorAction::CommitComposition | EditorAction::CancelComposition
        );
        if begins_composition {
            self.composition_selection = Some(self.session.snapshot().selection);
        }
        if updates_composition {
            if let Some(selection) = self.composition_selection {
                let revision = self.session.revision();
                self.session.set_selection(revision, selection)?;
            }
        }
        let before = self.serialize();
        let result = self
            .session
            .dispatch_current(action)
            .map(|update| self.record_mutation(before, update));
        if moves_table_focus {
            if let Ok(update) = &result {
                self.focus_target = Some(update.selection.focus.node);
                eprintln!(
                    "[freya][editor] focus-request action=table target={:?}",
                    update.selection.focus.node
                );
            }
        }
        if let Ok(update) = &result {
            if updates_composition {
                let anchor = self
                    .composition_selection
                    .map(|selection| selection.anchor)
                    .unwrap_or(update.selection.anchor);
                self.composition_selection = Some(Selection {
                    anchor,
                    focus: update.selection.focus,
                });
            } else if ends_composition {
                self.composition_selection = None;
            }
        }
        result
    }

    pub fn dispatch_text(&mut self, text: impl Into<String>) -> Result<EditorUpdate, EditorError> {
        let before = self.serialize();
        self.session
            .dispatch_text(text)
            .map(|update| self.record_mutation(before, update))
    }

    pub fn paste_markdown(
        &mut self,
        markdown: impl Into<String>,
    ) -> Result<EditorUpdate, EditorError> {
        self.dispatch(EditorAction::PasteMarkdown(markdown.into()))
    }

    pub fn selected_markdown(&self) -> Result<String, String> {
        selected_inline_markdown(self.session.document(), self.session.snapshot().selection)
    }

    pub fn selected_text(&self) -> Result<String, String> {
        selected_inline_text(self.session.document(), self.session.snapshot().selection)
    }

    /// Applies inline-code Markdown through Muya's real paste transaction so the
    /// resulting `CodeSpan` remains part of the editable document and history.
    pub fn apply_inline_code(&mut self) -> Result<EditorUpdate, String> {
        let selected = self.selected_text()?;
        let markdown = inline_code_markdown(&selected);
        self.paste_markdown(markdown)
            .map_err(|error| error.to_string())
    }

    /// Wraps the current real selection in a Markdown link. The destination is
    /// supplied by the Freya toolbar form rather than fabricated in document state.
    pub fn link_selection(&mut self, destination: &str) -> Result<EditorUpdate, String> {
        let selected = self.selected_text()?;
        let destination = destination.trim();
        if destination.is_empty() {
            return Err("link destination cannot be empty".to_string());
        }
        let label = escape_link_label(&selected);
        let destination = markdown_link_destination(destination);
        self.paste_markdown(format!("[{label}]({destination})"))
            .map_err(|error| error.to_string())
    }

    pub fn set_selection(&mut self, selection: Selection) -> Result<EditorUpdate, EditorError> {
        let revision = self.session.revision();
        self.session.set_selection(revision, selection)
    }

    pub fn move_caret_to_end_of_block(
        &mut self,
        node_id: muya_core::NodeId,
    ) -> Result<EditorUpdate, EditorError> {
        let (text_node, offset_utf16) = last_text_endpoint(self.session.document(), node_id)
            .ok_or(EditorError::Edit(EditError::NotTextNode(node_id)))?;
        let update = self.set_selection(Selection::collapsed(SelectionPoint {
            node: text_node,
            offset_utf16,
        }))?;
        self.focus_target = Some(update.selection.focus.node);
        Ok(update)
    }

    pub fn insert_paragraph(&mut self) -> Result<EditorUpdate, EditorError> {
        let before = self.serialize();
        let revision = self.session.revision();
        match self
            .session
            .dispatch_session_command(revision, SessionCommand::Core(Command::InsertParagraph))
        {
            Err(EditorError::Edit(EditError::UnsupportedStructure(_))) => {
                self.session.dispatch_session_command(
                    revision,
                    SessionCommand::ParagraphBoundary(ParagraphBoundaryCommand::InsertParagraph),
                )
            }
            result => result,
        }
        .map(|update| {
            self.focus_target = Some(update.selection.focus.node);
            self.record_mutation(before, update)
        })
    }

    pub fn delete_backward(&mut self) -> Result<EditorUpdate, EditorError> {
        self.dispatch(EditorAction::DeleteBackward)
    }

    pub fn delete_forward(&mut self) -> Result<EditorUpdate, EditorError> {
        let before = self.serialize();
        let snapshot = self.session.snapshot();
        if !snapshot.selection.is_collapsed() {
            return self
                .session
                .dispatch_session_command(
                    snapshot.revision,
                    SessionCommand::Core(Command::InsertText(String::new())),
                )
                .map(|update| self.record_mutation(before, update));
        }

        let caret = snapshot
            .selection
            .caret()
            .expect("a collapsed Muya selection must expose its caret");
        let (next_same_node, next_block_text) = {
            let document = self.session.document();
            let next_same_node = text_value(document, caret.node).and_then(|value| {
                next_utf16_boundary(value, caret.offset_utf16).map(|end| Selection {
                    anchor: caret,
                    focus: SelectionPoint {
                        node: caret.node,
                        offset_utf16: end,
                    },
                })
            });
            (next_same_node, next_block_text_node(document, caret.node))
        };

        if let Some(selection) = next_same_node {
            self.set_selection(selection)?;
            return self
                .session
                .dispatch_session_command(
                    snapshot.revision,
                    SessionCommand::Core(Command::InsertText(String::new())),
                )
                .map(|update| self.record_mutation(before, update));
        }

        if let Some(next_text) = next_block_text {
            self.set_selection(Selection::collapsed(SelectionPoint {
                node: next_text,
                offset_utf16: 0,
            }))?;
            return self
                .session
                .dispatch_session_command(
                    snapshot.revision,
                    SessionCommand::Grapheme(GraphemeCommand::DeleteBackward),
                )
                .map(|update| self.record_mutation(before, update));
        }

        Ok(snapshot)
    }

    pub fn undo(&mut self) -> Result<EditorUpdate, EditorError> {
        let before = self.serialize();
        self.session
            .undo()
            .map(|update| self.record_mutation(before, update))
    }

    pub fn redo(&mut self) -> Result<EditorUpdate, EditorError> {
        let before = self.serialize();
        self.session
            .redo()
            .map(|update| self.record_mutation(before, update))
    }

    pub fn save(&mut self) -> Result<(), EditorError> {
        let path = self.path.clone().ok_or(EditorError::MissingPath)?;
        let revision = self.session.revision();
        let started = Instant::now();
        eprintln!(
            "[freya][editor] action:start action=save path={} revision={revision}",
            path.display()
        );
        let markdown = self.serialize();
        match fs::write(&path, &markdown) {
            Ok(()) => {
                self.saved_markdown = markdown;
                self.dirty = false;
                self.last_edit_at = None;
                self.autosave_failure_revision = None;
                eprintln!(
                    "[freya][editor] action:complete action=save path={} revision={revision} duration_ms={}",
                    path.display(),
                    started.elapsed().as_millis()
                );
                Ok(())
            }
            Err(error) => {
                self.autosave_failure_revision = Some(revision);
                eprintln!(
                    "[freya][editor] action:failure action=save path={} revision={revision} duration_ms={} error={error}",
                    path.display(),
                    started.elapsed().as_millis()
                );
                Err(io_error(&path, error))
            }
        }
    }

    pub fn close(&mut self) -> Result<(), EditorError> {
        let path = self.path.clone().ok_or(EditorError::MissingPath)?;
        let revision = self.session.revision();
        let started = Instant::now();
        eprintln!(
            "[freya][editor] action:start action=close path={} revision={revision} dirty={}",
            path.display(),
            self.dirty
        );
        if self.dirty {
            self.save()?;
        }
        eprintln!(
            "[freya][editor] action:complete action=close path={} revision={revision} duration_ms={}",
            path.display(),
            started.elapsed().as_millis()
        );
        Ok(())
    }

    fn record_mutation(&mut self, before: String, update: EditorUpdate) -> EditorUpdate {
        self.dirty = self.saved_markdown != update.markdown;
        if before != update.markdown {
            self.last_edit_at = Some(Instant::now());
            self.autosave_failure_revision = None;
            eprintln!(
                "[freya][editor] action:complete action=mutation path={} revision={} dirty={}",
                self.path
                    .as_deref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<memory>".to_string()),
                update.revision,
                self.dirty
            );
        }
        update
    }
}

fn io_error(path: &Path, error: io::Error) -> EditorError {
    EditorError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

fn text_value(document: &muya_core::Document, node_id: muya_core::NodeId) -> Option<&str> {
    match &document.node(node_id)?.kind {
        NodeKind::Inline(InlineKind::Text { value }) => Some(value),
        NodeKind::Inline(InlineKind::CodeSpan { code }) => Some(code),
        _ => None,
    }
}

fn next_utf16_boundary(value: &str, offset: u32) -> Option<u32> {
    let mut cursor = 0u32;
    for character in value.chars() {
        if cursor == offset {
            return Some(offset + character.len_utf16() as u32);
        }
        cursor += character.len_utf16() as u32;
    }
    None
}

fn next_block_text_node(
    document: &muya_core::Document,
    node_id: muya_core::NodeId,
) -> Option<muya_core::NodeId> {
    let mut block = node_id;
    while let Some(parent) = document.node(block).and_then(|node| node.parent) {
        if parent == document.root {
            break;
        }
        block = parent;
    }
    let root_children = document.node(document.root)?.children.as_slice();
    let index = root_children
        .iter()
        .position(|candidate| *candidate == block)?;
    root_children
        .get(index + 1)
        .copied()
        .and_then(|next| first_text_node(document, next))
}

fn first_text_node(
    document: &muya_core::Document,
    node_id: muya_core::NodeId,
) -> Option<muya_core::NodeId> {
    let node = document.node(node_id)?;
    if matches!(
        node.kind,
        NodeKind::Inline(InlineKind::Text { .. } | InlineKind::CodeSpan { .. })
    ) {
        return Some(node_id);
    }
    node.children
        .iter()
        .find_map(|child| first_text_node(document, *child))
}

fn selected_inline_text(
    document: &muya_core::Document,
    selection: Selection,
) -> Result<String, String> {
    if selection.anchor.node != selection.focus.node {
        return Err("selection spans multiple Muya inline nodes".to_string());
    }
    let value = text_value(document, selection.anchor.node)
        .ok_or_else(|| "selection is not editable text".to_string())?;
    let start = selection
        .anchor
        .offset_utf16
        .min(selection.focus.offset_utf16);
    let end = selection
        .anchor
        .offset_utf16
        .max(selection.focus.offset_utf16);
    if start == end {
        return Err("formatting requires a non-empty selection".to_string());
    }
    utf16_slice(value, start, end)
}

fn last_text_endpoint(
    document: &muya_core::Document,
    node_id: muya_core::NodeId,
) -> Option<(muya_core::NodeId, u32)> {
    let node = document.node(node_id)?;
    if let NodeKind::Inline(inline) = &node.kind {
        let value = match inline {
            InlineKind::Text { value } => value,
            InlineKind::CodeSpan { code } => code,
            _ => return None,
        };
        return Some((node_id, value.encode_utf16().count() as u32));
    }
    node.children
        .iter()
        .rev()
        .find_map(|child| last_text_endpoint(document, *child))
}

fn selected_inline_markdown(
    document: &muya_core::Document,
    selection: Selection,
) -> Result<String, String> {
    if selection.anchor.node != selection.focus.node {
        return Err("copy selection spans multiple Muya text nodes".to_string());
    }
    let node = document
        .node(selection.anchor.node)
        .ok_or_else(|| "copy selection has no Muya text node".to_string())?;
    let value = text_value(document, selection.anchor.node)
        .ok_or_else(|| "copy selection is not text".to_string())?;
    let start = selection
        .anchor
        .offset_utf16
        .min(selection.focus.offset_utf16);
    let end = selection
        .anchor
        .offset_utf16
        .max(selection.focus.offset_utf16);
    if start == end {
        return Err("copy requires a non-empty selection".to_string());
    }
    let text = utf16_slice(value, start, end)?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::CodeSpan { .. })) {
        return Ok(inline_code_markdown(&text));
    }
    let mut wrappers = Vec::new();
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let parent_node = document
            .node(parent_id)
            .ok_or_else(|| "copy selection has an invalid Muya parent".to_string())?;
        match &parent_node.kind {
            NodeKind::Inline(InlineKind::Strong) => wrappers.push(("**", "**")),
            NodeKind::Inline(InlineKind::Emphasis) => wrappers.push(("*", "*")),
            NodeKind::Inline(InlineKind::Strike) => wrappers.push(("~~", "~~")),
            NodeKind::Inline(InlineKind::MarkFragment {
                mark: InlineMarkKind::Strong,
                ..
            }) => wrappers.push(("**", "**")),
            NodeKind::Inline(InlineKind::MarkFragment {
                mark: InlineMarkKind::Emphasis,
                ..
            }) => wrappers.push(("*", "*")),
            NodeKind::Inline(InlineKind::MarkFragment {
                mark: InlineMarkKind::Strike,
                ..
            }) => wrappers.push(("~~", "~~")),
            NodeKind::Block(_) => break,
            _ => {}
        }
        parent = parent_node.parent;
    }
    let mut markdown = String::new();
    for (open, _) in wrappers.iter().rev() {
        markdown.push_str(open);
    }
    markdown.push_str(&text);
    for (_, close) in &wrappers {
        markdown.push_str(close);
    }
    Ok(markdown)
}

fn inline_code_markdown(value: &str) -> String {
    let mut longest_run = 0usize;
    let mut current_run = 0usize;
    for character in value.chars() {
        if character == '`' {
            current_run += 1;
            longest_run = longest_run.max(current_run);
        } else {
            current_run = 0;
        }
    }
    let delimiter = "`".repeat(longest_run + 1);
    let needs_padding = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(' ') && value.ends_with(' '));
    if needs_padding {
        format!("{delimiter} {value} {delimiter}")
    } else {
        format!("{delimiter}{value}{delimiter}")
    }
}

fn escape_link_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn markdown_link_destination(value: &str) -> String {
    if value.chars().any(char::is_whitespace) || value.contains(['(', ')']) {
        format!("<{}>", value.replace('>', "%3E"))
    } else {
        value.to_string()
    }
}

fn utf16_slice(value: &str, start: u32, end: u32) -> Result<String, String> {
    let mut cursor = 0_u32;
    let mut selected = String::new();
    for character in value.chars() {
        let next = cursor + character.len_utf16() as u32;
        if cursor < start && start < next || cursor < end && end < next {
            return Err("copy selection splits a Unicode code point".to_string());
        }
        if cursor >= start && next <= end {
            selected.push(character);
        }
        cursor = next;
    }
    if end > cursor || start > end {
        return Err("copy selection is outside the Muya text node".to_string());
    }
    Ok(selected)
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

    fn select_all_first_inline(document: &mut EditorDocument) -> muya_core::NodeId {
        let paragraph = document
            .session()
            .document()
            .children(document.session().document().root)
            .next()
            .expect("fixture paragraph");
        let inline_id = document
            .session()
            .document()
            .children(paragraph.id)
            .next()
            .expect("fixture inline");
        let inline_id = inline_id.id;
        let value = text_value(document.session().document(), inline_id).expect("editable inline");
        let end = value.encode_utf16().count() as u32;
        document
            .set_selection(Selection {
                anchor: SelectionPoint {
                    node: inline_id,
                    offset_utf16: 0,
                },
                focus: SelectionPoint {
                    node: inline_id,
                    offset_utf16: end,
                },
            })
            .expect("select fixture inline");
        inline_id
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
    fn toolbar_block_actions_use_real_muya_commands() {
        let mut heading = EditorDocument::from_markdown("alpha");
        heading
            .dispatch(EditorAction::SetHeading(2))
            .expect("heading command");
        assert_eq!(heading.serialize(), "## alpha");

        let mut bullets = EditorDocument::from_markdown("alpha");
        bullets
            .dispatch(EditorAction::SetListKind(ListKind::Unordered))
            .expect("bullet list command");
        assert_eq!(bullets.serialize(), "- alpha");

        let mut ordered = EditorDocument::from_markdown("alpha");
        ordered
            .dispatch(EditorAction::SetListKind(ListKind::Ordered))
            .expect("ordered list command");
        assert_eq!(ordered.serialize(), "1. alpha");

        let mut task = EditorDocument::from_markdown("alpha");
        task.dispatch(EditorAction::SetListKind(ListKind::Task))
            .expect("task list command");
        assert_eq!(task.serialize(), "- [ ] alpha");

        let mut quote = EditorDocument::from_markdown("alpha");
        quote
            .dispatch(EditorAction::ToggleBlockQuote)
            .expect("quote command");
        assert_eq!(quote.serialize(), "> alpha");
    }

    #[test]
    fn toolbar_inline_code_and_link_replace_the_real_selection() {
        let mut code = EditorDocument::from_markdown("alpha");
        select_all_first_inline(&mut code);
        code.apply_inline_code().expect("inline-code command");
        assert_eq!(code.serialize(), "`alpha`");

        let code_span = code
            .session()
            .document()
            .children(code.session().document().root)
            .next()
            .and_then(|paragraph| {
                code.session()
                    .document()
                    .children(paragraph.id)
                    .find(|node| matches!(node.kind, NodeKind::Inline(InlineKind::CodeSpan { .. })))
            })
            .expect("rendered code span")
            .id;
        let (code_text, code_end) = last_text_endpoint(code.session().document(), code_span)
            .expect("code span must contain an editable text node");
        code.set_selection(Selection::collapsed(SelectionPoint {
            node: code_text,
            offset_utf16: code_end,
        }))
        .expect("place caret inside code");
        code.dispatch_text("!")
            .expect("code span must remain editable after toolbar formatting");
        assert_eq!(code.serialize(), "`alpha!`");

        let mut link = EditorDocument::from_markdown("alpha");
        select_all_first_inline(&mut link);
        link.link_selection("https://example.com")
            .expect("link command");
        assert_eq!(link.serialize(), "[alpha](https://example.com)");
    }

    #[test]
    fn inline_code_chooses_a_safe_backtick_delimiter() {
        assert_eq!(inline_code_markdown("alpha"), "`alpha`");
        assert_eq!(inline_code_markdown("a`b"), "``a`b``");
        assert_eq!(inline_code_markdown("`a`"), "`` `a` ``");
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
        assert!(
            document.is_dirty(),
            "a Muya mutation must mark the document dirty"
        );
        document
            .save()
            .expect("serialized Markdown must save to disk");
        assert!(
            !document.is_dirty(),
            "save must clear dirty only after fs::write succeeds"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "# !Title\n\nBody");

        fs::remove_file(path).expect("test fixture must be removed");
    }

    #[test]
    fn failed_save_keeps_the_real_document_dirty() {
        let parent = std::env::temp_dir().join(format!(
            "elephant-freya-editor-failure-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&parent).expect("fixture directory must be writable");
        let path = parent.join("note.md");
        fs::write(&path, "alpha").expect("fixture must be writable");
        let mut document = EditorDocument::load(&path).expect("fixture must load");
        document
            .dispatch_text("!")
            .expect("mutation must use the real Muya session");
        fs::remove_dir_all(&parent).expect("remove fixture to inject a save failure");

        assert!(
            document.save().is_err(),
            "the real write must report the filesystem error"
        );
        assert!(
            document.is_dirty(),
            "a failed write must not clear dirty state"
        );
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

    #[test]
    fn selection_update_keeps_revision_for_the_following_dispatch() {
        let mut session = EditorSession::from_markdown("alpha");
        let paragraph = session
            .document()
            .children(session.document().root)
            .next()
            .unwrap();
        let text = session.document().children(paragraph.id).next().unwrap();
        let before = session.snapshot();

        let selected = session
            .set_selection(
                before.revision,
                Selection::collapsed(SelectionPoint {
                    node: text.id,
                    offset_utf16: 1,
                }),
            )
            .expect("selection update must use the current revision");
        assert_eq!(selected.revision, before.revision);

        let inserted = session
            .dispatch(before.revision, EditorAction::InsertText("X".into()))
            .expect("the edit after selection must use the unchanged revision");
        assert_eq!(inserted.markdown, "aXlpha");
    }
}
