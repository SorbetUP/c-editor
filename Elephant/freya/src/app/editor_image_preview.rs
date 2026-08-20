use base64::{engine::general_purpose::STANDARD, Engine as _};
use freya::prelude::*;
use std::path::{Path, PathBuf};

use crate::theme;

pub(super) fn render(
    note_path: Option<&Path>,
    source: &str,
    alt: &str,
    palette: theme::ThemePalette,
) -> Element {
    match image_data_url(note_path, source) {
        Ok(Some(data_url)) => {
            let key = format!(
                "note-image:{}",
                blake3::hash(data_url.as_bytes()).to_hex()
            );
            let escaped = escape_xml(&data_url);
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 960 540\"><image href=\"{escaped}\" x=\"0\" y=\"0\" width=\"960\" height=\"540\" preserveAspectRatio=\"xMidYMid meet\"/></svg>"
            );
            rect()
                .width(Size::fill())
                .max_width(Size::px(720.))
                .height(Size::px(360.))
                .background(theme::color(palette.soft))
                .border(Border::new().fill(theme::color(palette.border)).width(1.))
                .with_corner_radius(8.)
                .a11y_alt(format!("Image preview {alt}"))
                .child(
                    SvgViewer::new((key, Bytes::from(svg.into_bytes())))
                        .width(Size::fill())
                        .height(Size::fill())
                        .show_loader(false),
                )
                .into_element()
        }
        Ok(None) => fallback(source, alt, "Remote image", palette),
        Err(error) => fallback(source, alt, &error, palette),
    }
}

fn fallback(
    source: &str,
    alt: &str,
    reason: &str,
    palette: theme::ThemePalette,
) -> Element {
    rect()
        .width(Size::fill())
        .max_width(Size::px(720.))
        .padding(Gaps::new_all(10.))
        .spacing(4.)
        .background(theme::color(palette.soft))
        .border(Border::new().fill(theme::color(palette.border)).width(1.))
        .with_corner_radius(8.)
        .a11y_alt(format!("Image preview unavailable {alt}"))
        .child(
            label()
                .font_size(13.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(palette.text))
                .text(if alt.is_empty() { "Image" } else { alt }.to_owned()),
        )
        .child(
            label()
                .font_size(11.)
                .color(theme::color(palette.muted))
                .text(format!("{reason}: {source}")),
        )
        .into_element()
}

fn image_data_url(note_path: Option<&Path>, source: &str) -> Result<Option<String>, String> {
    let source = source.trim();
    if source.starts_with("data:image/") {
        return Ok(Some(source.to_owned()));
    }
    if source.starts_with("http://") || source.starts_with("https://") {
        return Ok(None);
    }
    if source.is_empty() {
        return Err("Image source is empty".to_owned());
    }

    let source_path = PathBuf::from(source);
    let path = if source_path.is_absolute() {
        source_path
    } else {
        note_path
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."))
            .join(source_path)
    };
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("Unable to read image {}: {error}", path.display()))?;
    let mime = mime_type(&path).ok_or_else(|| {
        format!(
            "Unsupported image type: {}",
            path.extension()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
        )
    })?;
    Ok(Some(format!(
        "data:{mime};base64,{}",
        STANDARD.encode(bytes)
    )))
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

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn relative_local_image_becomes_embedded_data_url() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-note-image-{stamp}"));
        fs::create_dir_all(root.join(".assets")).unwrap();
        let note = root.join("Alpha.md");
        fs::write(&note, "note").unwrap();
        fs::write(root.join(".assets/pixel.png"), b"png-bytes").unwrap();

        let data = image_data_url(Some(&note), ".assets/pixel.png")
            .unwrap()
            .unwrap();
        assert!(data.starts_with("data:image/png;base64,"));
        assert!(!data.contains(".assets"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn remote_images_are_not_blockingly_downloaded_by_the_editor_renderer() {
        assert_eq!(
            image_data_url(None, "https://example.com/image.png").unwrap(),
            None
        );
    }
}
