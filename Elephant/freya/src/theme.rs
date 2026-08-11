//! Resolved values copied from the active Elephant shell theme tokens.
//! Source: `Elephant/frontend/app/styles/app-shell.css:2-9,17-24`.

use freya::prelude::Color;

pub const BG: (u8, u8, u8, u8) = (15, 20, 29, 255);
pub const SURFACE: (u8, u8, u8, u8) = (20, 26, 36, 255);
pub const SOFT: (u8, u8, u8, u8) = (27, 36, 50, 255);
pub const BORDER: (u8, u8, u8, u8) = (40, 50, 68, 255);
pub const BORDER_STRONG: (u8, u8, u8, u8) = (58, 70, 90, 255);
pub const TEXT: (u8, u8, u8, u8) = (238, 243, 251, 255);
pub const MUTED: (u8, u8, u8, u8) = (152, 163, 182, 255);
pub const PRIMARY: (u8, u8, u8, u8) = (94, 161, 255, 255);
pub const DANGER: (u8, u8, u8, u8) = (255, 107, 122, 255);

pub const TOPBAR_HEIGHT: f32 = 32.;
pub const TOPBAR_NAV_BUTTON_SIZE: f32 = 24.;
pub const TOPBAR_NAV_TOP: f32 = 4.;
pub const RAIL_WIDTH: f32 = 48.;
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
