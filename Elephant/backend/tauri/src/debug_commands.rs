use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn details_text(details: Option<&Value>) -> String {
  details.map(|value| value.to_string()).unwrap_or_default()
}

fn timestamp_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
}

fn ansi(code: &str) -> String {
  format!("{}[{code}m", 27_u8 as char)
}

fn level_color(level: &str) -> String {
  match level {
    "error" => ansi("31;1"),
    "warn" => ansi("33;1"),
    "debug" | "trace" => ansi("36"),
    _ => ansi("32"),
  }
}

fn stream_prefix(level: &str, message: &str) -> String {
  let module = message
    .strip_prefix('[')
    .and_then(|rest| rest.split(']').next())
    .unwrap_or("renderer");
  format!(
    "{}[{}][{}][{}]{}",
    level_color(level),
    timestamp_ms(),
    level,
    module,
    ansi("0")
  )
}

#[tauri::command]
pub fn tauri_debug_log(level: String, message: String, details: Option<Value>) -> bool {
  let normalized_level = match level.as_str() {
    "error" | "warn" | "debug" | "trace" | "info" => level.as_str(),
    _ => "info",
  };
  let prefix = stream_prefix(normalized_level, &message);
  let details = details_text(details.as_ref());
  match normalized_level {
    "error" | "warn" => eprintln!("{prefix} {message} {details}"),
    _ => println!("{prefix} {message} {details}"),
  }

  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_info_logs() {
    assert!(tauri_debug_log("info".to_string(), "boot".to_string(), None));
  }

  #[test]
  fn accepts_warning_logs() {
    assert!(tauri_debug_log("warn".to_string(), "warning".to_string(), Some(serde_json::json!({ "ok": true }))));
  }

  #[test]
  fn extracts_module_from_bracketed_messages() {
    assert!(stream_prefix("info", "[search] query:start").contains("[search]"));
  }
}
