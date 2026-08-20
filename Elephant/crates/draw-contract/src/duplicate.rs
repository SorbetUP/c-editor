use crate::{DrawingElement, DrawingScene, SelectionSet};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DuplicateSelectionOutcome {
    pub changed: usize,
    pub selection: SelectionSet,
}

impl DrawingScene {
    pub fn duplicate_selection_excalidraw(
        &mut self,
        selection: &SelectionSet,
        offset: [f32; 2],
        operation_nonce: u64,
    ) -> DuplicateSelectionOutcome {
        if selection.is_empty() || !offset[0].is_finite() || !offset[1].is_finite() {
            return DuplicateSelectionOutcome::default();
        }

        let mut source_ids = self
            .elements
            .iter()
            .filter(|element| {
                selection.contains(&element.id) && !element.is_deleted && !element.is_locked()
            })
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();
        if source_ids.is_empty() {
            return DuplicateSelectionOutcome::default();
        }

        loop {
            let before = source_ids.len();
            let snapshot = source_ids.clone();
            for element in &self.elements {
                if element.is_deleted {
                    continue;
                }
                if snapshot.contains(&element.id) && is_frame_like(element) {
                    for child in &self.elements {
                        if !child.is_deleted
                            && child.extra.get("frameId").and_then(Value::as_str)
                                == Some(element.id.as_str())
                        {
                            source_ids.insert(child.id.clone());
                        }
                    }
                }
                if snapshot.contains(&element.id) {
                    for bound_id in bound_text_ids(element) {
                        if self
                            .element_by_id(&bound_id)
                            .is_some_and(|bound| !bound.is_deleted)
                        {
                            source_ids.insert(bound_id);
                        }
                    }
                    if element.kind == "text" {
                        if let Some(container_id) =
                            element.extra.get("containerId").and_then(Value::as_str)
                        {
                            if self
                                .element_by_id(container_id)
                                .is_some_and(|container| !container.is_deleted)
                            {
                                source_ids.insert(container_id.to_owned());
                            }
                        }
                    }
                }
            }
            if source_ids.len() == before {
                break;
            }
        }

        let existing_ids = self
            .elements
            .iter()
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();
        let source_elements = self
            .elements
            .iter()
            .filter(|element| source_ids.contains(&element.id) && !element.is_deleted)
            .cloned()
            .collect::<Vec<_>>();
        if source_elements.is_empty() {
            return DuplicateSelectionOutcome::default();
        }

        let mut id_map = HashMap::<String, String>::new();
        let mut generated = existing_ids;
        for (ordinal, element) in source_elements.iter().enumerate() {
            let mut suffix = 0u64;
            let id = loop {
                let candidate = format!(
                    "dup-{:016x}-{:04x}-{:02x}",
                    operation_nonce, ordinal, suffix
                );
                if generated.insert(candidate.clone()) {
                    break candidate;
                }
                suffix = suffix.wrapping_add(1);
            };
            id_map.insert(element.id.clone(), id);
        }

        let mut group_map = HashMap::<String, String>::new();
        let mut group_ordinal = 0u64;
        for element in &source_elements {
            for group_id in group_ids(element) {
                group_map.entry(group_id.clone()).or_insert_with(|| {
                    let id = format!(
                        "group-{:016x}-{:04x}",
                        operation_nonce, group_ordinal
                    );
                    group_ordinal = group_ordinal.wrapping_add(1);
                    id
                });
            }
        }

        let mut duplicates = Vec::with_capacity(source_elements.len());
        for (ordinal, source) in source_elements.iter().enumerate() {
            let mut duplicate = source.clone();
            duplicate.id = id_map[&source.id].clone();
            duplicate.x += offset[0];
            duplicate.y += offset[1];
            duplicate.is_deleted = false;

            if let Some(groups) = duplicate
                .extra
                .get_mut("groupIds")
                .and_then(Value::as_array_mut)
            {
                for group in groups {
                    if let Some(old) = group.as_str() {
                        if let Some(new) = group_map.get(old) {
                            *group = Value::String(new.clone());
                        }
                    }
                }
            }

            remap_string_reference(&mut duplicate, "frameId", &id_map);
            remap_string_reference(&mut duplicate, "containerId", &id_map);
            remap_binding(&mut duplicate, "startBinding", &id_map);
            remap_binding(&mut duplicate, "endBinding", &id_map);
            remap_bound_elements(&mut duplicate, &id_map);

            duplicate.extra.remove("index");
            let seed = source
                .extra
                .get("seed")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .wrapping_mul(1_664_525)
                .wrapping_add(operation_nonce)
                .wrapping_add(ordinal as u64)
                .max(1);
            duplicate.extra.insert("seed".to_owned(), json!(seed));
            duplicate
                .extra
                .insert("updated".to_owned(), json!(operation_nonce));
            mark_changed(&mut duplicate);
            duplicates.push(duplicate);
        }

        let insertion_index = self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| source_ids.contains(&element.id))
            .map(|(index, _)| index)
            .max()
            .map_or(self.elements.len(), |index| index + 1);
        self.elements
            .splice(insertion_index..insertion_index, duplicates.clone());

        let arrow_bindings = duplicates
            .iter()
            .filter(|element| element.kind == "arrow")
            .flat_map(|arrow| {
                ["startBinding", "endBinding"].into_iter().filter_map(|key| {
                    arrow
                        .extra
                        .get(key)
                        .and_then(Value::as_object)
                        .and_then(|binding| binding.get("elementId"))
                        .and_then(Value::as_str)
                        .map(|target| (target.to_owned(), arrow.id.clone()))
                })
            })
            .collect::<Vec<_>>();

        let mut touched_targets = 0;
        for (target_id, arrow_id) in arrow_bindings {
            let Some(target) = self
                .elements
                .iter_mut()
                .find(|element| element.id == target_id && !element.is_deleted)
            else {
                continue;
            };
            if add_bound_arrow(target, &arrow_id) {
                touched_targets += 1;
            }
        }

        self.sync_fractional_indices();
        DuplicateSelectionOutcome {
            changed: duplicates.len() + touched_targets,
            selection: SelectionSet::from_ids(duplicates.into_iter().map(|element| element.id)),
        }
    }
}

fn is_frame_like(element: &DrawingElement) -> bool {
    matches!(element.kind.as_str(), "frame" | "magicframe")
}

fn bound_text_ids(element: &DrawingElement) -> Vec<String> {
    element
        .extra
        .get("boundElements")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|entry| entry.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn group_ids(element: &DrawingElement) -> Vec<String> {
    element
        .extra
        .get("groupIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect()
}

fn remap_string_reference(
    element: &mut DrawingElement,
    key: &str,
    id_map: &HashMap<String, String>,
) {
    let Some(current) = element.extra.get(key).and_then(Value::as_str) else {
        return;
    };
    if let Some(replacement) = id_map.get(current) {
        element
            .extra
            .insert(key.to_owned(), Value::String(replacement.clone()));
    }
}

fn remap_binding(
    element: &mut DrawingElement,
    key: &str,
    id_map: &HashMap<String, String>,
) {
    let Some(binding) = element.extra.get_mut(key).and_then(Value::as_object_mut) else {
        return;
    };
    let Some(current) = binding.get("elementId").and_then(Value::as_str) else {
        return;
    };
    if let Some(replacement) = id_map.get(current) {
        binding.insert("elementId".to_owned(), Value::String(replacement.clone()));
    }
}

fn remap_bound_elements(element: &mut DrawingElement, id_map: &HashMap<String, String>) {
    let Some(bound) = element
        .extra
        .get_mut("boundElements")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let remapped = bound
        .iter()
        .filter_map(|entry| {
            let old = entry.get("id")?.as_str()?;
            let new = id_map.get(old)?;
            let mut entry = entry.clone();
            entry["id"] = Value::String(new.clone());
            Some(entry)
        })
        .collect::<Vec<_>>();
    *bound = remapped;
}

fn add_bound_arrow(target: &mut DrawingElement, arrow_id: &str) -> bool {
    let bound = target
        .extra
        .entry("boundElements".to_owned())
        .or_insert_with(|| json!([]));
    if bound.is_null() {
        *bound = json!([]);
    }
    let Some(bound) = bound.as_array_mut() else {
        return false;
    };
    if bound.iter().any(|entry| {
        entry.get("id").and_then(Value::as_str) == Some(arrow_id)
            && entry.get("type").and_then(Value::as_str) == Some("arrow")
    }) {
        return false;
    }
    bound.push(json!({"type":"arrow","id":arrow_id}));
    mark_changed(target);
    true
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
