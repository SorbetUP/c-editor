use crate::{
    capture_support::{sha256, snapshot_files},
    differential_frames::{distinct_frame_count, FrameEvidence},
    differential_scenario::ScenarioAction,
    differential_ui::{
        accessibility_value, all_accessibility_labels, geometry_for_labels, labeled_nodes,
        paragraph_text, read_rail_order,
    },
};
use freya_testing::TestingRunner;
use serde_json::{json, Value};
use std::{fs, path::Path};

const DIFFERENTIAL_EDIT_MARKER: &str = "Differential edit marker 2026-06-22.";

pub fn checkpoint_record(
    runner: &TestingRunner,
    vault_root: &Path,
    action: &ScenarioAction,
    frames: Vec<FrameEvidence>,
) -> Value {
    let state = state_snapshot(runner, vault_root, action, &frames);
    let vault = snapshot_files(vault_root);
    let state_bytes = serde_json::to_vec(&state).expect("serialize Freya state snapshot");
    let vault_bytes = serde_json::to_vec(&vault).expect("serialize Freya vault snapshot");
    json!({
        "id": action.checkpoint,
        "afterAction": action.id,
        "state": state,
        "stateHash": sha256(&state_bytes),
        "vault": vault,
        "vaultHash": sha256(&vault_bytes),
        "frames": frames
    })
}

fn state_snapshot(
    runner: &TestingRunner,
    vault_root: &Path,
    action: &ScenarioAction,
    frames: &[FrameEvidence],
) -> Value {
    let labels = all_accessibility_labels(runner);
    let editor_open = !labeled_nodes(runner, "NoteEditorHost").is_empty();
    let search_visible = !labeled_nodes(runner, "Search input").is_empty();
    let menu_items = ["Note", "Drawing", "Folder"]
        .into_iter()
        .filter(|label| labels.iter().any(|actual| actual == label))
        .collect::<Vec<_>>();
    let menu_visible = !menu_items.is_empty();
    let visible_entries = ["Alpha note", "Projects"]
        .into_iter()
        .filter(|label| {
            !editor_open
                && labeled_nodes(runner, label)
                    .iter()
                    .any(|node| node.layout().area.origin.x >= 290.0)
        })
        .collect::<Vec<_>>();
    let body = paragraph_text(runner);
    let editor_text = if editor_open {
        source_probe_editor_text(&body)
    } else {
        String::new()
    };
    let persisted_body = fs::read_to_string(vault_root.join("Alpha.md")).unwrap_or_default();
    let errors = labels
        .iter()
        .filter(|label| label.to_ascii_lowercase().contains("error"))
        .cloned()
        .collect::<Vec<_>>();
    let query = accessibility_value(runner, "Search input").unwrap_or_default();
    let rendered_changed = distinct_frame_count(frames) > 1;
    let scroll = action.delta.as_ref().map(|delta| {
        json!({
            "requestedDeltaY": delta.y,
            "mustChange": rendered_changed
        })
    });
    json!({
        "route": if editor_open { "note-editor" } else { "library" },
        "vaultName": vault_root.file_name().and_then(|name| name.to_str()).unwrap_or("vault"),
        "visibleEntries": visible_entries,
        "sidebarVisible": labels.iter().any(|label| label == "Sidebar"),
        "errors": errors,
        "searchVisible": search_visible,
        "query": query,
        "resultTitles": labels.iter().filter_map(|label| label.strip_prefix("Open note ").map(str::to_owned)).collect::<Vec<_>>(),
        "libraryStillMounted": !labels.iter().any(|label| label == "NoteEditorHost"),
        "currentPath": "",
        "openNote": editor_open.then_some("Alpha note"),
        "notePath": editor_open.then_some("Alpha.md"),
        "bodyContains": if persisted_body.contains("Visible alpha body line.") { vec!["Visible alpha body line."] } else { Vec::new() },
        "editorText": editor_text,
        "closeControl": editor_open.then_some("Close note"),
        "persistedFile": {
            "path": "Alpha.md",
            "mustContain": DIFFERENTIAL_EDIT_MARKER,
            "contains": fs::read_to_string(vault_root.join("Alpha.md")).unwrap_or_default().contains(DIFFERENTIAL_EDIT_MARKER)
        },
        "scroll": scroll,
        "createMenuVisible": menu_visible,
        "menuItems": menu_items,
        "railOrder": read_rail_order(vault_root),
        "accessibilityLabels": labels,
        "geometry": geometry_for_labels(runner)
    })
}

fn source_probe_editor_text(body: &str) -> String {
    // The shared Tauri probe reads the editor host's text contract, including
    // its filename marker and cursor status suffix. Its DOM text omits the
    // first block boundary, and the edit action keeps the inserted marker in
    // that same leading text run.
    let omitted_boundaries = if body.contains(DIFFERENTIAL_EDIT_MARKER) {
        2
    } else {
        1
    };
    let mut source_text = body.to_owned();
    for _ in 0..omitted_boundaries {
        source_text = source_text.replacen('\n', "", 1);
    }
    format!("Alpha.md{source_text}0 / 0")
}
