//! Shared pure-Rust backend used by Elephant native hosts.
//!
//! Filesystem, metadata and FTS business logic live physically in this crate.
//! Host crates consume this API instead of source-including each other's files.

pub mod vault_layout;
pub mod fts;
pub mod vault;
