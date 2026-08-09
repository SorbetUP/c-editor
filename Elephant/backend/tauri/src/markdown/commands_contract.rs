use serde_json::Value;

use super::muya_deterministic as contract_engine;

#[tauri::command]
pub fn tauri_muya_contract(markdown: String) -> Value {
  contract_engine::deterministic_contract(&markdown)
}
