pub mod commands;
pub mod commands_contract;
pub mod muya_advanced;
pub mod muya_assets;
pub mod muya_clipboard;
pub mod muya_clipboard_commands;
pub mod muya_clipboard_html;
pub mod muya_compat;
pub mod muya_complete;
pub mod muya_deterministic;
pub mod muya_edges;
pub mod muya_engine;
pub mod muya_extras;
pub mod muya_frontmatter;
pub mod muya_inline;
pub mod muya_interactions;
pub mod muya_navigation;
pub mod muya_parity;
pub mod muya_session;
pub mod muya_state;
pub mod muya_surface;
pub mod muya_ui;
pub mod parser_v4;
pub mod renderer_v2;
pub mod types;

#[cfg(test)]
mod compat_extra_tests;

#[cfg(test)]
mod deterministic_parity_basic_tests;

#[cfg(test)]
mod deterministic_parity_render_tests;

#[cfg(test)]
mod editor_parity_tests;

#[cfg(test)]
mod muya_edge_parity_tests;

#[cfg(test)]
mod muya_snapshot_basic_tests;

#[cfg(test)]
mod muya_snapshot_render_tests;

#[cfg(test)]
mod parity_tests;

pub use parser_v4::parse_markdown_document;
pub use renderer_v2::{render_html, render_plain_text};
