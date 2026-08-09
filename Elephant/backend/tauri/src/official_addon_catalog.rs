// Kept as a thin module entrypoint so package transport, source validation and tests remain reviewable.
// Public Tauri commands implemented by install.rs:
// - tauri_official_addons_catalog_list
// - tauri_official_addons_catalog_install
include!("official_addon_catalog/catalog.rs");
include!("official_addon_catalog/source.rs");
include!("official_addon_catalog/install.rs");
include!("official_addon_catalog/tests.rs");
