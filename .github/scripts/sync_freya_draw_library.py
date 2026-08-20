#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


path = Path("Elephant/freya/src/app/drawing.rs")
text = path.read_text()
text = replace_once(
    text,
    '#[path = "drawing_storage.rs"]\nmod storage;\n',
    '#[path = "drawing_storage.rs"]\nmod storage;\n#[path = "drawing_library.rs"]\nmod drawing_library;\n',
    "drawing library module",
)
text = replace_once(
    text,
    'use serde_json::{json, Map, Value};\n',
    'use serde_json::{json, Map, Value};\nuse std::{path::PathBuf, time::{SystemTime, UNIX_EPOCH}};\n',
    "drawing library imports",
)

old_panel = '''fn library_panel(mut open: State<bool>) -> Element {
    rect()
        .position(Position::new_absolute().right(0.0).top(0.0).bottom(0.0))
        .width(Size::px(320.0))
        .padding(Gaps::new_all(18.0))
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .a11y_alt("Excalidraw Library")
        .child(label().font_size(22.0).color(Color::from_rgb(170, 160, 255)).text("Library"))
        .child(label().font_size(14.0).color(Color::from_rgb(190, 190, 198)).text("No library items in this vault."))
        .child(icon_button(X_ICON, "Close library", 34.0, Color::from_rgb(235, 235, 240), Color::TRANSPARENT, move |_| open.set(false)))
        .into_element()
}
'''
new_panel = '''fn library_panel(
    mut open: State<bool>,
    canvas: State<DrawingCanvasState>,
    library: State<elephant_draw::LibraryFile>,
    root: Option<PathBuf>,
    error: State<Option<String>>,
) -> Element {
    let items = library.read().library_items.clone();
    let mut rows = Vec::new();
    for item in items {
        let title = item.name.clone().unwrap_or_else(|| item.id.clone());
        let insert_item = item.clone();
        let delete_id = item.id.clone();
        let mut insert_canvas = canvas;
        let mut delete_library = library;
        let delete_root = root.clone();
        let mut delete_error = error;
        rows.push(
            rect()
                .width(Size::fill())
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.0)
                .padding(Gaps::new_all(6.0))
                .background(Color::from_rgb(48, 48, 58))
                .with_corner_radius(7.0)
                .a11y_alt(format!("Library item {title}"))
                .child(
                    label()
                        .width(Size::flex(1.0))
                        .font_size(13.0)
                        .color(Color::from_rgb(235, 235, 240))
                        .text(title),
                )
                .child(action_button("Insert", "Insert library item", 62.0, move |_| {
                    let fragment = elephant_draw::SceneFragment {
                        elements: insert_item.elements.clone(),
                        files: insert_item.files.clone(),
                    };
                    insert_canvas.write().paste_fragment(&fragment);
                }))
                .child(action_button("×", "Delete library item", 34.0, move |_| {
                    if !delete_library.write().remove(&delete_id) {
                        return;
                    }
                    if let Some(root) = delete_root.as_deref() {
                        let snapshot = delete_library.read().clone();
                        delete_error.set(drawing_library::save(root, &snapshot).err());
                    }
                })),
        );
    }

    let mut add_library = library;
    let add_canvas = canvas;
    let add_root = root.clone();
    let mut add_error = error;
    rect()
        .position(Position::new_absolute().right(0.0).top(0.0).bottom(0.0))
        .width(Size::px(340.0))
        .padding(Gaps::new_all(18.0))
        .spacing(10.0)
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(50))
        .a11y_alt("Excalidraw Library")
        .child(
            label()
                .font_size(22.0)
                .color(Color::from_rgb(170, 160, 255))
                .text("Library"),
        )
        .child(action_button(
            "Add selection",
            "Add selection to library",
            150.0,
            move |_| {
                let Some(fragment) = add_canvas.read().copy_selection_fragment() else {
                    return;
                };
                let created = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|duration| duration.as_millis() as u64)
                    .unwrap_or(0);
                let id = format!(
                    "elephant-library-{}-{}",
                    created,
                    add_library.read().library_items.len()
                );
                add_library.write().library_items.push(elephant_draw::LibraryItem {
                    id,
                    status: elephant_draw::LibraryItemStatus::Unpublished,
                    created,
                    elements: fragment.elements,
                    name: Some("Selection".to_owned()),
                    files: fragment.files,
                });
                if let Some(root) = add_root.as_deref() {
                    let snapshot = add_library.read().clone();
                    add_error.set(drawing_library::save(root, &snapshot).err());
                }
            },
        ))
        .maybe_child(rows.is_empty().then(|| {
            label()
                .font_size(14.0)
                .color(Color::from_rgb(190, 190, 198))
                .text("No library items yet. Select elements and add them.")
        }))
        .children(rows)
        .child(icon_button(
            X_ICON,
            "Close library",
            34.0,
            Color::from_rgb(235, 235, 240),
            Color::TRANSPARENT,
            move |_| open.set(false),
        ))
        .into_element()
}
'''
text = replace_once(text, old_panel, new_panel, "real library panel")

state_anchor = '''        let toolbar_error = use_state(|| Option::<String>::None);
        let active_tool = use_state(|| DrawingTool::Select);'''
state_new = '''        let toolbar_error = use_state(|| Option::<String>::None);
        let library_root = state
            .read()
            .vault
            .as_ref()
            .map(|vault| vault.root().to_path_buf());
        let initial_library_root = library_root.clone();
        let library_state = use_state(move || {
            initial_library_root
                .as_deref()
                .and_then(|root| drawing_library::load(root).ok())
                .unwrap_or_else(|| elephant_draw::LibraryFile::empty("elephant"))
        });
        let active_tool = use_state(|| DrawingTool::Select);'''
text = replace_once(text, state_anchor, state_new, "library view state")
text = replace_once(
    text,
    '            .maybe_child((*library_open.read()).then(|| library_panel(library_open)))\n',
    '''            .maybe_child((*library_open.read()).then(|| {
                library_panel(
                    library_open,
                    canvas_state,
                    library_state,
                    library_root.clone(),
                    toolbar_error,
                )
            }))
''',
    "library panel invocation",
)
path.write_text(text)
