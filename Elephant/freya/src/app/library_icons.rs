//! Library-local SVG icons matching the Lucide assets used by the Vue library.
//!
//! Keep this module independent from `navigation_icons.rs`: navigation is owned
//! by a different migration lot, while the library still needs the same
//! vector-icon contract instead of font glyphs.

use freya::{
    components::SvgViewer,
    prelude::{Color, ContainerSizeExt, Element, IntoElement, Size},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Icon {
    Plus,
    ArrowDownNarrowWide,
    ArrowUpNarrowWide,
    ArrowDownAz,
    ArrowDownZa,
    Grid3x3,
    List,
    FilePlus2,
    FolderPlus,
    FileText,
    Folder,
    PenLine,
    MoreHorizontal,
    Pencil,
    Trash2,
    PanelLeftOpen,
    PanelLeftClose,
    Pin,
    Excalidraw,
}

const PLUS: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>"#;
const ARROW_DOWN_NARROW_WIDE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 15 3 3 3-3"/><path d="M10 6v12"/><path d="M18 17V6"/><path d="M17 6h-1"/><path d="M18 10h-2"/><path d="M18 14h-3"/><path d="M18 18h-4"/></svg>"#;
const ARROW_UP_NARROW_WIDE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 9 3-3 3 3"/><path d="M10 6v12"/><path d="M18 17V6"/><path d="M17 6h-1"/><path d="M18 10h-2"/><path d="M18 14h-3"/><path d="M18 18h-4"/></svg>"#;
const ARROW_DOWN_AZ: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 16 4 4 4-4"/><path d="M7 20V4"/><path d="M20 8h-5"/><path d="M15 10V6.5A2.5 2.5 0 0 1 17.5 4 2.5 2.5 0 0 1 20 6.5V10"/><path d="M15 14h5l-5 6h5"/></svg>"#;
const ARROW_DOWN_ZA: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 16 4 4 4-4"/><path d="M7 20V4"/><path d="M15 4h5l-5 6h5"/><path d="M20 20v-3.5a2.5 2.5 0 0 0-5 0V20"/><path d="M15 18h5"/></svg>"#;
const GRID_3X3: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M3 15h18"/><path d="M9 3v18"/><path d="M15 3v18"/></svg>"#;
const LIST: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/><path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/></svg>"#;
const FILE_PLUS_2: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><path d="M12 18v-6"/><path d="M9 15h6"/></svg>"#;
const FOLDER_PLUS: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 10v6"/><path d="M9 13h6"/><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"#;
const FILE_TEXT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><path d="M8 13h8"/><path d="M8 17h8"/><path d="M8 9h2"/></svg>"#;
const FOLDER: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"#;
const PEN_LINE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>"#;
const MORE_HORIZONTAL: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="1" fill="currentColor"/><circle cx="19" cy="12" r="1" fill="currentColor"/><circle cx="5" cy="12" r="1" fill="currentColor"/></svg>"#;
const PENCIL: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>"#;
const TRASH_2: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>"#;
const PANEL_LEFT_OPEN: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m14 9-3 3 3 3"/></svg>"#;
const PANEL_LEFT_CLOSE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m14 15 3-3-3-3"/></svg>"#;
const PIN: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/></svg>"#;
const EXCALIDRAW: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="18" fill="#6C63FF"/><path d="M20 44c7.5-15.5 15.5-23.5 24-24" fill="none" stroke="#fff" stroke-width="5" stroke-linecap="round"/><path d="M18 46l8-2-6-6-2 8z" fill="#fff"/><path d="M42 18l4 4" stroke="#fff" stroke-width="5" stroke-linecap="round"/><circle cx="22" cy="22" r="4" fill="#fff" opacity=".85"/><path d="M42 42h7" stroke="#fff" stroke-width="4" stroke-linecap="round" opacity=".85"/></svg>"##;

pub(super) fn svg_icon(icon: Icon, color: Color, size: f32) -> Element {
    SvgViewer::new(source(icon))
        .width(Size::px(size))
        .height(Size::px(size))
        .show_loader(false)
        .color(color)
        .stroke(color)
        .stroke_width(2.)
        .into_element()
}

fn source(icon: Icon) -> &'static [u8] {
    match icon {
        Icon::Plus => PLUS,
        Icon::ArrowDownNarrowWide => ARROW_DOWN_NARROW_WIDE,
        Icon::ArrowUpNarrowWide => ARROW_UP_NARROW_WIDE,
        Icon::ArrowDownAz => ARROW_DOWN_AZ,
        Icon::ArrowDownZa => ARROW_DOWN_ZA,
        Icon::Grid3x3 => GRID_3X3,
        Icon::List => LIST,
        Icon::FilePlus2 => FILE_PLUS_2,
        Icon::FolderPlus => FOLDER_PLUS,
        Icon::FileText => FILE_TEXT,
        Icon::Folder => FOLDER,
        Icon::PenLine => PEN_LINE,
        Icon::MoreHorizontal => MORE_HORIZONTAL,
        Icon::Pencil => PENCIL,
        Icon::Trash2 => TRASH_2,
        Icon::PanelLeftOpen => PANEL_LEFT_OPEN,
        Icon::PanelLeftClose => PANEL_LEFT_CLOSE,
        Icon::Pin => PIN,
        Icon::Excalidraw => EXCALIDRAW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_lucide_sources_are_vector_icons_not_font_glyphs() {
        for icon in [
            Icon::Plus,
            Icon::ArrowDownNarrowWide,
            Icon::ArrowUpNarrowWide,
            Icon::ArrowDownAz,
            Icon::ArrowDownZa,
            Icon::Grid3x3,
            Icon::List,
            Icon::FilePlus2,
            Icon::FolderPlus,
            Icon::FileText,
            Icon::Folder,
            Icon::PenLine,
            Icon::MoreHorizontal,
            Icon::Pencil,
            Icon::Trash2,
            Icon::PanelLeftOpen,
            Icon::PanelLeftClose,
            Icon::Pin,
        ] {
            let svg = std::str::from_utf8(source(icon)).expect("library icon SVG is UTF-8");
            assert!(svg.contains("viewBox=\"0 0 24 24\""));
            assert!(svg.contains("stroke=\"currentColor\""));
            assert!(svg.contains("stroke-linecap=\"round\""));
        }
    }

    #[test]
    fn excalidraw_icon_keeps_the_shared_asset_branding() {
        let svg = std::str::from_utf8(source(Icon::Excalidraw)).expect("Excalidraw SVG is UTF-8");
        assert!(svg.contains("viewBox=\"0 0 64 64\""));
        assert!(svg.contains("#6C63FF"));
        assert!(svg.contains("#fff"));
    }
}
