//! Native Freya editor surface and Freya 0.4 hover compatibility.

use super::{route_notice, ShellState};

mod implementation {
    include!("editor_view_impl.rs");

    trait EditorHoverCompatExt {
        fn on_mouse_enter<F>(self, handler: F) -> Self
        where
            F: FnMut(()) + 'static;

        fn on_mouse_leave<F>(self, handler: F) -> Self
        where
            F: FnMut(()) + 'static;
    }

    impl EditorHoverCompatExt for freya::prelude::Rect {
        fn on_mouse_enter<F>(self, mut handler: F) -> Self
        where
            F: FnMut(()) + 'static,
        {
            use freya::prelude::*;
            self.on_pointer_enter(move |_| handler(()))
        }

        fn on_mouse_leave<F>(self, mut handler: F) -> Self
        where
            F: FnMut(()) + 'static,
        {
            use freya::prelude::*;
            self.on_pointer_leave(move |_| handler(()))
        }
    }
}

pub(super) use implementation::note_editor_host;
