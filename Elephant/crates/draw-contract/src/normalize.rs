use crate::DrawingScene;
use serde_json::{json, Map, Value};

impl DrawingScene {
    pub fn normalize_excalidraw_defaults(&mut self) -> usize {
        let mut changed = 0;
        for element in &mut self.elements {
            let mut element_changed = false;
            element_changed |= insert_missing(&mut element.extra, "roundness", Value::Null);
            element_changed |= insert_missing(&mut element.extra, "roughness", json!(1));
            element_changed |= insert_missing(&mut element.extra, "seed", json!(1));
            element_changed |= insert_missing(&mut element.extra, "version", json!(1));
            element_changed |= insert_missing(&mut element.extra, "versionNonce", json!(1));
            element_changed |= insert_missing(&mut element.extra, "index", Value::Null);
            element_changed |= insert_missing(&mut element.extra, "groupIds", json!([]));
            element_changed |= insert_missing(&mut element.extra, "frameId", Value::Null);
            element_changed |= insert_missing(&mut element.extra, "boundElements", Value::Null);
            element_changed |= insert_missing(&mut element.extra, "updated", json!(0));
            element_changed |= insert_missing(&mut element.extra, "link", Value::Null);
            element_changed |= insert_missing(&mut element.extra, "locked", json!(false));

            match element.kind.as_str() {
                "text" => {
                    element_changed |= insert_missing(&mut element.extra, "fontFamily", json!(5));
                    element_changed |= insert_missing(&mut element.extra, "textAlign", json!("left"));
                    element_changed |= insert_missing(&mut element.extra, "verticalAlign", json!("top"));
                    element_changed |= insert_missing(&mut element.extra, "containerId", Value::Null);
                    element_changed |= insert_missing(
                        &mut element.extra,
                        "originalText",
                        Value::String(element.text.clone()),
                    );
                    element_changed |= insert_missing(&mut element.extra, "autoResize", json!(true));
                    element_changed |= insert_missing(&mut element.extra, "lineHeight", json!(1.25));
                }
                "image" => {
                    element_changed |= insert_missing(&mut element.extra, "fileId", Value::Null);
                    element_changed |= insert_missing(&mut element.extra, "status", json!("pending"));
                    element_changed |= insert_missing(&mut element.extra, "scale", json!([1, 1]));
                    element_changed |= insert_missing(&mut element.extra, "crop", Value::Null);
                }
                "freedraw" => {
                    element_changed |= insert_missing(&mut element.extra, "pressures", json!([]));
                    element_changed |= insert_missing(&mut element.extra, "simulatePressure", json!(true));
                    element_changed |= insert_missing(
                        &mut element.extra,
                        "strokeOptions",
                        json!({"variability": "variable", "streamline": 0.5}),
                    );
                }
                "line" => {
                    element_changed |= insert_missing(&mut element.extra, "polygon", json!(false));
                    element_changed |= insert_missing(&mut element.extra, "startBinding", Value::Null);
                    element_changed |= insert_missing(&mut element.extra, "endBinding", Value::Null);
                }
                "arrow" => {
                    element_changed |= insert_missing(&mut element.extra, "elbowed", json!(false));
                    element_changed |= insert_missing(&mut element.extra, "startBinding", Value::Null);
                    element_changed |= insert_missing(&mut element.extra, "endBinding", Value::Null);
                    if element.extra.get("elbowed").and_then(Value::as_bool) == Some(true) {
                        element_changed |= insert_missing(&mut element.extra, "fixedSegments", Value::Null);
                        element_changed |= insert_missing(&mut element.extra, "startIsSpecial", Value::Null);
                        element_changed |= insert_missing(&mut element.extra, "endIsSpecial", Value::Null);
                    }
                }
                "frame" | "magicframe" => {
                    element_changed |= insert_missing(&mut element.extra, "name", Value::Null);
                }
                _ => {}
            }

            if element_changed {
                changed += 1;
            }
        }
        changed
    }
}

fn insert_missing(extra: &mut Map<String, Value>, key: &str, value: Value) -> bool {
    if extra.contains_key(key) {
        return false;
    }
    extra.insert(key.to_owned(), value);
    true
}
