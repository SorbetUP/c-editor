//! Resolved values copied from the active Elephant shell theme tokens.
//!
//! The light shell's inline `activeThemeTokens` from
//! `Elephant/shared/appearance.js` wins over the earlier CSS variables in
//! `app-shell.css`; Playwright's light capture is the runtime cross-check.

use freya::prelude::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub bg: (u8, u8, u8, u8),
    pub surface: (u8, u8, u8, u8),
    pub sidebar: (u8, u8, u8, u8),
    pub soft: (u8, u8, u8, u8),
    pub border: (u8, u8, u8, u8),
    pub border_strong: (u8, u8, u8, u8),
    pub text: (u8, u8, u8, u8),
    pub muted: (u8, u8, u8, u8),
    pub primary: (u8, u8, u8, u8),
    pub danger: (u8, u8, u8, u8),
}

impl ThemePalette {
    pub const fn new(
        bg: (u8, u8, u8, u8),
        surface: (u8, u8, u8, u8),
        sidebar: (u8, u8, u8, u8),
        soft: (u8, u8, u8, u8),
        border: (u8, u8, u8, u8),
        border_strong: (u8, u8, u8, u8),
        text: (u8, u8, u8, u8),
        muted: (u8, u8, u8, u8),
        primary: (u8, u8, u8, u8),
        danger: (u8, u8, u8, u8),
    ) -> Self {
        Self {
            bg,
            surface,
            sidebar,
            soft,
            border,
            border_strong,
            text,
            muted,
            primary,
            danger,
        }
    }
}

pub const LIGHT_PALETTE: ThemePalette = ThemePalette::new(
    (247, 249, 252, 255),
    (255, 255, 255, 255),
    (237, 242, 247, 255),
    (233, 239, 247, 255),
    (197, 207, 221, 255),
    (174, 186, 205, 255),
    (16, 24, 40, 255),
    (71, 84, 103, 255),
    (37, 99, 235, 255),
    (220, 38, 38, 255),
);

pub const DARK_PALETTE: ThemePalette = ThemePalette::new(
    (15, 20, 29, 255),
    (20, 26, 36, 255),
    (16, 23, 34, 255),
    (27, 36, 50, 255),
    (40, 50, 68, 255),
    (58, 70, 90, 255),
    (238, 243, 251, 255),
    (152, 163, 182, 255),
    (94, 161, 255, 255),
    (255, 107, 122, 255),
);

const APPLE_DARK: ThemePalette = ThemePalette::new(
    (29, 29, 31, 255),
    (44, 44, 46, 255),
    (25, 25, 27, 255),
    (58, 58, 60, 255),
    (63, 63, 70, 255),
    (99, 99, 102, 255),
    (245, 245, 247, 255),
    (161, 161, 170, 255),
    (10, 132, 255, 255),
    (255, 69, 58, 255),
);
const APPLE_LIGHT: ThemePalette = ThemePalette::new(
    (245, 245, 247, 255),
    (255, 255, 255, 255),
    (238, 238, 239, 255),
    (232, 232, 237, 255),
    (209, 209, 214, 255),
    (174, 174, 178, 255),
    (29, 29, 31, 255),
    (81, 81, 84, 255),
    (0, 122, 255, 255),
    (255, 59, 48, 255),
);
const GRAPHITE_DARK: ThemePalette = ThemePalette::new(
    (17, 17, 19, 255),
    (24, 24, 27, 255),
    (13, 13, 15, 255),
    (36, 36, 40, 255),
    (47, 47, 53, 255),
    (82, 82, 91, 255),
    (244, 244, 245, 255),
    (161, 161, 170, 255),
    (161, 161, 170, 255),
    (248, 113, 113, 255),
);
const GRAPHITE_LIGHT: ThemePalette = ThemePalette::new(
    (244, 244, 245, 255),
    (255, 255, 255, 255),
    (228, 228, 231, 255),
    (231, 231, 234, 255),
    (201, 201, 207, 255),
    (161, 161, 170, 255),
    (24, 24, 27, 255),
    (82, 82, 91, 255),
    (82, 82, 91, 255),
    (220, 38, 38, 255),
);
const NORD_DARK: ThemePalette = ThemePalette::new(
    (46, 52, 64, 255),
    (59, 66, 82, 255),
    (37, 43, 53, 255),
    (67, 76, 94, 255),
    (76, 86, 106, 255),
    (96, 112, 137, 255),
    (236, 239, 244, 255),
    (216, 222, 233, 255),
    (136, 192, 208, 255),
    (191, 97, 106, 255),
);
const NORD_LIGHT: ThemePalette = ThemePalette::new(
    (236, 239, 244, 255),
    (255, 255, 255, 255),
    (229, 233, 240, 255),
    (223, 229, 238, 255),
    (196, 204, 218, 255),
    (154, 168, 189, 255),
    (46, 52, 64, 255),
    (76, 86, 106, 255),
    (94, 129, 172, 255),
    (191, 97, 106, 255),
);
const SOLAR_DARK: ThemePalette = ThemePalette::new(
    (28, 25, 23, 255),
    (41, 37, 36, 255),
    (22, 19, 17, 255),
    (51, 43, 38, 255),
    (81, 68, 59, 255),
    (120, 101, 86, 255),
    (254, 243, 199, 255),
    (214, 185, 140, 255),
    (245, 158, 11, 255),
    (251, 113, 133, 255),
);
const SOLAR_LIGHT: ThemePalette = ThemePalette::new(
    (255, 247, 237, 255),
    (255, 251, 245, 255),
    (255, 237, 213, 255),
    (254, 223, 189, 255),
    (243, 201, 156, 255),
    (214, 154, 99, 255),
    (59, 36, 21, 255),
    (124, 79, 45, 255),
    (217, 119, 6, 255),
    (220, 38, 38, 255),
);
const FOREST_DARK: ThemePalette = ThemePalette::new(
    (16, 24, 19, 255),
    (23, 35, 27, 255),
    (12, 19, 15, 255),
    (32, 48, 37, 255),
    (49, 81, 60, 255),
    (75, 117, 90, 255),
    (238, 248, 239, 255),
    (168, 199, 173, 255),
    (74, 222, 128, 255),
    (251, 113, 133, 255),
);
const FOREST_LIGHT: ThemePalette = ThemePalette::new(
    (244, 247, 242, 255),
    (255, 255, 255, 255),
    (231, 239, 226, 255),
    (220, 232, 213, 255),
    (189, 206, 180, 255),
    (140, 171, 124, 255),
    (23, 32, 22, 255),
    (63, 95, 58, 255),
    (21, 128, 61, 255),
    (220, 38, 38, 255),
);
const BEIGE_DARK: ThemePalette = ThemePalette::new(
    (33, 28, 24, 255),
    (43, 37, 32, 255),
    (25, 21, 18, 255),
    (56, 47, 40, 255),
    (83, 70, 59, 255),
    (117, 98, 82, 255),
    (246, 234, 220, 255),
    (197, 173, 152, 255),
    (216, 160, 112, 255),
    (242, 139, 130, 255),
);
const BEIGE_LIGHT: ThemePalette = ThemePalette::new(
    (245, 239, 228, 255),
    (255, 250, 241, 255),
    (235, 225, 210, 255),
    (234, 220, 202, 255),
    (213, 194, 170, 255),
    (184, 157, 125, 255),
    (52, 40, 32, 255),
    (114, 93, 77, 255),
    (164, 91, 63, 255),
    (194, 65, 59, 255),
);
const PASTEL_DARK: ThemePalette = ThemePalette::new(
    (33, 27, 53, 255),
    (43, 36, 66, 255),
    (24, 19, 41, 255),
    (57, 48, 80, 255),
    (81, 69, 107, 255),
    (116, 99, 143, 255),
    (247, 240, 255, 255),
    (202, 190, 224, 255),
    (196, 181, 253, 255),
    (251, 142, 171, 255),
);
const PASTEL_LIGHT: ThemePalette = ThemePalette::new(
    (248, 245, 255, 255),
    (255, 254, 254, 255),
    (240, 235, 255, 255),
    (238, 232, 251, 255),
    (216, 207, 234, 255),
    (184, 169, 212, 255),
    (48, 41, 67, 255),
    (111, 100, 132, 255),
    (139, 92, 246, 255),
    (229, 107, 143, 255),
);
const GAMER_VIOLET_DARK: ThemePalette = ThemePalette::new(
    (12, 7, 22, 255),
    (22, 13, 37, 255),
    (9, 5, 16, 255),
    (33, 18, 52, 255),
    (58, 30, 82, 255),
    (102, 53, 139, 255),
    (248, 239, 255, 255),
    (196, 167, 216, 255),
    (168, 85, 247, 255),
    (251, 113, 133, 255),
);
const GAMER_VIOLET_LIGHT: ThemePalette = ThemePalette::new(
    (247, 242, 255, 255),
    (255, 255, 255, 255),
    (238, 228, 255, 255),
    (233, 221, 255, 255),
    (204, 181, 239, 255),
    (169, 134, 212, 255),
    (38, 21, 59, 255),
    (101, 79, 127, 255),
    (147, 51, 234, 255),
    (225, 29, 72, 255),
);

pub fn palette_for(theme_id: &str) -> ThemePalette {
    match theme_id {
        "dark" => DARK_PALETTE,
        "apple-dark" => APPLE_DARK,
        "apple-light" => APPLE_LIGHT,
        "graphite-dark" => GRAPHITE_DARK,
        "graphite-light" => GRAPHITE_LIGHT,
        "nord-dark" => NORD_DARK,
        "nord-light" => NORD_LIGHT,
        "solar-dark" => SOLAR_DARK,
        "solar-light" => SOLAR_LIGHT,
        "forest-dark" => FOREST_DARK,
        "forest-light" => FOREST_LIGHT,
        "beige-dark" => BEIGE_DARK,
        "beige-light" => BEIGE_LIGHT,
        "pastel-dark" => PASTEL_DARK,
        "pastel-light" => PASTEL_LIGHT,
        "gamer-violet-dark" => GAMER_VIOLET_DARK,
        "gamer-violet-light" => GAMER_VIOLET_LIGHT,
        _ => LIGHT_PALETTE,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeToken {
    Bg,
    Surface,
    Sidebar,
    Soft,
    Border,
    BorderStrong,
    Text,
    Muted,
    Primary,
    Danger,
}

pub fn token_color(palette: ThemePalette, token: ThemeToken) -> Color {
    color(match token {
        ThemeToken::Bg => palette.bg,
        ThemeToken::Surface => palette.surface,
        ThemeToken::Sidebar => palette.sidebar,
        ThemeToken::Soft => palette.soft,
        ThemeToken::Border => palette.border,
        ThemeToken::BorderStrong => palette.border_strong,
        ThemeToken::Text => palette.text,
        ThemeToken::Muted => palette.muted,
        ThemeToken::Primary => palette.primary,
        ThemeToken::Danger => palette.danger,
    })
}

// Legacy consumers still read these constants directly.  Keep their startup
// palette aligned with `ELEPHANTNOTE_THEME_TOKENS.light` until their owners
// accept the dynamic palette; dark runtime propagation remains owner-scoped.
pub const BG: (u8, u8, u8, u8) = (247, 249, 252, 255);
pub const SURFACE: (u8, u8, u8, u8) = (255, 255, 255, 255);
pub const SOFT: (u8, u8, u8, u8) = (233, 239, 247, 255);
pub const BORDER: (u8, u8, u8, u8) = (197, 207, 221, 255);
pub const BORDER_STRONG: (u8, u8, u8, u8) = (174, 186, 205, 255);
pub const TEXT: (u8, u8, u8, u8) = (16, 24, 40, 255);
pub const MUTED: (u8, u8, u8, u8) = (71, 84, 103, 255);
pub const PRIMARY: (u8, u8, u8, u8) = (37, 99, 235, 255);
pub const DANGER: (u8, u8, u8, u8) = (220, 38, 38, 255);

/// Source runtime fix: `.en-topstrip { height: 28px !important; }`.
pub const TOPBAR_HEIGHT: f32 = 28.;
pub const TOPBAR_NAV_BUTTON_SIZE: f32 = 24.;
pub const TOPBAR_NAV_TOP: f32 = 4.;
/// Source: `app-shell.css:362` `.en-rail { width: 56px !important; }`.
/// The Playwright light capture places the rail/sidebar boundary at 56 px;
/// `SIDEBAR_DEFAULT_WIDTH` remains the separate 232 px runtime contract.
pub const RAIL_WIDTH: f32 = 56.;
pub const RAIL_ACTION_SIZE: f32 = 34.;
pub const RAIL_GAP: f32 = 2.;
pub const RAIL_PADDING_TOP_DESKTOP: f32 = 8.;
pub const RAIL_PADDING_TOP_MACOS: f32 = 36.;
pub const RAIL_BOTTOM_PADDING_TOP: f32 = 6.;
pub const RAIL_BOTTOM_PADDING_BOTTOM: f32 = 8.;
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 232.;
pub const SIDEBAR_RESIZER_WIDTH: f32 = 12.;
pub const SIDEBAR_SCROLL_PADDING_TOP: f32 = 8.;
pub const SIDEBAR_ALL_NOTES_HEIGHT: f32 = 38.;
pub const CARD_HEIGHT: f32 = 176.;
pub const LIST_CARD_HEIGHT: f32 = 58.;

pub fn color(value: (u8, u8, u8, u8)) -> Color {
    Color::from_rgb(value.0, value.1, value.2)
}

pub fn mix(
    foreground: (u8, u8, u8, u8),
    background: (u8, u8, u8, u8),
    weight: f32,
) -> (u8, u8, u8, u8) {
    let weight = weight.clamp(0., 1.);
    let channel = |front: u8, back: u8| {
        (f32::from(front) * weight + f32::from(back) * (1. - weight)).round() as u8
    };
    (
        channel(foreground.0, background.0),
        channel(foreground.1, background.1),
        channel(foreground.2, background.2),
        channel(foreground.3, background.3),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_light_palette_uses_inline_sidebar_token() {
        assert_eq!(palette_for("light").bg, (247, 249, 252, 255));
        assert_eq!(palette_for("light").sidebar, (237, 242, 247, 255));
    }

    #[test]
    fn legacy_constants_match_light_startup_palette() {
        assert_eq!(BG, LIGHT_PALETTE.bg);
        assert_eq!(SURFACE, LIGHT_PALETTE.surface);
        assert_eq!(SOFT, LIGHT_PALETTE.soft);
        assert_eq!(BORDER, LIGHT_PALETTE.border);
        assert_eq!(BORDER_STRONG, LIGHT_PALETTE.border_strong);
        assert_eq!(TEXT, LIGHT_PALETTE.text);
        assert_eq!(MUTED, LIGHT_PALETTE.muted);
        assert_eq!(PRIMARY, LIGHT_PALETTE.primary);
        assert_eq!(DANGER, LIGHT_PALETTE.danger);
    }
}
