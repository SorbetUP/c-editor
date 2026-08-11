//! Native Freya rendering for the real Muya document owned by NoteEditorHost.
//!
//! The Vue host delegates the editing surface to `EditorWithTabs`/Muya. This
//! native tranche keeps that boundary: `EditorDocument` remains the source of
//! truth for undo/save, while this module walks its parsed Muya tree and turns
//! blocks and inline nodes into Freya paragraphs and containers.

use freya::prelude::*;
use muya_core::{
    model::{BlockKind, InlineKind, InlineMarkKind, ListKind, NodeKind},
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

#[derive(Clone, Copy, Default)]
struct BlockTextStyle {
    font_size: f32,
    bold: bool,
    italic: bool,
    code: bool,
    color: (u8, u8, u8, u8),
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

    let document = editor.session().document();
    let document_view = render_document(document);

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
                .child(save),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .scrollable(true)
                .padding(Gaps::new_all(18.))
                .background(theme::color(theme::SURFACE))
                .with_corner_radius(10.)
                .child(document_view),
        )
        .into_element()
}

fn render_document(document: &Document) -> Element {
    render_block_children(document, document.root)
}

fn render_block_children(document: &Document, parent: NodeId) -> Element {
    let children = document
        .children(parent)
        .map(|node| render_block(document, node.id))
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(10.)
        .children(children)
        .into_element()
}

fn render_block(document: &Document, node_id: NodeId) -> Element {
    let Some(node) = document.node(node_id) else {
        return label().text("[Muya node unavailable]").into_element();
    };

    match &node.kind {
        NodeKind::Block(BlockKind::Paragraph) => {
            render_inline_block(document, node_id, "Paragraph", BlockTextStyle::default())
        }
        NodeKind::Block(BlockKind::Heading { level }) => render_inline_block(
            document,
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
            .child(render_block_children(document, node_id))
            .a11y_alt("Block quote")
            .into_element(),
        NodeKind::Block(BlockKind::List { kind, start }) => {
            render_list(document, node_id, *kind, start.unwrap_or(1))
        }
        NodeKind::Block(BlockKind::ListItem { .. }) => render_block_children(document, node_id),
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
                document,
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
                    .map(|child| render_block(document, child.id))
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
                    .map(|child| render_block(document, child.id))
                    .collect::<Vec<_>>(),
            )
            .into_element(),
        NodeKind::Block(BlockKind::TableCell { header, .. }) => rect()
            .width(Size::fill())
            .padding(Gaps::new_all(6.))
            .border(Border::new().fill(theme::color(theme::BORDER)).width(1.))
            .child(if *header {
                render_inline_block(
                    document,
                    node_id,
                    "Table header cell",
                    BlockTextStyle {
                        bold: true,
                        ..BlockTextStyle::default()
                    },
                )
            } else {
                render_inline_block(document, node_id, "Table cell", BlockTextStyle::default())
            })
            .into_element(),
        NodeKind::Block(BlockKind::HtmlBlock) => {
            render_unsupported_block(document, node_id, "HTML block not rendered")
        }
        NodeKind::Block(BlockKind::MathBlock) => {
            render_unsupported_block(document, node_id, "Math block not rendered")
        }
        NodeKind::Block(BlockKind::FrontMatter { .. }) => {
            render_unsupported_block(document, node_id, "Front matter is not an editor block")
        }
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => render_unsupported_block(
            document,
            node_id,
            &format!("Footnote definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => render_unsupported_block(
            document,
            node_id,
            &format!("Reference definition not rendered: {label}"),
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => render_unsupported_block(
            document,
            node_id,
            &format!("Diagram not rendered: {language}"),
        ),
        NodeKind::Document | NodeKind::Inline(_) => render_block_children(document, node_id),
    }
}

fn render_list(document: &Document, node_id: NodeId, kind: ListKind, start: u64) -> Element {
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
                .child(render_block_children(document, item.id))
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
    document: &Document,
    node_id: NodeId,
    accessibility_label: &str,
    style: BlockTextStyle,
) -> Element {
    let mut spans = Vec::new();
    collect_inline_children(document, node_id, InlineStyle::default(), &mut spans);
    if spans.is_empty() {
        spans.push(styled_span(String::new(), InlineStyle::default()));
    }

    let mut view = paragraph()
        .width(Size::fill())
        .font_size(style.font_size)
        .color(theme::color(style.color))
        .spans_iter(spans.into_iter())
        .a11y_alt(accessibility_label.to_string());
    if style.bold {
        view = view.font_weight(FontWeight::BOLD);
    }
    if style.italic {
        view = view.font_slant(FontSlant::Italic);
    }
    if style.code {
        view = view.font_family("monospace");
    }
    view.into_element()
}

fn render_unsupported_block(document: &Document, node_id: NodeId, message: &str) -> Element {
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
            document,
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
    use crate::editor::EditorSession;

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
        let session = EditorSession::from_markdown(
            "# Heading\n\n> quote\n\n- item\n- [x] done\n\n```rust\nlet x = 1;\n```\n\n---",
        );
        let _rendered = render_document(session.document());
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
    }
}
