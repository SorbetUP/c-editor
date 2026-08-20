//! Shared production vault implementation.

pub mod config {
    use std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

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

pub mod types;
pub mod metadata;
pub mod entries;
