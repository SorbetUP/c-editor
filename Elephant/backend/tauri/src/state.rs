use serde_json::{Map, Value};
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::atomic_features::AtomicFeatureStore;
use crate::buffer_store::{BufferStore, EditorBuffer};
use crate::data_center::DataCenter;
use crate::keybindings::{Keybindings, KeybindingsStore};
use crate::preferences::Preferences;
use crate::recents::{Recents, RecentsStore};

pub struct AppState {
  pub prefs: Mutex<Option<Preferences>>,
  pub data: Mutex<Option<DataCenter>>,
  pub buffers: Option<BufferStore>,
  pub recents: Option<RecentsStore>,
  pub keybindings: Option<KeybindingsStore>,
  pub atomic: Mutex<Option<AtomicFeatureStore>>,
}

impl AppState {
  pub fn new(app: &AppHandle) -> Self {
    AppState {
      prefs: Mutex::new(Preferences::load(app)),
      data: Mutex::new(DataCenter::load(app)),
      buffers: BufferStore::load(app),
      recents: RecentsStore::load(app),
      keybindings: KeybindingsStore::load(app),
      atomic: Mutex::new(AtomicFeatureStore::load(app)),
    }
  }
}

type R<T> = Result<T, String>;

#[tauri::command]
pub fn tauri_prefs_get(state: State<'_, AppState>, key: String) -> R<Value> {
  let guard = state.prefs.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(p) => p.get(&key),
    None => Value::Null,
  })
}

#[tauri::command]
pub fn tauri_prefs_all(state: State<'_, AppState>) -> R<Value> {
  let guard = state.prefs.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(p) => p.get_all(),
    None => Value::Object(Map::new()),
  })
}

#[tauri::command]
pub fn tauri_prefs_set(state: State<'_, AppState>, key: String, value: Value) -> R<Value> {
  let mut guard = state.prefs.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(p) => p.set(&key, value).map_err(|e| e.to_string()),
    None => Err("preferences not available".into()),
  }
}

#[tauri::command]
pub fn tauri_prefs_set_many(state: State<'_, AppState>, items: Value) -> R<()> {
  let mut guard = state.prefs.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(p) => p.set_many(items).map_err(|e| e.to_string()),
    None => Err("preferences not available".into()),
  }
}

#[tauri::command]
pub fn tauri_user_data_get(state: State<'_, AppState>, key: String) -> R<Value> {
  let guard = state.data.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(d) => d.get(&key),
    None => Value::Null,
  })
}

#[tauri::command]
pub fn tauri_user_data_all(state: State<'_, AppState>) -> R<Value> {
  let guard = state.data.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(d) => d.get_all_sync(),
    None => Value::Object(Map::new()),
  })
}

#[tauri::command]
pub fn tauri_user_data_set(state: State<'_, AppState>, key: String, value: Value) -> R<Value> {
  let mut guard = state.data.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(d) => d.set(&key, value).map_err(|e| e.to_string()),
    None => Err("userData not available".into()),
  }
}

#[tauri::command]
pub fn tauri_user_data_set_many(state: State<'_, AppState>, items: Value) -> R<()> {
  let mut guard = state.data.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(d) => d.set_many(items).map_err(|e| e.to_string()),
    None => Err("userData not available".into()),
  }
}

#[tauri::command]
pub fn tauri_secret_set(state: State<'_, AppState>, name: String, value: String) -> R<()> {
  let mut guard = state.data.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(d) => {
      d.set(&name, Value::String(value)).map(|_| ()).map_err(|e| e.to_string())
    }
    None => Err("userData not available".into()),
  }
}

#[tauri::command]
pub fn tauri_secret_get(state: State<'_, AppState>, name: String) -> R<Value> {
  let guard = state.data.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(d) => d.get(&name),
    None => Value::Null,
  })
}

#[tauri::command]
pub fn tauri_secret_delete(state: State<'_, AppState>, name: String) -> R<()> {
  let mut guard = state.data.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(d) => {
      d.set(&name, Value::String(String::new())).map(|_| ()).map_err(|e| e.to_string())
    }
    None => Err("userData not available".into()),
  }
}

#[tauri::command]
pub fn tauri_buffer_save(state: State<'_, AppState>, window_id: String, buffer: EditorBuffer) -> R<()> {
  match &state.buffers {
    Some(b) => b.save(&window_id, &buffer).map_err(|e| e.to_string()),
    None => Err("buffer store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_buffer_load(state: State<'_, AppState>, window_id: String) -> R<EditorBuffer> {
  match &state.buffers {
    Some(b) => Ok(b.read(&window_id)),
    None => Err("buffer store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_buffer_clear(state: State<'_, AppState>, window_id: String) -> R<()> {
  match &state.buffers {
    Some(b) => b.clear(&window_id).map_err(|e| e.to_string()),
    None => Err("buffer store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_recents_list(state: State<'_, AppState>) -> R<Recents> {
  match &state.recents {
    Some(s) => Ok(s.list()),
    None => Ok(Recents::default()),
  }
}

#[tauri::command]
pub fn tauri_recents_add(state: State<'_, AppState>, path: String) -> R<Recents> {
  match &state.recents {
    Some(s) => s.add(path).map_err(|e| e.to_string()),
    None => Err("recents store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_recents_clear(state: State<'_, AppState>) -> R<()> {
  match &state.recents {
    Some(s) => s.clear().map_err(|e| e.to_string()),
    None => Err("recents store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_keybindings_get(state: State<'_, AppState>) -> R<Keybindings> {
  match &state.keybindings {
    Some(s) => Ok(s.get()),
    None => Ok(Keybindings::default()),
  }
}

#[tauri::command]
pub fn tauri_keybindings_save(state: State<'_, AppState>, bindings: Keybindings) -> R<()> {
  match &state.keybindings {
    Some(s) => s.save(bindings).map_err(|e| e.to_string()),
    None => Err("keybindings store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_atomic_features_list(state: State<'_, AppState>) -> R<serde_json::Value> {
  let guard = state.atomic.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(store) => serde_json::to_value(store.list()).unwrap_or(serde_json::Value::Null),
    None => serde_json::Value::Object(serde_json::Map::new()),
  })
}

#[tauri::command]
pub fn tauri_atomic_features_get(state: State<'_, AppState>, feature: String) -> R<bool> {
  let guard = state.atomic.lock().map_err(|e| e.to_string())?;
  Ok(match guard.as_ref() {
    Some(store) => store.get(&feature).as_bool().unwrap_or(false),
    None => false,
  })
}

#[tauri::command]
pub fn tauri_atomic_features_toggle(state: State<'_, AppState>, feature: String) -> R<bool> {
  let mut guard = state.atomic.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(store) => store.toggle(&feature).map_err(|e| e.to_string()),
    None => Err("atomic feature store not available".into()),
  }
}

#[tauri::command]
pub fn tauri_atomic_features_set(state: State<'_, AppState>, feature: String, enabled: bool) -> R<()> {
  let mut guard = state.atomic.lock().map_err(|e| e.to_string())?;
  match guard.as_mut() {
    Some(store) => store.set_enabled(&feature, enabled).map_err(|e| e.to_string()),
    None => Err("atomic feature store not available".into()),
  }
}