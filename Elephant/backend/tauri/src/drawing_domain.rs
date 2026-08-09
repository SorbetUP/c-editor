use serde_json::{json, Value};

pub fn empty_drawing() -> Value {
  json!({
    "kind": "drawing",
    "version": 1,
    "items": []
  })
}

pub fn is_drawing(value: &Value) -> bool {
  value.get("kind").and_then(Value::as_str) == Some("drawing")
    && value.get("items").and_then(Value::as_array).is_some()
}

pub fn drawing_filename(title: &str) -> String {
  let cleaned = title.trim().replace('/', "-");
  let stem = if cleaned.is_empty() { "Untitled Drawing" } else { &cleaned };
  format!("{}.drawing.json", stem)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creates_empty_drawing() {
    let drawing = empty_drawing();
    assert!(is_drawing(&drawing));
    assert!(drawing["items"].as_array().unwrap().is_empty());
  }

  #[test]
  fn creates_drawing_filename() {
    assert_eq!(drawing_filename("Sketch"), "Sketch.drawing.json");
  }
}
