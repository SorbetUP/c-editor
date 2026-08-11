//! Pointer and keyboard gesture state for shell controls.

use crate::navigation_contract::SidebarWidth;

use super::ShellState;

#[derive(Clone, Debug)]
pub(super) struct RailDragState {
    pub(super) source: String,
    pub(super) start_x: f64,
    pub(super) start_y: f64,
    pub(super) moved: bool,
}

#[derive(Clone, Debug)]
pub(super) struct SidebarResizeState {
    pub(super) start_x: f64,
    pub(super) start_width: SidebarWidth,
}

impl ShellState {
    pub(super) fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        self.persist_shell_preferences();
    }

    pub(super) fn begin_sidebar_resize(&mut self, x: f64) {
        self.sidebar_resize = Some(SidebarResizeState {
            start_x: x,
            start_width: self.sidebar_width,
        });
        eprintln!(
            "[freya][sidebar] action:resize-start width={} x={x}",
            self.sidebar_width.get()
        );
    }

    pub(super) fn update_sidebar_resize(&mut self, x: f64) {
        let Some(resize) = self.sidebar_resize.as_ref() else {
            return;
        };
        self.sidebar_width =
            SidebarWidth::from_number(f64::from(resize.start_width.get()) + x - resize.start_x);
    }

    pub(super) fn finish_sidebar_resize(&mut self, x: f64) {
        self.update_sidebar_resize(x);
        if self.sidebar_resize.take().is_some() {
            eprintln!(
                "[freya][sidebar] action:resize-complete width={} x={x}",
                self.sidebar_width.get()
            );
            self.persist_shell_preferences();
        }
    }

    pub(super) fn resize_sidebar_by(&mut self, delta: f64) {
        self.sidebar_width = SidebarWidth::from_number(f64::from(self.sidebar_width.get()) + delta);
        eprintln!(
            "[freya][sidebar] action:resize-keyboard width={}",
            self.sidebar_width.get()
        );
        self.persist_shell_preferences();
    }

    pub(super) fn begin_rail_drag(&mut self, source: &str, x: f64, y: f64) {
        self.rail_drag = Some(RailDragState {
            source: source.to_string(),
            start_x: x,
            start_y: y,
            moved: false,
        });
        self.rail_drop_target = None;
        eprintln!("[freya][rail] action:drag-start source={source}");
    }

    pub(super) fn update_rail_drag(&mut self, target: &str, x: f64, y: f64) {
        let Some(drag) = self.rail_drag.as_mut() else {
            return;
        };
        if drag.source == target {
            return;
        }
        if (x - drag.start_x).abs() >= 4. || (y - drag.start_y).abs() >= 4. {
            drag.moved = true;
            self.rail_drop_target = Some(target.to_string());
        }
    }

    pub(super) fn finish_rail_drag(&mut self, target: &str) -> bool {
        let Some(drag) = self.rail_drag.take() else {
            return false;
        };
        let was_drag = drag.moved;
        if was_drag && drag.source != target {
            if let Some(source_index) = self.rail_order.iter().position(|id| id == &drag.source) {
                let source = self.rail_order.remove(source_index);
                let target_index = self
                    .rail_order
                    .iter()
                    .position(|id| id == target)
                    .unwrap_or(self.rail_order.len());
                self.rail_order.insert(target_index, source);
                eprintln!(
                    "[freya][rail] action:drag-complete source={} target={target}",
                    drag.source
                );
                self.persist_shell_preferences();
            }
        }
        self.rail_drop_target = None;
        was_drag
    }
}
