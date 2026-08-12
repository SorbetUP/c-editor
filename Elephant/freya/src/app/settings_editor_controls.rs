//! Editor preference controls from the current Tauri SettingsPanel surface.

use freya::prelude::*;

use super::super::SettingsViewState;

pub(super) fn editor_settings(state: State<SettingsViewState>) -> Vec<Element> {
    let snapshot = state.read().clone();
    let runtime = snapshot.runtime;
    vec![
        super::preference_switch(
            state,
            "Editor footer",
            "Show word count, typography controls and the theme shortcut.",
            "showEditorFooter",
            runtime.bool_value("showEditorFooter"),
            "Show editor footer",
        ),
        super::preference_switch(
            state,
            "Tag prefix",
            "Display # before tag names in the editor.",
            "showTagHashInEditor",
            runtime.bool_value("showTagHashInEditor"),
            "Show tag prefix",
        ),
        super::preference_switch(
            state,
            "Quick insert menu",
            "Show the block command menu when its trigger is typed.",
            "hideQuickInsertHint",
            runtime.bool_value("hideQuickInsertHint"),
            "Show quick insert menu",
        ),
        super::text_preference(
            state,
            "Quick insert trigger",
            "The character that opens the insert menu. The default is /.",
            runtime.text_value("quickInsertTrigger"),
        ),
        super::preference_switch(
            state,
            "Pair brackets",
            "Automatically insert the matching closing bracket.",
            "autoPairBracket",
            runtime.bool_value("autoPairBracket"),
            "Automatically pair brackets",
        ),
        super::preference_switch(
            state,
            "Pair Markdown syntax",
            "Automatically close Markdown emphasis and formatting markers.",
            "autoPairMarkdownSyntax",
            runtime.bool_value("autoPairMarkdownSyntax"),
            "Automatically pair Markdown syntax",
        ),
        super::preference_switch(
            state,
            "Pair quotes",
            "Automatically insert the matching closing quote.",
            "autoPairQuote",
            runtime.bool_value("autoPairQuote"),
            "Automatically pair quotes",
        ),
        super::preference_switch(
            state,
            "Spellchecker",
            "Check spelling while writing.",
            "spellcheckerEnabled",
            runtime.bool_value("spellcheckerEnabled"),
            "Enable spellchecker",
        ),
        super::preference_switch(
            state,
            "Code block line numbers",
            "Display line numbers in fenced code blocks.",
            "codeBlockLineNumbers",
            runtime.bool_value("codeBlockLineNumbers"),
            "Show code block line numbers",
        ),
        super::integer_stepper(
            state,
            "Note margins",
            "Horizontal space around the title and text.",
            "noteEditorMargin",
            runtime.integer_value("noteEditorMargin"),
            8,
            48,
            4,
            " px",
        ),
        super::preference_switch(
            state,
            "Autosave",
            "Write changes to disk automatically.",
            "autoSave",
            runtime.bool_value("autoSave"),
            "Enable autosave",
        ),
        super::delay_preference(
            state,
            runtime.integer_value("autoSaveDelay") as u64,
            runtime.is_enabled("autoSaveDelay"),
        ),
    ]
}
