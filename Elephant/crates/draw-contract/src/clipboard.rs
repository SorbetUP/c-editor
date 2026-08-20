use crate::{DrawingElement, DrawingScene, SelectionSet};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneFragment {
    pub elements: Vec<DrawingElement>,
    #[serde(default)]
    pub files: Value,
}

impl DrawingScene {
    pub fn copy_selection_fragment(&self, selection: &SelectionSet) -> SceneFragment {
        let mut ids = selection.ids().map(str::to_owned).collect::<HashSet<_>>();
        if ids.is_empty() {
            return SceneFragment {
                elements: Vec::new(),
                files: Value::Object(Map::new()),
            };
        }
        loop {
            let before = ids.len();
            for element in &self.elements {
                if element.is_deleted {
                    continue;
                }
                let parent_selected = element
                    .extra
                    .get("frameId")
                    .and_then(Value::as_str)
                    .is_some_and(|frame_id| ids.contains(frame_id));
                let container_selected = element.kind == "text"
                    && element
                        .extra
                        .get("containerId")
                        .and_then(Value::as_str)
                        .is_some_and(|container_id| ids.contains(container_id));
                if parent_selected || container_selected {
                    ids.insert(element.id.clone());
                }
            }
            if ids.len() == before {
                break;
            }
        }

        let elements = self
            .elements
            .iter()
            .filter(|element| ids.contains(&element.id) && !element.is_deleted)
            .cloned()
            .collect::<Vec<_>>();
        let mut files = Map::new();
        if let Some(scene_files) = self.files.as_object() {
            for element in &elements {
                let Some(file_id) = element.extra.get("fileId").and_then(Value::as_str) else {
                    continue;
                };
                if let Some(file) = scene_files.get(file_id) {
                    files.entry(file_id.to_owned()).or_insert_with(|| file.clone());
                }
            }
        }
        SceneFragment {
            elements,
            files: Value::Object(files),
        }
    }

    pub fn paste_fragment(
        &mut self,
        fragment: &SceneFragment,
        offset: [f32; 2],
        namespace: &str,
    ) -> SelectionSet {
        if fragment.elements.is_empty()
            || namespace.is_empty()
            || !offset[0].is_finite()
            || !offset[1].is_finite()
        {
            return SelectionSet::new();
        }

        let existing = self
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<HashSet<_>>();
        let mut remap = HashMap::<String, String>::new();
        for (index, element) in fragment.elements.iter().enumerate() {
            let stem = format!("{namespace}-{}", element.id);
            let mut candidate = stem.clone();
            let mut attempt = index;
            while existing.contains(candidate.as_str())
                || remap.values().any(|value| value == &candidate)
            {
                attempt = attempt.saturating_add(1);
                candidate = format!("{stem}-{attempt}");
            }
            remap.insert(element.id.clone(), candidate);
        }

        let pasted_ids = remap.values().cloned().collect::<HashSet<_>>();
        let mut output = Vec::with_capacity(fragment.elements.len());
        for source in &fragment.elements {
            let mut element = source.clone();
            element.id = remap
                .get(&source.id)
                .expect("all fragment ids are remapped")
                .clone();
            element.x += offset[0];
            element.y += offset[1];
            element.is_deleted = false;
            rewrite_element_references(&mut element, &remap, &pasted_ids, namespace);
            mark_pasted(&mut element);
            output.push(element);
        }

        merge_fragment_files(&mut self.files, &fragment.files);
        self.elements.extend(output);
        self.sync_fractional_indices();
        SelectionSet::from_ids(pasted_ids)
    }

    pub fn duplicate_selection(
        &mut self,
        selection: &SelectionSet,
        offset: [f32; 2],
        namespace: &str,
    ) -> SelectionSet {
        let fragment = self.copy_selection_fragment(selection);
        self.paste_fragment(&fragment, offset, namespace)
    }
}

fn rewrite_element_references(
    element: &mut DrawingElement,
    remap: &HashMap<String, String>,
    pasted_ids: &HashSet<String>,
    namespace: &str,
) {
    for key in ["frameId", "containerId"] {
        let old = element.extra.get(key).and_then(Value::as_str).map(str::to_owned);
        if let Some(old) = old {
            element.extra.insert(
                key.to_owned(),
                remap
                    .get(&old)
                    .cloned()
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            );
        }
    }

    if let Some(groups) = element
        .extra
        .get_mut("groupIds")
        .and_then(Value::as_array_mut)
    {
        for group in groups.iter_mut() {
            if let Some(old) = group.as_str() {
                *group = Value::String(format!("{namespace}-group-{old}"));
            }
        }
    }

    for key in ["startBinding", "endBinding"] {
        let Some(binding) = element
            .extra
            .get_mut(key)
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        let old = binding
            .get("elementId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        match old.and_then(|id| remap.get(&id).cloned()) {
            Some(id) => {
                binding.insert("elementId".to_owned(), Value::String(id));
            }
            None => {
                element.extra.insert(key.to_owned(), Value::Null);
            }
        }
    }

    if let Some(bound) = element
        .extra
        .get_mut("boundElements")
        .and_then(Value::as_array_mut)
    {
        bound.retain_mut(|entry| {
            let Some(object) = entry.as_object_mut() else {
                return false;
            };
            let Some(old) = object
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
            else {
                return false;
            };
            let Some(next) = remap.get(&old) else {
                return false;
            };
            object.insert("id".to_owned(), Value::String(next.clone()));
            pasted_ids.contains(next)
        });
    }
}

fn merge_fragment_files(scene_files: &mut Value, fragment_files: &Value) {
    if !scene_files.is_object() {
        *scene_files = Value::Object(Map::new());
    }
    let Some(target) = scene_files.as_object_mut() else {
        return;
    };
    let Some(source) = fragment_files.as_object() else {
        return;
    };
    for (id, file) in source {
        target.entry(id.clone()).or_insert_with(|| file.clone());
    }
}

fn mark_pasted(element: &mut DrawingElement) {
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);
    element
        .extra
        .insert("version".to_owned(), Value::from(version));
    element.extra.insert(
        "versionNonce".to_owned(),
        Value::from(
            version
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223),
        ),
    );
}
