#[path = "../src/app/drawing_image.rs"]
mod drawing_image;

use std::path::Path;

#[test]
fn png_dimensions_and_data_url_are_preserved() {
    let mut bytes = vec![0u8; 24];
    bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes[16..20].copy_from_slice(&1920u32.to_be_bytes());
    bytes[20..24].copy_from_slice(&1080u32.to_be_bytes());

    let asset = drawing_image::asset_from_bytes(Path::new("photo.png"), &bytes).unwrap();
    assert_eq!(asset.mime_type, "image/png");
    assert!(asset.data_url.starts_with("data:image/png;base64,"));
    assert!(asset.file_id.starts_with("image-"));
    assert_eq!((asset.width, asset.height), (720.0, 405.0));
}

#[test]
fn gif_dimensions_are_read_without_decoding_pixels() {
    let mut bytes = b"GIF89a".to_vec();
    bytes.extend_from_slice(&320u16.to_le_bytes());
    bytes.extend_from_slice(&200u16.to_le_bytes());
    let asset = drawing_image::asset_from_bytes(Path::new("image.gif"), &bytes).unwrap();
    assert_eq!((asset.width, asset.height), (320.0, 200.0));
    assert_eq!(asset.mime_type, "image/gif");
}

#[test]
fn unsupported_and_empty_files_are_rejected() {
    assert!(drawing_image::asset_from_bytes(Path::new("x.png"), &[]).is_err());
    assert!(drawing_image::asset_from_bytes(Path::new("x.txt"), b"not an image").is_err());
}

#[test]
fn unknown_dimensions_use_a_safe_canvas_size_but_keep_real_bytes() {
    let asset = drawing_image::asset_from_bytes(Path::new("vector.svg"), b"<svg></svg>").unwrap();
    assert_eq!((asset.width, asset.height), (320.0, 180.0));
    assert_eq!(asset.mime_type, "image/svg+xml");
    assert!(asset.data_url.contains("PHN2Zz48L3N2Zz4="));
}
