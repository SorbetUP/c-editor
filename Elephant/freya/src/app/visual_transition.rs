//! Presentation-only timing for overlays.
//!
//! The shell state decides whether search is open. This module only mirrors
//! the source shell's delayed mount point, keeping animation details out of
//! the search state and renderer.

use freya::{
    animation::{use_animation, AnimNum, Ease},
    prelude::*,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SearchOverlayTransition {
    pub(super) backdrop_opacity: f32,
    pub(super) content_opacity: f32,
}

pub(super) fn use_search_overlay_transition(open: bool) -> SearchOverlayTransition {
    let animation = use_animation(|_| AnimNum::new(0., 1.).time(125).ease(Ease::Out));
    let mut animation_for_effect = animation;
    use_side_effect_with_deps(&open, move |open| {
        if *open {
            animation_for_effect.start();
        } else {
            animation_for_effect.reset();
        }
    });
    let value = animation.get().value();
    SearchOverlayTransition {
        // Tauri paints the backdrop one frame before the dialog content. The
        // small intermediate opacity is presentation-only; search state and
        // accessibility mounting remain controlled by the shell.
        backdrop_opacity: if !open || value <= 0. {
            0.
        } else if value < 0.5 {
            0.08
        } else {
            1.
        },
        content_opacity: if open && value >= 0.5 { 1. } else { 0. },
    }
}
