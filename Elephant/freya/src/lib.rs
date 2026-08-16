mod addon_adapter;
mod addon_packages;
pub mod addon_runtime;
mod addon_worker;
pub mod app;
pub mod canvas_contract;
pub mod canvas_runtime;
pub mod editor;
pub mod library_contract;
mod markdown_tags;
pub mod navigation_contract;
mod pi_adapter;
pub mod search_graph_contract;
pub mod settings_contract;
pub mod source_contracts;
mod sync_adapter;
pub mod theme;
pub mod vault_adapter;
mod vault_registry;

#[path = "../../backend/tauri/src/vault_layout.rs"]
pub mod vault_layout;
