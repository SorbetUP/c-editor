//! Presentation-only timing for overlays.
//!
//! The shell state decides whether search is open. This module only mirrors
//! the source shell's delayed mount point, keeping animation details out of
//! the search state and renderer.

use freya::{
    animation::{use_animation, AnimNum, Ease},
    prelude::*,
    sdk::use_timeout,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SearchOverlayTransition {
    pub(super) mounted: bool,
    pub(super) interactive: bool,
    pub(super) backdrop_opacity: f32,
    pub(super) content_opacity: f32,
}

pub(super) fn use_search_overlay_transition(open: bool) -> SearchOverlayTransition {
    let hide_timeout = use_timeout(|| Duration::from_millis(75));
    let unmount_timeout = use_timeout(|| Duration::from_millis(125));
    let open_animation = use_animation(|_| AnimNum::new(0., 1.).time(125).ease(Ease::Out));
    let close_animation = use_animation(|_| AnimNum::new(1., 0.).time(125).ease(Ease::Out));
    let ever_opened = use_state(|| false);
    let mounted = use_state(|| open);
    let closing = use_state(|| false);
    let hidden = use_state(|| false);
    let mut open_animation_for_effect = open_animation;
    let mut close_animation_for_effect = close_animation;
    let mut mounted_for_effect = mounted;
    let mut ever_opened_for_effect = ever_opened;
    let mut closing_for_effect = closing;
    let mut hidden_for_effect = hidden;
    let mut hide_timeout_for_effect = hide_timeout;
    let mut unmount_timeout_for_effect = unmount_timeout;
    use_side_effect_with_deps(&open, move |open| {
        if *open {
            ever_opened_for_effect.set(true);
            mounted_for_effect.set(true);
            closing_for_effect.set(false);
            hidden_for_effect.set(false);
            hide_timeout_for_effect.reset();
            unmount_timeout_for_effect.reset();
            open_animation_for_effect.start();
        } else if *ever_opened_for_effect.read() {
            // Match SearchModal.vue: close the functional route now, but
            // keep its visual tree alive until the CSS transition window has
            // elapsed. The root becomes non-interactive while doing so.
            mounted_for_effect.set(true);
            closing_for_effect.set(true);
            hidden_for_effect.set(false);
            hide_timeout_for_effect.reset();
            unmount_timeout_for_effect.reset();
            close_animation_for_effect.start();
        } else {
            mounted_for_effect.set(false);
        }
    });
    let mut mounted_for_expiry = mounted;
    let mut closing_for_expiry = closing;
    let mut hidden_for_expiry = hidden;
    use_side_effect(move || {
        if *closing_for_expiry.read() {
            if hide_timeout.elapsed() {
                hidden_for_expiry.set(true);
            }
            if unmount_timeout.elapsed() {
                mounted_for_expiry.set(false);
                closing_for_expiry.set(false);
            }
        }
    });
    let value = if open {
        open_animation.get().value()
    } else {
        close_animation.get().value()
    };
    let hidden = *hidden.read();
    SearchOverlayTransition {
        // Opening is functional state, so expose the tree in the same render
        // as the rail click; closing may wait for the bounded timeout.
        mounted: open || *mounted.read(),
        // A closing overlay remains visual-only. Pointer events must reach the
        // library immediately after the functional search state closes.
        interactive: open,
        // Tauri paints the backdrop one frame before the dialog content. The
        // small intermediate opacity is presentation-only; search state and
        // accessibility mounting remain controlled by the shell.
        backdrop_opacity: if !open && hidden {
            0.
        } else if !open {
            value
        } else if value < 0.5 {
            0.08
        } else {
            1.
        },
        content_opacity: if hidden || value < 0.5 { 0. } else { 1. },
    }
}
