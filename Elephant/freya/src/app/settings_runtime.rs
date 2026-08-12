//! Persistence and value transforms for the native settings surface.
//!
//! The renderer consumes the existing portable preference keys instead of
//! defining a Freya-only schema. The JSON object remains forward-compatible:
//! unknown keys are preserved when a supported setting is changed.

use crate::settings_contract::{DefaultValue, UiValueTransform, ValueKind, SETTINGS_PREFERENCES};
use serde_json::{Map, Value};
use std::{env, fs, io, path::PathBuf};

const PROFILE_OVERRIDE_ENV: &str = "ELEPHANT_FREYA_PROFILE";

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsRuntimeState {
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
        if matches!(
            contract.map(|item| item.transform),
            Some(UiValueTransform::InvertedBoolean)
        ) {
            !raw
        } else {
            raw
        }
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

    pub fn has_key(&self, key: &str) -> bool {
        self.preferences.contains_key(key)
    }

    pub fn string_list_value(&self, key: &str) -> Vec<String> {
        let Some(contract) = contract_for(key) else {
            return Vec::new();
        };
        self.value_for(Some(contract))
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect()
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
        if contract.value_kind != ValueKind::Boolean {
            return;
        }
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
        if !matches!(
            contract.value_kind,
            ValueKind::String | ValueKind::SingleCharacter
        ) {
            return;
        }
        let value = apply_text_transform(value, contract.transform);
        self.preferences
            .insert(key.to_owned(), Value::String(value));
        self.persist();
    }

    pub fn set_integer_preference(&mut self, key: &str, value: i64) {
        let Some(contract) = contract_for(key) else {
            return;
        };
        if contract.value_kind != ValueKind::Integer {
            return;
        }
        let value = apply_integer_transform(value, contract.transform);
        self.preferences
            .insert(key.to_owned(), Value::Number(value.into()));
        self.persist();
    }

    pub fn set_string_list_preference(&mut self, key: &str, value: Vec<String>) {
        let Some(contract) = contract_for(key) else {
            return;
        };
        if contract.value_kind != ValueKind::StringList {
            return;
        }
        self.preferences.insert(
            key.to_owned(),
            Value::Array(value.into_iter().map(Value::String).collect()),
        );
        self.persist();
    }

    pub fn toggle_string_list_value(&mut self, key: &str, item: &str) {
        let mut values = self.string_list_value(key);
        if let Some(index) = values.iter().position(|value| value == item) {
            values.remove(index);
        } else {
            values.push(item.to_owned());
        }
        self.set_string_list_preference(key, values);
    }

    pub fn cycle_auto_save_delay(&mut self) {
        if !self.is_enabled("autoSaveDelay") {
            return;
        }
        let next = match self.integer_value("autoSaveDelay") {
            250 => 500,
            500 => 1000,
            1000 => 2000,
            2000 => 5000,
            _ => 250,
        };
        self.set_integer_preference("autoSaveDelay", next);
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
    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "elephant-freya-settings-{}-{stamp}",
                std::process::id()
            ))
            .join(format!("{name}.json"))
    }

    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_dir_all(path.parent().expect("test directory"));
    }

    #[test]
    fn theme_and_editor_preferences_are_reloaded_from_disk() {
        let path = test_path("round-trip");
        let mut state = SettingsRuntimeState::load_from(path.clone());
        state.set_text_preference("theme", "nord-dark".to_owned());
        state.toggle_bool("autoPairBracket");
        state.set_integer_preference("noteEditorMargin", 44);

        let reloaded = SettingsRuntimeState::load_from(path.clone());
        assert_eq!(reloaded.text_value("theme"), "nord-dark");
        assert!(!reloaded.bool_value("autoPairBracket"));
        assert_eq!(reloaded.integer_value("noteEditorMargin"), 44);
        assert!(reloaded.load_error.is_none());

        cleanup(&path);
    }

    #[test]
    fn source_transforms_are_persisted_not_only_reflected_in_widget_state() {
        let path = test_path("transforms");
        let mut state = SettingsRuntimeState::load_from(path.clone());
        state.toggle_bool("hideQuickInsertHint");
        state.set_text_preference("quickInsertTrigger", "//".to_owned());
        state.set_integer_preference("noteEditorMargin", 100);

        let raw = fs::read_to_string(&path).expect("persisted preferences");
        let persisted: Value = serde_json::from_str(&raw).expect("valid persisted json");
        assert_eq!(persisted["hideQuickInsertHint"], Value::Bool(true));
        assert_eq!(
            persisted["quickInsertTrigger"],
            Value::String("/".to_owned())
        );
        assert_eq!(persisted["noteEditorMargin"], Value::Number(48.into()));

        let reloaded = SettingsRuntimeState::load_from(path.clone());
        assert!(!reloaded.bool_value("hideQuickInsertHint"));
        assert_eq!(reloaded.text_value("quickInsertTrigger"), "/");
        assert_eq!(reloaded.integer_value("noteEditorMargin"), 48);

        cleanup(&path);
    }

    #[test]
    fn updates_keep_unknown_forward_compatible_preferences() {
        let path = test_path("unknown-key");
        fs::create_dir_all(path.parent().expect("test directory")).expect("create test dir");
        fs::write(
            &path,
            r#"{"futureSetting":{"enabled":true},"theme":"light"}"#,
        )
        .expect("seed preferences");

        let mut state = SettingsRuntimeState::load_from(path.clone());
        state.set_text_preference("theme", "dark".to_owned());
        let reloaded = SettingsRuntimeState::load_from(path.clone());
        assert_eq!(reloaded.text_value("theme"), "dark");
        assert_eq!(
            reloaded.preferences.get("futureSetting"),
            Some(&serde_json::json!({"enabled": true}))
        );

        cleanup(&path);
    }

    #[test]
    fn disabled_autosave_delay_cannot_be_changed_through_runtime() {
        let path = test_path("disabled-delay");
        let mut state = SettingsRuntimeState::load_from(path.clone());
        state.set_integer_preference("autoSaveDelay", 5000);
        assert!(!state.bool_value("autoSave"));
        assert!(!state.is_enabled("autoSaveDelay"));

        state.cycle_auto_save_delay();
        assert_eq!(state.integer_value("autoSaveDelay"), 5000);

        state.toggle_bool("autoSave");
        assert!(state.is_enabled("autoSaveDelay"));
        state.cycle_auto_save_delay();
        assert_eq!(state.integer_value("autoSaveDelay"), 250);

        cleanup(&path);
    }

    #[test]
    fn icon_rail_hidden_round_trips_as_a_string_list() {
        let path = test_path("icon-rail-hidden");
        let mut state = SettingsRuntimeState::load_from(path.clone());
        state.toggle_string_list_value("iconRailHidden", "search");
        state.toggle_string_list_value("iconRailHidden", "vault");

        let reloaded = SettingsRuntimeState::load_from(path.clone());
        assert_eq!(
            reloaded.string_list_value("iconRailHidden"),
            vec!["search".to_owned(), "vault".to_owned()]
        );

        let mut reloaded = reloaded;
        reloaded.toggle_string_list_value("iconRailHidden", "search");
        assert_eq!(
            SettingsRuntimeState::load_from(path.clone()).string_list_value("iconRailHidden"),
            vec!["vault".to_owned()]
        );

        cleanup(&path);
    }

    #[test]
    fn setters_ignore_unknown_keys_and_wrong_value_kinds() {
        let path = test_path("type-safety");
        let mut state = SettingsRuntimeState::load_from(path.clone());

        state.toggle_bool("theme");
        state.set_text_preference("autoSave", "true".to_owned());
        state.set_integer_preference("quickInsertTrigger", 9);
        state.set_string_list_preference("theme", vec!["dark".to_owned()]);
        state.set_text_preference("not-a-setting", "ignored".to_owned());

        assert!(state.preferences.is_empty());
        assert!(!path.exists(), "invalid setters must not write a file");

        cleanup(&path);
    }

    #[test]
    fn malformed_or_non_object_preferences_surface_load_errors() {
        let malformed = test_path("malformed");
        fs::create_dir_all(malformed.parent().expect("test directory")).expect("create test dir");
        fs::write(&malformed, "{broken").expect("seed malformed json");
        let malformed_state = SettingsRuntimeState::load_from(malformed.clone());
        assert!(malformed_state.load_error.is_some());
        cleanup(&malformed);

        let array = test_path("array");
        fs::create_dir_all(array.parent().expect("test directory")).expect("create test dir");
        fs::write(&array, "[]").expect("seed non-object json");
        let array_state = SettingsRuntimeState::load_from(array.clone());
        assert_eq!(
            array_state.load_error.as_deref(),
            Some("Preferences must be a JSON object.")
        );
        cleanup(&array);
    }
}
