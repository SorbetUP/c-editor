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

use crate::theme;

use super::{route_notice, ShellState};

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

#[derive(Clone, Copy, Default, PartialEq)]
struct BlockTextStyle {
    font_size: f32,
    bold: bool,
    italic: bool,
    code: bool,
    color: (u8, u8, u8, u8),
}

#[derive(Clone, PartialEq)]
struct EditableInlineBlock {
    state: State<ShellState>,
    node_id: NodeId,
    accessibility_label: String,
    style: BlockTextStyle,
}

impl Component for EditableInlineBlock {
    fn render(&self) -> impl IntoElement {
        let snapshot = self.state.read().clone();
        let Some(editor) = snapshot.editor.as_ref() else {
            return route_notice("NoteEditorHost", "No note open").into_element();
        };

        let mut spans = Vec::new();
        collect_inline_children(
            editor.session().document(),
            self.node_id,
            InlineStyle::default(),
            &mut spans,
        );
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
        let mut state = self.state;
        let node_id = self.node_id;
        let previous_value = value.clone();
        let on_key_down = move |event: Event<KeyboardEventData>| {
            if let Err(error) = sync_muya_selection(state, node_id, &editable) {
                state.write().error = Some(error.clone());
                eprintln!("[freya][editor] action:failure action=selection error={error}");
                return;
            }

            let result = match &event.key {
                Key::Named(NamedKey::Enter) if event.modifiers.is_empty() => state
                    .write()
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot split without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .insert_paragraph()
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    }),
                Key::Named(NamedKey::Backspace) if event.modifiers.is_empty() => state
                    .write()
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot delete without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .delete_backward()
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    }),
                Key::Named(NamedKey::Delete) if event.modifiers.is_empty() => state
                    .write()
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot delete without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .delete_forward()
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    }),
                Key::Character(character)
                    if event.modifiers.contains(Modifiers::ctrl_or_meta())
                        && character.eq_ignore_ascii_case("s") =>
                {
                    state
                        .read()
                        .editor
                        .as_ref()
                        .ok_or_else(|| "cannot save without an open note".to_string())
                        .and_then(|editor| editor.save().map_err(|error| error.to_string()))
                }
                Key::Character(character)
                    if event.modifiers.contains(Modifiers::ctrl_or_meta())
                        && character.eq_ignore_ascii_case("z") =>
                {
                    let redo = event.modifiers.contains(Modifiers::SHIFT);
                    state
                        .write()
                        .editor
                        .as_mut()
                        .ok_or_else(|| "cannot change history without an open note".to_string())
                        .and_then(|editor| {
                            if redo {
                                editor.redo().map(|_| ()).map_err(|error| error.to_string())
                            } else {
                                editor.undo().map(|_| ()).map_err(|error| error.to_string())
                            }
                        })
                }
                Key::Character(character)
                    if event.modifiers.contains(Modifiers::ctrl_or_meta())
                        && character.eq_ignore_ascii_case("y") =>
                {
                    state
                        .write()
                        .editor
                        .as_mut()
                        .ok_or_else(|| "cannot change history without an open note".to_string())
                        .and_then(|editor| {
                            editor.redo().map(|_| ()).map_err(|error| error.to_string())
                        })
                }
                _ => {
                    editable.process_event(EditableEvent::KeyDown {
                        key: &event.key,
                        modifiers: event.modifiers,
                    });
                    let next_value = editable.editor().read().committed_text();
                    if next_value == previous_value {
                        sync_muya_selection(state, node_id, &editable)
                    } else {
                        apply_inline_delta(state, node_id, &previous_value, &next_value)
                    }
                }
            };

            match result {
                Ok(()) => {
                    sync_editable_from_muya(state, node_id, &mut editable);
                    eprintln!(
                        "[freya][editor] action:complete action=keyboard node={:?} key={}",
                        node_id, event.key
                    );
                }
                Err(error) => {
                    let error = error.to_string();
                    state.write().error = Some(error.clone());
                    editable.editor_mut().write().set(&previous_value);
                    eprintln!("[freya][editor] action:failure action=keyboard error={error}");
                }
            }
        };
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
            .font_size(self.style.font_size)
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

pub(super) fn note_editor_host(mut state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let Some(editor) = snapshot.editor else {
        return route_notice("NoteEditorHost", "No note open");
    };

    let undo = rect()
        .width(Size::px(52.))
        .height(Size::px(36.))
        .center()
        .background(theme::color(theme::SOFT))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            let error = {
                let mut shell = state.write();
                shell
                    .editor
                    .as_mut()
                    .and_then(|editor| editor.undo().err())
                    .map(|error| error.to_string())
            };
            if let Some(error) = error {
                eprintln!("[freya][editor] action:failure action=undo error={error}");
                state.write().error = Some(error);
            }
        })
        .a11y_alt("Undo")
        .child(label().text("↶"));
    let save = rect()
        .width(Size::px(72.))
        .height(Size::px(36.))
        .center()
        .background(theme::color(theme::PRIMARY))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            let error = state
                .read()
                .editor
                .as_ref()
                .and_then(|editor| editor.save().err())
                .map(|error| error.to_string());
            if let Some(error) = error {
                eprintln!("[freya][editor] action:failure action=save error={error}");
                state.write().error = Some(error);
            } else {
                eprintln!("[freya][editor] action:complete action=save");
            }
        })
        .a11y_alt("Save")
        .child(label().text("Save"));
    let close = rect()
        .width(Size::px(72.))
        .height(Size::px(36.))
        .center()
        .background(theme::color(theme::SOFT))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            eprintln!("[freya][editor] action:complete action=close");
            state.write().editor = None;
        })
        .a11y_alt("Close note")
        .child(label().text("Close"));

    let document = editor.session().document();
    let document_view = render_document(state, document);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .spacing(10.)
        .child(
            rect()
                .height(Size::px(44.))
                .horizontal()
                .spacing(8.)
                .child(undo)
                .child(save)
                .child(close),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .scrollable(true)
                .padding(Gaps::new_all(18.))
                .background(theme::color(theme::SURFACE))
                .with_corner_radius(10.)
                .a11y_alt("Editor scroll")
                .child(document_view),
        )
        .a11y_alt("NoteEditorHost")
        .into_element()
}

fn render_document(state: State<ShellState>, document: &Document) -> Element {
    render_block_children(state, document, document.root)
}

fn render_block_children(state: State<ShellState>, document: &Document, parent: NodeId) -> Element {
    let children = document
        .children(parent)
        .map(|node| render_block(state, document, node.id))
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(10.)
        .children(children)
        .into_element()
}

fn render_block(state: State<ShellState>, document: &Document, node_id: NodeId) -> Element {
    let Some(node) = document.node(node_id) else {
        return label().text("[Muya node unavailable]").into_element();
    };

    match &node.kind {
        NodeKind::Block(BlockKind::Paragraph) => {
            render_inline_block(state, node_id, "Paragraph", BlockTextStyle::default())
        }
        NodeKind::Block(BlockKind::Heading { level }) => render_inline_block(
            state,
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
            .child(render_block_children(state, document, node_id))
            .a11y_alt("Block quote")
            .into_element(),
        NodeKind::Block(BlockKind::List { kind, start }) => {
            render_list(state, document, node_id, *kind, start.unwrap_or(1))
        }
        NodeKind::Block(BlockKind::ListItem { .. }) => {
            render_block_children(state, document, node_id)
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
                    .map(|child| render_block(state, document, child.id))
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
                    .map(|child| render_block(state, document, child.id))
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
                    node_id,
                    "Table header cell",
                    BlockTextStyle {
                        bold: true,
                        ..BlockTextStyle::default()
                    },
                )
            } else {
                render_inline_block(state, node_id, "Table cell", BlockTextStyle::default())
            })
            .into_element(),
        NodeKind::Block(BlockKind::HtmlBlock) => {
            render_unsupported_block(state, node_id, "HTML block not rendered")
        }
        NodeKind::Block(BlockKind::MathBlock) => {
            render_unsupported_block(state, node_id, "Math block not rendered")
        }
        NodeKind::Block(BlockKind::FrontMatter { .. }) => {
            render_unsupported_block(state, node_id, "Front matter is not an editor block")
        }
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => render_unsupported_block(
            state,
            node_id,
            &format!("Footnote definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => render_unsupported_block(
            state,
            node_id,
            &format!("Reference definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => {
            render_unsupported_block(state, node_id, &format!("Diagram not rendered: {language}"))
        }
        NodeKind::Document | NodeKind::Inline(_) => render_block_children(state, document, node_id),
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

fn sync_muya_selection(
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

fn sync_editable_from_muya(state: State<ShellState>, block_id: NodeId, editable: &mut UseEditable) {
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

fn apply_inline_delta(
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
) -> Element {
    let items = document
        .children(node_id)
        .enumerate()
        .map(|(index, item)| {
            let marker = list_marker(kind, start.saturating_add(index as u64), item.id, document);
            rect()
                .width(Size::fill())
                .horizontal()
                .spacing(6.)
                .child(
                    label()
                        .font_size(16.)
                        .color(theme::color(theme::MUTED))
                        .text(marker),
                )
                .child(render_block_children(state, document, item.id))
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
    node_id: NodeId,
    accessibility_label: &str,
    style: BlockTextStyle,
) -> Element {
    EditableInlineBlock {
        state,
        node_id,
        accessibility_label: accessibility_label.to_string(),
        style,
    }
    .into_element()
}

fn render_unsupported_block(state: State<ShellState>, node_id: NodeId, message: &str) -> Element {
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
}
