//! Native Freya rendering for Elephant's real Muya-backed editor surface.
//!
//! Tauri's NoteEditorHost/TopBar/Toolbar/Footer remain the visual and
//! behavioral reference. The editable paragraphs below always operate on the
//! real `EditorDocument`; the chrome never substitutes a display-only buffer.

use freya::{
    clipboard::Clipboard,
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
#[path = "editor_links.rs"]
mod editor_links;
#[path = "editor_tag_interactions.rs"]
mod editor_tag_interactions;

const CONTENT_MAX: f32 = 780.;
const BODY_SIZE: f32 = 16.;
const BODY_LINE_HEIGHT: f32 = 1.58;
const TOPBAR_HEIGHT: f32 = 52.;
const COMPACT_TOPBAR_HEIGHT: f32 = 36.;
const TOPBAR_ACTION: f32 = 30.;
const TOOLBAR_HEIGHT: f32 = 56.;
const TOOLBAR_ACTION: f32 = 34.;
const FOOTER_HEIGHT: f32 = 50.;
const UI_FONT: &str = "sans-serif";
const CODE_FONT: &str = "monospace";
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
}

impl Default for BlockTextStyle {
    fn default() -> Self {
        Self {
            font_size: BODY_SIZE,
            line_height: BODY_LINE_HEIGHT,
            bold: false,
            italic: false,
            code: false,
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
                contains_node(editor.session().document(), self.node_id, target)
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

        let mut paragraph = paragraph()
            .a11y_id(a11y_id)
            .width(Size::fill())
            .holder(holder.read().clone())
            .a11y_focusable(true)
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
            .font_family(if self.style.code { CODE_FONT } else { UI_FONT })
            .font_size(self.style.font_size * self.text_scale)
            .line_height(self.style.line_height)
            .color(theme::color(self.palette.text));
        if self.style.bold {
            paragraph = paragraph.font_weight(FontWeight::BOLD);
        }
        if self.style.italic {
            paragraph = paragraph.font_slant(FontSlant::Italic);
        }
        paragraph.into_element()
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

fn dispatch_toolbar_action(
    mut state: State<ShellState>,
    autosave_generation: State<u64>,
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

#[allow(clippy::too_many_arguments)]
fn action_button(
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
                .font_family(UI_FONT)
                .font_size(if text.chars().count() > 2 { 11. } else { 14. })
                .font_weight(FontWeight::BOLD)
                .color(theme::color(if enabled {
                    foreground
                } else {
                    palette.muted
                }))
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
        let mut enter_state = state;
        let enter_key = hover_key.clone();
        let mut leave_state = state;
        let mut down_state = state;
        let down_key = pressed_key;
        let mut up_state = state;
        let up_key = hover_key;
        button = button
            .on_pointer_enter(move |_| {
                enter_state.write().set_hovered_target(enter_key.clone());
            })
            .on_pointer_leave(move |_| {
                leave_state.write().hovered_target = None;
            })
            .on_mouse_down(move |_| {
                down_state.write().set_hovered_target(down_key.clone());
            })
            .on_mouse_up(move |event| {
                up_state.write().set_hovered_target(up_key.clone());
                on_press(event);
            });
    }
    button.into_element()
}

fn submit_tag_edit(
    mut state: State<ShellState>,
    current_tags: Vec<String>,
    title: String,
    mut draft: State<String>,
    mut editing_index: State<Option<usize>>,
    mut form_open: State<bool>,
) {
    let tag = draft.read().trim().to_owned();
    if tag.is_empty() {
        *form_open.write() = false;
        *draft.write() = String::new();
        *editing_index.write() = None;
        return;
    }
    let mut next_tags = current_tags;
    if let Some(index) = *editing_index.read() {
        if index < next_tags.len() {
            next_tags[index] = tag;
        } else {
            next_tags.push(tag);
        }
    } else if !next_tags.iter().any(|current| current == &tag) {
        next_tags.push(tag);
    }
    let result = editor_tag_interactions::persist_tags(state.clone(), &next_tags, &title);
    match result {
        Ok(()) => {
            *form_open.write() = false;
            *draft.write() = String::new();
            *editing_index.write() = None;
            eprintln!("[freya][editor] action=update-tags status=complete");
        }
        Err(error) => state.write().error = Some(error),
    }
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
    let mut enter_state = state;
    let enter_key = hover_key;
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
        .on_pointer_enter(move |_| {
            enter_state.write().set_hovered_target(enter_key.clone());
        })
        .on_pointer_leave(move |_| {
            leave_state.write().hovered_target = None;
        })
        .child(
            label()
                .font_family(UI_FONT)
                .font_size(14.)
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

fn editable_tag_chip(
    state: State<ShellState>,
    interaction_id: &'static str,
    index: usize,
    text: String,
    current_tags: Vec<String>,
    title: String,
    palette: theme::ThemePalette,
    form_open: State<bool>,
    draft: State<String>,
    editing_index: State<Option<usize>>,
) -> Element {
    let hover_key = format!("editor-hover:{interaction_id}");
    let hovered = state.read().hovered_target.as_deref() == Some(hover_key.as_str());
    let mut enter_state = state;
    let enter_key = hover_key;
    let mut leave_state = state;
    let mut open_state = form_open;
    let mut draft_state = draft;
    let mut editing_state = editing_index;
    let delete_state = state;
    let delete_tags = current_tags;
    let delete_title = title;
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
        .on_pointer_enter(move |_| {
            enter_state.write().set_hovered_target(enter_key.clone());
        })
        .on_pointer_leave(move |_| {
            leave_state.write().hovered_target = None;
        })
        .on_mouse_up(move |event: Event<MouseEventData>| {
            event.stop_propagation();
            if event.button == Some(MouseButton::Right) {
                editor_tag_interactions::delete_tag(
                    delete_state,
                    &delete_tags,
                    &delete_title,
                    index,
                );
                return;
            }
            *open_state.write() = true;
            *draft_state.write() = delete_tags.get(index).cloned().unwrap_or_default();
            *editing_state.write() = Some(index);
        })
        .child(
            label()
                .font_family(UI_FONT)
                .font_size(14.)
                .color(theme::color(palette.text))
                .text(text.clone()),
        )
        // Preserve the Tauri chip's accessible name (`#tag`) while the
        // pointer handlers provide the edit/delete gestures.
        .a11y_alt(text)
        .into_element()
}

fn render_note_editor_host(mut state: State<ShellState>) -> Element {
    // Hooks are intentionally unconditional. Conditional sub-UI is rendered
    // after every hook has been registered, matching Freya's hook rules.
    let autosave_generation = use_state(|| 0_u64);
    let link_form_open = use_state(|| false);
    let link_value = use_state(String::new);
    let text_scale = use_state(|| 1.0_f32);
    let editor_dark_mode = use_state(|| false);
    let tag_form_open = use_state(|| false);
    let tag_draft = use_state(String::new);
    let tag_edit_index = use_state(|| None::<usize>);
    let tag_input_a11y_id = use_a11y();
    let tag_should_focus = *tag_form_open.read();
    use_side_effect(move || {
        if tag_should_focus {
            tag_input_a11y_id.request_focus();
        }
    });

    let generation_for_effect = autosave_generation;
    let state_for_effect = state;
    use_side_effect(move || {
        let generation = *generation_for_effect.read();
        let (enabled, dirty, delay) = {
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
        if !enabled || !dirty {
            return;
        }
        let mut save_state = state_for_effect;
        let generation_state = generation_for_effect;
        spawn(async move {
            Delay::new(delay).await;
            if *generation_state.read() != generation {
                return;
            }
            let result = {
                let mut shell = save_state.write();
                shell
                    .editor
                    .as_mut()
                    .filter(|editor| editor.autosave_due())
                    .map_or(Ok(()), |editor| editor.save())
            };
            if let Err(error) = result {
                save_state.write().error = Some(error.to_string());
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
        let mut position_state = scroll_position;
        let mut notifier = scroll_notifier;
        let mut shell_state = state;
        Callback::new(move |event: ScrollEvent| {
            let (changed, y) = {
                let mut position = position_state.write();
                let previous = *position;
                match event {
                    ScrollEvent::X(x) => position.0 = x,
                    ScrollEvent::Y(y) => position.1 = y,
                }
                (previous != *position, position.1)
            };
            if changed {
                let scroll_top = y.saturating_neg();
                if let Some(editor) = shell_state.write().editor.as_mut() {
                    editor.set_scroll_top(scroll_top);
                }
                notifier.write();
            }
            changed
        })
    });
    let get_scroll = use_state(|| {
        let position_state = scroll_position;
        Callback::new(move |_| *position_state.read())
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
    let editor_snapshot = editor.snapshot();
    let dirty = editor.is_dirty();
    let markdown = editor.serialize();
    let word_count = markdown.split_whitespace().count();
    let char_count = markdown.chars().count();
    let fallback_title = editor
        .path()
        .and_then(|path| path.file_stem())
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("Untitled")
        .to_string();
    let title = document_title(&markdown, &fallback_title);
    let editor_path_key = editor
        .path()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let title_value = use_state(|| title.clone());
    let title_a11y_id = use_a11y();
    let mut title_sync_state = title_value;
    let title_path_state = use_state(|| editor_path_key.clone());
    let mut title_path_sync_state = title_path_state;
    let rendered_title = title.clone();
    let rendered_path = editor_path_key.clone();
    use_side_effect(move || {
        if title_path_sync_state.read().as_str() != rendered_path.as_str() {
            title_path_sync_state.set(rendered_path.clone());
            title_sync_state.set(rendered_title.clone());
        }
    });
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
    let date = document_created_at(&markdown)
        .or_else(|| library_entry.map(|entry| entry.updated_at.as_str().to_string()))
        .map(|value| short_date(&value));
    let pinned = relative_path.as_deref().is_some_and(|path| {
        snapshot
            .library
            .pinned_paths
            .iter()
            .any(|candidate| candidate.as_str() == path)
    });
    let has_selection = !editor_snapshot.selection.is_collapsed();

    let title_for_submit = title.clone();
    let title_for_save_original = title.clone();
    let title_for_close_original = title.clone();
    let mut title_state = title_value;
    let title_input = Input::new(title_value)
        .width(Size::fill())
        .a11y_id(title_a11y_id)
        .on_submit(move |next_title: String| {
            let next_title = next_title.trim().to_owned();
            if next_title.is_empty() {
                title_state.set(title_for_submit.clone());
                return;
            }
            let result = {
                let mut shell = state.write();
                shell
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot rename without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .rename_title(&next_title)
                            .map_err(|error| error.to_string())
                            .and_then(|_| editor.save().map_err(|error| error.to_string()))
                    })
            };
            match result {
                Ok(()) => {
                    title_state.set(next_title);
                    eprintln!("[freya][editor] action=rename-title status=complete");
                }
                Err(error) => state.write().error = Some(error),
            }
        });

    let add_tag = {
        let mut open = tag_form_open;
        let mut draft = tag_draft;
        let mut editing = tag_edit_index;
        action_button(
            state,
            "add-tag",
            "+",
            "Add tag",
            true,
            *tag_form_open.read(),
            Some(palette.primary),
            TOPBAR_ACTION,
            true,
            palette,
            move |_| {
                *open.write() = true;
                *draft.write() = String::new();
                *editing.write() = None;
            },
        )
    };
    let tag_form = if *tag_form_open.read() {
        let mut open = tag_form_open;
        let mut draft = tag_draft;
        let mut editing = tag_edit_index;
        let submit_state = state;
        let submit_tags = tags.clone();
        let submit_title = title.clone();
        let cancel = action_button(
            state,
            "cancel-tag",
            "×",
            "Cancel tag",
            true,
            false,
            None,
            26.,
            true,
            palette,
            move |_| {
                *open.write() = false;
                *draft.write() = String::new();
                *editing.write() = None;
            },
        );
        let save_state = state;
        let save_tags = submit_tags.clone();
        let save_title = submit_title.clone();
        let save = action_button(
            state,
            "save-tag",
            "✓",
            "Save",
            true,
            false,
            Some(palette.primary),
            26.,
            true,
            palette,
            move |_| {
                submit_tag_edit(
                    save_state,
                    save_tags.clone(),
                    save_title.clone(),
                    tag_draft,
                    tag_edit_index,
                    tag_form_open,
                );
            },
        );
        let submit_state = submit_state;
        let submit_tags = submit_tags;
        let submit_title = submit_title;
        Some(
            rect()
                .height(Size::px(30.))
                .horizontal()
                .spacing(4.)
                .a11y_alt("Tag editor")
                .child(
                    rect()
                        .width(Size::px(110.))
                        .a11y_alt("Tag")
                        .on_mouse_up(move |_| tag_input_a11y_id.request_focus())
                        .child(
                            Input::new(tag_draft)
                                .width(Size::fill())
                                .a11y_id(tag_input_a11y_id)
                                .auto_focus(true)
                                .placeholder("Tag")
                                .on_submit(move |_| {
                                    submit_tag_edit(
                                        submit_state,
                                        submit_tags.clone(),
                                        submit_title.clone(),
                                        tag_draft,
                                        tag_edit_index,
                                        tag_form_open,
                                    );
                                }),
                        ),
                )
                .child(save)
                .child(cancel)
                .into_element(),
        )
    } else {
        None
    };

    let undo = action_button(
        state,
        "undo",
        "↶",
        "Undo",
        editor_snapshot.can_undo,
        false,
        None,
        TOPBAR_ACTION,
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
    let redo = action_button(
        state,
        "redo",
        "↷",
        "Redo",
        editor_snapshot.can_redo,
        false,
        None,
        TOPBAR_ACTION,
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
    let title_for_save = title_value;
    let fallback_title_for_save = fallback_title.clone();
    let save = action_button(
        state,
        "save",
        "✓",
        "Save",
        true,
        dirty,
        Some(palette.primary),
        TOPBAR_ACTION,
        true,
        palette,
        move |_| {
            let requested_title = title_for_save.read().trim().to_owned();
            let requested_title = if requested_title.is_empty() {
                fallback_title_for_save.clone()
            } else {
                requested_title
            };
            let result = state
                .write()
                .editor
                .as_mut()
                .ok_or_else(|| "cannot save without an open note".to_string())
                .and_then(|editor| {
                    if requested_title != title_for_save_original {
                        editor
                            .rename_title(&requested_title)
                            .map_err(|error| error.to_string())?;
                    }
                    editor.save().map_err(|error| error.to_string())
                });
            if let Err(error) = result {
                state.write().error = Some(error);
            }
        },
    );
    let pin_path = relative_path.clone();
    let pin = action_button(
        state,
        "pin",
        "⌖",
        if pinned { "Unpin note" } else { "Pin note" },
        pin_path.is_some(),
        pinned,
        Some(PIN_ACTIVE),
        TOPBAR_ACTION,
        true,
        palette,
        move |_| {
            let Some(path) = pin_path.as_deref() else {
                return;
            };
            let mut shell = state.write();
            shell.toggle_pinned(RelativePath::new(path));
        },
    );
    let title_for_close = title_value;
    let fallback_title_for_close = fallback_title.clone();
    let close = action_button(
        state,
        "close",
        "×",
        "Close note",
        true,
        false,
        None,
        TOPBAR_ACTION,
        true,
        palette,
        move |_| {
            let result = {
                let mut shell = state.write();
                let result = shell.editor.as_mut().map_or_else(
                    || Err("cannot close without an open note".to_string()),
                    |editor| {
                        let requested_title = title_for_close.read().trim().to_owned();
                        let requested_title = if requested_title.is_empty() {
                            fallback_title_for_close.clone()
                        } else {
                            requested_title
                        };
                        if requested_title != title_for_close_original {
                            editor
                                .rename_title(&requested_title)
                                .map_err(|error| error.to_string())?;
                        }
                        editor.close().map_err(|error| error.to_string())
                    },
                );
                if result.is_ok() {
                    shell.editor = None;
                }
                result
            };
            if let Err(error) = result {
                state.write().error = Some(error);
            }
        },
    );

    let bold = toolbar_command_button(
        state,
        autosave_generation,
        "bold",
        "B",
        "Bold",
        EditorAction::ToggleStrong,
        palette,
    );
    let italic = toolbar_command_button(
        state,
        autosave_generation,
        "italic",
        "I",
        "Italic",
        EditorAction::ToggleEmphasis,
        palette,
    );
    let strike = toolbar_command_button(
        state,
        autosave_generation,
        "strike",
        "S",
        "Strikethrough",
        EditorAction::ToggleStrike,
        palette,
    );
    let heading = toolbar_command_button(
        state,
        autosave_generation,
        "heading-2",
        "H2",
        "Heading 2",
        EditorAction::SetHeading(2),
        palette,
    );
    let bullets = toolbar_command_button(
        state,
        autosave_generation,
        "bullet-list",
        "•",
        "Bulleted list",
        EditorAction::SetListKind(ListKind::Unordered),
        palette,
    );
    let ordered = toolbar_command_button(
        state,
        autosave_generation,
        "ordered-list",
        "1.",
        "Numbered list",
        EditorAction::SetListKind(ListKind::Ordered),
        palette,
    );
    let task = toolbar_command_button(
        state,
        autosave_generation,
        "task-list",
        "☑",
        "Task list",
        EditorAction::SetListKind(ListKind::Task),
        palette,
    );
    let quote = toolbar_command_button(
        state,
        autosave_generation,
        "quote",
        "❞",
        "Quote",
        EditorAction::ToggleBlockQuote,
        palette,
    );
    let code_block = toolbar_command_button(
        state,
        autosave_generation,
        "code-block",
        "{}",
        "Code block",
        EditorAction::ToggleCodeBlock,
        palette,
    );
    let inline_code = action_button(
        state,
        "inline-code",
        "</>",
        if has_selection {
            "Inline code"
        } else {
            "Inline code requires a selection"
        },
        has_selection,
        false,
        None,
        TOOLBAR_ACTION,
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
    let link = {
        let mut open_state = link_form_open;
        action_button(
            state,
            "link",
            "↗",
            if has_selection {
                "Link"
            } else {
                "Link requires a selection"
            },
            has_selection,
            *link_form_open.read(),
            Some(palette.primary),
            TOOLBAR_ACTION,
            false,
            palette,
            move |_| {
                let next = !*open_state.read();
                *open_state.write() = next;
            },
        )
    };

    let link_form = if *link_form_open.read() {
        let mut submit_state = state;
        let mut submit_generation = autosave_generation;
        let mut submit_open = link_form_open;
        let mut submit_value = link_value;
        let cancel = {
            let mut open = link_form_open;
            let mut value = link_value;
            action_button(
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
                    *open.write() = false;
                    *value.write() = String::new();
                },
            )
        };
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
                                }
                                Err(error) => submit_state.write().error = Some(error),
                            }
                        }),
                )
                .child(cancel)
                .into_element(),
        )
    } else {
        None
    };

    let document_view = render_document(
        state,
        editor.session().document(),
        autosave_generation,
        palette,
        content_scale,
    );
    let link_actions = render_link_actions(
        state,
        editor_links::collect_links(editor.session().document()),
        palette,
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
        .height(Size::px(if compact {
            COMPACT_TOPBAR_HEIGHT
        } else {
            TOPBAR_HEIGHT
        }))
        .horizontal()
        .spacing(8.)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(0., 12., 0., 12.))
        .background(theme::color(palette.bg))
        .a11y_alt(if compact {
            "Editor topbar compact"
        } else {
            "Editor topbar"
        })
        .child(
            rect()
                .width(Size::flex(1.))
                .a11y_alt("Note title")
                .on_mouse_up(move |_| title_a11y_id.request_focus())
                .child(title_input),
        );
    let mut topbar_actions = rect()
        .width(Size::px(340.))
        .height(Size::fill())
        .horizontal()
        .spacing(8.)
        .main_align(Alignment::End)
        .cross_align(Alignment::Center);
    if let Some(date) = date {
        topbar_actions = topbar_actions.child(passive_chip(state, "date", date, palette, true));
    }
    for (index, tag) in tags.iter().take(2).enumerate() {
        topbar_actions = topbar_actions.child(editable_tag_chip(
            state,
            if index == 0 { "tag-1" } else { "tag-2" },
            index,
            format!("#{tag}"),
            tags.clone(),
            title.clone(),
            palette,
            tag_form_open,
            tag_draft,
            tag_edit_index,
        ));
    }
    if tags.len() > 2 {
        topbar_actions = topbar_actions.child(passive_chip(
            state,
            "tag-more",
            format!("+{}", tags.len() - 2),
            palette,
            true,
        ));
    }
    if let Some(tag_form) = tag_form {
        topbar_actions = topbar_actions.child(tag_form);
    } else {
        topbar_actions = topbar_actions.child(add_tag);
    }
    topbar_actions = topbar_actions
        .child(undo)
        .child(redo)
        .child(save)
        .child(pin)
        .child(close);
    topbar = topbar.child(topbar_actions);

    let toolbar = rect()
        .width(Size::fill())
        .height(Size::px(TOOLBAR_HEIGHT))
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

    let centered_body = rect().width(Size::fill()).center().child(
        rect()
            .width(Size::fill())
            .max_width(Size::px(CONTENT_MAX))
            .padding(Gaps::new(24., 20., 36., 20.))
            .child(document_view),
    );

    let scale_down = {
        let mut scale = text_scale;
        action_button(
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
                let next = (*scale.read() - 0.1).clamp(0.85, 1.3);
                *scale.write() = next;
            },
        )
    };
    let scale_reset = {
        let mut scale = text_scale;
        action_button(
            state,
            "text-scale-reset",
            format!("{}%", (content_scale * 100.).round() as i32),
            "Reset editor text size",
            true,
            (content_scale - 1.0).abs() < f32::EPSILON,
            Some(palette.primary),
            52.,
            true,
            palette,
            move |_| *scale.write() = 1.0,
        )
    };
    let scale_up = {
        let mut scale = text_scale;
        action_button(
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
                let next = (*scale.read() + 0.1).clamp(0.85, 1.3);
                *scale.write() = next;
            },
        )
    };
    let theme_toggle = {
        let dark = *editor_dark_mode.read();
        let mut dark_state = editor_dark_mode;
        action_button(
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
                let next = !*dark_state.read();
                *dark_state.write() = next;
            },
        )
    };

    let footer = rect()
        .width(Size::fill())
        .height(Size::px(FOOTER_HEIGHT))
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
                .child(status_label(format!("{word_count} words"), palette, false))
                .child(status_label(
                    format!("{char_count} characters"),
                    palette,
                    false,
                ))
                .child(status_label(
                    if dirty {
                        "Unsaved changes".to_string()
                    } else {
                        "Saved".to_string()
                    },
                    palette,
                    dirty,
                )),
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
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(1.))
                .background(theme::color(palette.border)),
        )
        .maybe_child(error_view)
        .child(toolbar)
        .maybe_child(link_form)
        .maybe_child(link_actions)
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

fn render_link_actions(
    state: State<ShellState>,
    links: Vec<editor_links::MarkdownLink>,
    palette: theme::ThemePalette,
) -> Option<Element> {
    if links.is_empty() {
        return None;
    }
    let mut controls = rect()
        .width(Size::fill())
        .padding(Gaps::new(6., 24., 6., 24.))
        .horizontal()
        .spacing(6.)
        .background(theme::color(palette.surface))
        .a11y_alt("Editor links");
    for link in links {
        let destination = link.destination.clone();
        let label_text = if link.label.is_empty() {
            destination.clone()
        } else {
            link.label
        };
        let accessibility_label = format!("Open link {label_text}");
        let press_destination = destination.clone();
        controls = controls.child(
            rect()
                .padding(Gaps::new(5., 8., 5., 8.))
                .with_corner_radius(6.)
                .background(theme::color(palette.soft))
                .a11y_alt(accessibility_label)
                .on_mouse_up(move |_| editor_links::activate(state, &press_destination))
                .child(
                    label()
                        .color(theme::color(palette.primary))
                        .text(label_text),
                ),
        );
    }
    Some(controls.into_element())
}

fn toolbar_command_button(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    interaction_id: &'static str,
    text: &'static str,
    label: &'static str,
    action: EditorAction,
    palette: theme::ThemePalette,
) -> Element {
    action_button(
        state,
        interaction_id,
        text,
        label,
        true,
        false,
        None,
        TOOLBAR_ACTION,
        false,
        palette,
        move |_| {
            dispatch_toolbar_action(state, autosave_generation, action.clone(), interaction_id)
        },
    )
}

fn status_label(text: String, palette: theme::ThemePalette, accent: bool) -> Element {
    label()
        .font_family(UI_FONT)
        .font_size(12.)
        .font_weight(if accent {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        })
        .color(theme::color(if accent {
            palette.primary
        } else {
            palette.muted
        }))
        .text(text)
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
    rect()
        .width(Size::fill())
        .spacing(12.)
        .children(
            document
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
                .collect::<Vec<_>>(),
        )
        .into_element()
}

fn render_code_block(
    mut state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    language: Option<&str>,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let mut nodes = Vec::new();
    editable_text_nodes(document, node_id, &mut nodes);
    let code = nodes
        .iter()
        .map(|(_, value)| value.as_str())
        .collect::<String>();
    let language = language.unwrap_or("text").to_owned();
    let execution = state.read().code_execution.clone();
    let running = execution.running && execution.block_id == Some(node_id);
    let has_output = execution.block_id == Some(node_id)
        && (!execution.output.is_empty() || execution.error.is_some());

    let copy_code = code.clone();
    let copy = action_button(
        state,
        "copy-code-block",
        "Copy",
        "Copy code block",
        !copy_code.is_empty(),
        false,
        None,
        62.,
        true,
        palette,
        move |_| {
            if let Err(error) = Clipboard::set(copy_code.clone()) {
                state.write().error = Some(format!("Unable to copy code block: {error:?}"));
            } else {
                eprintln!("[freya][editor] action=copy-code-block node={node_id:?}");
            }
        },
    );
    let run_code = code.clone();
    let run_language = language.clone();
    let run = action_button(
        state,
        "run-code-block",
        if running { "Running…" } else { "Run" },
        if running {
            "Code block is running"
        } else {
            "Run code block"
        },
        !running && !run_code.is_empty(),
        running,
        Some(palette.primary),
        72.,
        true,
        palette,
        move |_| {
            let Some(root) = state
                .read()
                .vault
                .as_ref()
                .map(|vault| vault.root().to_path_buf())
            else {
                state.write().error = Some("Cannot run code without an active vault.".to_owned());
                return;
            };
            {
                let mut shell = state.write();
                shell.error = None;
                shell.code_execution = super::code_execution::CodeExecutionState {
                    block_id: Some(node_id),
                    language: run_language.clone(),
                    running: true,
                    output: format!("Running {run_language}…"),
                    exit_code: None,
                    error: None,
                };
            }
            let execution_state = state;
            let language_for_execution = run_language.clone();
            let code_for_execution = run_code.clone();
            spawn(async move {
                let result = super::code_execution::execute(
                    &root,
                    &language_for_execution,
                    &code_for_execution,
                );
                let mut execution_state = execution_state;
                let mut shell = execution_state.write();
                shell.code_execution.running = false;
                match result {
                    Ok(result) => {
                        shell.code_execution.output = if result.output.is_empty() {
                            format!("Exited with code {}", result.exit_code)
                        } else {
                            result.output
                        };
                        shell.code_execution.exit_code = Some(result.exit_code);
                        eprintln!(
                            "[freya][editor] action=run-code-block-complete node={node_id:?}"
                        );
                    }
                    Err(error) => {
                        eprintln!("[freya][editor] action=run-code-block-failure node={node_id:?} error={error}");
                        shell.code_execution.error = Some(error.clone());
                        shell.code_execution.output.clear();
                        shell.code_execution.exit_code = Some("error".to_owned());
                    }
                }
            });
        },
    );

    let controls = rect()
        .width(Size::fill())
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .font_family(CODE_FONT)
                .font_size(11. * text_scale)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(palette.muted))
                .text(language),
        )
        .child(rect().horizontal().spacing(6.).child(copy).child(run));
    let mut frame = rect()
        .width(Size::fill())
        .spacing(8.)
        .padding(Gaps::new_all(12.))
        .background(theme::color(palette.soft))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .with_corner_radius(8.)
        .child(controls)
        .child(editable_block(
            state,
            autosave_generation,
            node_id,
            "Code block",
            BlockTextStyle {
                font_size: 14.,
                line_height: 1.45,
                code: true,
                ..BlockTextStyle::default()
            },
            palette,
            text_scale,
        ));
    if has_output {
        let output = execution.error.unwrap_or(execution.output);
        frame = frame.child(
            label()
                .font_family(CODE_FONT)
                .font_size(12. * text_scale)
                .color(theme::color(
                    if execution.exit_code.as_deref() == Some("0") {
                        palette.text
                    } else {
                        palette.danger
                    },
                ))
                .a11y_alt("Code execution output")
                .text(output),
        );
    }
    frame.a11y_alt("Code block").into_element()
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
        NodeKind::Block(BlockKind::Paragraph) => editable_block(
            state,
            autosave_generation,
            node_id,
            "Paragraph",
            BlockTextStyle::default(),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::Heading { level }) => editable_block(
            state,
            autosave_generation,
            node_id,
            &format!("Heading {level}"),
            BlockTextStyle {
                font_size: heading_size(*level),
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
            .child(render_block_children(
                state,
                document,
                node_id,
                autosave_generation,
                palette,
                text_scale,
            ))
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
        NodeKind::Block(BlockKind::CodeBlock { language, .. }) => render_code_block(
            state,
            document,
            node_id,
            language.as_deref().filter(|value| !value.is_empty()),
            autosave_generation,
            palette,
            text_scale,
        ),
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
            .child(editable_block(
                state,
                autosave_generation,
                node_id,
                if *header {
                    "Table header cell"
                } else {
                    "Table cell"
                },
                BlockTextStyle {
                    bold: *header,
                    ..BlockTextStyle::default()
                },
                palette,
                text_scale,
            ))
            .into_element(),
        NodeKind::Block(BlockKind::FrontMatter { .. }) => rect().into_element(),
        NodeKind::Block(BlockKind::HtmlBlock) => unsupported_block(
            state,
            autosave_generation,
            node_id,
            "HTML block not rendered",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::MathBlock) => unsupported_block(
            state,
            autosave_generation,
            node_id,
            "Math block not rendered",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Footnote definition not rendered: {label}"),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Reference definition not rendered: {label}"),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => unsupported_block(
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

fn editable_block(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    node_id: NodeId,
    label: &str,
    style: BlockTextStyle,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    EditableInlineBlock {
        state,
        autosave_generation,
        node_id,
        accessibility_label: label.to_string(),
        style,
        palette,
        text_scale,
    }
    .into_element()
}

fn unsupported_block(
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
                .font_family(UI_FONT)
                .font_size(13.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(palette.danger))
                .text(message.to_string()),
        )
        .child(editable_block(
            state,
            autosave_generation,
            node_id,
            message,
            BlockTextStyle {
                code: true,
                ..BlockTextStyle::default()
            },
            palette,
            text_scale,
        ))
        .a11y_alt(message.to_string())
        .into_element()
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
                editor_interactions::task_marker(
                    state,
                    autosave_generation,
                    item.id,
                    task_checked(item.id, document),
                    marker,
                )
            } else {
                label()
                    .font_family(UI_FONT)
                    .font_size(BODY_SIZE * text_scale)
                    .line_height(BODY_LINE_HEIGHT)
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
        .a11y_alt(match kind {
            ListKind::Unordered => "Unordered list",
            ListKind::Ordered => "Ordered list",
            ListKind::Task => "Task list",
        })
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
        ListKind::Task => if task_checked(item_id, document) {
            "☑"
        } else {
            "☐"
        }
        .to_string(),
    }
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
            InlineKind::Text { value } => {
                spans.push(styled_span(value.clone(), style, palette, text_scale))
            }
            InlineKind::Escaped { value } => {
                spans.push(styled_span(value.to_string(), style, palette, text_scale))
            }
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
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[Image non rendue: alt=\"{alt}\" source=\"{source}\"]"),
                    next,
                    palette,
                    text_scale,
                ));
            }
            InlineKind::AutoLink { destination } => {
                let mut next = style;
                next.link = true;
                spans.push(styled_span(destination.clone(), next, palette, text_scale));
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
            InlineKind::Emoji { value, .. } => {
                spans.push(styled_span(value.clone(), style, palette, text_scale))
            }
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
            InlineKind::FootnoteReference { label } => spans.push(styled_span(
                format!("[footnote: {label}]"),
                style,
                palette,
                text_scale,
            )),
            InlineKind::SoftBreak | InlineKind::HardBreak => {
                spans.push(styled_span("\n".to_string(), style, palette, text_scale))
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
    let mut span = Span::new(value).color(theme::color(if style.status {
        palette.danger
    } else if style.link {
        palette.primary
    } else {
        palette.text
    }));
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
        span = span.font_family(CODE_FONT);
    }
    if !matches!(style.script, ScriptStyle::Normal) {
        span = span.font_size(12. * text_scale);
    }
    span
}

fn heading_size(level: u8) -> f32 {
    match level {
        1 => 28.,
        2 => 24.,
        3 => 20.,
        4 => 18.,
        5 => 17.,
        _ => 16.,
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
            NodeKind::Inline(InlineKind::Text { value }) => nodes.push((child.id, value.clone())),
            NodeKind::Inline(InlineKind::CodeSpan { code }) => nodes.push((child.id, code.clone())),
            NodeKind::Document | NodeKind::Block(_) | NodeKind::Inline(_) => {
                editable_text_nodes(document, child.id, nodes)
            }
        }
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
    state
        .write()
        .editor
        .as_mut()
        .ok_or_else(|| "cannot update selection without an open note".to_string())?
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
        let mut nodes = Vec::new();
        editable_text_nodes(document, block_id, &mut nodes);
        let value = nodes
            .iter()
            .map(|(_, value)| value.as_str())
            .collect::<String>();
        let selection = block_offset_for_selection(&nodes, editor.session().snapshot().selection);
        (value, selection)
    };
    let mut inner = editable.editor_mut().write();
    if inner.committed_text() != value {
        inner.set(&value);
    }
    match selection {
        Some((anchor, focus)) if anchor == focus => inner.move_cursor_to(anchor as usize),
        Some((anchor, focus)) => inner.set_selection((anchor as usize, focus as usize)),
        None => inner.clear_selection(),
    }
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
    let (start_index, start_offset) = segment_at_offset(&nodes, delta.start_utf16)
        .ok_or_else(|| "editor selection has no Muya text target".to_string())?;
    let (end_index, end_offset) = segment_at_offset(&nodes, delta.end_utf16)
        .ok_or_else(|| "editor selection has no Muya text target".to_string())?;
    editor
        .set_selection(Selection {
            anchor: SelectionPoint {
                node: nodes[start_index].0,
                offset_utf16: start_offset,
            },
            focus: SelectionPoint {
                node: nodes[end_index].0,
                offset_utf16: end_offset,
            },
        })
        .map_err(|error| error.to_string())?;
    editor
        .dispatch_text(delta.inserted)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn segment_at_offset(nodes: &[(NodeId, String)], offset: u32) -> Option<(usize, u32)> {
    let mut cursor = 0;
    for (index, (_, value)) in nodes.iter().enumerate() {
        let end = cursor + value.encode_utf16().count() as u32;
        if offset <= end {
            return Some((index, offset - cursor));
        }
        cursor = end;
    }
    nodes
        .last()
        .map(|(_, value)| (nodes.len() - 1, value.encode_utf16().count() as u32))
}

fn point_at_offset(
    nodes: &[(NodeId, String)],
    offset: u32,
    is_end: bool,
) -> Option<SelectionPoint> {
    let mut cursor = 0u32;
    for (index, (node, value)) in nodes.iter().enumerate() {
        let end = cursor + value.encode_utf16().count() as u32;
        if offset < end || (offset == end && (!is_end || index + 1 == nodes.len())) {
            return Some(SelectionPoint {
                node: *node,
                offset_utf16: utf16_boundary_ceil(value, offset.saturating_sub(cursor)),
            });
        }
        if offset == end && is_end {
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

fn utf16_boundary_floor(value: &str, offset: u32) -> u32 {
    let mut cursor = 0;
    for character in value.chars() {
        if offset <= cursor {
            return cursor;
        }
        let next = cursor + character.len_utf16() as u32;
        if offset < next {
            return cursor;
        }
        cursor = next;
    }
    cursor
}

fn utf16_boundary_ceil(value: &str, offset: u32) -> u32 {
    let mut cursor = 0;
    for character in value.chars() {
        if offset <= cursor {
            return cursor;
        }
        cursor += character.len_utf16() as u32;
        if offset <= cursor {
            return cursor;
        }
    }
    cursor
}

fn editor_relative_path(snapshot: &ShellState, path: Option<&Path>) -> Option<String> {
    let root = snapshot.vault.as_ref()?.root();
    let relative = path?.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

fn frontmatter(markdown: &str) -> Option<&str> {
    let normalized = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let body = normalized.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    Some(&body[..end])
}

fn document_title(markdown: &str, fallback: &str) -> String {
    let body = if markdown.starts_with("---\n") {
        markdown
            .find("\n---")
            .map(|end| &markdown[end + 4..])
            .unwrap_or(markdown)
    } else {
        markdown
    };
    body.lines()
        .find_map(|line| {
            line.strip_prefix("# ")
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn document_created_at(markdown: &str) -> Option<String> {
    frontmatter(markdown)?.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        matches!(key.trim(), "createdAt" | "created_at")
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
    let raw = lines[index]
        .split_once(':')
        .map(|(_, raw)| raw.trim())
        .unwrap_or("");
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
                let value = trim_yaml_scalar(tag);
                if !value.is_empty() && !tags.contains(&value) {
                    tags.push(value);
                }
            } else if !trimmed.is_empty() {
                break;
            }
        }
    }
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
    let value = value.trim();
    if value.len() >= 10 {
        let date = &value[..10];
        if date.as_bytes().get(4) == Some(&b'-') && date.as_bytes().get(7) == Some(&b'-') {
            return date.to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_reference_typography_is_explicit() {
        assert_eq!(BlockTextStyle::default().font_size, 16.);
        assert_eq!(BlockTextStyle::default().line_height, 1.58);
        assert_eq!(heading_size(1), 28.);
        assert_eq!(heading_size(2), 24.);
        assert_eq!(heading_size(3), 20.);
    }

    #[test]
    fn metadata_reads_tauri_frontmatter_shape_without_rendering_it() {
        let markdown = "---\ncreatedAt: 2026-08-12T08:30:00Z\ntags: [rust, editor]\n---\n# Reference note\n\nBody";
        assert_eq!(document_title(markdown, "Fallback"), "Reference note");
        assert_eq!(
            document_created_at(markdown).as_deref(),
            Some("2026-08-12T08:30:00Z")
        );
        assert_eq!(document_tags(markdown), vec!["rust", "editor"]);
        assert_eq!(short_date("2026-08-12T08:30:00Z"), "2026-08-12");
    }

    #[test]
    fn multiline_tags_preserve_source_order() {
        let markdown = "---\ntags:\n  - alpha\n  - beta\ncreatedAt: '2026-01-03'\n---\n# Note";
        assert_eq!(document_tags(markdown), vec!["alpha", "beta"]);
    }

    #[test]
    fn editor_delta_uses_utf16_offsets_for_astral_text() {
        let delta = text_delta("A😀B", "A😃B").expect("emoji replacement is an edit");
        assert_eq!(delta.start_utf16, 1);
        assert_eq!(delta.end_utf16, 3);
        assert_eq!(delta.inserted, "😃");
    }
}
