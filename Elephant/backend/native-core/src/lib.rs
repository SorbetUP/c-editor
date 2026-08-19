//! Shared pure-Rust backend used by Elephant native hosts.
//!
//! During the migration the canonical source files still live under the Tauri
//! backend tree, but Freya no longer reaches into that tree itself. This crate
//! is the single compilation boundary for filesystem/metadata/FTS business
//! logic and can be moved physically later without changing either host API.

#[path = "../../tauri/src/vault_layout.rs"]
pub mod vault_layout;

#[path = "../../tauri/src/fts.rs"]
pub mod fts;

pub mod vault;
