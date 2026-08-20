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
    anchor = '''    pub fn selected_element_ids(&self) -> Vec<&str> {
        if self.selection.is_empty() {
            return self.selected_element_id().into_iter().collect();
        }
        self.selection.ids().collect()
    }
'''
    methods = '''
    pub fn focus_element(&mut self, id: &str, screen_center: [f32; 2]) -> bool {
        let Some(index) = self.document.element_index(id) else {
            return false;
        };
        let Some(element) = self.document.elements.get(index) else {
            return false;
        };
        if element.is_deleted || is_locked(element) {
            return false;
        }
        let (x, y, width, height) = element.bounds();
        self.selection = SelectionSet::from_ids(std::iter::once(id.to_owned()));
        self.selected = Some(index);
        self.interaction = Interaction::None;
        self.snap_guides.clear();
        self.active_tool = "selection".to_owned();
        self.viewport.pan = [
            screen_center[0] - (x + width / 2.0) * self.viewport.zoom,
            screen_center[1] - (y + height / 2.0) * self.viewport.zoom,
        ];
        self.changed();
        true
    }

'''
    text = replace_once(text, anchor, anchor + methods, "focus search result")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()

    helper_anchor = '''fn zoom_controls(canvas: State<DrawingCanvasState>) -> Element {'''
    helper = '''fn search_panel(
    mut open: State<bool>,
    query: State<String>,
    canvas: State<DrawingCanvasState>,
) -> Element {
    let needle = query.read().clone();
    let results = canvas.read().document.search_scene(&needle);
    let mut rows = Vec::new();
    for result in results.into_iter().take(30) {
        let id = result.element_id.clone();
        let field = match result.field {
            elephant_draw::SearchField::Text => "Text",
            elephant_draw::SearchField::FrameName => "Frame",
            elephant_draw::SearchField::Link => "Link",
        };
        let caption = format!("{field}: {}", result.value.replace('\n', " "));
        let mut target_canvas = canvas;
        let mut target_open = open;
        rows.push(action_button(caption, format!("Open search result {id}"), 430.0, move |_| {
            target_canvas.write().focus_element(&id, [350.0, 250.0]);
            target_open.set(false);
        }));
    }
    rect()
        .position(Position::new_absolute().left(130.0).right(130.0).top(78.0))
        .padding(Gaps::new_all(12.0))
        .spacing(8.0)
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(70))
        .with_corner_radius(10.0)
        .a11y_alt("Search drawing")
        .child(
            Input::new(query)
                .placeholder("Search text, frames and links")
                .auto_focus(true)
                .width(Size::fill()),
        )
        .maybe_child((!needle.is_empty() && rows.is_empty()).then(|| {
            label()
                .font_size(13.0)
                .color(Color::from_rgb(190, 190, 198))
                .text("No matches")
        }))
        .children(rows)
        .child(icon_button(
            X_ICON,
            "Close drawing search",
            30.0,
            Color::from_rgb(235, 235, 240),
            Color::TRANSPARENT,
            move |_| open.set(false),
        ))
        .into_element()
}

'''
    text = replace_once(text, helper_anchor, helper + helper_anchor, "search panel")

    state_anchor = '''        let library_open = use_state(|| false);
        let menu_open = use_state(|| false);'''
    state_new = '''        let library_open = use_state(|| false);
        let search_open = use_state(|| false);
        let search_query = use_state(String::new);
        let menu_open = use_state(|| false);'''
    text = replace_once(text, state_anchor, state_new, "search states")

    capture_anchor = '''        let mut keyboard_canvas = canvas_state;

        rect()'''
    capture_new = '''        let mut keyboard_canvas = canvas_state;
        let mut keyboard_search_open = search_open;
        let mut keyboard_search_query = search_query;

        rect()'''
    text = replace_once(text, capture_anchor, capture_new, "search keyboard state")

    command_anchor = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("c")) {'''
    command_new = '''                if command {
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("f")) {
                        keyboard_search_open.set(true);
                        event.stop_propagation();
                        return;
                    }
                    if matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("c")) {'''
    text = replace_once(text, command_anchor, command_new, "search shortcut")

    escape_old = '''                    Key::Named(NamedKey::Escape) => {
                        keyboard_canvas.write().cancel_interaction();
                        keyboard_tool.set(DrawingTool::Select);
                        event.stop_propagation();
                    }'''
    escape_new = '''                    Key::Named(NamedKey::Escape) => {
                        if *keyboard_search_open.read() {
                            keyboard_search_open.set(false);
                            keyboard_search_query.set(String::new());
                        } else {
                            keyboard_canvas.write().cancel_interaction();
                            keyboard_tool.set(DrawingTool::Select);
                        }
                        event.stop_propagation();
                    }'''
    text = replace_once(text, escape_old, escape_new, "search escape")

    overlay_anchor = '''            .maybe_child((*menu_open.read()).then(|| menu_panel(menu_open, canvas_state)))
'''
    overlay_new = overlay_anchor + '''            .maybe_child((*search_open.read()).then(|| {
                search_panel(search_open, search_query, canvas_state)
            }))
'''
    text = replace_once(text, overlay_anchor, overlay_new, "search overlay")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_view()
