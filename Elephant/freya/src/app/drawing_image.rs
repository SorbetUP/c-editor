use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingImageAsset {
    pub file_id: String,
    pub mime_type: String,
    pub data_url: String,
    pub width: f32,
    pub height: f32,
    pub natural_width: f32,
    pub natural_height: f32,
    pub created: u64,
}

pub fn asset_from_bytes(path: &Path, bytes: &[u8]) -> Result<DrawingImageAsset, String> {
    if bytes.is_empty() {
        return Err("Selected image is empty.".to_owned());
    }
    let mime_type = mime_type(path).ok_or_else(|| {
        format!(
            "Unsupported image type: {}",
            path.extension()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
        )
    })?;
    let (intrinsic_width, intrinsic_height) = image_dimensions(bytes).unwrap_or((320, 180));
    let (width, height) = fit_size(intrinsic_width, intrinsic_height, 720.0, 520.0);
    let hash = blake3::hash(bytes).to_hex().to_string();
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    Ok(DrawingImageAsset {
        file_id: format!("image-{hash}"),
        mime_type: mime_type.to_owned(),
        data_url: format!("data:{mime_type};base64,{}", STANDARD.encode(bytes)),
        width,
        height,
        natural_width: intrinsic_width.max(1) as f32,
        natural_height: intrinsic_height.max(1) as f32,
        created,
    })
}

#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn pick_image() -> Result<Option<DrawingImageAsset>, String> {
    let Some(path) = rfd::FileDialog::new()
        .set_title("Insert image")
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"])
        .pick_file()
    else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("Unable to read selected image {}: {error}", path.display()))?;
    asset_from_bytes(&path, &bytes).map(Some)
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub fn pick_image() -> Result<Option<DrawingImageAsset>, String> {
    Err("Native image picker is unavailable on this platform.".to_owned())
}

fn mime_type(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

fn fit_size(width: u32, height: u32, max_width: f32, max_height: f32) -> (f32, f32) {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    let scale = (max_width / width).min(max_height / height).min(1.0);
    ((width * scale).max(1.0), (height * scale).max(1.0))
}

fn image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    png_dimensions(bytes)
        .or_else(|| gif_dimensions(bytes))
        .or_else(|| bmp_dimensions(bytes))
        .or_else(|| jpeg_dimensions(bytes))
        .or_else(|| webp_dimensions(bytes))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    valid_dimensions(width, height)
}

fn gif_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 10 || (&bytes[..6] != b"GIF87a" && &bytes[..6] != b"GIF89a") {
        return None;
    }
    let width = u16::from_le_bytes(bytes[6..8].try_into().ok()?) as u32;
    let height = u16::from_le_bytes(bytes[8..10].try_into().ok()?) as u32;
    valid_dimensions(width, height)
}

fn bmp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 26 || &bytes[..2] != b"BM" {
        return None;
    }
    let width = i32::from_le_bytes(bytes[18..22].try_into().ok()?).unsigned_abs();
    let height = i32::from_le_bytes(bytes[22..26].try_into().ok()?).unsigned_abs();
    valid_dimensions(width, height)
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[..2] != [0xff, 0xd8] {
        return None;
    }
    let mut cursor = 2usize;
    while cursor + 4 <= bytes.len() {
        if bytes[cursor] != 0xff {
            cursor += 1;
            continue;
        }
        while cursor < bytes.len() && bytes[cursor] == 0xff {
            cursor += 1;
        }
        let marker = *bytes.get(cursor)?;
        cursor += 1;
        if matches!(marker, 0xd8 | 0xd9) {
            continue;
        }
        let length = u16::from_be_bytes([*bytes.get(cursor)?, *bytes.get(cursor + 1)?]) as usize;
        if length < 2 || cursor + length > bytes.len() {
            return None;
        }
        if matches!(
            marker,
            0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
        ) && length >= 7
        {
            let height = u16::from_be_bytes([bytes[cursor + 3], bytes[cursor + 4]]) as u32;
            let width = u16::from_be_bytes([bytes[cursor + 5], bytes[cursor + 6]]) as u32;
            return valid_dimensions(width, height);
        }
        cursor += length;
    }
    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    match &bytes[12..16] {
        b"VP8X" if bytes.len() >= 30 => {
            let width = 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]);
            let height = 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]);
            valid_dimensions(width, height)
        }
        _ => None,
    }
}

fn valid_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    (width > 0 && height > 0).then_some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_png_preserves_intrinsic_dimensions_while_fitting_display_size() {
        let mut png = vec![0_u8; 24];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[16..20].copy_from_slice(&2000_u32.to_be_bytes());
        png[20..24].copy_from_slice(&1000_u32.to_be_bytes());
        let asset = asset_from_bytes(Path::new("large.png"), &png).unwrap();
        assert_eq!((asset.natural_width, asset.natural_height), (2000.0, 1000.0));
        assert_eq!((asset.width, asset.height), (720.0, 360.0));
    }
}
