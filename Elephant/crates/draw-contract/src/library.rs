use crate::{DrawingElement, DrawingScene, SceneFragment, SelectionSet};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LibraryItemStatus {
    Published,
    Unpublished,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: String,
    pub status: LibraryItemStatus,
    pub created: u64,
    pub elements: Vec<DrawingElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub files: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    #[serde(rename = "type")]
    pub file_type: String,
    pub version: u32,
    pub source: String,
    #[serde(default)]
    pub library_items: Vec<LibraryItem>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl LibraryFile {
    pub fn empty(source: impl Into<String>) -> Self {
        Self {
            file_type: "excalidrawlib".to_owned(),
            version: 2,
            source: source.into(),
            library_items: Vec::new(),
            extra: Map::new(),
        }
    }

    pub fn add_from_selection(
        &mut self,
        scene: &DrawingScene,
        selection: &SelectionSet,
        id: impl Into<String>,
        created: u64,
        name: Option<String>,
    ) -> bool {
        let id = id.into();
        if id.is_empty() || self.library_items.iter().any(|item| item.id == id) {
            return false;
        }
        let fragment = scene.copy_selection_fragment(selection);
        if fragment.elements.is_empty() {
            return false;
        }
        self.library_items.push(LibraryItem {
            id,
            status: LibraryItemStatus::Unpublished,
            created,
            elements: fragment.elements,
            name,
            files: fragment.files,
        });
        true
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.library_items.len();
        self.library_items.retain(|item| item.id != id);
        self.library_items.len() != before
    }

    pub fn search(&self, query: &str) -> Vec<&LibraryItem> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return self.library_items.iter().collect();
        }
        self.library_items
            .iter()
            .filter(|item| {
                item.id.to_lowercase().contains(&needle)
                    || item
                        .name
                        .as_deref()
                        .is_some_and(|name| name.to_lowercase().contains(&needle))
                    || item.elements.iter().any(|element| {
                        element.text.to_lowercase().contains(&needle)
                            || element
                                .extra
                                .get("name")
                                .and_then(Value::as_str)
                                .is_some_and(|name| name.to_lowercase().contains(&needle))
                    })
            })
            .collect()
    }
}

impl DrawingScene {
    pub fn insert_library_item(
        &mut self,
        item: &LibraryItem,
        offset: [f32; 2],
        namespace: &str,
    ) -> SelectionSet {
        self.paste_fragment(
            &SceneFragment {
                elements: item.elements.clone(),
                files: item.files.clone(),
            },
            offset,
            namespace,
        )
    }
}
