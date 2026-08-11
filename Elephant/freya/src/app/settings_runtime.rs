//! Persistence and value transforms for the native settings surface.
//!
//! The renderer deliberately consumes this small runtime boundary instead of
//! defining a second Freya-only preferences schema.  The keys, defaults, and
//! UI transforms are the contracts observed in `settings_contract`; the JSON
//! shape is the flat object used by the Tauri preferences service.

use crate::settings_contract::{DefaultValue, UiValueTransform, ValueKind, SETTINGS_PREFERENCES};
use serde_json::{Map, Value};
use std::{env, fs, io, path::PathBuf};

const PROFILE_OVERRIDE_ENV: &str = "ELEPHANT_FREYA_PROFILE";

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsRuntimeState {
    /// The complete flat preferences object, including keys unknown to Freya.
    pub preferences: Map<String, Value>,
    pub persistence_path: PathBuf,
    pub feedback: Option<String>,
    pub load_error: Option<String>,
}

impl SettingsRuntimeState {
    pub fn load_from(path: impl Into<PathBuf>) -> Self {
        let persistence_path = path.into();
        let mut state = Self {
            preferences: Map::new(),
            persistence_path,
            feedback: None,
            load_error: None,
        };

        match fs::read_to_string(&state.persistence_path) {
            Ok(raw) => match serde_json::from_str::<Value>(&raw) {
                Ok(Value::Object(values)) => state.preferences = values,
                Ok(_) => state.load_error = Some("Preferences must be a JSON object.".to_owned()),
                Err(error) => state.load_error = Some(error.to_string()),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => state.load_error = Some(error.to_string()),
        }

        state
    }

    pub fn bool_value(&self, key: &str) -> bool {
        let contract = contract_for(key);
        let raw = self
            .value_for(contract)
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        matches!(
            contract.map(|item| item.transform),
            Some(UiValueTransform::InvertedBoolean)
        )
        .then_some(!raw)
        .unwrap_or(raw)
    }

    pub fn text_value(&self, key: &str) -> String {
        let Some(contract) = contract_for(key) else {
            return self
                .preferences
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        };
        let value = self
            .value_for(Some(contract))
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_default();
        apply_text_transform(value, contract.transform)
    }

    pub fn integer_value(&self, key: &str) -> i64 {
        let Some(contract) = contract_for(key) else {
            return self
                .preferences
                .get(key)
                .and_then(Value::as_i64)
                .unwrap_or_default();
        };
        let value = self
            .value_for(Some(contract))
            .and_then(|value| value.as_i64())
            .unwrap_or_default();
        apply_integer_transform(value, contract.transform)
    }

    pub fn is_enabled(&self, key: &str) -> bool {
        match contract_for(key).map(|item| item.transform) {
            Some(UiValueTransform::DisabledWhenAutosaveFalse) => self.bool_value("autoSave"),
            _ => true,
        }
    }

    pub fn toggle_bool(&mut self, key: &str) {
        let Some(contract) = contract_for(key) else {
            return;
        };
        let next_ui_value = !self.bool_value(key);
        let next_raw_value = if contract.transform == UiValueTransform::InvertedBoolean {
            !next_ui_value
        } else {
            next_ui_value
        };
        self.preferences
            .insert(key.to_owned(), Value::Bool(next_raw_value));
        self.persist();
    }

    pub fn set_text_preference(&mut self, key: &str, value: String) {
        let Some(contract) = contract_for(key) else {
            return;
        };
        let value = apply_text_transform(value, contract.transform);
        self.preferences
            .insert(key.to_owned(), Value::String(value));
        self.persist();
    }

    pub fn cycle_auto_save_delay(&mut self) {
        let next = match self.integer_value("autoSaveDelay") {
            250 => 500,
            500 => 1000,
            1000 => 2000,
            2000 => 5000,
            _ => 250,
        };
        self.preferences
            .insert("autoSaveDelay".to_owned(), Value::Number(next.into()));
        self.persist();
    }

    fn value_for(
        &self,
        contract: Option<&crate::settings_contract::PreferenceContract>,
    ) -> Option<Value> {
        let contract = contract?;
        Some(
            self.preferences
                .get(contract.key)
                .cloned()
                .unwrap_or_else(|| default_json_value(contract.default, contract.value_kind)),
        )
    }

    fn persist(&mut self) {
        let Some(parent) = self.persistence_path.parent() else {
            self.feedback = Some("Settings path has no parent directory.".to_owned());
            return;
        };
        let result = fs::create_dir_all(parent).and_then(|_| {
            let raw = serde_json::to_vec_pretty(&Value::Object(self.preferences.clone()))
                .map_err(io::Error::other)?;
            write_atomically(&self.persistence_path, &raw)
        });
        match result {
            Ok(()) => {
                self.feedback = Some("Settings saved".to_owned());
                eprintln!(
                    "[freya][settings] action:complete action=persist path={}",
                    self.persistence_path.display()
                );
            }
            Err(error) => {
                self.feedback = Some(format!("Settings could not be saved: {error}"));
                eprintln!(
                    "[freya][settings] action:failure action=persist path={} error={error}",
                    self.persistence_path.display()
                );
            }
        }
    }
}

impl Default for SettingsRuntimeState {
    fn default() -> Self {
        Self::load_from(default_preferences_path())
    }
}

pub fn default_preferences_path() -> PathBuf {
    if let Some(profile) = env::var_os(PROFILE_OVERRIDE_ENV) {
        return PathBuf::from(profile).join("preferences.json");
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("com.elephantnote.app")
                .join("preferences.json");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = env::var_os("APPDATA") {
            return PathBuf::from(app_data)
                .join("com.elephantnote.app")
                .join("preferences.json");
        }
    }

    let config_root = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    config_root
        .join("com.elephantnote.app")
        .join("preferences.json")
}

fn contract_for(key: &str) -> Option<&'static crate::settings_contract::PreferenceContract> {
    SETTINGS_PREFERENCES.iter().find(|item| item.key == key)
}

fn default_json_value(default: DefaultValue, kind: ValueKind) -> Value {
    match default {
        DefaultValue::Boolean(value) => Value::Bool(value),
        DefaultValue::Integer(value) => Value::Number(value.into()),
        DefaultValue::Text(value) | DefaultValue::RuntimeFallback(value) => match kind {
            ValueKind::StringList => Value::Array(
                value
                    .split(',')
                    .filter(|item| !item.is_empty())
                    .map(|item| Value::String(item.to_owned()))
                    .collect(),
            ),
            _ => Value::String(value.to_owned()),
        },
        DefaultValue::MissingFromStoreState => match kind {
            ValueKind::Boolean => Value::Bool(false),
            ValueKind::Integer => Value::Number(0.into()),
            ValueKind::StringList => Value::Array(Vec::new()),
            _ => Value::String(String::new()),
        },
    }
}

fn apply_text_transform(mut value: String, transform: UiValueTransform) -> String {
    if let UiValueTransform::MaxLength(maximum) = transform {
        value = value.chars().take(maximum).collect();
    }
    value
}

fn apply_integer_transform(value: i64, transform: UiValueTransform) -> i64 {
    match transform {
        UiValueTransform::Clamped { minimum, maximum } => value.clamp(minimum, maximum),
        _ => value,
    }
}

fn write_atomically(path: &PathBuf, raw: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, raw)?;
    fs::rename(temporary, path)
}
