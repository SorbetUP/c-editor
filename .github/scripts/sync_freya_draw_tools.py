#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    if old not in text:
        raise SystemExit(f"{label}: expected anchor not found")
    return text.replace(old, new, 1)


def sync_toolbar() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    text = replace_once(text, "    Image,\n    Eraser,\n}", "    Image,\n    Frame,\n    Eraser,\n}", "toolbar Frame enum")
    text = replace_once(text, "    Select,\n    Hand,", "    Select,\n    Lasso,\n    Hand,", "toolbar Lasso enum")
    if "const ALL: [Self; 11]" in text:
        text = text.replace("const ALL: [Self; 11]", "const ALL: [Self; 13]", 1)
    elif "const ALL: [Self; 12]" in text:
        text = text.replace("const ALL: [Self; 12]", "const ALL: [Self; 13]", 1)
    elif "const ALL: [Self; 13]" not in text:
        raise SystemExit("toolbar count: expected anchor not found")
    text = replace_once(text, "        Self::Image,\n        Self::Eraser,", "        Self::Image,\n        Self::Frame,\n        Self::Eraser,", "toolbar Frame list")
    text = replace_once(text, "        Self::Select,\n        Self::Rectangle,", "        Self::Select,\n        Self::Lasso,\n        Self::Rectangle,", "toolbar Lasso list")
    text = replace_once(text, '            Self::Image => "Image",\n            Self::Eraser => "Eraser",', '            Self::Image => "Image",\n            Self::Frame => "Frame",\n            Self::Eraser => "Eraser",', "toolbar Frame label")
    text = replace_once(text, '            Self::Select => "Selection",\n            Self::Hand => "Hand",', '            Self::Select => "Selection",\n            Self::Lasso => "Lasso",\n            Self::Hand => "Hand",', "toolbar Lasso label")
    text = replace_once(text, '            Self::Image => "9",\n            Self::Eraser => "0",', '            Self::Image => "9",\n            Self::Frame => "F",\n            Self::Eraser => "0",', "toolbar Frame shortcut")
    text = replace_once(text, '            Self::Select => "1",\n            Self::Hand => "H",', '            Self::Select => "1",\n            Self::Lasso => "V",\n            Self::Hand => "H",', "toolbar Lasso shortcut")
    selection_icon = '''            Self::Select => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m5 3 14 9-6 2-2 6Z"/></svg>"#,
            Self::Hand =>'''
    lasso_icon = '''            Self::Select => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m5 3 14 9-6 2-2 6Z"/></svg>"#,
            Self::Lasso => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 8c0 3-3 5-7 5S5 11 5 8s3-5 7-5 7 2 7 5Z"/><path d="M12 13c0 5 7 3 7 7 0 1-1 2-3 2"/></svg>"#,
            Self::Hand =>'''
    text = replace_once(text, selection_icon, lasso_icon, "Lasso icon")
    image_icon = '''            Self::Image => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>"#,
            Self::Eraser =>'''
    frame_icon = '''            Self::Image => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>"#,
            Self::Frame => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="1"/><path d="M8 4v16M16 4v16"/></svg>"#,
            Self::Eraser =>'''
    text = replace_once(text, image_icon, frame_icon, "Frame icon")
    if "fn main_toolbar_exposes_frame_and_lasso_tools()" not in text:
        text += '''

#[cfg(test)]
mod drawing_toolbar_tests {
    use super::DrawingTool;

    #[test]
    fn main_toolbar_exposes_frame_and_lasso_tools() {
        assert!(DrawingTool::ALL.contains(&DrawingTool::Frame));
        assert!(DrawingTool::ALL.contains(&DrawingTool::Lasso));
        assert_eq!(DrawingTool::Frame.label(), "Frame");
        assert_eq!(DrawingTool::Frame.shortcut(), "F");
        assert_eq!(DrawingTool::Lasso.label(), "Lasso");
        assert_eq!(DrawingTool::Lasso.shortcut(), "V");
    }
}
'''
    path.write_text(text)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    text = replace_once(text, "    Selection,\n    Hand,", "    Selection,\n    Lasso,\n    Hand,", "canvas Lasso enum")
    text = replace_once(text, '        match id {\n            "freehand"', '        match id {\n            "lasso" => Self::Lasso,\n            "freehand"', "canvas from_id")
    text = replace_once(text, '            Self::Selection => "selection",\n            Self::Hand => "hand",', '            Self::Selection => "selection",\n            Self::Lasso => "lasso",\n            Self::Hand => "hand",', "canvas Lasso id")
    text = replace_once(text, '            Self::Selection => "Selection",\n            Self::Hand => "Hand",', '            Self::Selection => "Selection",\n            Self::Lasso => "Lasso",\n            Self::Hand => "Hand",', "canvas Lasso label")
    text = replace_once(text, "                DrawingTool::Selection | DrawingTool::Hand | DrawingTool::Eraser => {", "                DrawingTool::Selection\n                | DrawingTool::Lasso\n                | DrawingTool::Hand\n                | DrawingTool::Eraser => {", "canvas pointer down")
    text = replace_once(text, "                DrawingTool::Selection | DrawingTool::Hand | DrawingTool::Eraser\n            ) {", "                DrawingTool::Selection\n                    | DrawingTool::Lasso\n                    | DrawingTool::Hand\n                    | DrawingTool::Eraser\n            ) {", "canvas pointer move")
    text = replace_once(text, "        .child(tool_button(state, DrawingTool::Selection))\n        .child(tool_button(state, DrawingTool::Freehand))", "        .child(tool_button(state, DrawingTool::Selection))\n        .child(tool_button(state, DrawingTool::Lasso))\n        .child(tool_button(state, DrawingTool::Freehand))", "canvas palette")
    text = replace_once(text, "        DrawingTool::Selection => elephant_draw::DrawingTool::Selection,\n        DrawingTool::Hand", "        DrawingTool::Selection => elephant_draw::DrawingTool::Selection,\n        DrawingTool::Lasso => elephant_draw::DrawingTool::Selection,\n        DrawingTool::Hand", "canvas core mapping")
    path.write_text(text)


def sync_scene() -> None:
    path = Path("Elephant/freya/src/app/drawing_scene.rs")
    text = path.read_text()
    text = replace_once(text, "    BoxSelect {\n        start_world: [f32; 2],\n        current_world: [f32; 2],\n    },\n    Pan {", "    BoxSelect {\n        start_world: [f32; 2],\n        current_world: [f32; 2],\n    },\n    LassoSelect {\n        points: Vec<[f32; 2]>,\n    },\n    Pan {", "scene Lasso interaction")
    text = replace_once(text, '            "Image" => "image",\n            "Eraser" => "eraser",', '            "Image" => "image",\n            "Frame" => "frame",\n            "Eraser" => "eraser",', "Frame state mapping")
    text = replace_once(text, '            "Hand" => "hand",\n            "Frame" => "frame",', '            "Hand" => "hand",\n            "Lasso" => "lasso",\n            "Frame" => "frame",', "Lasso state mapping")
    marquee = '''    pub fn selection_marquee_world(&self) -> Option<(f32, f32, f32, f32)> {
        let Interaction::BoxSelect {
            start_world,
            current_world,
        } = self.interaction
        else {
            return None;
        };
        Some(normalized_bounds(start_world, current_world))
    }
'''
    text = replace_once(text, marquee, marquee + '''
    pub fn selection_lasso_world(&self) -> Option<&[[f32; 2]]> {
        match &self.interaction {
            Interaction::LassoSelect { points } => Some(points.as_slice()),
            _ => None,
        }
    }
''', "scene Lasso accessor")
    text = replace_once(text, '''            "selection" => {
                if let Some(index) = self.hit_test(world) {''', '''            "lasso" => {
                self.clear_selection();
                self.interaction = Interaction::LassoSelect {
                    points: vec![world],
                };
                self.changed();
            }
            "selection" => {
                if let Some(index) = self.hit_test(world) {''', "scene Lasso begin")
    text = replace_once(text, '''                self.interaction = Interaction::BoxSelect {
                    start_world,
                    current_world: world,
                };
                self.changed();
            }
            Interaction::Pan {''', '''                self.interaction = Interaction::BoxSelect {
                    start_world,
                    current_world: world,
                };
                self.changed();
            }
            Interaction::LassoSelect { mut points } => {
                let world = self.to_world(point);
                let should_append = points.last().is_none_or(|last| {
                    let dx = world[0] - last[0];
                    let dy = world[1] - last[1];
                    dx * dx + dy * dy >= 4.0
                });
                if should_append {
                    points.push(world);
                }
                self.selection = self.document.select_in_lasso(&points, SelectionMode::Contained);
                self.selected = None;
                self.interaction = Interaction::LassoSelect { points };
                self.changed();
            }
            Interaction::Pan {''', "scene Lasso move")
    text = replace_once(text, "        if matches!(self.interaction, Interaction::BoxSelect { .. }) {", "        if matches!(\n            self.interaction,\n            Interaction::BoxSelect { .. } | Interaction::LassoSelect { .. }\n        ) {", "scene Lasso end")
    path.write_text(text)


def sync_render() -> None:
    path = Path("Elephant/freya/src/app/drawing_render.rs")
    text = path.read_text()
    marker = '''    if let Some(bounds) = state.selection_marquee_world() {
        output.push(selection_overlay(
            bounds,
            state.viewport,
            "Drawing selection marquee",
            24,
        ));
    }
    output
}'''
    replacement = '''    if let Some(bounds) = state.selection_marquee_world() {
        output.push(selection_overlay(
            bounds,
            state.viewport,
            "Drawing selection marquee",
            24,
        ));
    }
    if let Some(points) = state.selection_lasso_world() {
        output.push(svg::lasso(points, state.viewport));
    }
    output
}'''
    text = replace_once(text, marker, replacement, "render Lasso overlay")
    path.write_text(text)


def sync_render_svg() -> None:
    path = Path("Elephant/freya/src/app/drawing_render_svg.rs")
    text = path.read_text()
    if "pub(super) fn lasso(" in text:
        return
    anchor = "pub(super) fn selection(index: usize, element: &DrawingElement, viewport: Viewport) -> Element {"
    if anchor not in text:
        raise SystemExit("render svg Lasso anchor missing")
    lasso = '''pub(super) fn lasso(points: &[[f32; 2]], viewport: Viewport) -> Element {
    if points.len() < 2 {
        return rect().width(Size::px(1.0)).height(Size::px(1.0)).into_element();
    }
    let min_x = points.iter().map(|point| point[0]).fold(f32::INFINITY, f32::min);
    let min_y = points.iter().map(|point| point[1]).fold(f32::INFINITY, f32::min);
    let max_x = points.iter().map(|point| point[0]).fold(f32::NEG_INFINITY, f32::max);
    let max_y = points.iter().map(|point| point[1]).fold(f32::NEG_INFINITY, f32::max);
    let mut path = String::new();
    let mut signature = 0_u64;
    for (index, [x, y]) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(path, "{command} {} {} ", x - min_x, y - min_y);
        signature = signature
            .wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(x.to_bits()))
            .wrapping_add(u64::from(y.to_bits()).rotate_left(17));
    }
    if points.len() > 2 {
        path.push('Z');
    }
    let content = format!(
        "<path d=\"{path}\" fill=\"rgb(105,101,219)\" fill-opacity=\"0.08\" stroke=\"rgb(105,101,219)\" stroke-width=\"1.5\" stroke-dasharray=\"5 4\" stroke-linejoin=\"round\"/>"
    );
    padded_surface(
        ("drawing-lasso", signature),
        viewport,
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
        content,
        "Drawing lasso",
        4.0,
    )
}

'''
    path.write_text(text.replace(anchor, lasso + anchor, 1))


if __name__ == "__main__":
    sync_toolbar()
    sync_canvas()
    sync_scene()
    sync_render()
    sync_render_svg()
