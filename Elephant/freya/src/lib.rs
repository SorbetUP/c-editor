mod addon_adapter;
pub mod app;
pub mod editor;
pub mod library_contract;
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
