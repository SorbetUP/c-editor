#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def sync_canvas() -> None:
    path = Path("Elephant/freya/src/app/drawing_canvas.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "    Frame,\n}",
        "    Frame,\n    MagicFrame,\n    Embeddable,\n}",
        "canvas extra tool enum",
    )
    text = replace_once(
        text,
        '            "frame" => Self::Frame,\n            _ => Self::Selection,',
        '            "frame" => Self::Frame,\n            "magicframe" => Self::MagicFrame,\n            "embeddable" | "iframe" => Self::Embeddable,\n            _ => Self::Selection,',
        "canvas extra tool from id",
    )
    text = replace_once(
        text,
        '            Self::Frame => "frame",\n        }',
        '            Self::Frame => "frame",\n            Self::MagicFrame => "magicframe",\n            Self::Embeddable => "embeddable",\n        }',
        "canvas extra tool id",
    )
    text = replace_once(
        text,
        '            Self::Frame => "Frame",\n        }',
        '            Self::Frame => "Frame",\n            Self::MagicFrame => "Magic Frame",\n            Self::Embeddable => "Embed",\n        }',
        "canvas extra tool label",
    )
    text = replace_once(
        text,
        '''        DrawingTool::Frame => elephant_draw::DrawingTool::Frame,
    };''',
        '''        DrawingTool::Frame => elephant_draw::DrawingTool::Frame,
        DrawingTool::MagicFrame => elephant_draw::DrawingTool::MagicFrame,
        DrawingTool::Embeddable => elephant_draw::DrawingTool::Embeddable,
    };''',
        "canvas core extra tool mapping",
    )
    text = replace_once(
        text,
        '''            | DrawingTool::Ellipse
            | DrawingTool::Frame => {''',
        '''            | DrawingTool::Ellipse
            | DrawingTool::Frame
            | DrawingTool::MagicFrame
            | DrawingTool::Embeddable => {''',
        "canvas extra tool geometry",
    )
    path.write_text(text)


def sync_scene() -> None:
    path = Path("Elephant/freya/src/app/drawing_scene.rs")
    text = path.read_text()
    text = replace_once(
        text,
        '            "Frame" => "frame",\n            _ => "selection",',
        '            "Frame" => "frame",\n            "Magic Frame" => "magicframe",\n            "Embed" => "embeddable",\n            _ => "selection",',
        "scene extra tool mapping",
    )
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    text = replace_once(
        text,
        "    Frame,\n    Eraser,",
        "    Frame,\n    MagicFrame,\n    Embeddable,\n    Eraser,",
        "view extra tool enum",
    )
    text = replace_once(
        text,
        '            Self::Frame => "Frame",\n            Self::Eraser => "Eraser",',
        '            Self::Frame => "Frame",\n            Self::MagicFrame => "Magic Frame",\n            Self::Embeddable => "Embed",\n            Self::Eraser => "Eraser",',
        "view extra tool label",
    )
    text = replace_once(
        text,
        '            Self::Frame => "F",\n            Self::Eraser => "0",',
        '            Self::Frame => "F",\n            Self::MagicFrame | Self::Embeddable => "",\n            Self::Eraser => "0",',
        "view extra tool shortcut",
    )
    text = replace_once(
        text,
        '            Self::Frame => value == "f",\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        '            Self::Frame => value == "f",\n            Self::MagicFrame | Self::Embeddable => false,\n            Self::Eraser => matches!(value.as_str(), "e" | "0"),',
        "view extra tool shortcut matching",
    )
    icon_anchor = '''            Self::Frame => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M16 3h3a2 2 0 0 1 2 2v3"/><path d="M21 16v3a2 2 0 0 1-2 2h-3"/><path d="M8 21H5a2 2 0 0 1-2-2v-3"/></svg>"#,
            Self::Eraser =>'''
    icon_new = '''            Self::Frame => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M16 3h3a2 2 0 0 1 2 2v3"/><path d="M21 16v3a2 2 0 0 1-2 2h-3"/><path d="M8 21H5a2 2 0 0 1-2-2v-3"/></svg>"#,
            Self::MagicFrame => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="m12 7 .8 2.2L15 10l-2.2.8L12 13l-.8-2.2L9 10l2.2-.8Z"/><path d="m17 14 .5 1.5L19 16l-1.5.5L17 18l-.5-1.5L15 16l1.5-.5Z"/></svg>"#,
            Self::Embeddable => br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="m9 9-3 3 3 3"/><path d="m15 9 3 3-3 3"/></svg>"#,
            Self::Eraser =>'''
    text = replace_once(text, icon_anchor, icon_new, "view extra tool icons")

    panel_anchor = '''fn zoom_controls(canvas: State<DrawingCanvasState>) -> Element {'''
    panel = '''fn more_tools_panel(
    mut open: State<bool>,
    active: State<DrawingTool>,
    canvas: State<DrawingCanvasState>,
) -> Element {
    rect()
        .position(Position::new_absolute().right(110.0).top(78.0))
        .width(Size::px(150.0))
        .padding(Gaps::new_all(8.0))
        .spacing(5.0)
        .background(Color::from_rgb(35, 35, 42))
        .layer(Layer::OverlayLevel(55))
        .with_corner_radius(9.0)
        .a11y_alt("More drawing tools panel")
        .child(tool_button(
            DrawingTool::MagicFrame,
            *active.read(),
            active,
            canvas,
        ))
        .child(tool_button(
            DrawingTool::Embeddable,
            *active.read(),
            active,
            canvas,
        ))
        .child(icon_button(
            X_ICON,
            "Close more drawing tools",
            28.0,
            Color::from_rgb(235, 235, 240),
            Color::TRANSPARENT,
            move |_| open.set(false),
        ))
        .into_element()
}

'''
    text = replace_once(text, panel_anchor, panel + panel_anchor, "more tools panel")

    state_anchor = '''        let library_open = use_state(|| false);'''
    text = replace_once(
        text,
        state_anchor,
        '''        let more_tools_open = use_state(|| false);
        let library_open = use_state(|| false);''',
        "more tools state",
    )

    button_old = '''                            .child(icon_button(
                                SHAPES_ICON,
                                "More drawing tools",
                                38.0,
                                Color::from_rgb(235, 235, 240),
                                Color::TRANSPARENT,
                                |_| {},
                            )),'''
    button_new = '''                            .child(icon_button(
                                SHAPES_ICON,
                                "More drawing tools",
                                38.0,
                                Color::from_rgb(235, 235, 240),
                                Color::TRANSPARENT,
                                {
                                    let mut more = more_tools_open;
                                    move |_| more.set(!*more.read())
                                },
                            )),'''
    text = replace_once(text, button_old, button_new, "more tools button")

    overlay_anchor = '''            .maybe_child((*menu_open.read()).then(|| menu_panel(menu_open, canvas_state)))
'''
    overlay_new = '''            .maybe_child((*menu_open.read()).then(|| menu_panel(menu_open, canvas_state)))
            .maybe_child((*more_tools_open.read()).then(|| {
                more_tools_panel(more_tools_open, active_tool, canvas_state)
            }))
'''
    text = replace_once(text, overlay_anchor, overlay_new, "more tools overlay")
    path.write_text(text)


if __name__ == "__main__":
    sync_canvas()
    sync_scene()
    sync_view()
