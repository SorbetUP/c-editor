use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_elephant_android_vault::{
    ElephantAndroidVaultExt, ShadowRequest, ShareTextRequest, TreeState,
};

fn shadow_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("vaults").join("AndroidExternal"))
        .map_err(|error| error.to_string())
}

fn request(app: &AppHandle) -> Result<ShadowRequest, String> {
    let path = shadow_path(app)?;
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(ShadowRequest {
        shadow_path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn tauri_android_vault_pick(app: AppHandle) -> Result<TreeState, String> {
    app.elephant_android_vault()
        .pick_tree(request(&app)?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn tauri_android_vault_restore(app: AppHandle) -> Result<TreeState, String> {
    app.elephant_android_vault()
        .restore(request(&app)?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn tauri_android_vault_sync(app: AppHandle) -> Result<TreeState, String> {
    app.elephant_android_vault()
        .sync_to_tree(request(&app)?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn tauri_android_vault_clear(app: AppHandle) -> Result<TreeState, String> {
    app.elephant_android_vault()
        .clear(request(&app)?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn tauri_android_share_text(app: AppHandle, title: String, text: String) -> Result<(), String> {
    app.elephant_android_vault()
        .share_text(ShareTextRequest { title, text })
        .map_err(|error| error.to_string())
}
