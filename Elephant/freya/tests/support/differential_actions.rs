use crate::{
    differential_frames::FrameEvidence,
    differential_scenario::ScenarioAction,
    differential_timelines::{
        action_frames_after, drag_timeline, pointer_timeline, press_modified_key, scroll_timeline,
        settle,
    },
    differential_ui::{
        accessibility_value, all_accessibility_labels, click_label, click_note_card, labeled_nodes,
        paragraph_text,
    },
};
use freya::prelude::{Key, Modifiers, ModifiersExt, NamedKey};
use freya_testing::TestingRunner;
use std::{fs, path::Path};

pub fn run_action(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    vault_root: &Path,
) -> Vec<FrameEvidence> {
    match action.id.as_str() {
        "launch" => {
            let frames = action_frames_after(runner, output, action, |_| {});
            assert!(!labeled_nodes(runner, "All notes").is_empty());
            assert!(!labeled_nodes(runner, "Alpha note").is_empty());
            assert!(!labeled_nodes(runner, "Projects").is_empty());
            frames
        }
        "move-to-alpha-card" => pointer_timeline(
            runner,
            output,
            action,
            "Alpha note",
            &vec!["center".into(), "center-plus-x-24".into(), "center".into()],
        ),
        "open-search" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                click_label(runner, "Search")
            });
            assert!(
                !labeled_nodes(runner, "Search input").is_empty(),
                "HARD_ISSUE open-search: click did not expose the real Search input"
            );
            frames
        }
        "search-alpha" => {
            let query = action
                .input
                .clone()
                .or(action.text.clone())
                .expect("search input");
            let frames = action_frames_after(runner, output, action, |runner| {
                click_label(runner, "Search input");
                runner.write_text(&query);
                runner.sync_and_update();
            });
            settle(runner);
            assert!(
                all_accessibility_labels(runner)
                    .iter()
                    .any(|label| label.starts_with("Open note ")),
                "HARD_ISSUE search-alpha: shared write-text did not trigger the production Freya search; an extra Enter is forbidden; labels={:?}",
                all_accessibility_labels(runner)
            );
            let observed_query = accessibility_value(runner, "Search input").unwrap_or_else(|| {
                panic!(
                    "HARD_ISSUE search-alpha: the real Freya Search input exposes no accessibility value for the written query"
                )
            });
            assert_eq!(
                observed_query, query,
                "HARD_ISSUE search-alpha: accessibility value does not match shared write-text"
            );
            frames
        }
        "close-search" => {
            let repeat = action.repeat.unwrap_or(1);
            let frames = action_frames_after(runner, output, action, move |runner| {
                for _ in 0..repeat {
                    runner.press_key(Key::Named(NamedKey::Escape));
                }
            });
            assert!(
                labeled_nodes(runner, "Search input").is_empty(),
                "HARD_ISSUE close-search: Escape did not close the real search surface"
            );
            frames
        }
        "navigate-all-notes" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                click_label(runner, "All notes")
            });
            assert!(
                !labeled_nodes(runner, "All notes").is_empty(),
                "HARD_ISSUE navigate-all-notes: All notes target is not present after navigation"
            );
            frames
        }
        "open-alpha-note" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                click_note_card(runner, "Alpha note")
            });
            assert!(
                !labeled_nodes(runner, "NoteEditorHost").is_empty(),
                "HARD_ISSUE open-alpha-note: the real NoteEditorHost was not mounted"
            );
            assert!(
                !labeled_nodes(runner, "Paragraph").is_empty(),
                "HARD_ISSUE open-alpha-note: the real Paragraph target is not exposed"
            );
            assert!(
                !labeled_nodes(runner, "Editor scroll").is_empty(),
                "HARD_ISSUE open-alpha-note: the real Editor scroll target is not exposed"
            );
            frames
        }
        "edit-alpha-note" => {
            let marker = action
                .text
                .clone()
                .expect("edit-alpha-note text from shared scenario");
            let marker_for_input = marker.clone();
            let keys = action.keys_before_text.clone().unwrap_or_default();
            let frames = action_frames_after(runner, output, action, move |runner| {
                click_label(runner, "Paragraph");
                for key in &keys {
                    match key.as_str() {
                        "Control+End" => press_modified_key(
                            runner,
                            Key::Named(NamedKey::End),
                            Modifiers::ctrl_or_meta(),
                        ),
                        "Enter" => runner.press_key(Key::Named(NamedKey::Enter)),
                        other => panic!("Freya action edit-alpha-note: unsupported key {other:?}"),
                    }
                }
                runner.write_text(&marker_for_input);
                runner.sync_and_update();
            });
            settle(runner);
            let disk = fs::read_to_string(vault_root.join("Alpha.md"))
                .expect("read edited Alpha.md from the shared fixture");
            assert!(
                disk.contains(&marker),
                "Freya action edit-alpha-note: production autosave did not persist the marker"
            );
            assert!(
                paragraph_text(runner).contains(&marker),
                "HARD_ISSUE edit-alpha-note: Paragraph accessibility target did not receive the marker"
            );
            frames
        }
        "scroll-alpha-note" => scroll_timeline(runner, output, action),
        "close-alpha-note" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                click_label(runner, "Close note")
            });
            assert!(
                labeled_nodes(runner, "NoteEditorHost").is_empty(),
                "HARD_ISSUE close-alpha-note: the real NoteEditorHost remained mounted"
            );
            frames
        }
        "open-create-menu" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                click_label(runner, "Create")
            });
            assert!(
                !labeled_nodes(runner, "Note").is_empty()
                    && !labeled_nodes(runner, "Drawing").is_empty()
                    && !labeled_nodes(runner, "Folder").is_empty(),
                "HARD_ISSUE open-create-menu: the real create menu entries were not exposed"
            );
            frames
        }
        "move-through-create-menu" => pointer_timeline(
            runner,
            output,
            action,
            "Note",
            &action.pointer_path.clone().unwrap_or_else(|| {
                vec![
                    "left".into(),
                    "center".into(),
                    "right".into(),
                    "center".into(),
                ]
            }),
        ),
        "close-create-menu" => {
            let frames = action_frames_after(runner, output, action, |runner| {
                runner.press_key(Key::Named(NamedKey::Escape))
            });
            assert!(
                labeled_nodes(runner, "Note").is_empty(),
                "HARD_ISSUE close-create-menu: Escape did not close the real create menu"
            );
            frames
        }
        "drag-search-rail-item" => drag_timeline(runner, output, action, vault_root),
        other => panic!("shared scenario contains an unsupported action {other:?}"),
    }
}
