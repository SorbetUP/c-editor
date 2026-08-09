use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowRequest {
    pub shadow_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeState {
    pub configured: bool,
    pub uri: Option<String>,
    pub display_name: Option<String>,
    pub shadow_path: String,
    pub files_copied: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareTextRequest {
    pub title: String,
    pub text: String,
}
