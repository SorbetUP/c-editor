//! Focused regression proof for the modern Tauri SearchModal contract.
//!
//! `AppShell.vue` mounts SearchModal as a sibling of MainContent: opening
//! Search must overlay the current workspace, not replace or reset it.

use elephant_freya::app::app_with_vault;
use freya::prelude::{AccessibilityRole, Key, NamedKey};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-search-overlay-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create Projects");
        fs::write(
            root.join("Alpha.md"),
            "# Alpha\n\nSearch overlay fixture.\n",
        )
        .expect("write Alpha");
        fs::write(root.join("Beta.md"), "# Beta\n\nSearch overlay fixture.\n").expect("write Beta");
        fs::write(
            root.join("Gamma.md"),
            "# Gamma\n\nSearch overlay fixture.\n",
        )
        .expect("write Gamma");
        fs::write(root.join("Projects/Plan.md"), "# Plan\n\nNested fixture.\n")
            .expect("write Plan");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("areas must be ordered")
        })
        .unwrap_or_else(|| panic!("missing accessible label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
    runner.sync_and_update();
}

fn library_card_count(runner: &TestingRunner, label: &str) -> usize {
    nodes(runner, label)
        .into_iter()
        .filter(|node| {
            let size = node.layout().area.size;
            size.width >= 180. && size.height >= 70.
        })
        .count()
}

#[test]
fn opening_and_closing_search_preserves_the_underlying_library_cards() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1170., 768.).into(),
        |_| (),
        1.,
    );

    assert_eq!(library_card_count(&runner, "Alpha"), 1);
    assert_eq!(library_card_count(&runner, "Projects"), 1);

    click_label(&mut runner, "Search notes");
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();

    assert_eq!(
        nodes(&runner, "Search input").len(),
        1,
        "SEARCH-OVERLAY-001: Search must expose its real modal input"
    );
    assert_eq!(
        library_card_count(&runner, "Alpha"),
        1,
        "SEARCH-OVERLAY-001: opening Search must keep the Alpha library card mounted underneath"
    );
    assert_eq!(
        library_card_count(&runner, "Projects"),
        1,
        "SEARCH-OVERLAY-001: opening Search must preserve folder cards too"
    );

    // With an empty query, Escape closes the modal in the same two-stage
    // keyboard contract used by the modern SearchModal.
    runner.press_key(Key::Named(NamedKey::Escape));
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();

    assert!(
        nodes(&runner, "Search input").is_empty(),
        "SEARCH-OVERLAY-002: Escape with an empty query must unmount Search after its close transition"
    );
    assert_eq!(library_card_count(&runner, "Alpha"), 1);
    assert_eq!(library_card_count(&runner, "Projects"), 1);
}

#[test]
fn search_overlay_renders_all_results_and_enter_opens_the_cyclic_selection() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1170., 768.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Search notes");
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();
    click_label(&mut runner, "Search input");
    runner.write_text("Search overlay fixture");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
    runner.sync_and_update();

    assert_eq!(nodes(&runner, "Open note Alpha").len(), 1);
    assert_eq!(nodes(&runner, "Open note Beta").len(), 1);
    assert_eq!(nodes(&runner, "Open note Gamma").len(), 1);
    assert_eq!(nodes(&runner, "Open concept Alpha").len(), 1);
    assert_eq!(nodes(&runner, "Graph workspace").len(), 0);
    assert_eq!(nodes(&runner, "Search mode: exact").len(), 0);

    let dialog = nodes(&runner, "Search dialog")
        .into_iter()
        .next()
        .expect("Search dialog must be mounted");
    let dialog_area = dialog.layout().area;
    assert_eq!(dialog_area.size.width, 720.);
    assert!(
        (dialog_area.origin.x - 225.).abs() < 1.,
        "SEARCH-GEOMETRY-001: dialog x actual={} expected=225 width={} viewport=1170",
        dialog_area.origin.x,
        dialog_area.size.width
    );
    assert!(
        (dialog_area.origin.y - 92.16).abs() < 1.,
        "SEARCH-GEOMETRY-002: dialog y actual={} expected=92.16",
        dialog_area.origin.y
    );
    assert!(dialog_area.size.height <= 72. + 768. * 0.6 + 1.);

    let note_areas = ["Alpha", "Beta", "Gamma"]
        .into_iter()
        .map(|title| {
            nodes(&runner, &format!("Open note {title}"))
                .into_iter()
                .next()
                .expect("note result must be mounted")
                .layout()
                .area
        })
        .collect::<Vec<_>>();
    let concept_bottom = nodes(&runner, "Open concept Alpha")
        .into_iter()
        .next()
        .expect("concept result must be mounted")
        .layout()
        .area
        .max_y();
    assert!(
        note_areas.iter().all(|area| area.min_y() > concept_bottom),
        "SEARCH-OVERLAY-003: note results must follow the concept section without overlap"
    );
    for pair in note_areas.windows(2) {
        assert!(
            pair[0].max_y() <= pair[1].min_y() || pair[1].max_y() <= pair[0].min_y(),
            "SEARCH-OVERLAY-003: note cards must not overlap"
        );
    }

    runner.press_key(Key::Named(NamedKey::ArrowDown));
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::ArrowDown));
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();

    assert_eq!(nodes(&runner, "NoteEditorHost").len(), 1);
    assert_eq!(
        runner
            .find_many(|node, element| {
                let accessibility = element.accessibility();
                (accessibility.builder.role() == AccessibilityRole::TextInput
                    && accessibility.builder.label() == Some("Gamma"))
                    .then_some(node)
            })
            .len(),
        1
    );
}
