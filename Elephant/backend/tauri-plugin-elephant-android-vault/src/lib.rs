use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(not(target_os = "android"))]
mod desktop;
mod error;
#[cfg(target_os = "android")]
mod mobile;
mod models;

pub use error::{Error, Result};
pub use models::*;

#[cfg(not(target_os = "android"))]
pub use desktop::ElephantAndroidVault;
#[cfg(target_os = "android")]
pub use mobile::ElephantAndroidVault;

pub trait ElephantAndroidVaultExt<R: Runtime> {
    fn elephant_android_vault(&self) -> &ElephantAndroidVault<R>;
}

impl<R: Runtime, T: Manager<R>> ElephantAndroidVaultExt<R> for T {
    fn elephant_android_vault(&self) -> &ElephantAndroidVault<R> {
        self.state::<ElephantAndroidVault<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("elephant-android-vault")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let vault = mobile::init(app, api)?;
            #[cfg(not(target_os = "android"))]
            let vault = desktop::init(app, api)?;
            app.manage(vault);
            Ok(())
        })
        .build()
}
