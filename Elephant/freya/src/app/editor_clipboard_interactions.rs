//! Clipboard shortcuts routed through Freya's runtime provider.

use freya::{clipboard::Clipboard, prelude::*};

use super::super::ShellState;

pub(super) fn route_clipboard(
    mut state: State<ShellState>,
    key: &Key,
    modifiers: Modifiers,
) -> Option<Result<(), String>> {
    if !modifiers.contains(Modifiers::ctrl_or_meta()) {
        return None;
    }
    match key {
        Key::Character(character) if character.eq_ignore_ascii_case("c") => {
            let markdown = state
                .read()
                .editor
                .as_ref()
                .ok_or_else(|| "cannot copy without an open note".to_string())
                .and_then(|editor| editor.selected_markdown());
            Some(markdown.and_then(|markdown| {
                Clipboard::set(markdown)
                    .map_err(|error| format!("unable to write clipboard: {error:?}"))
            }))
        }
        Key::Character(character) if character.eq_ignore_ascii_case("v") => {
            Some(match Clipboard::get() {
                Ok(markdown) => state
                    .write()
                    .editor
                    .as_mut()
                    .ok_or_else(|| "cannot paste without an open note".to_string())
                    .and_then(|editor| {
                        editor
                            .paste_markdown(markdown)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    }),
                Err(error) => Err(format!("unable to read clipboard: {error:?}")),
            })
        }
        _ => None,
    }
}
