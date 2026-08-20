from pathlib import Path

APP = Path("Elephant/freya/src/app.rs")
VIEW = Path("Elephant/freya/src/app/editor_view.rs")
TEST = Path("Elephant/freya/tests/editor_image_markdown_freya_testing.rs")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


app = APP.read_text()
if "mod editor_image_preview;" not in app:
    app = replace_once(app, "mod editor_view;\n", "mod editor_view;\nmod editor_image_preview;\n", "image preview module")
APP.write_text(app)

text = VIEW.read_text()

text = replace_once(
    text,
    '''        NodeKind::Block(BlockKind::Paragraph) => editable_block(
            state,
            autosave_generation,
            node_id,
            "Paragraph",
            BlockTextStyle::default(),
            palette,
            text_scale,
        ),''',
    '''        NodeKind::Block(BlockKind::Paragraph) => render_paragraph(
            state,
            document,
            node_id,
            autosave_generation,
            palette,
            text_scale,
        ),''',
    "paragraph renderer",
)

paragraph_helper = r'''fn collect_inline_images(
    document: &Document,
    parent: NodeId,
    images: &mut Vec<(String, String)>,
) {
    for child in document.children(parent) {
        if let NodeKind::Inline(InlineKind::Image { source, alt, .. }) = &child.kind {
            images.push((source.clone(), alt.clone()));
        }
        collect_inline_images(document, child.id, images);
    }
}

fn render_paragraph(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    let mut images = Vec::new();
    collect_inline_images(document, node_id, &mut images);
    if images.is_empty() {
        return editable_block(
            state,
            autosave_generation,
            node_id,
            "Paragraph",
            BlockTextStyle::default(),
            palette,
            text_scale,
        );
    }

    let note_path = state
        .read()
        .editor
        .as_ref()
        .and_then(|editor| editor.path().map(std::path::Path::to_path_buf));
    let previews = images
        .iter()
        .map(|(source, alt)| {
            super::editor_image_preview::render(note_path.as_deref(), source, alt, palette)
        })
        .collect::<Vec<_>>();

    rect()
        .width(Size::fill())
        .spacing(8.)
        .child(editable_block(
            state,
            autosave_generation,
            node_id,
            "Paragraph",
            BlockTextStyle::default(),
            palette,
            text_scale,
        ))
        .children(previews)
        .a11y_alt("Paragraph with image")
        .into_element()
}

'''
if "fn render_paragraph(" not in text:
    text = replace_once(text, "fn render_block(\n", paragraph_helper + "fn render_block(\n", "paragraph helper")

text = replace_once(
    text,
    '''        NodeKind::Block(BlockKind::HtmlBlock) => unsupported_block(
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
        ),''',
    '''        NodeKind::Block(BlockKind::HtmlBlock) => semantic_source_block(
            state,
            autosave_generation,
            node_id,
            "HTML",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::MathBlock) => semantic_source_block(
            state,
            autosave_generation,
            node_id,
            "Math",
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::FootnoteDefinition { label }) => render_footnote_definition(
            state,
            document,
            node_id,
            label,
            autosave_generation,
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::ReferenceDefinition { label }) => semantic_source_block(
            state,
            autosave_generation,
            node_id,
            &format!("Reference {label}"),
            palette,
            text_scale,
        ),
        NodeKind::Block(BlockKind::Diagram { language }) => semantic_source_block(
            state,
            autosave_generation,
            node_id,
            &format!("{language} diagram"),
            palette,
            text_scale,
        ),''',
    "special block renderer",
)

special_helper = r'''fn render_footnote_definition(
    state: State<ShellState>,
    document: &Document,
    node_id: NodeId,
    footnote_label: &str,
    autosave_generation: State<u64>,
    palette: theme::ThemePalette,
    text_scale: f32,
) -> Element {
    rect()
        .width(Size::fill())
        .spacing(6.)
        .padding(Gaps::new_all(8.))
        .background(theme::color(palette.soft))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .with_corner_radius(7.)
        .child(
            label()
                .font_family(UI_FONT)
                .font_size(12. * text_scale)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(palette.muted))
                .text(format!("[^{footnote_label}]")),
        )
        .child(render_block_children(
            state,
            document,
            node_id,
            autosave_generation,
            palette,
            text_scale,
        ))
        .a11y_alt(format!("Footnote {footnote_label}"))
        .into_element()
}

fn semantic_source_block(
'''
if "fn render_footnote_definition(" not in text:
    text = replace_once(text, "fn unsupported_block(\n", special_helper, "semantic block helper")
else:
    text = text.replace("fn unsupported_block(\n", "fn semantic_source_block(\n", 1)

text = text.replace(
    ".color(theme::color(palette.danger))\n                .text(message.to_string()),",
    ".color(theme::color(palette.muted))\n                .text(message.to_string()),",
    1,
)

text = replace_once(
    text,
    '''            InlineKind::Escaped { value } => {
                spans.push(styled_span(value.to_string(), style, palette, text_scale))
            }''',
    '''            InlineKind::Escaped { value } => {
                if node.children.is_empty() {
                    spans.push(styled_span(value.to_string(), style, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, style, palette, text_scale, spans);
                }
            }''',
    "escaped inline",
)
text = replace_once(
    text,
    '''            InlineKind::Image { source, alt, .. } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[Image non rendue: alt=\\\"{alt}\\\" source=\\\"{source}\\\"]"),
                    next,
                    palette,
                    text_scale,
                ));
            }''',
    '''            InlineKind::Image { alt, .. } => {
                let mut next = style;
                next.emphasis = true;
                if node.children.is_empty() {
                    spans.push(styled_span(alt.clone(), next, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, next, palette, text_scale, spans);
                }
            }''',
    "image inline",
)
text = replace_once(
    text,
    '''            InlineKind::AutoLink { destination } => {
                let mut next = style;
                next.link = true;
                spans.push(styled_span(destination.clone(), next, palette, text_scale));
            }''',
    '''            InlineKind::AutoLink { destination } => {
                let mut next = style;
                next.link = true;
                if node.children.is_empty() {
                    spans.push(styled_span(destination.clone(), next, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, next, palette, text_scale, spans);
                }
            }''',
    "autolink inline",
)
text = replace_once(
    text,
    '''            InlineKind::InlineHtml { raw } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[HTML inline non rendu: {raw}]"),
                    next,
                    palette,
                    text_scale,
                ));
            }''',
    '''            InlineKind::InlineHtml { raw } => {
                let mut next = style;
                next.code = true;
                if node.children.is_empty() {
                    spans.push(styled_span(raw.clone(), next, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, next, palette, text_scale, spans);
                }
            }''',
    "inline html",
)
text = replace_once(
    text,
    '''            InlineKind::InlineMath { source } => {
                let mut next = style;
                next.status = true;
                spans.push(styled_span(
                    format!("[Math inline non rendu: {source}]"),
                    next,
                    palette,
                    text_scale,
                ));
            }''',
    '''            InlineKind::InlineMath { source } => {
                let mut next = style;
                next.code = true;
                if node.children.is_empty() {
                    spans.push(styled_span(source.clone(), next, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, next, palette, text_scale, spans);
                }
            }''',
    "inline math",
)
text = replace_once(
    text,
    '''            InlineKind::Emoji { value, .. } => {
                spans.push(styled_span(value.clone(), style, palette, text_scale))
            }''',
    '''            InlineKind::Emoji { value, .. } => {
                if node.children.is_empty() {
                    spans.push(styled_span(value.clone(), style, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, style, palette, text_scale, spans);
                }
            }''',
    "emoji inline",
)
text = replace_once(
    text,
    '''            InlineKind::FootnoteReference { label } => spans.push(styled_span(
                format!("[footnote: {label}]"),
                style,
                palette,
                text_scale,
            )),
            InlineKind::SoftBreak | InlineKind::HardBreak => {
                spans.push(styled_span("\\n".to_string(), style, palette, text_scale))
            }''',
    '''            InlineKind::FootnoteReference { label } => {
                let mut next = style;
                next.script = ScriptStyle::Superscript;
                if node.children.is_empty() {
                    spans.push(styled_span(label.clone(), next, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, next, palette, text_scale, spans);
                }
            }
            InlineKind::SoftBreak | InlineKind::HardBreak => {
                if node.children.is_empty() {
                    spans.push(styled_span("\\n".to_string(), style, palette, text_scale));
                } else {
                    collect_inline_children(document, node_id, style, palette, text_scale, spans);
                }
            }''',
    "footnote/break inline",
)

forbidden = ["not rendered", "non rendu", "non rendue", "fn unsupported_block("]
leaked = [marker for marker in forbidden if marker in text]
if leaked:
    raise SystemExit(f"renderer placeholder markers remain: {leaked}")
required = ["fn render_paragraph(", "fn render_footnote_definition(", "fn semantic_source_block(", "super::editor_image_preview::render"]
missing = [marker for marker in required if marker not in text]
if missing:
    raise SystemExit(f"final renderer patch incomplete: {missing}")
VIEW.write_text(text)

if not TEST.exists():
    TEST.write_text(r'''use base64::{engine::general_purpose::STANDARD, Engine as _};
use elephant_freya::app::app_with_vault;
use freya_testing::TestingRunner;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn markdown_image_is_rendered_by_the_real_freya_note_surface_and_persists() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("elephant-freya-image-note-{stamp}"));
    fs::create_dir_all(root.join(".assets")).expect("create assets");
    let png = STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
        .expect("decode fixture PNG");
    fs::write(root.join(".assets/pixel.png"), png).expect("write image");
    let markdown = "# Image note\n\n![pixel](.assets/pixel.png)\n";
    fs::write(root.join("Alpha.md"), markdown).expect("write note");
    let note_path: PathBuf = root.join("Alpha.md");
    let app_root = root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let alpha = runner
        .find_many(|node, element| {
            (element.accessibility().builder.label() == Some("Alpha")).then_some(node)
        })
        .into_iter()
        .next()
        .expect("Alpha note card");
    runner.click_cursor(alpha.layout().area.center().to_f64());
    runner.sync_and_update();

    assert!(runner.find(|node, element| {
        (element.accessibility().builder.label() == Some("Image preview pixel")).then_some(node)
    }).is_some(), "local Markdown image must have a real native preview");

    let evidence = std::env::temp_dir().join("elephant-freya-note-image.png");
    runner.render_to_file(&evidence);
    assert!(fs::metadata(&evidence).expect("image screenshot").len() > 0);

    let save = runner
        .find_many(|node, element| {
            (element.accessibility().builder.label() == Some("Save")).then_some(node)
        })
        .into_iter()
        .next()
        .expect("Save button");
    runner.click_cursor(save.layout().area.center().to_f64());
    runner.sync_and_update();
    assert_eq!(fs::read_to_string(&note_path).expect("saved note"), markdown);

    let _ = fs::remove_dir_all(root);
}
''')
