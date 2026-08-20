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
    anchor = '''    pub fn set_selected_stroke_width(&mut self, width: f32) -> bool {'''
    methods = '''    pub fn set_selected_fill_style(&mut self, style: impl Into<String>) -> bool {
        let style = style.into();
        if !matches!(style.as_str(), "hachure" | "cross-hatch" | "solid" | "zigzag") {
            return false;
        }
        self.mutate_selected(move |element| {
            if element.fill_style == style {
                return false;
            }
            element.fill_style = style.clone();
            true
        })
    }

    pub fn set_selected_roughness(&mut self, roughness: u64) -> bool {
        let roughness = roughness.min(2);
        self.mutate_selected(move |element| {
            let next = json!(roughness);
            if element.extra.get("roughness") == Some(&next) {
                return false;
            }
            element.extra.insert("roughness".to_owned(), next);
            true
        })
    }

    pub fn set_selected_roundness(&mut self, rounded: bool) -> bool {
        self.mutate_selected(move |element| {
            if !matches!(element.kind.as_str(), "rectangle" | "diamond" | "line" | "arrow") {
                return false;
            }
            let next = if rounded { json!({"type": 3}) } else { Value::Null };
            if element.extra.get("roundness") == Some(&next) {
                return false;
            }
            element.extra.insert("roundness".to_owned(), next);
            true
        })
    }

    pub fn set_selected_arrowheads(
        &mut self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> bool {
        let start = start.map(str::to_owned);
        let end = end.map(str::to_owned);
        self.mutate_selected(move |element| {
            if element.kind != "arrow" {
                return false;
            }
            if element.start_arrowhead == start && element.end_arrowhead == end {
                return false;
            }
            element.start_arrowhead = start.clone();
            element.end_arrowhead = end.clone();
            true
        })
    }

    pub fn set_selected_font_size(&mut self, font_size: f32) -> bool {
        if !font_size.is_finite() || font_size < 1.0 {
            return false;
        }
        self.mutate_selected(move |element| {
            if element.kind != "text" || (element.font_size - font_size).abs() <= f32::EPSILON {
                return false;
            }
            element.font_size = font_size;
            let layout = elephant_draw::layout_text(element);
            if element.extra.get("autoResize").and_then(Value::as_bool).unwrap_or(true) {
                element.width = layout.width;
                element.height = layout.height;
            }
            true
        })
    }

    pub fn set_selected_font_family(&mut self, family: u64) -> bool {
        self.mutate_selected(move |element| {
            if element.kind != "text" {
                return false;
            }
            let next = json!(family);
            if element.extra.get("fontFamily") == Some(&next) {
                return false;
            }
            element.extra.insert("fontFamily".to_owned(), next);
            let layout = elephant_draw::layout_text(element);
            if element.extra.get("autoResize").and_then(Value::as_bool).unwrap_or(true) {
                element.width = layout.width;
                element.height = layout.height;
            }
            true
        })
    }

    pub fn set_selected_text_align(&mut self, align: &str) -> bool {
        if !matches!(align, "left" | "center" | "right") {
            return false;
        }
        let align = align.to_owned();
        self.mutate_selected(move |element| {
            if element.kind != "text" {
                return false;
            }
            let next = Value::String(align.clone());
            if element.extra.get("textAlign") == Some(&next) {
                return false;
            }
            element.extra.insert("textAlign".to_owned(), next);
            true
        })
    }

    pub fn flip_selected_images(&mut self, horizontal: bool) -> bool {
        self.mutate_selected(move |element| {
            if element.kind != "image" {
                return false;
            }
            let mut scale = element
                .extra
                .get("scale")
                .and_then(Value::as_array)
                .filter(|scale| scale.len() == 2)
                .map(|scale| [
                    scale[0].as_f64().unwrap_or(1.0) as f32,
                    scale[1].as_f64().unwrap_or(1.0) as f32,
                ])
                .unwrap_or([1.0, 1.0]);
            let axis = usize::from(!horizontal);
            scale[axis] = if scale[axis] < 0.0 { scale[axis].abs() } else { -scale[axis].abs() };
            element.extra.insert("scale".to_owned(), json!(scale));
            true
        })
    }

    pub fn reset_selected_image_crop(&mut self) -> bool {
        self.mutate_selected(|element| {
            if element.kind != "image" || element.extra.get("crop").is_none_or(Value::is_null) {
                return false;
            }
            element.extra.insert("crop".to_owned(), Value::Null);
            true
        })
    }

'''
    text = replace_once(text, anchor, methods + anchor, "advanced property methods")
    path.write_text(text)


def sync_view() -> None:
    path = Path("Elephant/freya/src/app/drawing.rs")
    text = path.read_text()
    anchor = '''fn unlock_all(canvas: State<DrawingCanvasState>) {'''
    panel = '''fn advanced_properties_panel(canvas: State<DrawingCanvasState>) -> Element {
    let snapshot = canvas.read();
    let ids = snapshot.selected_element_ids();
    let mut has_shape = false;
    let mut has_arrow = false;
    let mut has_text = false;
    let mut has_image = false;
    for id in ids {
        if let Some(element) = snapshot.document.element_by_id(id) {
            has_shape |= matches!(element.kind.as_str(), "rectangle" | "ellipse" | "diamond");
            has_arrow |= element.kind == "arrow";
            has_text |= element.kind == "text";
            has_image |= element.kind == "image";
        }
    }
    drop(snapshot);

    let text_color = Color::from_rgb(235, 235, 240);
    let mut hachure = canvas;
    let mut cross = canvas;
    let mut solid = canvas;
    let mut zigzag = canvas;
    let mut rough0 = canvas;
    let mut rough1 = canvas;
    let mut rough2 = canvas;
    let mut sharp = canvas;
    let mut rounded = canvas;
    let mut arrow_open = canvas;
    let mut arrow_triangle = canvas;
    let mut arrow_circle = canvas;
    let mut arrow_diamond = canvas;
    let mut arrow_none = canvas;
    let mut font16 = canvas;
    let mut font20 = canvas;
    let mut font28 = canvas;
    let mut font36 = canvas;
    let mut font_hand = canvas;
    let mut font_sans = canvas;
    let mut font_mono = canvas;
    let mut align_left = canvas;
    let mut align_center = canvas;
    let mut align_right = canvas;
    let mut flip_h = canvas;
    let mut flip_v = canvas;
    let mut reset_crop = canvas;

    rect()
        .position(Position::new_absolute().left(314.0).top(72.0))
        .width(Size::px(300.0))
        .padding(Gaps::new(12.0, 12.0, 12.0, 12.0))
        .background(Color::from_rgb(35, 35, 42))
        .with_corner_radius(10.0)
        .layer(Layer::OverlayLevel(35))
        .a11y_alt("Excalidraw advanced properties")
        .maybe_child(has_shape.then(|| {
            rect()
                .spacing(5.0)
                .child(label().font_size(13.0).color(text_color).text("Fill style"))
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("/", "Hachure fill", 48.0, move |_| { hachure.write().set_selected_fill_style("hachure"); }))
                        .child(action_button("×", "Cross hatch fill", 48.0, move |_| { cross.write().set_selected_fill_style("cross-hatch"); }))
                        .child(action_button("■", "Solid fill", 48.0, move |_| { solid.write().set_selected_fill_style("solid"); }))
                        .child(action_button("≈", "Zigzag fill", 48.0, move |_| { zigzag.write().set_selected_fill_style("zigzag"); })),
                )
        }))
        .child(label().font_size(13.0).color(text_color).text("Roughness"))
        .child(
            rect().horizontal().spacing(5.0)
                .child(action_button("0", "Architect roughness", 44.0, move |_| { rough0.write().set_selected_roughness(0); }))
                .child(action_button("1", "Artist roughness", 44.0, move |_| { rough1.write().set_selected_roughness(1); }))
                .child(action_button("2", "Cartoonist roughness", 44.0, move |_| { rough2.write().set_selected_roughness(2); }))
                .child(action_button("□", "Sharp corners", 44.0, move |_| { sharp.write().set_selected_roundness(false); }))
                .child(action_button("▢", "Rounded corners", 44.0, move |_| { rounded.write().set_selected_roundness(true); })),
        )
        .maybe_child(has_arrow.then(|| {
            rect()
                .spacing(5.0)
                .child(label().font_size(13.0).color(text_color).text("Arrowhead"))
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("→", "Arrow arrowhead", 44.0, move |_| { arrow_open.write().set_selected_arrowheads(None, Some("arrow")); }))
                        .child(action_button("▶", "Triangle arrowhead", 44.0, move |_| { arrow_triangle.write().set_selected_arrowheads(None, Some("triangle")); }))
                        .child(action_button("●", "Circle arrowhead", 44.0, move |_| { arrow_circle.write().set_selected_arrowheads(None, Some("circle")); }))
                        .child(action_button("◆", "Diamond arrowhead", 44.0, move |_| { arrow_diamond.write().set_selected_arrowheads(None, Some("diamond")); }))
                        .child(action_button("—", "No arrowhead", 44.0, move |_| { arrow_none.write().set_selected_arrowheads(None, None); })),
                )
        }))
        .maybe_child(has_text.then(|| {
            rect()
                .spacing(5.0)
                .child(label().font_size(13.0).color(text_color).text("Text"))
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("16", "Text size 16", 42.0, move |_| { font16.write().set_selected_font_size(16.0); }))
                        .child(action_button("20", "Text size 20", 42.0, move |_| { font20.write().set_selected_font_size(20.0); }))
                        .child(action_button("28", "Text size 28", 42.0, move |_| { font28.write().set_selected_font_size(28.0); }))
                        .child(action_button("36", "Text size 36", 42.0, move |_| { font36.write().set_selected_font_size(36.0); })),
                )
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("Hand", "Hand drawn font", 58.0, move |_| { font_hand.write().set_selected_font_family(5); }))
                        .child(action_button("Sans", "Sans font", 58.0, move |_| { font_sans.write().set_selected_font_family(2); }))
                        .child(action_button("Mono", "Monospace font", 58.0, move |_| { font_mono.write().set_selected_font_family(3); })),
                )
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("L", "Align text left", 42.0, move |_| { align_left.write().set_selected_text_align("left"); }))
                        .child(action_button("C", "Align text center", 42.0, move |_| { align_center.write().set_selected_text_align("center"); }))
                        .child(action_button("R", "Align text right", 42.0, move |_| { align_right.write().set_selected_text_align("right"); })),
                )
        }))
        .maybe_child(has_image.then(|| {
            rect()
                .spacing(5.0)
                .child(label().font_size(13.0).color(text_color).text("Image"))
                .child(
                    rect().horizontal().spacing(5.0)
                        .child(action_button("↔", "Flip image horizontally", 54.0, move |_| { flip_h.write().flip_selected_images(true); }))
                        .child(action_button("↕", "Flip image vertically", 54.0, move |_| { flip_v.write().flip_selected_images(false); }))
                        .child(action_button("Crop 0", "Reset image crop", 70.0, move |_| { reset_crop.write().reset_selected_image_crop(); })),
                )
        }))
        .into_element()
}

'''
    text = replace_once(text, anchor, panel + anchor, "advanced properties panel")

    invoke_anchor = '''            .maybe_child(
                (!canvas_state.read().selected_element_ids().is_empty())
                    .then(|| properties_panel(canvas_state)),
            )'''
    invoke_new = invoke_anchor + '''
            .maybe_child(
                (!canvas_state.read().selected_element_ids().is_empty())
                    .then(|| advanced_properties_panel(canvas_state)),
            )'''
    text = replace_once(text, invoke_anchor, invoke_new, "advanced properties invocation")
    path.write_text(text)


if __name__ == "__main__":
    sync_scene()
    sync_view()
