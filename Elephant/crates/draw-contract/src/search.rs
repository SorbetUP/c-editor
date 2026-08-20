use crate::{DrawingElement, DrawingScene};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchField {
    Text,
    FrameName,
    Link,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneSearchMatch {
    pub element_id: String,
    pub field: SearchField,
    pub value: String,
}

impl DrawingScene {
    pub fn search_scene(&self, query: &str) -> Vec<SceneSearchMatch> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        let mut matches = Vec::new();
        for element in self.visible_elements() {
            if !element.text.is_empty() && element.text.to_lowercase().contains(&needle) {
                matches.push(SceneSearchMatch {
                    element_id: element.id.clone(),
                    field: SearchField::Text,
                    value: element.text.clone(),
                });
            }
            if let Some(name) = element.extra.get("name").and_then(Value::as_str) {
                if name.to_lowercase().contains(&needle) {
                    matches.push(SceneSearchMatch {
                        element_id: element.id.clone(),
                        field: SearchField::FrameName,
                        value: name.to_owned(),
                    });
                }
            }
            if let Some(link) = element.extra.get("link").and_then(Value::as_str) {
                if link.to_lowercase().contains(&needle) {
                    matches.push(SceneSearchMatch {
                        element_id: element.id.clone(),
                        field: SearchField::Link,
                        value: link.to_owned(),
                    });
                }
            }
        }
        matches
    }

    pub fn element_link(&self, id: &str) -> Option<&str> {
        self.element_by_id(id)?
            .extra
            .get("link")
            .and_then(Value::as_str)
            .filter(|link| !link.is_empty())
    }

    pub fn set_element_link(&mut self, id: &str, link: Option<&str>) -> bool {
        let Some(index) = self.element_index(id) else {
            return false;
        };
        if self.elements[index].is_locked() {
            return false;
        }
        let next = link
            .map(str::trim)
            .filter(|link| !link.is_empty())
            .map(|link| Value::String(normalize_link(link)))
            .unwrap_or(Value::Null);
        if self.elements[index].extra.get("link") == Some(&next) {
            return false;
        }
        self.elements[index].extra.insert("link".to_owned(), next);
        mark_changed(&mut self.elements[index]);
        true
    }
}

fn normalize_link(link: &str) -> String {
    let trimmed = link.trim();
    if trimmed.starts_with('#')
        || trimmed.contains("://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
    {
        return trimmed.to_owned();
    }
    if trimmed.contains('@') && !trimmed.contains('/') {
        return format!("mailto:{trimmed}");
    }
    format!("https://{trimmed}")
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
    element.extra.insert("versionNonce".to_owned(), json!(nonce));
}
