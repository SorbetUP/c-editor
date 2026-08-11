use elephant_freya::app::app;
use freya::prelude::*;

/// Freya 0.4.x installs a blocking native `Fatal Error` message dialog for
/// release-mode panics. On macOS that dialog can outlive the terminal process
/// after Ctrl+C because the panic hook is blocked inside the OS modal alert.
///
/// `LaunchConfig::with_future` factories are invoked by Freya after it installs
/// its release panic hook and before the event loop starts rendering windows.
/// Replacing the hook here therefore preserves terminal diagnostics while
/// making a fatal panic terminate immediately instead of leaving a stuck modal
/// dialog behind.
fn install_non_blocking_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("[freya][fatal] {panic_info}");
        eprintln!("{}", std::backtrace::Backtrace::force_capture());
        std::process::exit(1);
    }));
}

fn main() {
    eprintln!("[freya][lifecycle] action:start component=ElephantShell");
    launch(
        LaunchConfig::new()
            .with_future(|_| {
                // This closure body runs synchronously inside Freya's launch
                // path, after Freya's own release panic hook is installed.
                install_non_blocking_panic_hook();
                async {}
            })
            .with_window(
                WindowConfig::new(app)
                    .with_title("Elephant")
                    .with_size(1280., 840.)
                    .with_min_size(900., 600.),
            ),
    );
}
