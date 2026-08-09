use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

const RECENTS_FILE: &str = "recently-used-documents.json";
const MAX_RECENTS: usize = 20;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(transparent)]
pub struct Recents(pub Vec<String>);

pub struct RecentsStore {
  path: PathBuf,
}

impl RecentsStore {
  pub fn load(app: &AppHandle) -> Option<Self> {
    let dir = app.path().app_config_dir().ok()?;
    fs::create_dir_all(&dir).ok()?;
    Some(RecentsStore { path: dir.join(RECENTS_FILE) })
  }

  pub fn list(&self) -> Recents {
    read_json_or(&self.path, Recents::default())
  }

  pub fn add(&self, path: String) -> std::io::Result<Recents> {
    let mut list = self.list();
    list.0.retain(|p| p != &path);
    list.0.insert(0, path);
    list.0.truncate(MAX_RECENTS);
    write_json_atomically(&self.path, &list)?;
    Ok(list)
  }

  pub fn clear(&self) -> std::io::Result<()> {
    let empty = Recents::default();
    write_json_atomically(&self.path, &empty)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn add_dedupes_and_prepends() {
    let dir = std::env::temp_dir().join(format!("elephantnote_recents_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("recents.json");
    let store = RecentsStore { path };
    let r = store.add("/a.md".into()).unwrap();
    assert_eq!(r.0, vec!["/a.md"]);
    let r = store.add("/b.md".into()).unwrap();
    assert_eq!(r.0, vec!["/b.md", "/a.md"]);
    let r = store.add("/a.md".into()).unwrap();
    assert_eq!(r.0, vec!["/a.md", "/b.md"]);
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn truncate_to_max() {
    let dir = std::env::temp_dir().join(format!("elephantnote_recents_trunc_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("recents.json");
    let store = RecentsStore { path };
    for i in 0..(MAX_RECENTS + 5) {
      store.add(format!("/{i}.md")).unwrap();
    }
    let list = store.list();
    assert_eq!(list.0.len(), MAX_RECENTS);
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn clear_empties_list() {
    let dir = std::env::temp_dir().join(format!("elephantnote_recents_clear_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("recents.json");
    let store = RecentsStore { path };
    store.add("/x.md".into()).unwrap();
    store.clear().unwrap();
    let list = store.list();
    assert!(list.0.is_empty());
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn json_shape_is_array_of_strings() {
    let r = Recents(vec!["/a".into(), "/b".into()]);
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v, json!(["/a", "/b"]));
  }
}