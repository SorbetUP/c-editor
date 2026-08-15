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
use std::time::{SystemTime, UNIX_EPOCH};
use muya_core::{
    model::{BlockKind, InlineKind, InlineMarkKind, ListKind, NodeKind},
    selection::{Selection, SelectionPoint},
    Document, NodeId,
};

use crate::{
    app::navigation_icons::{svg_icon, Icon},
    editor::Delay,
    theme,
};

use super::{route_notice, ShellState};

#[path = "editor_interactions.rs"]
mod editor_interactions;

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
    bold: bool,
    italic: bool,
    code: bool,
    color: (u8, u8, u8, u8),
}

impl Default for BlockTextStyle {
    fn default() -> Self {
        Self {
            // Muya's default editor text is 16px. A derived default made the
            // native paragraph renderer pass 0px text with a fully transparent
            // color to Freya, leaving ordinary note bodies invisible.
            font_size: 16.,
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
                &mut spans,
            );
        }
        coalesce_adjacent_spans(&mut spans);
        if spans.is_empty() {
            spans.push(styled_span(String::new(), InlineStyle::default()));
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
        use_side_effect_with_deps(&should_focus, move |should_focus| {
            if *should_focus {
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
        let document_revision = snapshot
            .editor
            .as_ref()
            .map_or(0, |editor| editor.session().revision());
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
            .key(("editor-paragraph", self.node_id, document_revision))
            .a11y_id(a11y_id)
            // Muya's table navigation needs real focusable cell owners. Keep
            // ordinary paragraphs on the same focus path as before so native
            // clipboard and IME events continue to reach the editable node.
            .a11y_focusable(self.accessibility_label.contains("Table"))
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
            .font_size(self.style.font_size)
            .line_height(if self.style.code { 1.4 } else { 1.6 })
            .color(theme::color(self.style.color));
        if self.style.bold {
            view = view.font_weight(FontWeight::BOLD);
        }
        if self.style.italic {
            view = view.font_slant(FontSlant::Italic);
        }
        if self.style.code {
            view = view.font_family("monospace");
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

fn render_note_editor_host(state: State<ShellState>) -> Element {
    let autosave_generation = use_state(|| 0_u64);
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

    // The managed controller owns the scroll offset outside the ScrollView's
    // element tree. Subscribe the host component to its notifier as well so a
    // wheel event invalidates the rendered offset, not only the persisted
    // editor metadata.
    let _scroll_revision = scroll_notifier.read();

    let snapshot = state.read().clone();
    let Some(editor) = snapshot.editor else {
        return route_notice("NoteEditorHost", "No note open");
    };

    let document = editor.session().document();
    let markdown = editor.serialize();
    let metadata = note_metadata(&markdown);
    let document_view = render_document(
        state,
        document,
        autosave_generation,
        Some(metadata.title.as_str()),
    );
    let compact = editor.topbar_compact();
    let topbar_height = if compact { 36. } else { 52. };
    let error_view = snapshot.error.map(|error| {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(6.))
            .background(theme::color(theme::DANGER))
            .a11y_alt("Editor error")
            .child(label().color(theme::color(theme::SURFACE)).text(error))
            .into_element()
    });

    let title_value = State::create(metadata.title.clone());
    let title_state = state;
    let title = rect()
        .height(Size::fill())
        .width(Size::fill())
        .center()
        .font_size(28.)
        .font_weight(FontWeight::BOLD)
        .a11y_alt("Note title")
        .child(
            Input::new(title_value)
                .flat()
                .width(Size::fill())
                .theme_layout(
                    InputLayoutThemePartial::new().inner_margin(Gaps::new_all(0.)),
                )
                .theme_colors(
                    InputColorsThemePartial::new()
                        .color(theme::color(theme::TEXT))
                        .background(Color::TRANSPARENT)
                        .focus_background(Color::TRANSPARENT)
                        .border_fill(Color::TRANSPARENT)
                        .focus_border_fill(Color::TRANSPARENT),
                )
                .on_submit(move |next_title: String| {
                    update_editor_title(title_state, &next_title);
                }),
        );
    let note_relative_path = snapshot
        .vault
        .as_ref()
        .and_then(|vault| {
            editor
                .path()
                .and_then(|path| path.strip_prefix(vault.root()).ok())
        })
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let is_pinned = snapshot
        .library
        .pinned_paths
        .iter()
        .any(|path| path.as_str() == note_relative_path);
    let mut metadata_rail = rect()
        .position(Position::new_absolute().right(46.).top(0.))
        .horizontal()
        .spacing(4.)
        .child(metadata_chip(metadata.date, "Note date"));
    for tag in metadata.tags {
        metadata_rail = metadata_rail.child(metadata_chip(format!("#{tag}"), "Note tag"));
    }
    let mut add_tag_state = state;
    let add_tag = rect()
        .width(Size::px(30.))
        .height(Size::px(30.))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .center()
        .a11y_alt("Add tag")
        .on_mouse_up(move |_| {
            add_tag_state.write().editor_tag_draft = Some(String::new());
        })
        .child(label().font_size(20.).text("+"));
    let mut pin_state = state;
    let pin_path = note_relative_path.clone();
    let pin_button = rect()
        .width(Size::px(30.))
        .height(Size::px(30.))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .background(if is_pinned {
            theme::color(theme::SOFT)
        } else {
            Color::TRANSPARENT
        })
        .with_corner_radius(8.)
        .center()
        .a11y_alt(if is_pinned { "Unpin note" } else { "Pin note" })
        .on_mouse_up(move |event: Event<MouseEventData>| {
            event.stop_propagation();
            if !pin_path.is_empty() {
                pin_state.write().toggle_pinned(
                    crate::library_contract::RelativePath::from(pin_path.as_str()),
                );
            }
        })
        .child(svg_icon(
            Icon::Pin,
            if is_pinned {
                theme::color(theme::PRIMARY)
            } else {
                theme::color(theme::TEXT)
            },
            20.,
        ));
    let tag_editor = snapshot.editor_tag_draft.clone().map(|draft| {
        let tag_value = State::create(draft);
        let tag_state = state;
        rect()
            .width(Size::px(128.))
            .height(Size::px(30.))
            .a11y_alt("Tag input")
            .child(
                Input::new(tag_value)
                    .flat()
                    .width(Size::fill())
                    .placeholder("Tag")
                    .on_submit(move |tag: String| submit_editor_tag(tag_state, &tag)),
            )
            .into_element()
    });
    metadata_rail = metadata_rail
        .maybe_child(tag_editor)
        .child(add_tag)
        .child(pin_button);
    let close_state_topbar = state;
    let mut topbar = rect()
        .width(Size::fill())
        .height(Size::px(topbar_height))
        .padding(Gaps::new(0., 12., 0., 2.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .spacing(4.)
        .a11y_alt(if compact {
            "Editor topbar compact"
        } else {
            "Editor topbar"
        })
        .on_mouse_up(move |event: Event<MouseEventData>| {
            if event.global_location.x >= 1200. && event.global_location.y <= 70. {
                close_note(close_state_topbar);
            }
        })
        .child(title)
        .child(metadata_rail);
    let close_state = state;
    let close_button = rect()
        .position(Position::new_absolute().right(12.).top(0.))
        .width(Size::px(30.))
        .height(Size::px(30.))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .center()
        .a11y_alt("Close note")
        .child(svg_icon(Icon::X, theme::color(theme::TEXT), 20.));
    topbar = topbar.child(
        close_button
            .on_mouse_up(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                close_note(close_state);
            }),
    );

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .child(topbar)
        .maybe_child(error_view)
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .padding(Gaps::new(27., 12., 100., 2.))
                .background(theme::color(theme::BG))
                .a11y_alt("Editor scroll")
                .child(
                    ScrollView::new_controlled(scroll_controller)
                        .width(Size::fill())
                        .height(Size::fill())
                        .child(document_view),
                ),
        )
        .a11y_alt("NoteEditorHost")
        .into_element()
}

fn close_note(mut state: State<ShellState>) {
    let result = {
        let mut shell = state.write();
        let result = shell.editor.as_mut().map_or_else(
            || Err("cannot close without an open note".to_string()),
            |editor| editor.close().map_err(|error| error.to_string()),
        );
        if result.is_ok() {
            shell.editor = None;
            shell.editor_tag_draft = None;
        }
        result
    };
    if let Err(error) = result {
        eprintln!("[freya][editor] action:failure action=close error={error}");
        state.write().error = Some(error);
    }
}

#[derive(Default)]
struct NoteMetadata {
    title: String,
    date: String,
    tags: Vec<String>,
}

fn note_metadata(markdown: &str) -> NoteMetadata {
    let mut metadata = NoteMetadata {
        title: "Untitled".to_owned(),
        date: today_date(),
        ..NoteMetadata::default()
    };
    let Some(frontmatter) = markdown
        .strip_prefix("---\n")
        .and_then(|value| value.split_once("\n---"))
        .map(|(value, _)| value)
    else {
        return metadata;
    };
    for line in frontmatter.lines() {
        let Some((key, raw_value)) = line.split_once(':') else {
            continue;
        };
        let value = raw_value.trim().trim_matches('"').trim_matches('\'');
        match key.trim() {
            "title" if !value.is_empty() => metadata.title = value.to_owned(),
            "createdAt" | "created" if value.len() >= 10 => {
                metadata.date = value[..10].to_owned()
            }
            "tags" => {
                metadata.tags = value
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|tag| tag.trim().trim_matches('"').trim_matches('\'').to_owned())
                    .filter(|tag| !tag.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    metadata
}

fn update_editor_title(mut state: State<ShellState>, requested: &str) {
    let next_markdown = {
        let shell = state.read();
        shell.editor.as_ref().map(|editor| {
            let markdown = editor.serialize();
            let current = note_metadata(&markdown).title;
            let next = requested.trim();
            if next.is_empty() || next == current {
                None
            } else {
                Some(rewrite_note_title(&markdown, &current, next))
            }
        })
    }
    .flatten();
    if let Some(markdown) = next_markdown {
        if let Some(editor) = state.write().editor.as_mut() {
            editor.replace_markdown(markdown);
            eprintln!("[freya][editor] action=metadata:update field=title");
        }
    }
}

fn submit_editor_tag(mut state: State<ShellState>, requested: &str) {
    let tag = requested.trim();
    let next_markdown = {
        let shell = state.read();
        shell.editor.as_ref().and_then(|editor| {
            if tag.is_empty() {
                return None;
            }
            let markdown = editor.serialize();
            let mut metadata = note_metadata(&markdown);
            if metadata.tags.iter().any(|current| current == tag) {
                return None;
            }
            metadata.tags.push(tag.to_owned());
            Some(rewrite_note_tags(&markdown, &metadata.tags))
        })
    };
    if let Some(markdown) = next_markdown {
        if let Some(editor) = state.write().editor.as_mut() {
            editor.replace_markdown(markdown);
            eprintln!("[freya][editor] action=metadata:update field=tags");
        }
    }
    state.write().editor_tag_draft = None;
}

fn rewrite_note_title(markdown: &str, previous: &str, next: &str) -> String {
    let with_frontmatter = rewrite_frontmatter_line(
        markdown,
        "title",
        &format!("title: \"{}\"", next.replace('"', "\\\"")),
    );
    let heading = format!("# {previous}");
    let replacement = format!("# {next}");
    let lines = with_frontmatter
        .split('\n')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let content_start = if lines.first().is_some_and(|line| line == "---") {
        lines
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, line)| line.as_str() == "---")
            .map_or(0, |(index, _)| index + 1)
    } else {
        0
    };
    let Some(heading_index) = lines
        .iter()
        .enumerate()
        .skip(content_start)
        .find_map(|(index, line)| (line == &heading).then_some(index))
    else {
        return with_frontmatter;
    };
    let mut rewritten = lines;
    rewritten[heading_index] = replacement;
    rewritten.join("\n")
}

fn rewrite_note_tags(markdown: &str, tags: &[String]) -> String {
    let encoded = tags
        .iter()
        .map(|tag| format!("\"{}\"", tag.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    rewrite_frontmatter_line(markdown, "tags", &format!("tags: [{encoded}]"))
}

fn rewrite_frontmatter_line(markdown: &str, key: &str, replacement: &str) -> String {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return format!("---\n{replacement}\n---\n\n{markdown}");
    };
    let Some((frontmatter, suffix)) = rest.split_once("\n---") else {
        return markdown.to_owned();
    };
    let mut found = false;
    let mut lines = frontmatter
        .lines()
        .map(|line| {
            if line
                .split_once(':')
                .is_some_and(|(candidate, _)| candidate.trim() == key)
            {
                found = true;
                replacement.to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();
    if !found {
        lines.push(replacement.to_owned());
    }
    format!("---\n{}\n---{}", lines.join("\n"), suffix)
}

fn today_date() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64 / 86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn metadata_chip(text: String, accessibility: &str) -> Element {
    let width = if accessibility == "Note date" {
        98.
    } else {
        20. + text.chars().count() as f32 * 8.
    };
    rect()
        .width(Size::px(width))
        .height(Size::px(30.))
        .padding(Gaps::new(0., 8., 0., 8.))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .with_corner_radius(8.)
        .center()
        .a11y_alt(accessibility)
        .child(label().font_size(14.).text(text))
        .into_element()
}

fn render_document(
    state: State<ShellState>,
    document: &Document,
    autosave_generation: State<u64>,
    hidden_title: Option<&str>,
) -> Element {
    render_block_children(
        state,
        document,
        document.root,
        autosave_generation,
        hidden_title,
    )
}

fn render_block_children(
    state: State<ShellState>,
    document: &Document,
    parent: NodeId,
    autosave_generation: State<u64>,
    hidden_title: Option<&str>,
) -> Element {
    let mut title_checked = hidden_title.is_none();
    let children = document
        .children(parent)
        .filter_map(|node| {
            if matches!(node.kind, NodeKind::Block(BlockKind::FrontMatter { .. })) {
                return None;
            }
            if !title_checked {
                title_checked = true;
                if matches!(node.kind, NodeKind::Block(BlockKind::Heading { level: 1 }))
                    && hidden_title.is_some_and(|title| {
                        plain_block_text(document, node.id).trim() == title.trim()
                    })
                {
                    return None;
                }
            }
            Some(render_block(state, document, node.id, autosave_generation))
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(5.)
        .children(children)
        .into_element()
}

fn plain_block_text(document: &Document, node_id: NodeId) -> String {
    let mut text = String::new();
    for child in document.children(node_id) {
        match &child.kind {
            NodeKind::Inline(InlineKind::Text { value }) => text.push_str(value),
            NodeKind::Document | NodeKind::Block(_) | NodeKind::Inline(_) => {
                text.push_str(&plain_block_text(document, child.id));
            }
        }
    }
    text
}

fn render_block(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    autosave_generation: State<u64>,
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
        ),
        NodeKind::Block(BlockKind::Heading { level }) => render_inline_block(
            state,
            autosave_generation,
            node_id,
            &format!("Heading {level}"),
            BlockTextStyle {
                font_size: heading_font_size(*level),
                bold: true,
                ..BlockTextStyle::default()
            },
        ),
        NodeKind::Block(BlockKind::BlockQuote) => rect()
            .width(Size::fill())
            .padding(Gaps::new_all(10.))
            .border(
                Border::new()
                    .fill(theme::color(theme::BORDER_STRONG))
                    .width(1.),
            )
            .child(render_block_children(
                state,
                document,
                node_id,
                autosave_generation,
                None,
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
        ),
        NodeKind::Block(BlockKind::ListItem { .. }) => {
            render_block_children(state, document, node_id, autosave_generation, None)
        }
        NodeKind::Block(BlockKind::CodeBlock { language, .. }) => {
            let mut children = Vec::new();
            if let Some(language) = language.as_deref().filter(|value| !value.is_empty()) {
                children.push(
                    label()
                        .font_size(12.)
                        .font_weight(FontWeight::BOLD)
                        .color(theme::color(theme::MUTED))
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
                    code: true,
                    color: theme::TEXT,
                    ..BlockTextStyle::default()
                },
            ));
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(12.))
                .background(theme::color(theme::BG))
                .with_corner_radius(6.)
                .children(children)
                .a11y_alt("Code block")
                .into_element()
        }
        NodeKind::Block(BlockKind::ThematicBreak) => rect()
            .width(Size::fill())
            .height(Size::px(1.))
            .background(theme::color(theme::BORDER_STRONG))
            .a11y_alt("Thematic break")
            .into_element(),
        NodeKind::Block(BlockKind::Table) => rect()
            .width(Size::fill())
            .spacing(4.)
            .padding(Gaps::new_all(6.))
            .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
            .children(
                document
                    .children(node_id)
                    .map(|child| render_block(state, document, child.id, autosave_generation))
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
                    .map(|child| render_block(state, document, child.id, autosave_generation))
                    .collect::<Vec<_>>(),
            )
            .into_element(),
        NodeKind::Block(BlockKind::TableCell { header, .. }) => rect()
            .width(Size::fill())
            .padding(Gaps::new_all(6.))
            .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
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
                )
            } else {
                render_inline_block(
                    state,
                    autosave_generation,
                    node_id,
                    "Table cell",
                    BlockTextStyle::default(),
                )
            })
            .into_element(),
        NodeKind::Block(BlockKind::HtmlBlock) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            "HTML block not rendered",
        ),
        NodeKind::Block(BlockKind::MathBlock) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            "Math block not rendered",
        ),
        NodeKind::Block(BlockKind::FrontMatter { .. }) => rect()
            .height(Size::px(0.))
            .into_element(),
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Footnote definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Reference definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => render_unsupported_block(
            state,
            autosave_generation,
            node_id,
            &format!("Diagram not rendered: {language}"),
        ),
        NodeKind::Document | NodeKind::Inline(_) => {
            render_block_children(state, document, node_id, autosave_generation, None)
        }
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
        collect_inline_children(document, block_id, InlineStyle::default(), &mut spans);
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

fn render_list(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    kind: ListKind,
    start: u64,
    autosave_generation: State<u64>,
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
                    .font_size(16.)
                    .color(theme::color(theme::MUTED))
                    .text(marker)
                    .into_element()
            };
            rect()
                .width(Size::fill())
                .horizontal()
                .spacing(6.)
                .child(marker_view)
                .child(render_block_children(
                    state,
                    document,
                    item.id,
                    autosave_generation,
                    None,
                ))
                .into_element()
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(6.)
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
) -> Element {
    EditableInlineBlock {
        state,
        autosave_generation,
        node_id,
        accessibility_label: accessibility_label.to_string(),
        style,
    }
    .into_element()
}

fn render_unsupported_block(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    node_id: NodeId,
    message: &str,
) -> Element {
    rect()
        .width(Size::fill())
        .spacing(4.)
        .padding(Gaps::new_all(8.))
        .background(theme::color(theme::SOFT))
        .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
        .child(
            label()
                .font_size(13.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(theme::DANGER))
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
        ))
        .a11y_alt(message.to_string())
        .into_element()
}

fn collect_inline_children(
    document: &Document,
    parent: NodeId,
    style: InlineStyle,
    spans: &mut Vec<Span<'static>>,
) {
    for child in document.children(parent) {
        collect_inline_node(document, child.id, style, spans);
    }
}

fn collect_inline_node(
    document: &Document,
    node_id: NodeId,
    style: InlineStyle,
    spans: &mut Vec<Span<'static>>,
) {
    let Some(node) = document.node(node_id) else {
        return;
    };

    match &node.kind {
        NodeKind::Inline(kind) => match kind {
            InlineKind::Text { value } => spans.push(styled_span(value.clone(), style)),
            InlineKind::Escaped { value } => spans.push(styled_span(value.to_string(), style)),
            InlineKind::Emphasis => {
                let mut next = style;
                next.emphasis = true;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::Strong => {
                let mut next = style;
                next.strong = true;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::Strike => {
                let mut next = style;
                next.strike = true;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::MarkFragment { mark, .. } => {
                let mut next = style;
                match mark {
                    InlineMarkKind::Emphasis => next.emphasis = true,
                    InlineMarkKind::Strong => next.strong = true,
                    InlineMarkKind::Strike => next.strike = true,
                }
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::CodeSpan { code } => {
                let mut next = style;
                next.code = true;
                spans.push(styled_span(code.clone(), next));
            }
            InlineKind::Link { .. } => {
                let mut next = style;
                next.link = true;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::Image { source, alt, .. } => {
                let text = format!("[Image non rendue: alt=\"{alt}\" source=\"{source}\"]");
                let mut next = style;
                next.status = true;
                spans.push(styled_span(text, next));
            }
            InlineKind::AutoLink { destination } => {
                let mut next = style;
                next.link = true;
                spans.push(styled_span(destination.clone(), next));
            }
            InlineKind::InlineHtml { raw } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(format!("[HTML inline non rendu: {raw}]"), next));
            }
            InlineKind::InlineMath { source } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[Math inline non rendu: {source}]"),
                    next,
                ));
            }
            InlineKind::Emoji { value, .. } => spans.push(styled_span(value.clone(), style)),
            InlineKind::Superscript => {
                let mut next = style;
                next.script = ScriptStyle::Superscript;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::Subscript => {
                let mut next = style;
                next.script = ScriptStyle::Subscript;
                collect_inline_children(document, node_id, next, spans);
            }
            InlineKind::FootnoteReference { label } => {
                spans.push(styled_span(format!("[footnote: {label}]"), style));
            }
            InlineKind::SoftBreak | InlineKind::HardBreak => {
                spans.push(styled_span("\n".to_string(), style));
            }
        },
        NodeKind::Document | NodeKind::Block(_) => {
            collect_inline_children(document, node_id, style, spans)
        }
    }
}

fn styled_span(value: String, style: InlineStyle) -> Span<'static> {
    let color = if style.status {
        theme::DANGER
    } else if style.link {
        theme::PRIMARY
    } else if style.code {
        theme::MUTED
    } else {
        theme::TEXT
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
        span = span.font_family("monospace");
    }
    match style.script {
        ScriptStyle::Normal => {}
        ScriptStyle::Superscript | ScriptStyle::Subscript => {
            span = span.font_size(12.);
        }
    }
    span
}

fn coalesce_adjacent_spans(spans: &mut Vec<Span<'static>>) {
    let mut merged: Vec<Span<'static>> = Vec::with_capacity(spans.len());
    for span in spans.drain(..) {
        if let Some(previous) = merged.last_mut() {
            if previous.text_style_data == span.text_style_data {
                previous.text = format!("{}{}", previous.text, span.text).into();
                continue;
            }
        }
        merged.push(span);
    }
    *spans = merged;
}

fn heading_font_size(level: u8) -> f32 {
    match level {
        1 => 30.,
        2 => 26.,
        3 => 22.,
        4 => 19.,
        5 => 17.,
        _ => 16.,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{EditorDocument, EditorSession};

    fn spans_for(markdown: &str) -> Vec<Span<'static>> {
        let session = EditorSession::from_markdown(markdown);
        let document = session.document();
        let mut spans = Vec::new();
        collect_inline_children(document, document.root, InlineStyle::default(), &mut spans);
        spans
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
    fn default_block_text_is_visible_at_the_editor_base_style() {
        let style = BlockTextStyle::default();

        assert_eq!(style.font_size, 16.);
        assert_eq!(style.color, theme::TEXT);
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
        collect_inline_children(&document, document.root, InlineStyle::default(), &mut spans);
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
            "Heading 1",
            "Block quote",
            "Unordered list",
            "Code block",
            "Thematic break",
        ] {
            assert!(
                runner
                    .find(|_, element| {
                        (element.accessibility().builder.label() == Some(label)).then_some(())
                    })
                    .is_some(),
                "structured block {label:?} must be present in the Freya tree"
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

    #[test]
    fn metadata_updates_preserve_frontmatter_and_visible_heading() {
        let markdown = "---\ntitle: \"Alpha note\"\ntags: [\"e2e\"]\n---\n\n# Alpha note\n\nBody\n";
        let renamed = rewrite_note_title(markdown, "Alpha note", "Renamed");
        assert!(renamed.contains("title: \"Renamed\""));
        assert!(renamed.contains("\n# Renamed\n"));

        let tagged = rewrite_note_tags(&renamed, &["e2e".to_owned(), "native".to_owned()]);
        assert!(tagged.contains("tags: [\"e2e\", \"native\"]"));
        assert!(tagged.contains("\n# Renamed\n"));
    }
}
