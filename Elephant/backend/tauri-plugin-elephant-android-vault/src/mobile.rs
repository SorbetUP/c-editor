use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{models::*, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.elephantnote.androidvault";

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<ElephantAndroidVault<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ElephantAndroidVaultPlugin")?;
    Ok(ElephantAndroidVault(handle))
}

pub struct ElephantAndroidVault<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> ElephantAndroidVault<R> {
    pub fn pick_tree(&self, payload: ShadowRequest) -> Result<TreeState> {
        self.0
            .run_mobile_plugin("pickTree", payload)
            .map_err(Into::into)
    }

    pub fn restore(&self, payload: ShadowRequest) -> Result<TreeState> {
        self.0
            .run_mobile_plugin("restore", payload)
            .map_err(Into::into)
    }

    pub fn sync_to_tree(&self, payload: ShadowRequest) -> Result<TreeState> {
        self.0
            .run_mobile_plugin("syncToTree", payload)
            .map_err(Into::into)
    }

    pub fn clear(&self, payload: ShadowRequest) -> Result<TreeState> {
        self.0
            .run_mobile_plugin("clear", payload)
            .map_err(Into::into)
    }

    pub fn share_text(&self, payload: ShareTextRequest) -> Result<()> {
        self.0
            .run_mobile_plugin("shareText", payload)
            .map_err(Into::into)
    }
}
