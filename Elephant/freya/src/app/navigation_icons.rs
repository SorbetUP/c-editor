//! Small SVG icons matching the Lucide paths used by the Vue shell.
//!
//! The Vue source renders these as `@lucide/vue` SVGs. `SvgViewer` keeps the
//! same stroke/fill semantics in Freya without a font-glyph fallback.

use freya::{
    components::SvgViewer,
    prelude::{Color, ContainerSizeExt, Element, IntoElement, Size},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Icon {
    Search,
    PanelLeftClose,
    PanelLeftOpen,
    Vault,
    Settings,
    ChevronLeft,
    ChevronRight,
    Inbox,
    MoreHorizontal,
    ArrowDownNarrowWide,
    ArrowUpNarrowWide,
    ArrowDownAz,
    ArrowDownZa,
    Grid3x3,
    List,
    Plus,
    Pin,
    Folder,
    FileText,
    X,
}

const SEARCH: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21 21-4.34-4.34"/><circle cx="11" cy="11" r="8"/></svg>"#;
const PANEL_LEFT_CLOSE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m16 15-3-3 3-3"/></svg>"#;
const PANEL_LEFT_OPEN: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m14 9 3 3-3 3"/></svg>"#;
const VAULT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="7.5" cy="7.5" r=".5" fill="currentColor"/><path d="m7.9 7.9 2.7 2.7"/><circle cx="16.5" cy="7.5" r=".5" fill="currentColor"/><path d="m13.4 10.6 2.7-2.7"/><circle cx="7.5" cy="16.5" r=".5" fill="currentColor"/><path d="m7.9 16.1 2.7-2.7"/><circle cx="16.5" cy="16.5" r=".5" fill="currentColor"/><path d="m13.4 13.4 2.7 2.7"/><circle cx="12" cy="12" r="2"/></svg>"#;
const SETTINGS: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/><circle cx="12" cy="12" r="3"/></svg>"#;
const CHEVRON_LEFT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>"#;
const CHEVRON_RIGHT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>"#;
const INBOX: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></svg>"#;
const MORE_HORIZONTAL: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>"#;
const ARROW_DOWN_NARROW_WIDE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 15 3 3 3-3"/><path d="M10 6v12"/><path d="M17 6h4"/><path d="M17 10h4"/><path d="M17 14h4"/><path d="M17 18h4"/></svg>"#;
const ARROW_UP_NARROW_WIDE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 9 3-3 3 3"/><path d="M10 6v12"/><path d="M17 6h4"/><path d="M17 10h4"/><path d="M17 14h4"/><path d="M17 18h4"/></svg>"#;
const ARROW_DOWN_AZ: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 16 3 3 3-3"/><path d="M6 19V5"/><path d="M11 6h5"/><path d="M11 10h4"/><path d="M11 14h3"/><path d="M11 18h2"/></svg>"#;
const ARROW_DOWN_ZA: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 16 3 3 3-3"/><path d="M6 19V5"/><path d="M11 6h2"/><path d="M11 10h3"/><path d="M11 14h4"/><path d="M11 18h5"/></svg>"#;
const GRID_3X3: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M3 15h18"/><path d="M9 3v18"/><path d="M15 3v18"/></svg>"#;
const LIST: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h.01"/><path d="M3 18h.01"/><path d="M3 6h.01"/><path d="M8 12h13"/><path d="M8 18h13"/><path d="M8 6h13"/></svg>"#;
const PLUS: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>"#;
const PIN: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/></svg>"#;
const FOLDER: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/></svg>"#;
const FILE_TEXT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v6h6"/><path d="M8 13h8"/><path d="M8 17h8"/><path d="M8 9h2"/></svg>"#;
const X: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#;

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
        Icon::Search => SEARCH,
        Icon::PanelLeftClose => PANEL_LEFT_CLOSE,
        Icon::PanelLeftOpen => PANEL_LEFT_OPEN,
        Icon::Vault => VAULT,
        Icon::Settings => SETTINGS,
        Icon::ChevronLeft => CHEVRON_LEFT,
        Icon::ChevronRight => CHEVRON_RIGHT,
        Icon::Inbox => INBOX,
        Icon::MoreHorizontal => MORE_HORIZONTAL,
        Icon::ArrowDownNarrowWide => ARROW_DOWN_NARROW_WIDE,
        Icon::ArrowUpNarrowWide => ARROW_UP_NARROW_WIDE,
        Icon::ArrowDownAz => ARROW_DOWN_AZ,
        Icon::ArrowDownZa => ARROW_DOWN_ZA,
        Icon::Grid3x3 => GRID_3X3,
        Icon::List => LIST,
        Icon::Plus => PLUS,
        Icon::Pin => PIN,
        Icon::Folder => FOLDER,
        Icon::FileText => FILE_TEXT,
        Icon::X => X,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lucide_sources_keep_the_expected_svg_contract() {
        for icon in [
            Icon::Search,
            Icon::PanelLeftClose,
            Icon::PanelLeftOpen,
            Icon::Vault,
            Icon::Settings,
            Icon::ChevronLeft,
            Icon::ChevronRight,
            Icon::Inbox,
            Icon::MoreHorizontal,
            Icon::ArrowDownNarrowWide,
            Icon::ArrowUpNarrowWide,
            Icon::ArrowDownAz,
            Icon::ArrowDownZa,
            Icon::Grid3x3,
            Icon::List,
            Icon::Plus,
            Icon::Pin,
            Icon::Folder,
            Icon::FileText,
            Icon::X,
        ] {
            let svg = std::str::from_utf8(source(icon)).expect("Lucide source is UTF-8");
            assert!(svg.contains("viewBox=\"0 0 24 24\""));
            assert!(svg.contains("stroke=\"currentColor\""));
            assert!(svg.contains("stroke-linecap=\"round\""));
        }
    }
}
