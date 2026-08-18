//! Desktop entry point for the native Freya shell.
//!
//! Keeping the launcher in this crate makes `pnpm freya:dev` exercise the
//! same production app component that the Freya acceptance tests mount.

use elephant_freya::app;
use freya::prelude::*;

#[cfg(target_os = "macos")]
use freya::winit::platform::macos::WindowAttributesExtMacOS;

/// Freya 0.4.x installs a blocking native `Fatal Error` message dialog for
/// release-mode panics. On macOS that dialog can outlive the terminal process
/// after Ctrl+C because the panic hook is blocked inside the OS modal alert.
/// Keep terminal diagnostics and make fatal failures terminate immediately.
fn install_non_blocking_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("[freya][fatal] {panic_info}");
        eprintln!("{}", std::backtrace::Backtrace::force_capture());
        std::process::exit(1);
    }));
}

fn main() {
    eprintln!("[freya][runtime] action=launch runtime=freya");
    let window = WindowConfig::new(app::app)
        .with_title("ElephantNote · Freya")
        .with_size(1280., 840.)
        .with_min_size(720., 480.)
        .with_icon(LaunchConfig::window_icon(include_bytes!(
            "../../assets/static/icon.png"
        )))
        .with_transparency(true)
        .with_window_attributes(|attributes, _| {
            #[cfg(target_os = "macos")]
            {
                return attributes
                    .with_titlebar_transparent(true)
                    .with_title_hidden(true)
                    .with_fullsize_content_view(true);
            }
            #[cfg(not(target_os = "macos"))]
            attributes
        });
    launch(
        LaunchConfig::new()
            .with_future(|_| {
                install_non_blocking_panic_hook();
                async {}
            })
            .with_window(window),
    );
}
