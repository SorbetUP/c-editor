use elephant_freya::app::app;
use freya::prelude::*;

fn main() {
    eprintln!("[freya][lifecycle] action:start component=ElephantShell");
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_title("Elephant")
                .with_size(1280., 840.)
                .with_min_size(900., 600.),
        ),
    );
}
