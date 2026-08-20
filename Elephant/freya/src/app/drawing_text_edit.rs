use freya::prelude::*;
use serde_json::Value;

use super::drawing_scene::DrawingCanvasState;

#[derive(PartialEq)]
pub(super) struct TextEditOverlay {
    pub canvas: State<DrawingCanvasState>,
    pub editing: State<Option<usize>>,
    pub index: usize,
}

impl Component for TextEditOverlay {
    fn render_key(&self) -> DiffKey {
        self.index.into()
    }

    fn render(&self) -> impl IntoElement {
        let canvas = self.canvas;
        let index = self.index;
        let snapshot = canvas.read();
        let Some(element) = snapshot.document.elements.get(index).cloned() else {
            return rect().into_element();
        };
        if element.is_deleted || element.kind != "text" {
            return rect().into_element();
        }
        let viewport = snapshot.viewport;
        drop(snapshot);

        let initial = element.text.clone();
        let holder = use_state(ParagraphHolder::default);
        let mut editable = use_editable(move || initial.clone(), EditableConfig::new);
        let focus = use_focus();
        let mut focus_once = focus;
        use_side_effect(move || {
            focus_once.request_focus();
        });

        let layout = elephant_draw::layout_text(&element);
        let width = layout.width.max(element.width).max(120.0);
        let height = layout
            .height
            .max(element.height)
            .max(element.font_size * 1.25)
            .max(30.0);
        let cursor = editable.editor().read().cursor_pos();
        let highlights = editable
            .editor()
            .read()
            .get_selection()
            .map(|selection| vec![selection])
            .unwrap_or_default();

        let mut down_editable = editable;
        let down_holder = holder;
        let down_focus = focus;
        let mut move_editable = editable;
        let move_holder = holder;
        let mut release_editable = editable;
        let mut key_editable = editable;
        let mut key_canvas = canvas;
        let mut key_editing = self.editing;
        let mut key_up_editable = editable;
        let mut key_up_canvas = canvas;

        rect()
            .position(
                Position::new_absolute()
                    .left(viewport.pan[0] + element.x * viewport.zoom)
                    .top(viewport.pan[1] + element.y * viewport.zoom),
            )
            .width(Size::px(width * viewport.zoom))
            .height(Size::px(height * viewport.zoom))
            .padding(Gaps::new_all(2.0))
            .background(Color::from_argb(18, 255, 255, 255))
            .border(Border::new().fill(Color::from_rgb(105, 101, 219)).width(1.0))
            .with_corner_radius(3.0)
            .layer(Layer::OverlayLevel(45))
            .a11y_alt("Drawing text editor")
            .child(
                paragraph()
                    .a11y_id(focus.a11y_id())
                    .width(Size::fill())
                    .height(Size::fill())
                    .font_size(element.font_size.max(1.0) * viewport.zoom)
                    .color(parse_color(&element.stroke_color))
                    .cursor_index(cursor)
                    .highlights(highlights)
                    .holder(holder.read().clone())
                    .on_mouse_down(move |event: Event<MouseEventData>| {
                        down_focus.request_focus();
                        down_editable.process_event(EditableEvent::Down {
                            location: event.element_location,
                            editor_line: EditorLine::SingleParagraph,
                            holder: &down_holder.read(),
                        });
                        event.stop_propagation();
                    })
                    .on_mouse_move(move |event: Event<MouseEventData>| {
                        move_editable.process_event(EditableEvent::Move {
                            location: event.element_location,
                            editor_line: EditorLine::SingleParagraph,
                            holder: &move_holder.read(),
                        });
                        event.stop_propagation();
                    })
                    .on_global_pointer_up(move |_| {
                        release_editable.process_event(EditableEvent::Release);
                    })
                    .on_key_down(move |event: Event<KeyboardEventData>| {
                        if matches!(event.key, Key::Named(NamedKey::Escape)) {
                            sync_editor(&mut key_canvas, index, &key_editable.editor().read().to_string());
                            key_editing.set(None);
                            event.stop_propagation();
                            return;
                        }
                        key_editable.process_event(EditableEvent::KeyDown {
                            key: &event.key,
                            modifiers: event.modifiers,
                        });
                        sync_editor(&mut key_canvas, index, &key_editable.editor().read().to_string());
                        event.stop_propagation();
                    })
                    .on_key_up(move |event: Event<KeyboardEventData>| {
                        key_up_editable.process_event(EditableEvent::KeyUp { key: &event.key });
                        sync_editor(
                            &mut key_up_canvas,
                            index,
                            &key_up_editable.editor().read().to_string(),
                        );
                        event.stop_propagation();
                    })
                    .span(editable.editor().read().to_string()),
            )
            .into_element()
    }
}

fn sync_editor(canvas: &mut State<DrawingCanvasState>, index: usize, text: &str) {
    let mut state = canvas.write();
    let mut changed = false;
    {
        let Some(element) = state.document.elements.get_mut(index) else {
            return;
        };
        if element.is_deleted || element.kind != "text" || element.text == text {
            return;
        }
        element.text = text.to_owned();
        element
            .extra
            .insert("originalText".to_owned(), Value::String(text.to_owned()));
        let layout = elephant_draw::layout_text(element);
        if element
            .extra
            .get("autoResize")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            element.width = layout.width.max(1.0);
            element.height = layout.height.max(element.font_size.max(1.0));
        }
        changed = true;
    }
    if changed {
        state.touch_element(index);
    }
}

fn parse_color(value: &str) -> Color {
    let value = value.trim();
    if value.len() == 7 && value.starts_with('#') {
        let r = u8::from_str_radix(&value[1..3], 16).ok();
        let g = u8::from_str_radix(&value[3..5], 16).ok();
        let b = u8::from_str_radix(&value[5..7], 16).ok();
        if let (Some(r), Some(g), Some(b)) = (r, g, b) {
            return Color::from_rgb(r, g, b);
        }
    }
    Color::from_rgb(30, 30, 30)
}
