use crate::{DrawingElement, DrawingScene};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct TextLineLayout {
    pub text: String,
    pub x: f32,
    pub baseline_y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<TextLineLayout>,
    pub width: f32,
    pub height: f32,
    pub line_height_px: f32,
}

pub fn font_family_css(font_family: u64) -> &'static str {
    match font_family {
        1 => "Virgil, Excalifont, cursive",
        2 => "Helvetica, Arial, sans-serif",
        3 => "Cascadia, 'Cascadia Code', monospace",
        5 => "Excalifont, Virgil, cursive",
        6 => "Nunito, sans-serif",
        7 => "'Lilita One', sans-serif",
        8 => "'Comic Shanns', 'Comic Sans MS', cursive",
        9 => "'Liberation Sans', Arial, sans-serif",
        10 => "Assistant, sans-serif",
        _ => "Excalifont, Virgil, sans-serif",
    }
}

pub fn layout_text(element: &DrawingElement) -> TextLayout {
    let font_size = element.font_size.max(1.0);
    let line_height = element
        .extra
        .get("lineHeight")
        .and_then(Value::as_f64)
        .unwrap_or(1.25) as f32;
    let line_height_px = (font_size * line_height.max(0.1)).max(1.0);
    let font_family = element
        .extra
        .get("fontFamily")
        .and_then(Value::as_u64)
        .unwrap_or(5);
    let char_width = estimated_char_width(font_size, font_family);
    let auto_resize = element
        .extra
        .get("autoResize")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let max_width = (!auto_resize && element.width > char_width)
        .then_some(element.width.max(char_width));

    let mut rows = Vec::<String>::new();
    for logical_line in element.text.split('\n') {
        if let Some(max_width) = max_width {
            rows.extend(wrap_line(logical_line, max_width, char_width));
        } else {
            rows.push(logical_line.to_owned());
        }
    }
    if rows.is_empty() {
        rows.push(String::new());
    }

    let measured_width = rows
        .iter()
        .map(|line| measure_line(line, char_width))
        .fold(0.0_f32, f32::max)
        .max(1.0);
    let width = if auto_resize {
        measured_width
    } else {
        element
            .width
            .max(measured_width.min(element.width.max(1.0)))
            .max(1.0)
    };
    let measured_height = (rows.len() as f32 * line_height_px).max(line_height_px);
    let height = if auto_resize {
        measured_height
    } else {
        element.height.max(measured_height).max(line_height_px)
    };

    let text_align = element
        .extra
        .get("textAlign")
        .and_then(Value::as_str)
        .unwrap_or("left");
    let vertical_align = element
        .extra
        .get("verticalAlign")
        .and_then(Value::as_str)
        .unwrap_or("top");
    let block_height = rows.len() as f32 * line_height_px;
    let top = match vertical_align {
        "middle" => ((height - block_height) / 2.0).max(0.0),
        "bottom" => (height - block_height).max(0.0),
        _ => 0.0,
    };

    let lines = rows
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let line_width = measure_line(&text, char_width);
            let x = match text_align {
                "center" => ((width - line_width) / 2.0).max(0.0),
                "right" => (width - line_width).max(0.0),
                _ => 0.0,
            };
            TextLineLayout {
                text,
                x,
                baseline_y: top + font_size + index as f32 * line_height_px,
            }
        })
        .collect();

    TextLayout {
        lines,
        width,
        height,
        line_height_px,
    }
}

impl DrawingScene {
    pub fn set_text_content(&mut self, id: &str, content: impl Into<String>) -> bool {
        let content = content.into();
        let Some(index) = self.elements.iter().position(|element| {
            element.id == id && element.kind == "text" && !element.is_deleted && !element.is_locked()
        }) else {
            return false;
        };
        if self.elements[index].text == content {
            return false;
        }

        {
            let element = &mut self.elements[index];
            element.text = content.clone();
            element
                .extra
                .insert("originalText".to_owned(), Value::String(content));
            element
                .extra
                .entry("autoResize".to_owned())
                .or_insert(Value::Bool(true));
            mark_text_changed(element);
        }

        let layout = layout_text(&self.elements[index]);
        if self.elements[index]
            .extra
            .get("autoResize")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            self.elements[index].width = layout.width;
            self.elements[index].height = layout.height;
        }
        true
    }
}

fn estimated_char_width(font_size: f32, font_family: u64) -> f32 {
    let factor = match font_family {
        3 | 8 => 0.62,
        7 => 0.58,
        1 | 5 => 0.57,
        _ => 0.55,
    };
    (font_size * factor).max(0.5)
}

fn measure_line(line: &str, char_width: f32) -> f32 {
    line.chars()
        .map(|character| {
            if character == '\t' {
                char_width * 4.0
            } else if character.is_ascii_whitespace() {
                char_width * 0.7
            } else if character.is_ascii_punctuation() {
                char_width * 0.8
            } else {
                char_width
            }
        })
        .sum::<f32>()
        .max(0.0)
}

fn wrap_line(line: &str, max_width: f32, char_width: f32) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    if measure_line(line, char_width) <= max_width {
        return vec![line.to_owned()];
    }

    let mut output = Vec::new();
    let mut current = String::new();
    for word in line.split_inclusive(char::is_whitespace) {
        let candidate = format!("{current}{word}");
        if !current.is_empty() && measure_line(&candidate, char_width) > max_width {
            output.push(current.trim_end().to_owned());
            current.clear();
        }
        if measure_line(word, char_width) <= max_width {
            current.push_str(word);
            continue;
        }
        for character in word.chars() {
            let mut candidate = current.clone();
            candidate.push(character);
            if !current.is_empty() && measure_line(&candidate, char_width) > max_width {
                output.push(current.trim_end().to_owned());
                current.clear();
            }
            current.push(character);
        }
    }
    if !current.is_empty() {
        output.push(current.trim_end().to_owned());
    }
    if output.is_empty() {
        output.push(String::new());
    }
    output
}

fn mark_text_changed(element: &mut DrawingElement) {
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);
    let nonce = element
        .extra
        .get("versionNonce")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    element.extra.insert("version".to_owned(), json!(version));
    element
        .extra
        .insert("versionNonce".to_owned(), json!(nonce));
}
