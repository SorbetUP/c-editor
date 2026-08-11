#[path = "../../backend/tauri/src/vault/types.rs"]
pub mod types;

pub mod config {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now_string() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    }

    pub fn basename(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Personal")
            .to_string()
    }
}

#[path = "../../backend/tauri/src/vault/entries.rs"]
pub mod entries;
#[path = "../../backend/tauri/src/vault/metadata.rs"]
pub mod metadata;
