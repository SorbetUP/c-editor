//! Freya boundary for the existing OpenAI-compatible PI provider runtime.

use serde_json::Value;

#[path = "../../backend/tauri/src/pi_runtime.rs"]
mod production;

pub(super) use production::PiChatResult;

pub(super) async fn complete(
    model: &str,
    messages: &[Value],
    payload: &Value,
) -> Result<PiChatResult, String> {
    production::chat_completion(model, messages, payload).await
}
