//! Native editor policy shared by the real Muya lifecycle.
//!
//! Preferences use the same canonical profile file as the Tauri app config.
//! Restart scroll state is deliberately not implemented here: the production
//! buffered state is owned by Tauri's `buffer_store/window_buffers` contract,
//! which cannot be reached from this assigned Freya write set without a host
//! command. The editor keeps scroll state only for its current lifecycle.

use std::{
    env, fs,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};

const PROFILE_OVERRIDE_ENV: &str = "ELEPHANT_FREYA_PROFILE";
const TAURI_IDENTIFIER: &str = "com.elephantnote.app";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EditorPreferences {
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            auto_save: false,
            auto_save_delay_ms: 5000,
        }
    }
}

pub fn preferences_path() -> PathBuf {
    if let Some(profile) = env::var_os(PROFILE_OVERRIDE_ENV) {
        return PathBuf::from(profile).join("preferences.json");
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(TAURI_IDENTIFIER)
            .join("preferences.json");
    }
    #[cfg(target_os = "windows")]
    if let Some(app_data) = env::var_os("APPDATA") {
        return PathBuf::from(app_data)
            .join(TAURI_IDENTIFIER)
            .join("preferences.json");
    }
    let root = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    root.join(TAURI_IDENTIFIER).join("preferences.json")
}

pub fn load_preferences() -> EditorPreferences {
    let mut preferences = EditorPreferences::default();
    let Ok(raw) = fs::read_to_string(preferences_path()) else {
        return preferences;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return preferences;
    };
    let Some(values) = value.as_object() else {
        return preferences;
    };
    if let Some(value) = values.get("autoSave").and_then(serde_json::Value::as_bool) {
        preferences.auto_save = value;
    }
    if let Some(value) = values
        .get("autoSaveDelay")
        .and_then(serde_json::Value::as_u64)
    {
        preferences.auto_save_delay_ms = value;
    }
    preferences
}

/// A non-blocking runtime delay used only to debounce autosave work.
pub struct Delay {
    deadline: Instant,
    wake: Arc<Mutex<Option<Waker>>>,
    started: bool,
}

impl Delay {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            wake: Arc::new(Mutex::new(None)),
            started: false,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }
        if let Ok(mut wake) = self.wake.lock() {
            *wake = Some(context.waker().clone());
        }
        if !self.started {
            self.started = true;
            let wake = Arc::clone(&self.wake);
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            thread::spawn(move || {
                thread::sleep(remaining);
                if let Ok(mut wake) = wake.lock() {
                    if let Some(waker) = wake.take() {
                        waker.wake();
                    }
                }
            });
        }
        Poll::Pending
    }
}
