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

    if "use elephant_draw::{LaserTrails, LASER_SIZE};" not in text:
        first_semicolon = text.find("};")
        if first_semicolon < 0:
            raise SystemExit("laser scene import: elephant_draw export block missing")
        first_semicolon += 2
        text = text[:first_semicolon] + "\nuse elephant_draw::{LaserTrails, LASER_SIZE};" + text[first_semicolon:]

    text = replace_once(
        text,
        "    history: HistoryState,\n",
        "    history: HistoryState,\n    laser_trails: LaserTrails,\n",
        "laser trail state",
    )
    text = replace_once(
        text,
        "            history: HistoryState::default(),\n",
        "            history: HistoryState::default(),\n            laser_trails: LaserTrails::default(),\n",
        "laser trail initialization",
    )

    tool_anchor = '            "Embed" => "embeddable",\n            _ => "selection",'
    if tool_anchor in text:
        text = text.replace(
            tool_anchor,
            '            "Embed" => "embeddable",\n            "Laser" => "laser",\n            _ => "selection",',
            1,
        )
    elif '            "Laser" => "laser",' not in text:
        raise SystemExit("scene laser mapping: expected Embed mapping")

    methods_anchor = '''    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }
'''
    methods = '''    pub fn begin_laser_path(&mut self, world: [f32; 2], timestamp_ms: f64) {
        self.laser_trails
            .start_path(world[0], world[1], timestamp_ms);
        self.changed();
    }

    pub fn move_laser_path(&mut self, world: [f32; 2], timestamp_ms: f64) -> bool {
        let changed = self
            .laser_trails
            .add_point_to_path(world[0], world[1], timestamp_ms);
        if changed {
            self.changed();
        }
        changed
    }

    pub fn end_laser_path(&mut self) -> bool {
        let changed = self.laser_trails.end_path();
        if changed {
            self.changed();
        }
        changed
    }

    pub fn laser_outlines_at(&self, now_ms: f64) -> Vec<Vec<[f32; 2]>> {
        self.laser_trails.stroke_outlines(
            now_ms,
            Some(LASER_SIZE / self.viewport.zoom.max(0.01)),
        )
    }

    pub fn prune_laser_at(&mut self, now_ms: f64) {
        let before = self.laser_trails.past().len();
        self.laser_trails.prune(now_ms);
        if before != self.laser_trails.past().len() {
            self.changed();
        }
    }

    pub fn has_current_laser_path(&self) -> bool {
        self.laser_trails.has_current_trail()
    }

'''
    text = replace_once(text, methods_anchor, methods + methods_anchor, "laser state methods")
    path.write_text(text)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()

    text = replace_once(
        text,
        "use freya::prelude::*;",
        "use freya::{animation::*, prelude::*};",
        "laser animation import",
    )
    if "fn laser_now_ms() -> f64" not in text:
        import_anchor = "use serde_json::{json, Map, Value};\n"
        if import_anchor not in text:
            raise SystemExit("laser clock import anchor missing")
        text = text.replace(
            import_anchor,
            import_anchor + "use std::time::{SystemTime, UNIX_EPOCH};\n",
            1,
        )

    text = replace_once(
        text,
        "    MagicFrame,\n    Embeddable,\n}",
        "    MagicFrame,\n    Embeddable,\n    Laser,\n}",
        "canvas laser enum",
    )
    text = replace_once(
        text,
        '            "embeddable" | "iframe" => Self::Embeddable,\n            _ => Self::Selection,',
        '            "embeddable" | "iframe" => Self::Embeddable,\n            "laser" => Self::Laser,\n            _ => Self::Selection,',
        "canvas laser from id",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "embeddable",\n        }',
        '            Self::Embeddable => "embeddable",\n            Self::Laser => "laser",\n        }',
        "canvas laser id",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "Embed",\n        }',
        '            Self::Embeddable => "Embed",\n            Self::Laser => "Laser",\n        }',
        "canvas laser label",
    )

    state_anchor = '''    let editing_text_state = use_state(|| Option::<usize>::None);
'''
    animation = '''    let mut laser_animation = use_animation(|_| AnimNum::new(0.0, 1.0).time(1_050));
    // Reading the animated value makes Freya repaint for the full 1s decay.
    let _laser_frame = laser_animation.read().value();
'''
    text = replace_once(text, state_anchor, state_anchor + animation, "laser animation state")

    capture_anchor = '''    let mut pointer_state = state;
'''
    captures = '''    let mut pointer_laser_animation = laser_animation;
    let mut move_laser_animation = laser_animation;
    let mut release_laser_animation = laser_animation;
'''
    text = replace_once(text, capture_anchor, capture_anchor + captures, "laser animation captures")

    image_anchor = '''                DrawingTool::Image => {
                    pointer_gesture.set(None);
                    pointer_down.set(false);
                    match drawing_image::pick_image() {'''
    laser_branch = '''                DrawingTool::Laser => {
                    let world = pointer_state.read().to_world(screen);
                    pointer_state
                        .write()
                        .begin_laser_path(world, laser_now_ms());
                    pointer_gesture.set(None);
                    pointer_laser_animation.reset();
                    pointer_laser_animation.start();
                }
'''
    text = replace_once(text, image_anchor, laser_branch + image_anchor, "laser pointer branch")

    move_anchor = '''            if let Some(gesture) = *move_gesture.read() {
                draw_gesture(&mut move_state.write(), gesture, location);
            } else if matches!(
                DrawingTool::from_id(&move_state.read().active_tool),
                DrawingTool::Selection
                    | DrawingTool::Lasso
                    | DrawingTool::Hand
                    | DrawingTool::Eraser
            ) {
                move_state.write().move_pointer(location);
            }'''
    move_new = '''            if DrawingTool::from_id(&move_state.read().active_tool) == DrawingTool::Laser {
                let world = move_state.read().to_world(location);
                move_state
                    .write()
                    .move_laser_path(world, laser_now_ms());
                if !move_laser_animation.is_running() {
                    move_laser_animation.reset();
                    move_laser_animation.start();
                }
            } else if let Some(gesture) = *move_gesture.read() {
                draw_gesture(&mut move_state.write(), gesture, location);
            } else if matches!(
                DrawingTool::from_id(&move_state.read().active_tool),
                DrawingTool::Selection
                    | DrawingTool::Lasso
                    | DrawingTool::Hand
                    | DrawingTool::Eraser
            ) {
                move_state.write().move_pointer(location);
            }'''
    text = replace_once(text, move_anchor, move_new, "laser pointer move")

    release_anchor = '''            release_pointer_down.set(false);
            if let Some(gesture) = *local_end_gesture.read() {
                finalize_gesture(&mut local_end_state.write(), gesture);
            }
            local_end_state.write().end_pointer();
            local_end_gesture.set(None);
            event.stop_propagation();'''
    release_new = '''            release_pointer_down.set(false);
            if DrawingTool::from_id(&local_end_state.read().active_tool) == DrawingTool::Laser {
                local_end_state.write().end_laser_path();
                release_laser_animation.reset();
                release_laser_animation.start();
            } else {
                if let Some(gesture) = *local_end_gesture.read() {
                    finalize_gesture(&mut local_end_state.write(), gesture);
                }
                local_end_state.write().end_pointer();
            }
            local_end_gesture.set(None);
            event.stop_propagation();'''
    text = replace_once(text, release_anchor, release_new, "laser pointer release")

    text = replace_once(
        text,
        '''        DrawingTool::Embeddable => elephant_draw::DrawingTool::Embeddable,
    };''',
        '''        DrawingTool::Embeddable => elephant_draw::DrawingTool::Embeddable,
        DrawingTool::Laser => elephant_draw::DrawingTool::Freehand,
    };''',
        "laser exhaustive core mapping",
    )

    helper_anchor = '''fn point(value: CursorPoint) -> [f32; 2] {'''
    helper = '''fn laser_now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1_000.0
}

'''
    text = replace_once(text, helper_anchor, helper + helper_anchor, "laser clock helper")
    path.write_text(text)


def sync_render() -> None:
    path = Path("Elephant/freya/src/app/drawing_render.rs")
    text = path.read_text()
    if "fn laser_now_ms() -> f64" not in text:
        text = text.replace(
            "use freya::prelude::*;\n",
            "use freya::prelude::*;\nuse std::time::{SystemTime, UNIX_EPOCH};\n",
            1,
        )
    anchor = '''    for (index, guide) in state.snap_guides().iter().enumerate() {
        output.push(snap_guide(guide, state.viewport, index));
    }
    output
}'''
    new = '''    for (index, guide) in state.snap_guides().iter().enumerate() {
        output.push(snap_guide(guide, state.viewport, index));
    }
    for (index, outline) in state.laser_outlines_at(laser_now_ms()).iter().enumerate() {
        output.push(svg::laser(outline, state.viewport, index));
    }
    output
}

fn laser_now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1_000.0
}'''
    text = replace_once(text, anchor, new, "laser overlay render")
    path.write_text(text)


def sync_render_svg() -> None:
    path = Path("Elephant/freya/src/app/drawing_render_svg.rs")
    text = path.read_text()
    if "pub(super) fn laser(" in text:
        return
    anchor = '''pub(super) fn selection(index: usize, element: &DrawingElement, viewport: Viewport) -> Element {'''
    if anchor not in text:
        raise SystemExit("laser svg anchor missing")
    laser = '''pub(super) fn laser(outline: &[[f32; 2]], viewport: Viewport, index: usize) -> Element {
    if outline.len() < 3 {
        return rect()
            .width(Size::px(1.0))
            .height(Size::px(1.0))
            .into_element();
    }
    let min_x = outline
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let min_y = outline
        .iter()
        .map(|point| point[1])
        .fold(f32::INFINITY, f32::min);
    let max_x = outline
        .iter()
        .map(|point| point[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = outline
        .iter()
        .map(|point| point[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let mut path = String::new();
    let mut signature = 0_u64;
    for (point_index, [x, y]) in outline.iter().enumerate() {
        let command = if point_index == 0 { 'M' } else { 'L' };
        let _ = write!(path, "{command} {} {} ", x - min_x, y - min_y);
        signature = signature
            .wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(x.to_bits()))
            .wrapping_add(u64::from(y.to_bits()).rotate_left(17));
    }
    path.push('Z');
    let content = format!("<path d=\\\"{path}\\\" fill=\\\"red\\\" stroke=\\\"none\\\"/>");
    padded_surface(
        ("drawing-laser", index, signature),
        viewport,
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
        content,
        "Drawing laser trail",
        2.0,
    )
}

'''
    path.write_text(text.replace(anchor, laser + anchor, 1))


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "    MagicFrame,\n    Embeddable,\n    Eraser,",
        "    MagicFrame,\n    Embeddable,\n    Laser,\n    Eraser,",
        "view laser enum",
    )
    text = replace_once(
        text,
        '            Self::Embeddable => "Embed",\n            Self::Eraser => "Eraser",',
        '            Self::Embeddable => "Embed",\n            Self::Laser => "Laser",\n            Self::Eraser => "Eraser",',
        "view laser label",
    )
    text = replace_once(
        text,
        '            Self::MagicFrame | Self::Embeddable => "",\n            Self::Eraser => "0",',
        '            Self::MagicFrame | Self::Embeddable => "",\n            Self::Laser => "K",\n            Self::Eraser => "0",',
        "view laser shortcut",
    )
    text = replace_once(
        text,
        '            Self::MagicFrame | Self::Embeddable => false,\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        '            Self::MagicFrame | Self::Embeddable => false,\n            Self::Laser => value == "k",\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        "view laser shortcut matching",
    )
    icon_anchor = '''            Self::Embeddable => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m9 9-3 3 3 3"/><path d="m15 9 3 3-3 3"/></svg>"#,
            Self::Eraser =>'''
    icon_new = '''            Self::Embeddable => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m9 9-3 3 3 3"/><path d="m15 9 3 3-3 3"/></svg>"#,
            Self::Laser => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 21 14 10"/><path d="m12 4 2 2"/><path d="m18 3 .5 2.5L21 6l-2.5.5L18 9l-.5-2.5L15 6l2.5-.5Z"/></svg>"#,
            Self::Eraser =>'''
    text = replace_once(text, icon_anchor, icon_new, "view laser icon")

    embed_panel = '''        .child(tool_button(
            DrawingTool::Embeddable,
            *active.read(),
            active,
            canvas,
        ))'''
    laser_panel = embed_panel + '''
        .child(tool_button(
            DrawingTool::Laser,
            *active.read(),
            active,
            canvas,
        ))'''
    text = replace_once(text, embed_panel, laser_panel, "laser more-tools button")

    keyboard_anchor = '''                    Key::Character(value) if !command => {
                        if let Some(tool) = DrawingTool::ALL'''
    keyboard_new = '''                    Key::Character(value) if !command => {
                        if value.eq_ignore_ascii_case("k") {
                            keyboard_tool.set(DrawingTool::Laser);
                            keyboard_canvas.write().set_active_tool_label("Laser");
                            event.stop_propagation();
                            return;
                        }
                        if let Some(tool) = DrawingTool::ALL'''
    text = replace_once(text, keyboard_anchor, keyboard_new, "laser keyboard shortcut")
    path.write_text(text)


def sync_test() -> None:
    Path("Elephant/freya/tests/drawing_laser_freya_testing.rs").write_text(r'''#[path = "../src/app/drawing_canvas.rs"]
mod drawing_canvas;

use drawing_canvas::DrawingCanvasState;
use serde_json::json;

#[test]
fn laser_never_enters_the_serialized_excalidraw_scene() {
    let raw = serde_json::to_string(&json!({
        "type":"excalidraw",
        "elements":[],
        "appState":{"viewBackgroundColor":"#ffffff"},
        "files":{}
    })).unwrap();
    let mut state = DrawingCanvasState::from_json(&raw).unwrap();
    state.set_active_tool_label("Laser");
    state.begin_laser_path([90.0, 130.0], 100.0);
    for (index, point) in [[130.0, 150.0], [180.0, 120.0], [230.0, 160.0], [300.0, 125.0]]
        .into_iter()
        .enumerate()
    {
        state.move_laser_path(point, 110.0 + index as f64 * 10.0);
    }
    assert!(state.document.elements.is_empty());
    assert!(state.has_current_laser_path());
    let during = state.laser_outlines_at(145.0);
    assert_eq!(during.len(), 1);
    assert!(during[0].len() > 20);
    assert_eq!(state.serialize_json().unwrap(), serde_json::to_string_pretty(&state.document).unwrap());
    assert!(!state.serialize_json().unwrap().contains("laser"));

    assert!(state.end_laser_path());
    assert!(!state.has_current_laser_path());
    assert!(!state.laser_outlines_at(500.0).is_empty());
    assert!(state.laser_outlines_at(1_200.0).is_empty());
    assert!(state.document.elements.is_empty());
}
''')


def main() -> None:
    sync_scene()
    sync_canvas()
    sync_render()
    sync_render_svg()
    sync_view()
    sync_test()


if __name__ == "__main__":
    main()
