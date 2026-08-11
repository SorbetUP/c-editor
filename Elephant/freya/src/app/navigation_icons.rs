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
}

const SEARCH: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21 21-4.34-4.34"/><circle cx="11" cy="11" r="8"/></svg>"#;
const PANEL_LEFT_CLOSE: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m16 15-3-3 3-3"/></svg>"#;
const PANEL_LEFT_OPEN: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m14 9 3 3-3 3"/></svg>"#;
const VAULT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="7.5" cy="7.5" r=".5" fill="currentColor"/><path d="m7.9 7.9 2.7 2.7"/><circle cx="16.5" cy="7.5" r=".5" fill="currentColor"/><path d="m13.4 10.6 2.7-2.7"/><circle cx="7.5" cy="16.5" r=".5" fill="currentColor"/><path d="m7.9 16.1 2.7-2.7"/><circle cx="16.5" cy="16.5" r=".5" fill="currentColor"/><path d="m13.4 13.4 2.7 2.7"/><circle cx="12" cy="12" r="2"/></svg>"#;
const SETTINGS: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/><circle cx="12" cy="12" r="3"/></svg>"#;
const CHEVRON_LEFT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>"#;
const CHEVRON_RIGHT: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>"#;
const INBOX: &[u8] = br#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></svg>"#;

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
        ] {
            let svg = std::str::from_utf8(source(icon)).expect("Lucide source is UTF-8");
            assert!(svg.contains("viewBox=\"0 0 24 24\""));
            assert!(svg.contains("stroke=\"currentColor\""));
            assert!(svg.contains("stroke-linecap=\"round\""));
        }
    }
}
