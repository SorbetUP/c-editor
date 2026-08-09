use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{models::*, Error, Result};

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<ElephantAndroidVault<R>> {
    Ok(ElephantAndroidVault(std::marker::PhantomData))
}

pub struct ElephantAndroidVault<R: Runtime>(pub(crate) std::marker::PhantomData<fn() -> R>);

impl<R: Runtime> ElephantAndroidVault<R> {
    fn unsupported<T>(&self) -> Result<T> {
        Err(Error::Unsupported(
            "Android document-tree storage is unavailable on this platform.".into(),
        ))
    }

    pub fn pick_tree(&self, _payload: ShadowRequest) -> Result<TreeState> {
        self.unsupported()
    }
    pub fn restore(&self, _payload: ShadowRequest) -> Result<TreeState> {
        self.unsupported()
    }
    pub fn sync_to_tree(&self, _payload: ShadowRequest) -> Result<TreeState> {
        self.unsupported()
    }
    pub fn share_text(&self, _request: ShareTextRequest) -> Result<()> {
        self.unsupported()
    }

    pub fn clear(&self, _payload: ShadowRequest) -> Result<TreeState> {
        self.unsupported()
    }
}
