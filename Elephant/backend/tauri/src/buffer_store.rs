use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::infra::{read_json_or, write_json_atomically};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct EditorBuffer {
  #[serde(default)]
  pub open_tabs: Vec<serde_json::Value>,
  #[serde(default)]
  pub unsaved_markdown: serde_json::Map<String, serde_json::Value>,
  #[serde(default)]
  pub window_state: Option<serde_json::Value>,
}

pub struct BufferStore {
  dir: PathBuf,
}

impl BufferStore {
  pub fn load(app: &AppHandle) -> Option<Self> {
    let dir = app.path().app_data_dir().ok()?.join("window_buffers");
    fs::create_dir_all(&dir).ok()?;
    Some(BufferStore { dir })
  }

  pub fn path(&self, window_id: &str) -> PathBuf {
    let safe = window_id.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect::<String>();
    self.dir.join(format!("{safe}.json"))
  }

  pub fn read(&self, window_id: &str) -> EditorBuffer {
    read_json_or(self.path(window_id), EditorBuffer::default())
  }

  pub fn save(&self, window_id: &str, buffer: &EditorBuffer) -> std::io::Result<()> {
    write_json_atomically(self.path(window_id), buffer)
  }

  pub fn clear(&self, window_id: &str) -> std::io::Result<()> {
    let path = self.path(window_id);
    if path.exists() {
      fs::remove_file(path)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn window_id_is_sanitized_for_path() {
    let dir = std::env::temp_dir().join(format!("elephantnote_buf_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let store = BufferStore { dir };
    let p1 = store.path("../escape");
    let p2 = store.path("safe-id");
    assert!(p1.file_name().unwrap().to_string_lossy().ends_with(".json"));
    assert!(!p1.to_string_lossy().contains(".."));
    assert_eq!(p2.file_name().unwrap().to_string_lossy(), "safe-id.json");
    fs::remove_dir_all(&store.dir).ok();
  }

  #[test]
  fn save_read_roundtrip() {
    let dir = std::env::temp_dir().join(format!("elephantnote_buf_rt_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let store = BufferStore { dir };
    let mut buffer = EditorBuffer::default();
    buffer.open_tabs = vec![json!({ "pathname": "/tmp/note.md" })];
    buffer.unsaved_markdown.insert("tab1".into(), json!("**draft**"));
    store.save("win1", &buffer).unwrap();
    let loaded = store.read("win1");
    assert_eq!(loaded.open_tabs.len(), 1);
    assert!(loaded.unsaved_markdown.contains_key("tab1"));
store.clear("win1").unwrap();
    assert!(!store.path("win1").exists());
    let cleared = store.read("win1");
    assert!(cleared.open_tabs.is_empty());
    fs::remove_dir_all(&store.dir).ok();
  }
}