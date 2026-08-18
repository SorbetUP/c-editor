//! Task marker interaction routing.

use freya::prelude::*;
use muya_core::NodeId;

use crate::editor::EditorAction;
use crate::theme;

use super::super::ShellState;

pub(crate) fn task_marker(
    state: State<ShellState>,
    autosave_generation: State<u64>,
    item_id: NodeId,
    checked: bool,
    marker: String,
    palette: theme::ThemePalette,
) -> Element {
    let label_text = if checked {
        "Task checked"
    } else {
        "Task unchecked"
    };
    let mut marker_state = state;
    let mut marker_generation = autosave_generation;
    rect()
        .width(Size::px(24.))
        .height(Size::px(24.))
        .center()
        .background(theme::color(if checked {
            palette.primary
        } else {
            palette.surface
        }))
        .border(
            Border::new()
                .fill(theme::color(palette.border_strong))
                .width(2.),
        )
        .with_corner_radius(4.)
        .a11y_alt(label_text)
        .on_mouse_up(move |_| {
            let result = {
                let mut shell = marker_state.write();
                shell
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot toggle task without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .dispatch(EditorAction::SetTaskChecked {
                                item: item_id,
                                checked: !checked,
                                auto_check: false,
                            })
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    })
                    .and_then(|_| shell.save_open_editor().map_err(|error| error.to_string()))
            };
            match result {
                Ok(()) => {
                    *marker_generation.write() += 1;
                    eprintln!(
                        "[freya][editor] action:complete action=toggle-task item={:?} checked={}",
                        item_id, !checked
                    );
                }
                Err(error) => {
                    eprintln!(
                        "[freya][editor] action:failure action=toggle-task item={:?} error={error}",
                        item_id
                    );
                    marker_state.write().error = Some(error);
                }
            }
        })
        .child(
            label()
                .font_size(14.)
                .font_weight(FontWeight::BOLD)
                .color(theme::color(if checked {
                    palette.surface
                } else {
                    palette.text
                }))
                .text(if checked { "✓".to_owned() } else { marker }),
        )
        .into_element()
}
