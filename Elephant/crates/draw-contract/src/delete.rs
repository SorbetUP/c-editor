use crate::{DrawingElement, DrawingScene, SelectionSet};
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeleteSelectionOutcome {
    pub changed: usize,
    pub selection: SelectionSet,
}

impl DrawingScene {
    pub fn delete_selection_excalidraw(
        &mut self,
        selection: &SelectionSet,
    ) -> DeleteSelectionOutcome {
        if selection.is_empty() {
            return DeleteSelectionOutcome::default();
        }

        let selected_frames = self
            .elements
            .iter()
            .filter(|element| {
                selection.contains(&element.id)
                    && !element.is_deleted
                    && !element.is_locked()
                    && is_frame_like(element)
            })
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();

        let protected_by_deleted_frame = self
            .elements
            .iter()
            .filter(|element| {
                !element.is_deleted
                    && element
                        .extra
                        .get("frameId")
                        .and_then(Value::as_str)
                        .is_some_and(|frame_id| selected_frames.contains(frame_id))
            })
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();

        let mut deleted_ids = self
            .elements
            .iter()
            .filter(|element| {
                selection.contains(&element.id)
                    && !element.is_deleted
                    && !element.is_locked()
                    && !protected_by_deleted_frame.contains(&element.id)
            })
            .map(|element| element.id.clone())
            .collect::<HashSet<_>>();

        loop {
            let before = deleted_ids.len();
            for element in &self.elements {
                if element.is_deleted || protected_by_deleted_frame.contains(&element.id) {
                    continue;
                }
                if element.kind == "text"
                    && element
                        .extra
                        .get("containerId")
                        .and_then(Value::as_str)
                        .is_some_and(|container_id| deleted_ids.contains(container_id))
                {
                    deleted_ids.insert(element.id.clone());
                }
            }
            if deleted_ids.len() == before {
                break;
            }
        }

        if deleted_ids.is_empty() && selected_frames.is_empty() {
            return DeleteSelectionOutcome::default();
        }

        let mut changed = 0;
        let mut survivors = Vec::new();

        for element in &mut self.elements {
            if element.is_deleted {
                continue;
            }

            let detached_from_selected_frame = element
                .extra
                .get("frameId")
                .and_then(Value::as_str)
                .is_some_and(|frame_id| selected_frames.contains(frame_id));

            if detached_from_selected_frame && !selected_frames.contains(&element.id) {
                element.extra.insert("frameId".to_owned(), Value::Null);
                mark_changed(element);
                changed += 1;
                if !is_bound_text(element) {
                    survivors.push(element.id.clone());
                }
            }

            if deleted_ids.contains(&element.id) {
                element.is_deleted = true;
                mark_changed(element);
                changed += 1;
            }
        }

        if !deleted_ids.is_empty() {
            for element in &mut self.elements {
                if element.is_deleted {
                    continue;
                }

                let mut touched = false;
                for key in ["startBinding", "endBinding"] {
                    let references_deleted = element
                        .extra
                        .get(key)
                        .and_then(Value::as_object)
                        .and_then(|binding| binding.get("elementId"))
                        .and_then(Value::as_str)
                        .is_some_and(|id| deleted_ids.contains(id));
                    if references_deleted {
                        element.extra.insert(key.to_owned(), Value::Null);
                        touched = true;
                    }
                }

                if let Some(bound) = element
                    .extra
                    .get_mut("boundElements")
                    .and_then(Value::as_array_mut)
                {
                    let before = bound.len();
                    bound.retain(|entry| {
                        entry
                            .get("id")
                            .and_then(Value::as_str)
                            .is_none_or(|id| !deleted_ids.contains(id))
                    });
                    touched |= bound.len() != before;
                }

                if touched {
                    mark_changed(element);
                    changed += 1;
                }
            }
        }

        DeleteSelectionOutcome {
            changed,
            selection: SelectionSet::from_ids(survivors),
        }
    }
}

fn is_frame_like(element: &DrawingElement) -> bool {
    matches!(element.kind.as_str(), "frame" | "magicframe")
}

fn is_bound_text(element: &DrawingElement) -> bool {
    element.kind == "text"
        && element
            .extra
            .get("containerId")
            .and_then(Value::as_str)
            .is_some()
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
