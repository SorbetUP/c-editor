use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::AppHandle;

use crate::vault::config as vault_config;
use crate::vault_layout;

const MAX_ASSET_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuyaAssetResult {
    pub path: String,
    pub full_path: String,
    pub file_name: String,
    pub bytes_written: usize,
}

fn extension_for_mime(mime_type: &str) -> Option<&'static str> {
    match mime_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/svg+xml" => Some("svg"),
        "image/avif" => Some("avif"),
        _ => None,
    }
}

fn sanitize_stem(file_name: Option<&str>) -> String {
    let stem = file_name
        .and_then(|name| Path::new(name).file_stem())
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    let safe = stem
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if safe.is_empty() {
        "image".to_string()
    } else {
        safe.chars().take(64).collect()
    }
}

fn unique_asset_path(assets: &Path, stem: &str, extension: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    for suffix in 0..10_000_u32 {
        let file_name = if suffix == 0 {
            format!("{stem}-{millis}.{extension}")
        } else {
            format!("{stem}-{millis}-{suffix}.{extension}")
        };
        let candidate = assets.join(file_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    assets.join(format!("{stem}-{millis}-overflow.{extension}"))
}

pub fn write_asset(
    vault_root: &Path,
    file_name: Option<&str>,
    mime_type: &str,
    bytes: &[u8],
) -> Result<MuyaAssetResult, String> {
    let extension = extension_for_mime(mime_type)
        .ok_or_else(|| format!("unsupported Muya asset MIME type: {mime_type}"))?;
    if bytes.is_empty() {
        return Err("Muya asset is empty".to_string());
    }
    if bytes.len() > MAX_ASSET_BYTES {
        return Err(format!(
            "Muya asset exceeds the {} MiB limit",
            MAX_ASSET_BYTES / 1024 / 1024
        ));
    }

    let assets = vault_layout::assets_dir(vault_root);
    fs::create_dir_all(&assets).map_err(|error| error.to_string())?;
    let path = unique_asset_path(&assets, &sanitize_stem(file_name), extension);
    fs::write(&path, bytes).map_err(|error| error.to_string())?;
    let relative = path
        .strip_prefix(vault_root)
        .map_err(|_| "Muya asset path escaped the active vault".to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(MuyaAssetResult {
        path: relative,
        full_path: path.to_string_lossy().to_string(),
        file_name,
        bytes_written: bytes.len(),
    })
}

#[tauri::command]
pub fn tauri_muya_asset_write(
    app: AppHandle,
    file_name: Option<String>,
    mime_type: String,
    bytes: Vec<u8>,
) -> Result<MuyaAssetResult, String> {
    let vault = vault_config::get_active_vault(&app)?;
    write_asset(
        Path::new(&vault.path),
        file_name.as_deref(),
        &mime_type,
        &bytes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("elephant-muya-{name}-{nonce}"))
    }

    #[test]
    fn writes_images_only_inside_hidden_assets() {
        let root = temporary_directory("asset");
        fs::create_dir_all(&root).unwrap();
        let result = write_asset(
            &root,
            Some("../unsafe name.png"),
            "image/png",
            &[1, 2, 3, 4],
        )
        .unwrap();
        assert!(result.path.starts_with(".assets/"));
        assert!(!result.path.contains(".."));
        assert_eq!(fs::read(result.full_path).unwrap(), vec![1, 2, 3, 4]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn rejects_unsupported_or_oversized_assets() {
        let root = temporary_directory("reject");
        fs::create_dir_all(&root).unwrap();
        assert!(write_asset(&root, None, "application/octet-stream", &[1]).is_err());
        assert!(write_asset(&root, None, "image/png", &[]).is_err());
        fs::remove_dir_all(root).ok();
    }
}

