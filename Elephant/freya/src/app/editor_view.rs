//! Freya conversion boundary for NoteEditorHost/runtimeEditor.

use freya::prelude::*;

use crate::{editor::EditorDocument, theme};

use super::{route_notice, ShellState};

pub(super) fn note_editor_host(mut state: State<ShellState>) -> Element {
    let snapshot = state.read().clone();
    let Some(editor) = snapshot.editor else {
        return route_notice("NoteEditorHost", "No note open");
    };
    let markdown = editor.serialize();
    let undo = rect()
        .width(Size::px(52.))
        .height(Size::px(36.))
        .center()
        .background(theme::color(theme::SOFT))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            let error = {
                let mut shell = state.write();
                shell
                    .editor
                    .as_mut()
                    .and_then(|editor| editor.undo().err())
                    .map(|error| error.to_string())
            };
            if let Some(error) = error {
                eprintln!("[freya][editor] action:failure action=undo error={error}");
                state.write().error = Some(error);
            }
        })
        .a11y_alt("Undo")
        .child(label().text("↶"));
    let save = rect()
        .width(Size::px(72.))
        .height(Size::px(36.))
        .center()
        .background(theme::color(theme::PRIMARY))
        .with_corner_radius(8.)
        .on_mouse_up(move |_| {
            let error = state
                .read()
                .editor
                .as_ref()
                .and_then(|editor| editor.save().err())
                .map(|error| error.to_string());
            if let Some(error) = error {
                eprintln!("[freya][editor] action:failure action=save error={error}");
                state.write().error = Some(error);
            } else {
                eprintln!("[freya][editor] action:complete action=save");
            }
        })
        .a11y_alt("Save")
        .child(label().text("Save"));
    let lines = markdown
        .lines()
        .map(|line| {
            label()
                .color(theme::color(theme::TEXT))
                .text(line.to_string())
                .into_element()
        })
        .collect::<Vec<_>>();
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .spacing(10.)
        .child(
            rect()
                .height(Size::px(44.))
                .horizontal()
                .spacing(8.)
                .child(undo)
                .child(save),
        )
        .child(
            rect()
                .height(Size::fill())
                .padding(Gaps::new_all(18.))
                .background(theme::color(theme::SURFACE))
                .with_corner_radius(10.)
                .child(rect().children(lines)),
        )
        .into_element()
}

#[allow(dead_code)]
fn _document_type_is_kept_for_source_boundary(_: &EditorDocument) {}
