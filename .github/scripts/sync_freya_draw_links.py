#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def sync_scene() -> None:
    path = Path("Elephant/freya/src/app/drawing_scene.rs")
    text = path.read_text()
    anchor = '''    pub fn selected_element(&self) -> Option<&DrawingElement> {
'''
    methods = '''    pub fn selected_link(&self) -> Option<&str> {
        let id = self.selected_element_id()?;
        self.document.element_link(id)
    }

    pub fn set_selected_link(&mut self, link: Option<&str>) -> bool {
        let Some(id) = self.selected_element_id().map(str::to_owned) else {
            return false;
        };
        let before = self.document.clone();
        if !self.document.set_element_link(&id, link) {
            return false;
        }
        self.history.push(before);
        self.changed();
        true
    }

'''
    text = replace_once(text, anchor, methods + anchor, "link state methods")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()

    signature_old = '''fn properties_panel(canvas: State<DrawingCanvasState>) -> Element {'''
    signature_new = '''fn properties_panel(
    canvas: State<DrawingCanvasState>,
    link_open: State<bool>,
    link_value: State<String>,
) -> Element {'''
    text = replace_once(text, signature_old, signature_new, "property panel link args")

    vars_anchor = '''    let mut back = canvas;

    rect()'''
    vars_new = '''    let mut back = canvas;
    let link_canvas = canvas;
    let mut edit_link_open = link_open;
    let mut edit_link_value = link_value;
    let mut remove_link = canvas;

    rect()'''
    text = replace_once(text, vars_anchor, vars_new, "property link vars")

    layers_tail = '''                .child(action_button("↙", "Send to back", 48.0, move |_| {
                    back.write().send_selection_to_back();
                })),
        )
        .into_element()
}'''
    link_section = '''                .child(action_button("↙", "Send to back", 48.0, move |_| {
                    back.write().send_selection_to_back();
                })),
        )
        .child(label().font_size(15.0).color(text).text("Link"))
        .child(
            rect()
                .horizontal()
                .spacing(8.0)
                .child(action_button("Edit", "Edit selected element link", 62.0, move |_| {
                    edit_link_value.set(
                        link_canvas
                            .read()
                            .selected_link()
                            .unwrap_or_default()
                            .to_owned(),
                    );
                    edit_link_open.set(true);
                }))
                .child(action_button("Remove", "Remove selected element link", 70.0, move |_| {
                    remove_link.write().set_selected_link(None);
                })),
        )
        .into_element()
}'''
    text = replace_once(text, layers_tail, link_section, "property link controls")

    panel_anchor = '''fn zoom_controls(canvas: State<DrawingCanvasState>) -> Element {'''
    panel = '''fn link_panel(
    mut open: State<bool>,
    value: State<String>,
    canvas: State<DrawingCanvasState>,
) -> Element {
    let mut save_canvas = canvas;
    let save_value = value;
    let mut save_open = open;
    rect()
        .position(Position::new_absolute().left(180.0).top(92.0))
        .width(Size::px(420.0))
        .padding(Gaps::new_all(12.0))
        .spacing(8.0)
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(80))
        .with_corner_radius(10.0)
        .a11y_alt("Edit drawing link")
        .child(
            Input::new(value)
                .placeholder("https://example.com or person@example.com")
                .auto_focus(true)
                .width(Size::fill())
                .on_submit(move |submitted| {
                    save_canvas.write().set_selected_link(Some(&submitted));
                    save_open.set(false);
                }),
        )
        .child(action_button("Save", "Save drawing link", 70.0, move |_| {
            save_canvas
                .write()
                .set_selected_link(Some(save_value.read().as_str()));
            save_open.set(false);
        }))
        .child(icon_button(
            X_ICON,
            "Close link editor",
            30.0,
            Color::from_rgb(235, 235, 240),
            Color::TRANSPARENT,
            move |_| open.set(false),
        ))
        .into_element()
}

'''
    text = replace_once(text, panel_anchor, panel + panel_anchor, "link editor overlay")

    state_anchor = '''        let search_open = use_state(|| false);
        let search_query = use_state(String::new);
        let menu_open = use_state(|| false);'''
    state_new = '''        let search_open = use_state(|| false);
        let search_query = use_state(String::new);
        let link_open = use_state(|| false);
        let link_value = use_state(String::new);
        let menu_open = use_state(|| false);'''
    text = replace_once(text, state_anchor, state_new, "link states")

    capture_anchor = '''        let mut keyboard_search_open = search_open;
        let mut keyboard_search_query = search_query;

        rect()'''
    capture_new = '''        let mut keyboard_search_open = search_open;
        let mut keyboard_search_query = search_query;
        let mut keyboard_link_open = link_open;

        rect()'''
    text = replace_once(text, capture_anchor, capture_new, "link keyboard state")

    escape_old = '''                        if *keyboard_search_open.read() {
                            keyboard_search_open.set(false);
                            keyboard_search_query.set(String::new());
                        } else {
                            keyboard_canvas.write().cancel_interaction();
                            keyboard_tool.set(DrawingTool::Select);
                        }'''
    escape_new = '''                        if *keyboard_link_open.read() {
                            keyboard_link_open.set(false);
                        } else if *keyboard_search_open.read() {
                            keyboard_search_open.set(false);
                            keyboard_search_query.set(String::new());
                        } else {
                            keyboard_canvas.write().cancel_interaction();
                            keyboard_tool.set(DrawingTool::Select);
                        }'''
    text = replace_once(text, escape_old, escape_new, "link escape")

    properties_old = '''                    .then(|| properties_panel(canvas_state)),'''
    properties_new = '''                    .then(|| properties_panel(canvas_state, link_open, link_value)),'''
    text = replace_once(text, properties_old, properties_new, "link property invocation")

    overlay_anchor = '''            .maybe_child((*search_open.read()).then(|| {
                search_panel(search_open, search_query, canvas_state)
            }))
'''
    overlay_new = overlay_anchor + '''            .maybe_child((*link_open.read()).then(|| {
                link_panel(link_open, link_value, canvas_state)
            }))
'''
    text = replace_once(text, overlay_anchor, overlay_new, "link overlay")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_view()
