//! Editor preference rows from the source settings panel.

use freya::prelude::*;

use super::super::SettingsViewState;

pub(super) fn editor_settings(state: State<SettingsViewState>) -> Vec<Element> {
    let snapshot = state.read().clone();
    let runtime = snapshot.runtime;
    let specs = [
        (
            "Editor footer",
            "Show word count, typography controls and the theme shortcut.",
            "showEditorFooter",
            "Show editor footer",
        ),
        (
            "Tag prefix",
            "Display # before tag names in the editor.",
            "showTagHashInEditor",
            "Show tag prefix",
        ),
        (
            "Quick insert menu",
            "Show the block command menu when its trigger is typed.",
            "hideQuickInsertHint",
            "Show quick insert menu",
        ),
        (
            "Pair brackets",
            "Automatically insert the matching closing bracket.",
            "autoPairBracket",
            "Automatically pair brackets",
        ),
        (
            "Pair Markdown syntax",
            "Automatically close Markdown emphasis and formatting markers.",
            "autoPairMarkdownSyntax",
            "Automatically pair Markdown syntax",
        ),
        (
            "Pair quotes",
            "Automatically insert the matching closing quote.",
            "autoPairQuote",
            "Automatically pair quotes",
        ),
        (
            "Spellchecker",
            "Check spelling while writing.",
            "spellcheckerEnabled",
            "Enable spellchecker",
        ),
        (
            "Code block line numbers",
            "Display line numbers in fenced code blocks.",
            "codeBlockLineNumbers",
            "Show code block line numbers",
        ),
        (
            "Autosave",
            "Write changes to disk automatically.",
            "autoSave",
            "Enable autosave",
        ),
    ];
    let mut controls = Vec::with_capacity(specs.len() + 2);
    for (index, (title, description, key, alt)) in specs.into_iter().enumerate() {
        if index == 3 {
            controls.push(super::text_preference(
                state,
                "Quick insert trigger",
                "The character that opens the insert menu. The default is /.",
                runtime.text_value("quickInsertTrigger"),
            ));
        }
        controls.push(super::preference_switch(
            state,
            title,
            description,
            key,
            runtime.bool_value(key),
            alt,
        ));
    }
    controls.push(super::delay_preference(
        state,
        runtime.integer_value("autoSaveDelay") as u64,
        runtime.is_enabled("autoSaveDelay"),
    ));
    controls
}
