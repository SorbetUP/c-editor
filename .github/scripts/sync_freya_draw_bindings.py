#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


path = Path("Elephant/freya/src/app/drawing_canvas.rs")
text = path.read_text()

old_text_branch = '''                    let index = if let Some(index) = existing {
                        canvas.checkpoint();
                        index
                    } else {
                        let index = canvas.document.elements.len();
                        canvas.checkpoint();
                        let mut element = new_element(DrawingTool::Text, world, index);
                        element.text.clear();
                        element.extra.insert("originalText".to_owned(), Value::String(String::new()));
                        element.width = element.font_size.max(20.0);
                        element.height = element.font_size.max(20.0) * 1.25;
                        canvas.document.elements.push(element);
                        canvas.touch_element(index);
                        index
                    };'''
new_text_branch = '''                    let index = if let Some(index) = existing {
                        canvas.checkpoint();
                        index
                    } else {
                        let container = canvas
                            .document
                            .elements
                            .iter()
                            .enumerate()
                            .rev()
                            .find_map(|(index, element)| {
                                (!element.is_deleted
                                    && !element.is_locked()
                                    && matches!(element.kind.as_str(), "rectangle" | "ellipse" | "diamond" | "arrow")
                                    && element.hit_test(world))
                                .then_some(index)
                            });
                        let index = canvas.document.elements.len();
                        canvas.checkpoint();
                        let mut element = new_element(DrawingTool::Text, world, index);
                        element.text.clear();
                        element.extra.insert("originalText".to_owned(), Value::String(String::new()));
                        element.width = element.font_size.max(20.0);
                        element.height = element.font_size.max(20.0) * 1.25;
                        let container_id = container.and_then(|container_index| {
                            let container = canvas.document.elements.get(container_index)?;
                            let (x, y, width, height) = container.bounds();
                            element.x = x + width / 2.0 - element.width / 2.0;
                            element.y = y + height / 2.0 - element.height / 2.0;
                            Some(container.id.clone())
                        });
                        let text_id = element.id.clone();
                        canvas.document.elements.push(element);
                        if let Some(container_id) = container_id {
                            canvas.document.bind_text_to_container(&text_id, &container_id);
                        }
                        canvas.touch_element(index);
                        index
                    };'''
text = replace_once(text, old_text_branch, new_text_branch, "bound text creation")

old_release = '''            release_pointer_down.set(false);
            local_end_state.write().end_pointer();
            local_end_gesture.set(None);
            event.stop_propagation();'''
new_release = '''            release_pointer_down.set(false);
            if let Some(gesture) = *local_end_gesture.read() {
                finalize_gesture(&mut local_end_state.write(), gesture);
            }
            local_end_state.write().end_pointer();
            local_end_gesture.set(None);
            event.stop_propagation();'''
text = replace_once(text, old_release, new_release, "gesture finalization")

helper_anchor = '''fn update_linear_dimensions(element: &mut DrawingElement) {'''
helpers = '''fn finalize_gesture(canvas: &mut DrawingCanvasState, gesture: Gesture) {
    if gesture.tool != DrawingTool::Arrow {
        return;
    }
    let Some(arrow) = canvas.document.elements.get(gesture.index) else {
        return;
    };
    if arrow.is_deleted || arrow.points.len() < 2 {
        return;
    }
    let arrow_id = arrow.id.clone();
    let start = [arrow.x + arrow.points[0][0], arrow.y + arrow.points[0][1]];
    let last = arrow.points.len() - 1;
    let end = [arrow.x + arrow.points[last][0], arrow.y + arrow.points[last][1]];
    let start_target = bind_target(&canvas.document, &arrow_id, start);
    let end_target = bind_target(&canvas.document, &arrow_id, end);
    let mut changed = false;
    if let Some((target, fixed)) = start_target {
        changed |= canvas.document.bind_arrow_endpoint(
            &arrow_id,
            elephant_draw::ArrowEndpoint::Start,
            &target,
            0.0,
            1.0,
            Some(fixed),
        );
    }
    if let Some((target, fixed)) = end_target {
        changed |= canvas.document.bind_arrow_endpoint(
            &arrow_id,
            elephant_draw::ArrowEndpoint::End,
            &target,
            0.0,
            1.0,
            Some(fixed),
        );
    }
    if changed {
        canvas.revision = canvas.revision.wrapping_add(1);
    }
}

fn bind_target(scene: &DrawingScene, arrow_id: &str, point: [f32; 2]) -> Option<(String, [f32; 2])> {
    scene
        .elements
        .iter()
        .rev()
        .find(|element| {
            element.id != arrow_id
                && !element.is_deleted
                && !element.is_locked()
                && matches!(
                    element.kind.as_str(),
                    "rectangle" | "ellipse" | "diamond" | "text" | "image" | "iframe" | "embeddable" | "frame" | "magicframe"
                )
                && element.hit_test_with_tolerance(point, 10.0)
        })
        .map(|element| {
            let (x, y, width, height) = element.bounds();
            let fixed = [
                if width > f32::EPSILON { ((point[0] - x) / width).clamp(0.0, 1.0) } else { 0.5 },
                if height > f32::EPSILON { ((point[1] - y) / height).clamp(0.0, 1.0) } else { 0.5 },
            ];
            (element.id.clone(), fixed)
        })
}

'''
text = replace_once(text, helper_anchor, helpers + helper_anchor, "binding helpers")
path.write_text(text)
