//! Compatibility wrapper for Freya 0.4 pointer hover events.
//!
//! The editor implementation uses semantic mouse-enter/leave helper names to
//! mirror Tauri's hover CSS. Freya exposes those transitions as pointer
//! enter/leave events, so keep the adapter here and the renderer focused on the
//! editor surface itself.

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

include!("editor_view_impl.rs");
