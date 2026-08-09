use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

type R<T> = Result<T, String>;
const DEFAULT_PI_BASE_URL: &str = "http://127.0.0.1:8317/v1";
const DEFAULT_CODEX_CLIENT_VERSION: &str = "elephantnote";

pub struct PiChatResult {
  pub answer: String,
  pub provider: String,
  pub model: String,
  pub base_url: String,
}

fn text(value: &Value, keys: &[&str]) -> String {
  if let Some(raw) = value.as_str() {
    return raw.trim().to_string();
  }
  keys
    .iter()
    .find_map(|key| value.get(*key).and_then(Value::as_str))
    .map(str::trim)
    .unwrap_or("")
    .to_string()
}

fn pi_config(payload: &Value) -> &Value {
  payload
    .get("pi")
    .or_else(|| payload.pointer("/aiConfig/pi"))
    .or_else(|| payload.pointer("/aiConfig/providers/pi"))
    .or_else(|| payload.pointer("/config/pi"))
    .or_else(|| payload.pointer("/config/providers/pi"))
    .unwrap_or(&Value::Null)
}

pub fn pi_base_url(payload: &Value) -> String {
  let config = pi_config(payload);
  let from_payload = text(payload, &["piBaseUrl", "baseUrl"]);
  let from_config = text(config, &["baseUrl", "piBaseUrl"]);
  let raw = if !from_payload.is_empty() {
    from_payload
  } else if !from_config.is_empty() {
    from_config
  } else {
    DEFAULT_PI_BASE_URL.to_string()
  };
  raw.trim_end_matches('/').to_string()
}

pub fn pi_api_key(payload: &Value) -> String {
  let config = pi_config(payload);
  let from_payload = text(payload, &["piApiKey", "apiKey", "token"]);
  if !from_payload.is_empty() {
    return from_payload;
  }
  text(config, &["apiKey", "piApiKey", "token"])
}

pub fn codex_client_version(payload: &Value) -> String {
  let config = pi_config(payload);
  let from_payload = text(payload, &["clientVersion", "codexClientVersion"]);
  let from_config = text(config, &["clientVersion", "codexClientVersion"]);
  if !from_payload.is_empty() {
    from_payload
  } else if !from_config.is_empty() {
    from_config
  } else {
    DEFAULT_CODEX_CLIENT_VERSION.to_string()
  }
}

fn client() -> R<Client> {
  Client::builder()
    .timeout(Duration::from_secs(120))
    .build()
    .map_err(|error| error.to_string())
}

fn with_pi_headers(request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
  if api_key.trim().is_empty() {
    request
  } else {
    request.bearer_auth(api_key.trim())
  }
}

pub fn models_url(base_url: &str) -> String {
  format!("{}/models", base_url.trim_end_matches('/'))
}

pub fn codex_models_url(base_url: &str, client_version: &str) -> String {
  format!(
    "{}/models?client_version={}",
    base_url.trim_end_matches('/'),
    client_version.trim()
  )
}

pub fn chat_completions_url(base_url: &str) -> String {
  format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

pub fn normalize_openai_model_ids(data: &Value) -> Vec<String> {
  data
    .get("data")
    .and_then(Value::as_array)
    .map(|models| {
      models
        .iter()
        .filter_map(|model| model.get("id").and_then(Value::as_str).map(str::to_string))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

pub fn normalize_codex_model_slugs(data: &Value) -> Vec<String> {
  data
    .get("models")
    .and_then(Value::as_array)
    .map(|models| {
      models
        .iter()
        .filter_map(|model| {
          model
            .get("slug")
            .or_else(|| model.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

async fn request_json(url: String, payload: &Value) -> R<Value> {
  let api_key = pi_api_key(payload);
  let response = with_pi_headers(client()?.get(url), &api_key)
    .send()
    .await
    .map_err(|error| error.to_string())?;
  let status = response.status();
  let data = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
  if !status.is_success() {
    return Err(pi_error_text(&data, format!("PI returned HTTP {status}.")));
  }
  Ok(data)
}

pub async fn list_models(payload: &Value) -> R<Value> {
  request_json(models_url(&pi_base_url(payload)), payload).await
}

pub async fn list_codex_models(payload: &Value) -> R<Value> {
  request_json(codex_models_url(&pi_base_url(payload), &codex_client_version(payload)), payload).await
}

pub async fn chat_completion(model: &str, messages: &[Value], payload: &Value) -> R<PiChatResult> {
  let base_url = pi_base_url(payload);
  let body = json!({
    "model": model,
    "messages": messages,
    "temperature": payload.get("temperature").and_then(Value::as_f64).unwrap_or(0.2),
    "max_tokens": payload.get("maxTokens").or_else(|| payload.get("max_tokens")).and_then(Value::as_u64).unwrap_or(900),
    "stream": false
  });
  let api_key = pi_api_key(payload);
  let response = with_pi_headers(client()?.post(chat_completions_url(&base_url)).json(&body), &api_key)
    .send()
    .await
    .map_err(|error| error.to_string())?;
  let status = response.status();
  let data = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
  if !status.is_success() {
    return Err(pi_error_text(&data, format!("PI chat/completions returned HTTP {status}.")));
  }
  let answer = data
    .pointer("/choices/0/message/content")
    .and_then(Value::as_str)
    .or_else(|| data.pointer("/choices/0/text").and_then(Value::as_str))
    .unwrap_or("")
    .trim()
    .to_string();
  if answer.is_empty() {
    return Err("PI chat/completions returned an empty answer.".to_string());
  }
  Ok(PiChatResult {
    answer,
    provider: "pi-cliproxyapi".to_string(),
    model: model.to_string(),
    base_url,
  })
}

fn pi_error_text(data: &Value, fallback: String) -> String {
  data
    .pointer("/error/message")
    .and_then(Value::as_str)
    .or_else(|| data.get("message").and_then(Value::as_str))
    .filter(|value| !value.trim().is_empty())
    .map(str::to_string)
    .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_pi_base_url_matches_cliproxyapi_default_port() {
    assert_eq!(pi_base_url(&json!({})), "http://127.0.0.1:8317/v1");
  }

  #[test]
  fn reads_pi_base_url_from_ai_config() {
    let payload = json!({ "aiConfig": { "pi": { "baseUrl": "http://127.0.0.1:9000/v1/" } } });
    assert_eq!(pi_base_url(&payload), "http://127.0.0.1:9000/v1");
  }

  #[test]
  fn builds_pi_openai_and_codex_model_urls() {
    assert_eq!(models_url("http://127.0.0.1:8317/v1"), "http://127.0.0.1:8317/v1/models");
    assert_eq!(codex_models_url("http://127.0.0.1:8317/v1", "elephantnote"), "http://127.0.0.1:8317/v1/models?client_version=elephantnote");
  }

  #[test]
  fn normalizes_openai_compatible_model_list() {
    let ids = normalize_openai_model_ids(&json!({ "object": "list", "data": [{ "id": "gpt-5.5", "object": "model" }] }));
    assert_eq!(ids, vec!["gpt-5.5".to_string()]);
  }

  #[test]
  fn normalizes_codex_client_model_list() {
    let ids = normalize_codex_model_slugs(&json!({ "models": [{ "slug": "gpt-5.5", "display_name": "GPT-5.5" }] }));
    assert_eq!(ids, vec!["gpt-5.5".to_string()]);
  }
}
