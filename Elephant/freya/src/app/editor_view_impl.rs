//! Native Freya rendering for the real Muya document owned by NoteEditorHost.
//!
//! The Vue host delegates the editing surface to `EditorWithTabs`/Muya. This
//! native tranche keeps that boundary: `EditorDocument` remains the source of
//! truth for undo/save, while this module walks its parsed Muya tree and turns
//! blocks and inline nodes into Freya paragraphs and containers.

use freya::{
    prelude::*,
    text_edit::{use_editable, EditableConfig, EditableEvent, EditorLine, TextEditor, UseEditable},
};
use muya_core::{
    model::{BlockKind, InlineKind, InlineMarkKind, ListKind, NodeKind},
    selection::{Selection, SelectionPoint},
    Document, NodeId,
};
use std::path::Path;

use crate::{
    editor::{Delay, EditorAction},
    library_contract::RelativePath,
    theme,
};

use super::{route_notice, ShellState};

#[path = "editor_interactions.rs"]
mod editor_interactions;

const EDITOR_CONTENT_MAX: f32 = 780.;
const EDITOR_BODY_SIZE: f32 = 16.;
const EDITOR_BODY_LINE_HEIGHT: f32 = 1.58;
const EDITOR_TOPBAR_HEIGHT: f32 = 52.;
const EDITOR_COMPACT_TOPBAR_HEIGHT: f32 = 36.;
const EDITOR_TOPBAR_ACTION_SIZE: f32 = 30.;
const EDITOR_TOOLBAR_HEIGHT: f32 = 56.;
const EDITOR_TOOLBAR_ACTION_SIZE: f32 = 34.;
const EDITOR_FOOTER_HEIGHT: f32 = 50.;
const EDITOR_FONT_FAMILY: &str = "sans-serif";
const EDITOR_CODE_FONT_FAMILY: &str = "monospace";
const PIN_ACTIVE: (u8, u8, u8, u8) = (250, 204, 21, 255);

#[derive(Clone, Copy, Default)]
struct InlineStyle {
    strong: bool,
    emphasis: bool,
    strike: bool,
    code: bool,
    link: bool,
    script: ScriptStyle,
    status: bool,
}

#[derive(Clone, Copy, Default)]
enum ScriptStyle {
    #[default]
    Normal,
    Superscript,
    Subscript,
}

#[derive(Clone, Copy, PartialEq)]
struct BlockTextStyle {
    font_size: f32,
    line_height: f32,
    bold: bool,
    italic: bool,
    code: bool,
    color: (u8, u8, u8, u8),
}

impl Default for BlockTextStyle {
    fn default() -> Self {
        Self {
            font_size: EDITOR_BODY_SIZE,
            line_height: EDITOR_BODY_LINE_HEIGHT,
            bold: false,
            italic: false,
            code: false,
            color: theme::TEXT,
        }
    }
}

#[derive(Clone, PartialEq)]
struct EditableInlineBlock {
    state: State<ShellState>,
    autosave_generation: State<u64>,
    node_id: NodeId,
    accessibility_label: String,
    style: BlockTextStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
}

impl Component for EditableInlineBlock {
    fn render(&self) -> impl IntoElement {
        let snapshot = self.state.read().clone();
        let mut spans = Vec::new();
        if let Some(editor) = snapshot.editor.as_ref() {
            collect_inline_children(
                editor.session().document(),
                self.node_id,
                InlineStyle::default(),
                self.palette,
                self.text_scale,
                &mut spans,
            );
        }
        if spans.is_empty() {
            spans.push(styled_span(
                String::new(),
                InlineStyle::default(),
                self.palette,
                self.text_scale,
            ));
        }
        let value = spans
            .iter()
            .map(|span| span.text.as_ref())
            .collect::<String>();
        let mut editable = use_editable(|| value.clone(), EditableConfig::new);
        let a11y_id = use_a11y();
        let holder = use_state(ParagraphHolder::default);

        let should_focus = snapshot.editor.as_ref().is_some_and(|editor| {
            editor.focus_target().is_some_and(|target| {
                let contains = contains_node(editor.session().document(), self.node_id, target);
                if contains {
                    eprintln!(
                        "[freya][editor] focus-apply block={:?} target={:?}",
                        self.node_id, target
                    );
                }
                contains
            })
        });
        use_side_effect(move || {
            if should_focus {
                a11y_id.request_focus();
            }
        });

        if snapshot.editor.is_none() {
            return route_notice("NoteEditorHost", "No note open").into_element();
        }

        if editable.editor().read().committed_text() != value {
            let mut inner = editable.editor_mut().write();
            inner.set(&value);
            inner.clear_selection();
        }

        let cursor_index = editable.editor().read().cursor_pos();
        let highlights = editable
            .editor()
            .read()
            .get_visible_selection(EditorLine::SingleParagraph);
        let state = self.state;
        let autosave_generation = self.autosave_generation;
        let node_id = self.node_id;
        let previous_value = value.clone();
        let on_key_down = editor_interactions::key_down_handler(
            state,
            node_id,
            editable,
            previous_value,
            autosave_generation,
        );
        let on_ime_preedit =
            editor_interactions::ime_preedit_handler(state, node_id, editable, autosave_generation);
        let on_key_up = move |event: Event<KeyboardEventData>| {
            editable.process_event(EditableEvent::KeyUp { key: &event.key });
        };
        let on_mouse_down = move |event: Event<MouseEventData>| {
            a11y_id.request_focus();
            editable.process_event(EditableEvent::Down {
                location: event.element_location,
                editor_line: EditorLine::SingleParagraph,
                holder: &holder.read(),
            });
        };
        let on_mouse_move = move |event: Event<MouseEventData>| {
            editable.process_event(EditableEvent::Move {
                location: event.element_location,
                editor_line: EditorLine::SingleParagraph,
                holder: &holder.read(),
            });
        };
        let on_pointer_up = move |_| editable.process_event(EditableEvent::Release);

        let mut view = paragraph()
            .a11y_id(a11y_id)
            .width(Size::fill())
            .holder(holder.read().clone())
            .cursor_index(cursor_index)
            .highlights(highlights.map(|selection| vec![selection]))
            .spans_iter(spans.into_iter())
            .a11y_alt(self.accessibility_label.clone())
            .on_mouse_down(on_mouse_down)
            .on_mouse_move(on_mouse_move)
            .on_global_pointer_press(on_pointer_up)
            .on_key_down(on_key_down)
            .on_key_up(on_key_up)
            .on_ime_preedit(on_ime_preedit)
            .font_size(self.style.font_size * self.text_scale)
            .line_height(self.style.line_height)
            .font_family(if self.style.code {
                EDITOR_CODE_FONT_FAMILY
            } else {
                EDITOR_FONT_FAMILY
            })
            .color(editor_color(self.palette, self.style.color));
        if self.style.bold {
            view = view.font_weight(FontWeight::BOLD);
        }
        if self.style.italic {
            view = view.font_slant(FontSlant::Italic);
        }
        view.into_element()
    }
}

pub(super) fn note_editor_host(state: State<ShellState>) -> Element {
    NoteEditorHost { state }.into_element()
}

#[derive(PartialEq)]
struct NoteEditorHost {
    state: State<ShellState>,
}

impl Component for NoteEditorHost {
    fn render(&self) -> impl IntoElement {
        render_note_editor_host(self.state)
    }
}

fn dispatch_toolbar_action(
    mut state: State<ShellState>,
    mut autosave_generation: State<u64>,
    action: EditorAction,
    action_name: &'static str,
) {
    let result = state
        .write()
        .editor
        .as_mut()
        .ok_or_else(|| "cannot format without an open note".to_string())
        .and_then(|editor| {
            editor
                .dispatch(action)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
    finish_toolbar_result(state, autosave_generation, action_name, result);
}

fn finish_toolbar_result(
    mut state: State<ShellState>,
    mut autosave_generation: State<u64>,
    action_name: &'static str,
    result: Result<(), String>,
) {
    match result {
        Ok(()) => {
            *autosave_generation.write() += 1;
            eprintln!("[freya][editor] action:complete action={action_name}");
        }
        Err(error) => {
            eprintln!("[freya][editor] action:failure action={action_name} error={error}");
            state.write().error = Some(error);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn editor_action_button(
    state: State<ShellState>,
    interaction_id: &'static str,
    text: impl Into<String>,
    accessibility_label: impl Into<String>,
    enabled: bool,
    active: bool,
    active_accent: Option<(u8, u8, u8, u8)>,
    size: f32,
    bordered: bool,
    palette: theme::ThemePalette,
    mut on_press: impl FnMut(Event<MouseEventData>) + 'static,
) -> Element {
    let text = text.into();
    let accessibility_label = accessibility_label.into();
    let hover_key = format!("editor-hover:{interaction_id}");
    let pressed_key = format!("editor-pressed:{interaction_id}");
    let interaction = state.read().hovered_target.clone();
    let hovered = enabled && interaction.as_deref() == Some(hover_key.as_str());
    let pressed = enabled && interaction.as_deref() == Some(pressed_key.as_str());
    let background = if pressed {
        theme::mix(palette.primary, palette.soft, 0.16)
    } else if hovered || active {
        palette.soft
    } else {
        palette.surface
    };
    let foreground = active_accent.filter(|_| active).unwrap_or(palette.text);
    let text_size = if text.chars().count() > 2 { 11. } else { 14. };

    let mut button = rect()
        .width(Size::px(size))
        .height(Size::px(size))
        .center()
        .opacity(if enabled { 1. } else { 0.38 })
        .background(theme::color(background))
        .with_corner_radius(8.)
        .a11y_alt(accessibility_label)
        .child(
            label()
                .font_size(text_size)
                .font_family(EDITOR_FONT_FAMILY)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(if enabled { foreground } else { palette.muted }))
                .text(text),
        );
    if bordered {
        button = button.border(
            Border::new()
                .fill(theme::color(if active {
                    active_accent.unwrap_or(palette.border_strong)
                } else {
                    palette.border
                }))
                .width(1.),
        );
    }
    if enabled {
        let mut hover_state = state;
        let hover_key_enter = hover_key.clone();
        let mut leave_state = state;
        let mut down_state = state;
        let pressed_key_down = pressed_key.clone();
        let mut up_state = state;
        let hover_key_up = hover_key;
        button = button
            .on_mouse_enter(move |_| {
                hover_state
                    .write()
                    .set_hovered_target(hover_key_enter.clone());
            })
            .on_mouse_leave(move |_| {
                leave_state.write().hovered_target = None;
            })
            .on_mouse_down(move |_| {
                down_state
                    .write()
                    .set_hovered_target(pressed_key_down.clone());
            })
            .on_mouse_up(move |event| {
                up_state.write().set_hovered_target(hover_key_up.clone());
                on_press(event);
            });
    }
    button.into_element()
}

fn passive_chip(
    state: State<ShellState>,
    interaction_id: &'static str,
    text: String,
    palette: theme::ThemePalette,
    muted: bool,
) -> Element {
    let hover_key = format!("editor-hover:{interaction_id}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let mut hover_state = state;
    let hover_key_enter = hover_key;
    let mut leave_state = state;
    rect()
        .height(Size::px(30.))
        .padding(Gaps::new(0., 8., 0., 8.))
        .center()
        .background(theme::color(if hovered {
            palette.soft
        } else {
            palette.surface
        }))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .with_corner_radius(8.)
        .on_mouse_enter(move |_| {
            hover_state
                .write()
                .set_hovered_target(hover_key_enter.clone());
        })
        .on_mouse_leave(move |_| {
            leave_state.write().hovered_target = None;
        })
        .child(
            label()
                .font_size(14.)
                .font_family(EDITOR_FONT_FAMILY)
                .color(theme::color(if muted {
                    palette.muted
                } else {
                    palette.text
                }))
                .text(text.clone()),
        )
        .a11y_alt(text)
        .into_element()
}

fn render_note_editor_host(mut state: State<ShellState>) -> Element {
    let mut autosave_generation = use_state(|| 0_u64);
    let mut link_form_open = use_state(|| false);
    let mut link_value = use_state(String::new);
    let mut text_scale = use_state(|| 1.0_f32);
    let mut editor_dark_mode = use_state(|| false);

    let generation_for_effect = autosave_generation;
    let state_for_effect = state;
    use_side_effect(move || {
        let generation = *generation_for_effect.read();
        let (autosave_enabled, dirty, delay) = {
            let shell = state_for_effect.read();
            let Some(editor) = shell.editor.as_ref() else {
                return;
            };
            (
                editor.autosave_enabled(),
                editor.is_dirty(),
                editor.autosave_delay(),
            )
        };
        if !autosave_enabled || !dirty {
            return;
        }
        let mut state_for_save = state_for_effect;
        let generation_for_save = generation_for_effect;
        spawn(async move {
            Delay::new(delay).await;
            if *generation_for_save.read() != generation {
                return;
            }
            let result = {
                let mut shell = state_for_save.write();
                shell
                    .editor
                    .as_mut()
                    .filter(|editor| editor.autosave_due())
                    .map_or(Ok(()), |editor| editor.save())
            };
            if let Err(error) = result {
                eprintln!("[freya][editor] action:failure action=autosave error={error}");
                state_for_save.write().error = Some(error.to_string());
            }
        });
    });

    let initial_scroll_top = state
        .read()
        .editor
        .as_ref()
        .map_or(0, |editor| editor.scroll_top());
    let scroll_position = use_state(|| (0_i32, initial_scroll_top));
    let scroll_notifier = use_state(|| ());
    let scroll_requests = use_state(Vec::<ScrollRequest>::new);
    let on_scroll = use_state(|| {
        let mut scroll_position = scroll_position;
        let mut scroll_notifier = scroll_notifier;
        let mut state = state;
        Callback::new(move |event: ScrollEvent| {
            let (changed, y) = {
                let mut position = scroll_position.write();
                let previous = *position;
                match event {
                    ScrollEvent::X(x) => position.0 = x,
                    ScrollEvent::Y(y) => position.1 = y,
                }
                (previous != *position, position.1)
            };
            if changed {
                let scroll_top = y.saturating_neg();
                if let Some(editor) = state.write().editor.as_mut() {
                    editor.set_scroll_top(scroll_top);
                }
                eprintln!("[freya][editor] action:complete action=scroll scroll_top={scroll_top}");
                scroll_notifier.write();
            }
            changed
        })
    });
    let get_scroll = use_state(|| {
        let scroll_position = scroll_position;
        Callback::new(move |_| *scroll_position.read())
    });
    let scroll_controller =
        ScrollController::managed(scroll_notifier, scroll_requests, on_scroll, get_scroll);

    let snapshot = state.read().clone();
    let Some(editor) = snapshot.editor.clone() else {
        return route_notice("NoteEditorHost", "No note open");
    };

    let palette = if *editor_dark_mode.read() {
        theme::DARK_PALETTE
    } else {
        theme::LIGHT_PALETTE
    };
    let content_scale = *text_scale.read();
    let compact = editor.topbar_compact();
    let topbar_height = if compact {
        EDITOR_COMPACT_TOPBAR_HEIGHT
    } else {
        EDITOR_TOPBAR_HEIGHT
    };
    let editor_snapshot = editor.snapshot();
    let dirty = editor.is_dirty();
    let markdown = editor.serialize();
    let word_count = markdown.split_whitespace().count();
    let char_count = markdown.chars().count();
    let fallback_title = editor
        .path()
        .and_then(|path| path.file_stem())
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Untitled")
        .to_string();
    let note_title = document_title(&markdown, &fallback_title);
    let relative_path = editor_relative_path(&snapshot, editor.path());
    let library_entry = relative_path.as_deref().and_then(|path| {
        snapshot
            .library
            .entries
            .iter()
            .chain(snapshot.library.root_entries.iter())
            .find(|entry| entry.path.as_str() == path)
    });
    let mut tags = document_tags(&markdown);
    if tags.is_empty() {
        tags = library_entry
            .map(|entry| entry.tags.clone())
            .unwrap_or_default();
    }
    let note_date = document_created_at(&markdown)
        .or_else(|| library_entry.map(|entry| entry.updated_at.as_str().to_string()))
        .map(|value| short_date(&value));
    let is_pinned = relative_path.as_deref().is_some_and(|path| {
        snapshot
            .library
            .pinned_paths
            .iter()
            .any(|candidate| candidate.as_str() == path)
    });
    let selection_nonempty = !editor_snapshot.selection.is_collapsed();

    let undo = editor_action_button(
        state,
        "undo",
        "↶",
        "Undo",
        editor_snapshot.can_undo,
        false,
        None,
        EDITOR_TOPBAR_ACTION_SIZE,
        true,
        palette,
        move |_| {
            let result = state
                .write()
                .editor
                .as_mut()
                .ok_or_else(|| "cannot undo without an open note".to_string())
                .and_then(|editor| editor.undo().map(|_| ()).map_err(|error| error.to_string()));
            finish_toolbar_result(state, autosave_generation, "undo", result);
        },
    );
    let redo = editor_action_button(
        state,
        "redo",
        "↷",
        "Redo",
        editor_snapshot.can_redo,
        false,
        None,
        EDITOR_TOPBAR_ACTION_SIZE,
        true,
        palette,
        move |_| {
            let result = state
                .write()
                .editor
                .as_mut()
                .ok_or_else(|| "cannot redo without an open note".to_string())
                .and_then(|editor| editor.redo().map(|_| ()).map_err(|error| error.to_string()));
            finish_toolbar_result(state, autosave_generation, "redo", result);
        },
    );
    let save = editor_action_button(
        state,
        "save",
        "✓",
        "Save",
        true,
        dirty,
        Some(palette.primary),
        EDITOR_TOPBAR_ACTION_SIZE,
        true,
        palette,
        move |_| {
            let result = state
                .write()
                .editor
                .as_mut()
                .ok_or_else(|| "cannot save without an open note".to_string())
                .and_then(|editor| editor.save().map_err(|error| error.to_string()));
            if let Err(error) = result {
                eprintln!("[freya][editor] action:failure action=save error={error}");
                state.write().error = Some(error);
            } else {
                eprintln!("[freya][editor] action:complete action=save");
            }
        },
    );
    let pin_path = relative_path.clone();
    let pin = editor_action_button(
        state,
        "pin",
        "⌖",
        if is_pinned { "Unpin note" } else { "Pin note" },
        pin_path.is_some(),
        is_pinned,
        Some(PIN_ACTIVE),
        EDITOR_TOPBAR_ACTION_SIZE,
        true,
        palette,
        move |_| {
            let Some(path) = pin_path.as_deref() else {
                return;
            };
            let mut shell = state.write();
            if shell
                .library
                .pinned_paths
                .iter()
                .any(|candidate| candidate.as_str() == path)
            {
                shell
                    .library
                    .pinned_paths
                    .retain(|candidate| candidate.as_str() != path);
                eprintln!("[freya][editor] action:complete action=unpin path={path}");
            } else {
                shell.library.pinned_paths.push(RelativePath::new(path));
                eprintln!("[freya][editor] action:complete action=pin path={path}");
            }
        },
    );
    let close = editor_action_button(
        state,
        "close",
        "×",
        "Close note",
        true,
        false,
        None,
        EDITOR_TOPBAR_ACTION_SIZE,
        true,
        palette,
        move |_| {
            let result = {
                let mut shell = state.write();
                let result = shell.editor.as_mut().map_or_else(
                    || Err("cannot close without an open note".to_string()),
                    |editor| editor.close().map_err(|error| error.to_string()),
                );
                if result.is_ok() {
                    shell.editor = None;
                }
                result
            };
            if let Err(error) = result {
                eprintln!("[freya][editor] action:failure action=close error={error}");
                state.write().error = Some(error);
            }
        },
    );

    let bold = editor_action_button(
        state,
        "bold",
        "B",
        "Bold",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(state, autosave_generation, EditorAction::ToggleStrong, "bold")
        },
    );
    let italic = editor_action_button(
        state,
        "italic",
        "I",
        "Italic",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::ToggleEmphasis,
                "italic",
            )
        },
    );
    let strike = editor_action_button(
        state,
        "strike",
        "S",
        "Strikethrough",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::ToggleStrike,
                "strikethrough",
            )
        },
    );
    let inline_code = editor_action_button(
        state,
        "inline-code",
        "</>",
        if selection_nonempty {
            "Inline code"
        } else {
            "Inline code requires a selection"
        },
        selection_nonempty,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            let result = state
                .write()
                .editor
                .as_mut()
                .ok_or_else(|| "cannot format without an open note".to_string())
                .and_then(|editor| editor.apply_inline_code().map(|_| ()));
            finish_toolbar_result(state, autosave_generation, "inline-code", result);
        },
    );
    let mut open_link_form = link_form_open;
    let link = editor_action_button(
        state,
        "link",
        "↗",
        if selection_nonempty {
            "Link"
        } else {
            "Link requires a selection"
        },
        selection_nonempty,
        *link_form_open.read(),
        Some(palette.primary),
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            *open_link_form.write() = !*open_link_form.read();
        },
    );
    let heading = editor_action_button(
        state,
        "heading-2",
        "H2",
        "Heading 2",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::SetHeading(2),
                "heading-2",
            )
        },
    );
    let bullets = editor_action_button(
        state,
        "bullet-list",
        "•",
        "Bulleted list",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::SetListKind(ListKind::Unordered),
                "bullet-list",
            )
        },
    );
    let ordered = editor_action_button(
        state,
        "ordered-list",
        "1.",
        "Numbered list",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::SetListKind(ListKind::Ordered),
                "ordered-list",
            )
        },
    );
    let task = editor_action_button(
        state,
        "task-list",
        "☑",
        "Task list",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::SetListKind(ListKind::Task),
                "task-list",
            )
        },
    );
    let quote = editor_action_button(
        state,
        "quote",
        "❞",
        "Quote",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::ToggleBlockQuote,
                "quote",
            )
        },
    );
    let code_block = editor_action_button(
        state,
        "code-block",
        "{}",
        "Code block",
        true,
        false,
        None,
        EDITOR_TOOLBAR_ACTION_SIZE,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(
                state,
                autosave_generation,
                EditorAction::ToggleCodeBlock,
                "code-block",
            )
        },
    );

    let link_form = if *link_form_open.read() {
        let mut submit_state = state;
        let mut submit_generation = autosave_generation;
        let mut submit_open = link_form_open;
        let mut submit_value = link_value;
        let cancel = editor_action_button(
            state,
            "cancel-link",
            "×",
            "Cancel link",
            true,
            false,
            None,
            30.,
            true,
            palette,
            move |_| {
                *link_form_open.write() = false;
                *link_value.write() = String::new();
            },
        );
        Some(
            rect()
                .width(Size::fill())
                .height(Size::px(42.))
                .horizontal()
                .spacing(8.)
                .padding(Gaps::new(4., 24., 4., 24.))
                .background(theme::color(palette.surface))
                .a11y_alt("Link destination editor")
                .child(
                    Input::new(link_value)
                        .width(Size::px(300.))
                        .placeholder("https://")
                        .on_submit(move |value: String| {
                            let result = submit_state
                                .write()
                                .editor
                                .as_mut()
                                .ok_or_else(|| "cannot link without an open note".to_string())
                                .and_then(|editor| editor.link_selection(&value).map(|_| ()));
                            match result {
                                Ok(()) => {
                                    *submit_generation.write() += 1;
                                    *submit_open.write() = false;
                                    *submit_value.write() = String::new();
                                    eprintln!(
                                        "[freya][editor] action:complete action=link destination={value}"
                                    );
                                }
                                Err(error) => {
                                    eprintln!(
                                        "[freya][editor] action:failure action=link error={error}"
                                    );
                                    submit_state.write().error = Some(error);
                                }
                            }
                        }),
                )
                .child(cancel)
                .into_element(),
        )
    } else {
        None
    };

    let document = editor.session().document();
    let document_view = render_document(
        state,
        document,
        autosave_generation,
        palette,
        content_scale,
    );
    let error_view = snapshot.error.map(|error| {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(6.))
            .background(theme::color(palette.danger))
            .a11y_alt("Editor error")
            .child(label().color(theme::color(palette.surface)).text(error))
            .into_element()
    });

    let mut topbar = rect()
        .width(Size::fill())
        .height(Size::px(topbar_height))
        .horizontal()
        .spacing(8.)
        .center()
        .padding(Gaps::new(0., 12., 0., 12.))
        .background(theme::color(palette.bg))
        .a11y_alt(if compact {
            "Editor topbar compact"
        } else {
            "Editor topbar"
        })
        .child(
            rect()
                .width(Size::fill())
                .child(
                    label()
                        .font_size(if compact { 19. } else { 28. })
                        .line_height(1.2)
                        .font_family(EDITOR_FONT_FAMILY)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(palette.text))
                        .max_lines(1)
                        .text(note_title.clone()),
                ),
        );
    if let Some(date) = note_date {
        topbar = topbar.child(passive_chip(state, "date", date, palette, true));
    }
    for (index, tag) in tags.iter().take(2).enumerate() {
        let id = if index == 0 { "tag-1" } else { "tag-2" };
        topbar = topbar.child(passive_chip(
            state,
            id,
            format!("#{tag}"),
            palette,
            false,
        ));
    }
    if tags.len() > 2 {
        topbar = topbar.child(passive_chip(
            state,
            "tag-more",
            format!("+{}", tags.len() - 2),
            palette,
            true,
        ));
    }
    topbar = topbar
        .child(undo)
        .child(redo)
        .child(save)
        .child(pin)
        .child(close);

    let topbar_separator = rect()
        .width(Size::fill())
        .height(Size::px(1.))
        .horizontal()
        .background(theme::color(palette.bg))
        .child(rect().width(Size::px(12.)))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(1.))
                .background(theme::color(theme::mix(
                    palette.border,
                    palette.bg,
                    0.42,
                ))),
        )
        .child(rect().width(Size::px(12.)));

    let toolbar = rect()
        .width(Size::fill())
        .height(Size::px(EDITOR_TOOLBAR_HEIGHT))
        .horizontal()
        .spacing(14.)
        .padding(Gaps::new(0., 24., 0., 24.))
        .cross_align(Alignment::Center)
        .background(theme::color(palette.surface))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .a11y_alt("Editor toolbar")
        .child(heading)
        .child(bold)
        .child(italic)
        .child(strike)
        .child(link)
        .child(bullets)
        .child(ordered)
        .child(task)
        .child(inline_code)
        .child(quote)
        .child(code_block);

    let editor_body = rect()
        .width(Size::fill())
        .child(document_view);

    let centered_body = rect()
        .width(Size::fill())
        .center()
        .child(
            rect()
                .width(Size::fill())
                .max_width(Size::px(EDITOR_CONTENT_MAX))
                .padding(Gaps::new(24., 20., 36., 20.))
                .child(editor_body),
        );

    let scale_label = format!("{}%", (content_scale * 100.).round() as i32);
    let scale_down = {
        let mut text_scale = text_scale;
        editor_action_button(
            state,
            "text-scale-down",
            "A−",
            "Decrease editor text size",
            content_scale > 0.85,
            false,
            None,
            36.,
            true,
            palette,
            move |_| {
                *text_scale.write() = (*text_scale.read() - 0.1).clamp(0.85, 1.3);
            },
        )
    };
    let scale_reset = {
        let mut text_scale = text_scale;
        editor_action_button(
            state,
            "text-scale-reset",
            scale_label,
            "Reset editor text size",
            true,
            (content_scale - 1.0).abs() < f32::EPSILON,
            Some(palette.primary),
            52.,
            true,
            palette,
            move |_| {
                *text_scale.write() = 1.0;
            },
        )
    };
    let scale_up = {
        let mut text_scale = text_scale;
        editor_action_button(
            state,
            "text-scale-up",
            "A+",
            "Increase editor text size",
            content_scale < 1.3,
            false,
            None,
            36.,
            true,
            palette,
            move |_| {
                *text_scale.write() = (*text_scale.read() + 0.1).clamp(0.85, 1.3);
            },
        )
    };
    let theme_toggle = {
        let dark = *editor_dark_mode.read();
        let mut editor_dark_mode = editor_dark_mode;
        editor_action_button(
            state,
            "theme-toggle",
            if dark { "☀" } else { "☾" },
            if dark {
                "Use light editor theme"
            } else {
                "Use dark editor theme"
            },
            true,
            dark,
            Some(palette.primary),
            36.,
            true,
            palette,
            move |_| {
                let next = !*editor_dark_mode.read();
                *editor_dark_mode.write() = next;
                eprintln!(
                    "[freya][editor] action:complete action=editor-theme mode={}",
                    if next { "dark" } else { "light" }
                );
            },
        )
    };

    let footer = rect()
        .width(Size::fill())
        .height(Size::px(EDITOR_FOOTER_HEIGHT))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(0., 24., 0., 24.))
        .background(theme::color(palette.surface))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .a11y_alt("Editor footer")
        .child(
            rect()
                .horizontal()
                .spacing(12.)
                .child(
                    label()
                        .font_size(12.)
                        .font_family(EDITOR_FONT_FAMILY)
                        .color(theme::color(palette.muted))
                        .text(format!("{word_count} words")),
                )
                .child(
                    label()
                        .font_size(12.)
                        .font_family(EDITOR_FONT_FAMILY)
                        .color(theme::color(palette.muted))
                        .text(format!("{char_count} characters")),
                )
                .child(
                    label()
                        .font_size(12.)
                        .font_family(EDITOR_FONT_FAMILY)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(if dirty {
                            palette.primary
                        } else {
                            palette.muted
                        }))
                        .text(if dirty { "Unsaved changes" } else { "Saved" }),
                ),
        )
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(scale_down)
                .child(scale_reset)
                .child(scale_up)
                .child(theme_toggle),
        );

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::color(palette.surface))
        .child(topbar)
        .child(topbar_separator)
        .maybe_child(error_view)
        .child(toolbar)
        .maybe_child(link_form)
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .background(theme::color(palette.surface))
                .a11y_alt("Editor scroll")
                .child(
                    ScrollView::new_controlled(scroll_controller)
                        .width(Size::fill())
                        .height(Size::fill())
                        .child(centered_body),
                ),
        )
        .child(footer)
        .a11y_alt("NoteEditorHost")
        .into_element()
}

fn render_document(
    state: State<ShellState>,
    document: &Document,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let children = document
        .children(document.root)
        .filter(|node| !matches!(node.kind, NodeKind::Block(BlockKind::FrontMatter { .. })))
        .map(|node| {
            render_block(
                state,
                document,
                node.id,
                autosave_generation,
                palette,
                text_scale,
            )
        })
        .collect::<Vec<_>>();
    rect()
        .width(Size::fill())
        .spacing(12.)
        .children(children)
        .into_element()
}

fn render_block_children(
    state: State<ShellState>,
    document: &Document,
    parent: NodeId,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let children = document
        .children(parent)
        .map(|node| {
            render_block(
                state,
                document,
                node.id,
                autosave_generation,
                palette,
                text_scale,
            )
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(12.)
        .children(children)
        .into_element()
}

fn render_block(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let Some(node) = document.node(node_id) else {
        return label().text("[Muya node unavailable]").into_element();
    };

    match &node.kind {
        NodeKind::Block(BlockKind::Paragraph) => render_inline_block(
            state,
            autosave_generation,
            node_id,
            "Paragraph",
            BlockTextStyle::default(),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::Heading { level }) => render_inline_block(
            state,
            autosave_generation,
            node_id,
            &format!("Heading {level}"),
            BlockTextStyle {
                font_size: heading_font_size(*level),
                line_height: 1.3,
                bold: true,
                ..BlockTextStyle::default()
            },
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::BlockQuote) => rect()
            .width(Size::fill())
            .horizontal()
            .spacing(12.)
            .child(
                rect()
                    .width(Size::px(3.))
                    .height(Size::fill())
                    .background(theme::color(palette.border_strong))
                    .with_corner_radius(2.),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(4.))
                    .color(theme::color(palette.muted))
                    .child(render_block_children(
                        state,
                        document,
                        node_id,
                        autosave_generation,
                        palette,
                        text_scale,
                    )),
            )
            .a11y_alt("Block quote")
            .into_element(),
        NodeKind::Block(BlockKind::List { kind, start }) => render_list(
            state,
            document,
            node_id,
            *kind,
            start.unwrap_or(1),
            autosave_generation,
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::ListItem { .. }) => render_block_children(
            state,
            document,
            node_id,
            autosave_generation,
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::CodeBlock { language, .. }) => {
            let mut children = Vec::new();
            if let Some(language) = language.as_deref().filter(|value| !value.is_empty()) {
                children.push(
                    label()
                        .font_size(11. * text_scale)
                        .line_height(1.25)
                        .font_family(EDITOR_CODE_FONT_FAMILY)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(palette.muted))
                        .text(language.to_string())
                        .into_element(),
                );
            }
            children.push(render_inline_block(
                state,
                autosave_generation,
                node_id,
                "Code block",
                BlockTextStyle {
                    font_size: 14.,
                    line_height: 1.45,
                    code: true,
                    color: theme::TEXT,
                    ..BlockTextStyle::default()
                },
                palette,
                text_scale,
            ));
            rect()
                .width(Size::fill())
                .spacing(8.)
                .padding(Gaps::new_all(12.))
                .background(theme::color(palette.soft))
                .border(Border::new().fill(theme::color(palette.border)).width(1.))
                .with_corner_radius(8.)
                .children(children)
                .a11y_alt("Code block")
                .into_element()
        }
        NodeKind::Block(BlockKind::ThematicBreak) => rect()
            .width(Size::fill())
            .height(Size::px(1.))
            .background(theme::color(palette.border_strong))
            .a11y_alt("Thematic break")
            .into_element(),
        NodeKind::Block(BlockKind::Table) => rect()
            .width(Size::fill())
            .spacing(4.)
            .padding(Gaps::new_all(6.))
            .border(Border::new().fill(theme::color(palette.border)).width(1.))
            .with_corner_radius(7.)
            .children(
                document
                    .children(node_id)
                    .map(|child| {
                        render_block(
                            state,
                            document,
                            child.id,
                            autosave_generation,
                            palette,
                            text_scale,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .a11y_alt("Table")
            .into_element(),
        NodeKind::Block(BlockKind::TableRow) => rect()
            .width(Size::fill())
            .horizontal()
            .spacing(4.)
            .children(
                document
                    .children(node_id)
                    .map(|child| {
                        render_block(
                            state,
                            document,
                            child.id,
                            autosave_generation,
                            palette,
                            text_scale,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .into_element(),
        NodeKind::Block(BlockKind::TableCell { header, .. }) => rect()
            .width(Size::fill())
            .padding(Gaps::new_all(6.))
            .border(Border::new().fill(theme::color(palette.border)).width(1.))
            .child(if *header {
                render_inline_block(
                    state,
                    autosave_generation,
                    node_id,
                    "Table header cell",
                    BlockTextStyle {
                        bold: true,
                        ..BlockTextStyle::default()
                    },
                    palette,
                    text_scale,
                )
            } else {
                render_inline_block(
                    state,
                    autosave_generation,
                    node_id,
                    "Table cell",
                    BlockTextStyle::default(),
                    palette,
                    text_scale,
                )
            })
            .into_element(),
        NodeKind::Block(BlockKind::HtmlBlock) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            "HTML block not rendered",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::MathBlock) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            "Math block not rendered",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::FrontMatter { .. }) => rect().into_element(),
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Footnote definition not rendered: {label}"),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Reference definition not rendered: {label}"),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Diagram not rendered: {language}"),
            palette,
            text_scale,
        ),
        NodeKind::Document | NodeKind::Inline(_) => render_block_children(
            state,
            document,
            node_id,
            autosave_generation,
            palette,
            text_scale,
        ),
    }
}

struct TextDelta {
    start_utf16: u32,
    end_utf16: u32,
    inserted: String,
}

fn text_delta(before: &str, after: &str) -> Option<TextDelta> {
    if before == after {
        return None;
    }
    let before_chars = before.chars().collect::<Vec<_>>();
    let after_chars = after.chars().collect::<Vec<_>>();
    let mut prefix = 0;
    while prefix < before_chars.len()
        && prefix < after_chars.len()
        && before_chars[prefix] == after_chars[prefix]
    {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < before_chars.len().saturating_sub(prefix)
        && suffix < after_chars.len().saturating_sub(prefix)
        && before_chars[before_chars.len() - suffix - 1]
            == after_chars[after_chars.len() - suffix - 1]
    {
        suffix += 1;
    }
    let start_utf16 = before_chars[..prefix]
        .iter()
        .map(|character| character.len_utf16() as u32)
        .sum();
    let end_utf16 = before_chars[..before_chars.len() - suffix]
        .iter()
        .map(|character| character.len_utf16() as u32)
        .sum();
    Some(TextDelta {
        start_utf16,
        end_utf16,
        inserted: after_chars[prefix..after_chars.len() - suffix]
            .iter()
            .collect(),
    })
}

fn editable_text_nodes(document: &Document, parent: NodeId, nodes: &mut Vec<(NodeId, String)>) {
    for child in document.children(parent) {
        match &child.kind {
            NodeKind::Inline(InlineKind::Text { value }) => {
                nodes.push((child.id, value.clone()));
            }
            NodeKind::Inline(InlineKind::CodeSpan { code }) => {
                nodes.push((child.id, code.clone()));
            }
            NodeKind::Document | NodeKind::Block(_) | NodeKind::Inline(_) => {
                editable_text_nodes(document, child.id, nodes);
            }
        }
    }
}

pub(super) fn has_table_ancestor(document: &Document, mut node_id: NodeId) -> bool {
    loop {
        let Some(node) = document.node(node_id) else {
            return false;
        };
        if matches!(node.kind, NodeKind::Block(BlockKind::Table)) {
            return true;
        }
        let Some(parent) = node.parent else {
            return false;
        };
        node_id = parent;
    }
}

pub(super) fn contains_node(document: &Document, root: NodeId, target: NodeId) -> bool {
    if root == target {
        return true;
    }
    document.node(root).is_some_and(|node| {
        node.children
            .iter()
            .any(|child| contains_node(document, *child, target))
    })
}

pub(super) fn sync_muya_selection(
    mut state: State<ShellState>,
    block_id: NodeId,
    editable: &UseEditable,
) -> Result<(), String> {
    let (start, end) = {
        let editor = editable.editor().read();
        (
            editor.selection().start() as u32,
            editor.selection().end() as u32,
        )
    };
    let desired = {
        let snapshot = state.read();
        let editor = snapshot
            .editor
            .as_ref()
            .ok_or_else(|| "cannot update selection without an open note".to_string())?;
        let mut nodes = Vec::new();
        editable_text_nodes(editor.session().document(), block_id, &mut nodes);
        let (start, end) =
            normalize_selection_offsets(&nodes, start, end, editor.session().snapshot().selection);
        let anchor = point_at_offset(&nodes, start, false)
            .ok_or_else(|| "editable block has no Muya text target".to_string())?;
        let focus = point_at_offset(&nodes, end, start != end)
            .ok_or_else(|| "editable block has no Muya selection target".to_string())?;
        Selection { anchor, focus }
    };
    let mut shell = state.write();
    let editor = shell
        .editor
        .as_mut()
        .ok_or_else(|| "cannot update selection without an open note".to_string())?;
    editor
        .set_selection(desired)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(super) fn sync_editable_from_muya(
    state: State<ShellState>,
    block_id: NodeId,
    editable: &mut UseEditable,
) {
    let (value, selection) = {
        let snapshot = state.read();
        let Some(editor) = snapshot.editor.as_ref() else {
            return;
        };
        let document = editor.session().document();
        if document.node(block_id).is_none() {
            return;
        }
        let mut spans = Vec::new();
        collect_inline_children(
            document,
            block_id,
            InlineStyle::default(),
            theme::LIGHT_PALETTE,
            1.0,
            &mut spans,
        );
        let value = spans
            .iter()
            .map(|span| span.text.as_ref())
            .collect::<String>();
        let mut nodes = Vec::new();
        editable_text_nodes(document, block_id, &mut nodes);
        let muya_selection = editor.session().snapshot().selection;
        let selection = block_offset_for_selection(&nodes, muya_selection);
        (value, selection)
    };

    let mut inner = editable.editor_mut().write();
    if inner.committed_text() != value {
        inner.set(&value);
    }
    if let Some((anchor, focus)) = selection {
        if anchor == focus {
            inner.move_cursor_to(anchor as usize);
        } else {
            inner.set_selection((anchor as usize, focus as usize));
        }
    } else {
        inner.clear_selection();
    }
}

fn point_at_offset(
    nodes: &[(NodeId, String)],
    offset_utf16: u32,
    is_end: bool,
) -> Option<SelectionPoint> {
    let mut cursor = 0u32;
    for (index, (node, value)) in nodes.iter().enumerate() {
        let end = cursor + value.encode_utf16().count() as u32;
        if offset_utf16 < end || (offset_utf16 == end && (!is_end || index + 1 == nodes.len())) {
            let local = utf16_boundary_ceil(value, offset_utf16.saturating_sub(cursor));
            return Some(SelectionPoint {
                node: *node,
                offset_utf16: local,
            });
        }
        if offset_utf16 == end && is_end {
            if let Some((next, _)) = nodes.get(index + 1) {
                return Some(SelectionPoint {
                    node: *next,
                    offset_utf16: 0,
                });
            }
        }
        cursor = end;
    }
    nodes.last().map(|(node, value)| SelectionPoint {
        node: *node,
        offset_utf16: value.encode_utf16().count() as u32,
    })
}

fn utf16_boundary_ceil(value: &str, offset_utf16: u32) -> u32 {
    let mut cursor = 0u32;
    for character in value.chars() {
        if offset_utf16 <= cursor {
            return cursor;
        }
        cursor += character.len_utf16() as u32;
        if offset_utf16 <= cursor {
            return cursor;
        }
    }
    cursor
}

fn utf16_boundary_floor(value: &str, offset_utf16: u32) -> u32 {
    let mut cursor = 0u32;
    for character in value.chars() {
        if offset_utf16 <= cursor {
            return cursor;
        }
        let next = cursor + character.len_utf16() as u32;
        if offset_utf16 < next {
            return cursor;
        }
        cursor = next;
    }
    cursor
}

fn normalize_selection_offsets(
    nodes: &[(NodeId, String)],
    start: u32,
    end: u32,
    current: Selection,
) -> (u32, u32) {
    let value = nodes
        .iter()
        .map(|(_, value)| value.as_str())
        .collect::<String>();
    if start != end {
        return (
            utf16_boundary_floor(&value, start),
            utf16_boundary_ceil(&value, end),
        );
    }

    let floor = utf16_boundary_floor(&value, start);
    let ceil = utf16_boundary_ceil(&value, start);
    if floor == ceil {
        return (start, end);
    }

    let current_offset = block_offset_for_selection(nodes, current)
        .filter(|(anchor, focus)| anchor == focus)
        .map(|(anchor, _)| anchor);
    if current_offset.is_some_and(|offset| offset > start) {
        (floor, floor)
    } else {
        (ceil, ceil)
    }
}

fn block_offset_for_selection(
    nodes: &[(NodeId, String)],
    selection: Selection,
) -> Option<(u32, u32)> {
    Some((
        block_offset_for_point(nodes, selection.anchor)?,
        block_offset_for_point(nodes, selection.focus)?,
    ))
}

fn block_offset_for_point(nodes: &[(NodeId, String)], point: SelectionPoint) -> Option<u32> {
    let mut cursor = 0u32;
    for (node, value) in nodes {
        if *node == point.node {
            return Some(cursor + point.offset_utf16);
        }
        cursor += value.encode_utf16().count() as u32;
    }
    None
}

pub(super) fn apply_inline_delta(
    mut state: State<ShellState>,
    block_id: NodeId,
    before: &str,
    after: &str,
) -> Result<(), String> {
    let delta = text_delta(before, after).ok_or_else(|| "empty editor delta".to_string())?;
    let mut shell = state.write();
    let editor = shell
        .editor
        .as_mut()
        .ok_or_else(|| "cannot edit without an open note".to_string())?;
    let mut nodes = Vec::new();
    editable_text_nodes(editor.session().document(), block_id, &mut nodes);
    let flattened = nodes
        .iter()
        .map(|(_, value)| value.as_str())
        .collect::<String>();
    if flattened != before {
        return Err("editor changed while the previous keystroke was pending".to_string());
    }
    let (start_node_index, start_offset) = segment_at_offset(&nodes, delta.start_utf16)
        .ok_or_else(|| "editor selection has no Muya text target".to_string())?;
    let (end_node_index, end_offset) = segment_at_offset(&nodes, delta.end_utf16)
        .ok_or_else(|| "editor selection has no Muya text target".to_string())?;
    let (start_node, _current) = &nodes[start_node_index];
    let (end_node, _current) = &nodes[end_node_index];
    editor
        .set_selection(Selection {
            anchor: SelectionPoint {
                node: *start_node,
                offset_utf16: start_offset,
            },
            focus: SelectionPoint {
                node: *end_node,
                offset_utf16: end_offset,
            },
        })
        .map_err(|error| error.to_string())?;
    editor
        .dispatch_text(delta.inserted)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn segment_at_offset(segments: &[(NodeId, String)], offset_utf16: u32) -> Option<(usize, u32)> {
    let mut cursor = 0;
    for (index, (_, text)) in segments.iter().enumerate() {
        let end = cursor + text.encode_utf16().count() as u32;
        if offset_utf16 <= end {
            return Some((index, offset_utf16 - cursor));
        }
        cursor = end;
    }
    segments
        .last()
        .map(|(_, text)| (segments.len() - 1, text.encode_utf16().count() as u32))
}

#[allow(clippy::too_many_arguments)]
fn render_list(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    kind: ListKind,
    start: u64,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let items = document
        .children(node_id)
        .enumerate()
        .map(|(index, item)| {
            let marker = list_marker(kind, start.saturating_add(index as u64), item.id, document);
            let marker_view = if kind == ListKind::Task {
                let checked = task_checked(item.id, document);
                editor_interactions::task_marker(
                    state,
                    autosave_generation,
                    item.id,
                    checked,
                    marker,
                )
            } else {
                label()
                    .font_size(EDITOR_BODY_SIZE * text_scale)
                    .line_height(EDITOR_BODY_LINE_HEIGHT)
                    .font_family(EDITOR_FONT_FAMILY)
                    .color(theme::color(palette.muted))
                    .text(marker)
                    .into_element()
            };
            rect()
                .width(Size::fill())
                .horizontal()
                .spacing(8.)
                .child(marker_view)
                .child(render_block_children(
                    state,
                    document,
                    item.id,
                    autosave_generation,
                    palette,
                    text_scale,
                ))
                .into_element()
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(7.)
        .children(items)
        .a11y_alt(list_accessibility_label(kind))
        .into_element()
}

fn task_checked(item_id: NodeId, document: &Document) -> bool {
    document
        .node(item_id)
        .and_then(|node| match &node.kind {
            NodeKind::Block(BlockKind::ListItem { checked }) => *checked,
            _ => None,
        })
        .unwrap_or(false)
}

fn list_marker(kind: ListKind, number: u64, item_id: NodeId, document: &Document) -> String {
    match kind {
        ListKind::Unordered => "•".to_string(),
        ListKind::Ordered => format!("{number}."),
        ListKind::Task => {
            let checked = document
                .node(item_id)
                .and_then(|node| match &node.kind {
                    NodeKind::Block(BlockKind::ListItem { checked }) => *checked,
                    _ => None,
                })
                .unwrap_or(false);
            if checked {
                "☑".to_string()
            } else {
                "☐".to_string()
            }
        }
    }
}

fn list_accessibility_label(kind: ListKind) -> &'static str {
    match kind {
        ListKind::Unordered => "Unordered list",
        ListKind::Ordered => "Ordered list",
        ListKind::Task => "Task list",
    }
}

fn render_inline_block(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    node_id: NodeId,
    accessibility_label: &str,
    style: BlockTextStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    EditableInlineBlock {
        state,
        autosave_generation,
        node_id,
        accessibility_label: accessibility_label.to_string(),
        style,
        palette,
        text_scale,
    }
    .into_element()
}

fn render_unsupported_block(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    node_id: NodeId,
    message: &str,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    rect()
        .width(Size::fill())
        .spacing(4.)
        .padding(Gaps::new_all(8.))
        .background(theme::color(palette.soft))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .with_corner_radius(7.)
        .child(
            label()
                .font_size(13. * text_scale)
                .font_family(EDITOR_FONT_FAMILY)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(palette.danger))
                .text(message.to_string()),
        )
        .child(render_inline_block(
            state,
            autosave_generation,
            node_id,
            message,
            BlockTextStyle {
                code: true,
                color: theme::MUTED,
                ..BlockTextStyle::default()
            },
            palette,
            text_scale,
        ))
        .a11y_alt(message.to_string())
        .into_element()
}

fn collect_inline_children(
    document: &Document,
    parent: NodeId,
    style: InlineStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
    spans: &mut Vec<Span<'static>>,
) {
    for child in document.children(parent) {
        collect_inline_node(document, child.id, style, palette, text_scale, spans);
    }
}

fn collect_inline_node(
    document: &Document,
    node_id: NodeId,
    style: InlineStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
    spans: &mut Vec<Span<'static>>,
) {
    let Some(node) = document.node(node_id) else {
        return;
    };

    match &node.kind {
        NodeKind::Inline(kind) => match kind {
            InlineKind::Text { value } => spans.push(styled_span(
                value.clone(),
                style,
                palette,
                text_scale,
            )),
            InlineKind::Escaped { value } => spans.push(styled_span(
                value.to_string(),
                style,
                palette,
                text_scale,
            )),
            InlineKind::Emphasis => {
                let mut next = style;
                next.emphasis = true;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::Strong => {
                let mut next = style;
                next.strong = true;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::Strike => {
                let mut next = style;
                next.strike = true;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::MarkFragment { mark, .. } => {
                let mut next = style;
                match mark {
                    InlineMarkKind::Emphasis => next.emphasis = true,
                    InlineMarkKind::Strong => next.strong = true,
                    InlineMarkKind::Strike => next.strike = true,
                }
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::CodeSpan { code } => {
                let mut next = style;
                next.code = true;
                spans.push(styled_span(code.clone(), next, palette, text_scale));
            }
            InlineKind::Link { .. } => {
                let mut next = style;
                next.link = true;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::Image { source, alt, .. } => {
                let text = format!("[Image non rendue: alt=\"{alt}\" source=\"{source}\"]");
                let mut next = style;
                next.status = true;
                spans.push(styled_span(text, next, palette, text_scale));
            }
            InlineKind::AutoLink { destination } => {
                let mut next = style;
                next.link = true;
                spans.push(styled_span(
                    destination.clone(),
                    next,
                    palette,
                    text_scale,
                ));
            }
            InlineKind::InlineHtml { raw } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[HTML inline non rendu: {raw}]"),
                    next,
                    palette,
                    text_scale,
                ));
            }
            InlineKind::InlineMath { source } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[Math inline non rendu: {source}]"),
                    next,
                    palette,
                    text_scale,
                ));
            }
            InlineKind::Emoji { value, .. } => spans.push(styled_span(
                value.clone(),
                style,
                palette,
                text_scale,
            )),
            InlineKind::Superscript => {
                let mut next = style;
                next.script = ScriptStyle::Superscript;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::Subscript => {
                let mut next = style;
                next.script = ScriptStyle::Subscript;
                collect_inline_children(document, node_id, next, palette, text_scale, spans);
            }
            InlineKind::FootnoteReference { label } => {
                spans.push(styled_span(
                    format!("[footnote: {label}]"),
                    style,
                    palette,
                    text_scale,
                ));
            }
            InlineKind::SoftBreak | InlineKind::HardBreak => {
                spans.push(styled_span(
                    "\n".to_string(),
                    style,
                    palette,
                    text_scale,
                ));
            }
        },
        NodeKind::Document | NodeKind::Block(_) => {
            collect_inline_children(document, node_id, style, palette, text_scale, spans)
        }
    }
}

fn styled_span(
    value: String,
    style: InlineStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Span<'static> {
    let color = if style.status {
        palette.danger
    } else if style.link {
        palette.primary
    } else {
        palette.text
    };
    let mut span = Span::new(value).color(theme::color(color));
    if style.strong {
        span = span.font_weight(FontWeight::BOLD);
    }
    if style.emphasis {
        span = span.font_slant(FontSlant::Italic);
    }
    if style.strike {
        span = span.text_decoration(TextDecoration::LineThrough);
    }
    if style.code {
        span = span.font_family(EDITOR_CODE_FONT_FAMILY);
    }
    match style.script {
        ScriptStyle::Normal => {}
        ScriptStyle::Superscript | ScriptStyle::Subscript => {
            span = span.font_size(12. * text_scale);
        }
    }
    span
}

fn editor_color(palette: theme::ThemePalette, legacy: (u8, u8, u8, u8)) -> Color {
    if legacy == theme::TEXT {
        theme::color(palette.text)
    } else if legacy == theme::MUTED {
        theme::color(palette.muted)
    } else if legacy == theme::PRIMARY {
        theme::color(palette.primary)
    } else if legacy == theme::DANGER {
        theme::color(palette.danger)
    } else if legacy == theme::SOFT {
        theme::color(palette.soft)
    } else {
        theme::color(legacy)
    }
}

fn heading_font_size(level: u8) -> f32 {
    match level {
        1 => 28.,
        2 => 24.,
        3 => 20.,
        4 => 18.,
        5 => 17.,
        _ => 16.,
    }
}

fn editor_relative_path(snapshot: &ShellState, path: Option<&Path>) -> Option<String> {
    let root = snapshot.vault.as_ref()?.root();
    let relative = path?.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

fn frontmatter(markdown: &str) -> Option<&str> {
    let normalized = markdown.strip_prefix("\u{feff}").unwrap_or(markdown);
    let body = normalized.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    Some(&body[..end])
}

fn document_title(markdown: &str, fallback: &str) -> String {
    let mut in_frontmatter = markdown.starts_with("---\n");
    for line in markdown.lines() {
        if in_frontmatter {
            if line == "---" && line.as_ptr() != markdown.as_ptr() {
                in_frontmatter = false;
            }
            continue;
        }
        if let Some(title) = line.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    fallback.to_string()
}

fn document_created_at(markdown: &str) -> Option<String> {
    let frontmatter = frontmatter(markdown)?;
    frontmatter.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == "createdAt" || key.trim() == "created_at")
            .then(|| trim_yaml_scalar(value))
            .filter(|value| !value.is_empty())
    })
}

fn document_tags(markdown: &str) -> Vec<String> {
    let Some(frontmatter) = frontmatter(markdown) else {
        return Vec::new();
    };
    let lines = frontmatter.lines().collect::<Vec<_>>();
    let Some(index) = lines.iter().position(|line| {
        line.split_once(':')
            .is_some_and(|(key, _)| key.trim() == "tags")
    }) else {
        return Vec::new();
    };
    let (_, raw) = lines[index]
        .split_once(':')
        .expect("tags line already validated");
    let raw = raw.trim();
    let mut tags = if raw.starts_with('[') && raw.ends_with(']') {
        raw[1..raw.len() - 1]
            .split(',')
            .map(trim_yaml_scalar)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
    } else if !raw.is_empty() {
        raw.split(',')
            .map(trim_yaml_scalar)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if raw.is_empty() {
        for line in lines.iter().skip(index + 1) {
            let trimmed = line.trim();
            if let Some(tag) = trimmed.strip_prefix("- ") {
                let tag = trim_yaml_scalar(tag);
                if !tag.is_empty() {
                    tags.push(tag);
                }
                continue;
            }
            if !trimmed.is_empty() {
                break;
            }
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn trim_yaml_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn short_date(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 10 {
        let date = &trimmed[..10];
        if date.as_bytes().get(4) == Some(&b'-') && date.as_bytes().get(7) == Some(&b'-') {
            return date.to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorDocument, EditorSession};

    fn spans_for(markdown: &str) -> Vec<Span<'static>> {
        let session = EditorSession::from_markdown(markdown);
        let document = session.document();
        let mut spans = Vec::new();
        collect_inline_children(
            document,
            document.root,
            InlineStyle::default(),
            theme::LIGHT_PALETTE,
            1.0,
            &mut spans,
        );
        spans
    }

    #[test]
    fn default_body_typography_matches_editor_reference_scale() {
        let style = BlockTextStyle::default();
        assert_eq!(style.font_size, 16.);
        assert_eq!(style.line_height, 1.58);
        assert_eq!(style.color, theme::TEXT);
        assert!(!style.bold);
        assert!(!style.code);
    }

    #[test]
    fn heading_scale_tracks_note_editor_header_and_content_hierarchy() {
        assert_eq!(heading_font_size(1), 28.);
        assert_eq!(heading_font_size(2), 24.);
        assert_eq!(heading_font_size(3), 20.);
        assert!(heading_font_size(1) > heading_font_size(2));
        assert!(heading_font_size(2) > heading_font_size(3));
    }

    #[test]
    fn metadata_matches_tauri_title_tags_and_created_at_sources() {
        let markdown = "---\ncreatedAt: 2026-08-12T08:30:00Z\ntags: [rust, editor]\n---\n# Reference note\n\nBody";
        assert_eq!(document_title(markdown, "Fallback"), "Reference note");
        assert_eq!(document_created_at(markdown).as_deref(), Some("2026-08-12T08:30:00Z"));
        assert_eq!(document_tags(markdown), vec!["editor", "rust"]);
        assert_eq!(short_date("2026-08-12T08:30:00Z"), "2026-08-12");
    }

    #[test]
    fn multiline_frontmatter_tags_are_supported() {
        let markdown = "---\ntags:\n  - alpha\n  - beta\ncreatedAt: '2026-01-03'\n---\n# Note";
        assert_eq!(document_tags(markdown), vec!["alpha", "beta"]);
        assert_eq!(document_created_at(markdown).as_deref(), Some("2026-01-03"));
    }

    #[test]
    fn renders_muya_inline_nodes_as_styled_freya_spans() {
        let spans =
            spans_for("# **bold** *emphasis* ~~strike~~ `code` [link](https://example.com)");
        let text = spans
            .iter()
            .map(|span| span.text.as_ref())
            .collect::<String>();

        assert!(text.contains("bold"));
        assert!(text.contains("emphasis"));
        assert!(text.contains("strike"));
        assert!(text.contains("code"));
        assert!(text.contains("link"));
        assert!(spans.iter().any(|span| span.text == "bold"));
    }

    #[test]
    fn exposes_non_rendered_muya_content_as_explicit_states() {
        let mut document = Document::new();
        let paragraph = document.allocate(NodeKind::Block(BlockKind::Paragraph), None);
        document.append_child(document.root, paragraph);
        let image = document.allocate(
            NodeKind::Inline(InlineKind::Image {
                source: "diagram.png".to_string(),
                title: None,
                alt: "diagram".to_string(),
            }),
            None,
        );
        let html = document.allocate(
            NodeKind::Inline(InlineKind::InlineHtml {
                raw: "<span>raw</span>".to_string(),
            }),
            None,
        );
        let math = document.allocate(
            NodeKind::Inline(InlineKind::InlineMath {
                source: "x^2".to_string(),
            }),
            None,
        );
        document.append_child(paragraph, image);
        document.append_child(paragraph, html);
        document.append_child(paragraph, math);

        let mut spans = Vec::new();
        collect_inline_children(
            &document,
            document.root,
            InlineStyle::default(),
            theme::LIGHT_PALETTE,
            1.0,
            &mut spans,
        );
        let text = spans
            .iter()
            .map(|span| span.text.as_ref())
            .collect::<String>();

        assert!(text.contains("Image non rendue"));
        assert!(text.contains("HTML inline non rendu"));
        assert!(text.contains("Math inline non rendu"));
    }

    #[test]
    fn render_document_walks_structured_blocks_without_using_markdown_lines() {
        let markdown =
            "# Heading\n\n> quote\n\n- item\n- [x] done\n\n```rust\nlet x = 1;\n```\n\n---";
        let session = EditorSession::from_markdown(markdown);
        let runner = freya_testing::prelude::launch_test(move || {
            let state = use_state(move || {
                let mut shell = ShellState::empty();
                shell.editor = Some(EditorDocument::from_markdown(markdown));
                shell
            });
            note_editor_host(state)
        });
        let block_kinds = session
            .document()
            .children(session.document().root)
            .filter_map(|node| match &node.kind {
                NodeKind::Block(kind) => Some(kind),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(block_kinds
            .iter()
            .any(|kind| matches!(kind, BlockKind::Heading { level: 1 })));
        assert!(block_kinds
            .iter()
            .any(|kind| matches!(kind, BlockKind::BlockQuote)));
        assert!(block_kinds
            .iter()
            .any(|kind| matches!(kind, BlockKind::List { .. })));
        assert!(block_kinds
            .iter()
            .any(|kind| matches!(kind, BlockKind::CodeBlock { .. })));
        assert!(block_kinds
            .iter()
            .any(|kind| matches!(kind, BlockKind::ThematicBreak)));
        for label in [
            "Editor topbar",
            "Editor toolbar",
            "Editor footer",
            "Heading 1",
            "Block quote",
            "Unordered list",
            "Code block",
            "Thematic break",
            "Heading 2",
            "Bulleted list",
            "Numbered list",
            "Task list",
            "Quote",
        ] {
            assert!(
                runner
                    .find(|_, element| {
                        (element.accessibility().builder.label() == Some(label)).then_some(())
                    })
                    .is_some(),
                "editor surface element {label:?} must be present in the Freya tree"
            );
        }
    }

    #[test]
    fn editor_delta_uses_utf16_offsets_for_non_bmp_text() {
        let delta = text_delta("A😀B", "A😃B").expect("the emoji replacement is an edit");
        assert_eq!(delta.start_utf16, 1);
        assert_eq!(delta.end_utf16, 3);
        assert_eq!(delta.inserted, "😃");
    }
}
