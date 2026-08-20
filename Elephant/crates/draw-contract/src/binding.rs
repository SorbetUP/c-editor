use crate::{DrawingElement, DrawingScene};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrowEndpoint {
    Start,
    End,
}

impl ArrowEndpoint {
    fn key(self) -> &'static str {
        match self {
            Self::Start => "startBinding",
            Self::End => "endBinding",
        }
    }
}

impl DrawingScene {
    pub fn bind_arrow_endpoint(
        &mut self,
        arrow_id: &str,
        endpoint: ArrowEndpoint,
        target_id: &str,
        focus: f32,
        gap: f32,
        fixed_point: Option<[f32; 2]>,
    ) -> bool {
        if arrow_id == target_id || !focus.is_finite() || !gap.is_finite() {
            return false;
        }
        if fixed_point.is_some_and(|point| !point[0].is_finite() || !point[1].is_finite()) {
            return false;
        }

        let Some(arrow_index) = self.element_index(arrow_id) else {
            return false;
        };
        let Some(target_index) = self.element_index(target_id) else {
            return false;
        };
        if self.elements[arrow_index].kind != "arrow"
            || self.elements[arrow_index].is_locked()
            || self.elements[target_index].is_locked()
            || !is_bindable(&self.elements[target_index])
        {
            return false;
        }

        let previous_target =
            binding_target(&self.elements[arrow_index], endpoint).map(str::to_owned);
        if previous_target.as_deref() == Some(target_id)
            && binding_matches(
                &self.elements[arrow_index],
                endpoint,
                target_id,
                focus,
                gap,
                fixed_point,
            )
        {
            return false;
        }

        if let Some(previous_target) = previous_target {
            if let Some(index) = self.element_index(&previous_target) {
                remove_bound_element(&mut self.elements[index], arrow_id, "arrow");
            }
        }

        let binding = binding_value(target_id, focus, gap, fixed_point);
        self.elements[arrow_index]
            .extra
            .insert(endpoint.key().to_owned(), binding);
        mark_changed(&mut self.elements[arrow_index]);
        add_bound_element(&mut self.elements[target_index], arrow_id, "arrow");
        true
    }

    pub fn unbind_arrow_endpoint(&mut self, arrow_id: &str, endpoint: ArrowEndpoint) -> bool {
        let Some(arrow_index) = self.element_index(arrow_id) else {
            return false;
        };
        if self.elements[arrow_index].kind != "arrow" || self.elements[arrow_index].is_locked() {
            return false;
        }
        let Some(target_id) =
            binding_target(&self.elements[arrow_index], endpoint).map(str::to_owned)
        else {
            return false;
        };

        self.elements[arrow_index]
            .extra
            .insert(endpoint.key().to_owned(), Value::Null);
        mark_changed(&mut self.elements[arrow_index]);
        if let Some(target_index) = self.element_index(&target_id) {
            remove_bound_element(&mut self.elements[target_index], arrow_id, "arrow");
        }
        true
    }

    pub fn bind_text_to_container(&mut self, text_id: &str, container_id: &str) -> bool {
        if text_id == container_id {
            return false;
        }
        let Some(text_index) = self.element_index(text_id) else {
            return false;
        };
        let Some(container_index) = self.element_index(container_id) else {
            return false;
        };
        if self.elements[text_index].kind != "text"
            || self.elements[text_index].is_locked()
            || self.elements[container_index].is_locked()
            || !is_text_container(&self.elements[container_index])
        {
            return false;
        }

        let previous = self.elements[text_index]
            .extra
            .get("containerId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if previous.as_deref() == Some(container_id) {
            return false;
        }
        if let Some(previous) = previous {
            if let Some(index) = self.element_index(&previous) {
                remove_bound_element(&mut self.elements[index], text_id, "text");
            }
        }

        self.elements[text_index].extra.insert(
            "containerId".to_owned(),
            Value::String(container_id.to_owned()),
        );
        mark_changed(&mut self.elements[text_index]);
        add_bound_element(&mut self.elements[container_index], text_id, "text");
        true
    }

    pub fn unbind_text(&mut self, text_id: &str) -> bool {
        let Some(text_index) = self.element_index(text_id) else {
            return false;
        };
        if self.elements[text_index].kind != "text" || self.elements[text_index].is_locked() {
            return false;
        }
        let Some(container_id) = self.elements[text_index]
            .extra
            .get("containerId")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            return false;
        };

        self.elements[text_index]
            .extra
            .insert("containerId".to_owned(), Value::Null);
        mark_changed(&mut self.elements[text_index]);
        if let Some(container_index) = self.element_index(&container_id) {
            remove_bound_element(&mut self.elements[container_index], text_id, "text");
        }
        true
    }
}

fn is_bindable(element: &DrawingElement) -> bool {
    matches!(
        element.kind.as_str(),
        "rectangle" | "ellipse" | "diamond" | "text" | "image" | "frame"
    )
}

fn is_text_container(element: &DrawingElement) -> bool {
    matches!(
        element.kind.as_str(),
        "rectangle" | "ellipse" | "diamond" | "arrow" | "frame"
    )
}

fn binding_value(target_id: &str, focus: f32, gap: f32, fixed_point: Option<[f32; 2]>) -> Value {
    let mut binding = Map::new();
    binding.insert("elementId".to_owned(), Value::String(target_id.to_owned()));
    binding.insert("focus".to_owned(), json!(focus.clamp(-1.0, 1.0)));
    binding.insert("gap".to_owned(), json!(gap.max(0.0)));
    binding.insert(
        "fixedPoint".to_owned(),
        fixed_point.map_or(Value::Null, |point| json!(point)),
    );
    Value::Object(binding)
}

fn binding_target(element: &DrawingElement, endpoint: ArrowEndpoint) -> Option<&str> {
    element
        .extra
        .get(endpoint.key())
        .and_then(Value::as_object)
        .and_then(|binding| binding.get("elementId"))
        .and_then(Value::as_str)
}

fn binding_matches(
    element: &DrawingElement,
    endpoint: ArrowEndpoint,
    target_id: &str,
    focus: f32,
    gap: f32,
    fixed_point: Option<[f32; 2]>,
) -> bool {
    let Some(binding) = element.extra.get(endpoint.key()).and_then(Value::as_object) else {
        return false;
    };
    binding.get("elementId").and_then(Value::as_str) == Some(target_id)
        && binding.get("focus").and_then(Value::as_f64) == Some(f64::from(focus.clamp(-1.0, 1.0)))
        && binding.get("gap").and_then(Value::as_f64) == Some(f64::from(gap.max(0.0)))
        && fixed_point_matches(binding.get("fixedPoint"), fixed_point)
}

fn fixed_point_matches(value: Option<&Value>, expected: Option<[f32; 2]>) -> bool {
    match expected {
        None => value.is_none() || value == Some(&Value::Null),
        Some(expected) => value
            .and_then(Value::as_array)
            .filter(|array| array.len() == 2)
            .is_some_and(|array| {
                array[0].as_f64() == Some(f64::from(expected[0]))
                    && array[1].as_f64() == Some(f64::from(expected[1]))
            }),
    }
}

fn add_bound_element(element: &mut DrawingElement, id: &str, kind: &str) {
    let bound = element
        .extra
        .entry("boundElements".to_owned())
        .or_insert_with(|| json!([]));
    if bound.is_null() {
        *bound = json!([]);
    }
    let Some(bound) = bound.as_array_mut() else {
        return;
    };
    if bound.iter().any(|entry| {
        entry.get("id").and_then(Value::as_str) == Some(id)
            && entry.get("type").and_then(Value::as_str) == Some(kind)
    }) {
        return;
    }
    bound.push(json!({"type": kind, "id": id}));
    mark_changed(element);
}

fn remove_bound_element(element: &mut DrawingElement, id: &str, kind: &str) {
    let Some(bound) = element
        .extra
        .get_mut("boundElements")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let before = bound.len();
    bound.retain(|entry| {
        entry.get("id").and_then(Value::as_str) != Some(id)
            || entry.get("type").and_then(Value::as_str) != Some(kind)
    });
    if bound.len() != before {
        mark_changed(element);
    }
}

fn mark_changed(element: &mut DrawingElement) {
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
