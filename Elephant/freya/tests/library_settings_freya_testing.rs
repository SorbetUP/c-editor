use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey, Rect};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const LONG_NOTE_TITLE: &str =
    "A deliberately long library note title used to exercise card wrapping";

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-library-settings-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        fs::write(
            root.join("Broken.md"),
            "# Broken\n\nThis file will be removed after render.\n",
        )
        .expect("write fixture note");
        fs::write(root.join("Rename.md"), "# Rename\n\nRename me.\n")
            .expect("write rename fixture note");
        fs::write(root.join("Delete.md"), "# Delete\n\nDelete me.\n")
            .expect("write delete fixture note");
        fs::write(
            root.join("MissingAction.md"),
            "# MissingAction\n\nThis file will disappear before delete.\n",
        )
        .expect("write missing action fixture note");
        fs::write(root.join("A.md"), "# A\n\nShort fixture name.\n")
            .expect("write short-name fixture note");
        fs::write(
            root.join(format!("{LONG_NOTE_TITLE}.md")),
            format!("# {LONG_NOTE_TITLE}\n\nLong-name fixture note.\n"),
        )
        .expect("write long-name fixture note");

        let folder = root.join("Folder");
        let subfolder = folder.join("Subfolder");
        fs::create_dir_all(&subfolder).expect("create nested fixture folders");
        fs::write(folder.join("Inside.md"), "# Inside\n\nFolder fixture.\n")
            .expect("write folder fixture note");
        fs::write(
            subfolder.join("Nested.md"),
            "# Nested\n\nSubfolder fixture.\n",
        )
        .expect("write nested fixture note");
        fs::create_dir_all(root.join("Empty Folder")).expect("create empty fixture folder");

        Self { root }
    }

    fn add_bulk_notes(&self, count: usize) {
        for index in 0..count {
            let title = format!("Bulk {index:03}");
            fs::write(
                self.root.join(format!("{title}.md")),
                format!("# {title}\n\nPagination fixture note {index}.\n"),
            )
            .unwrap_or_else(|error| panic!("write bulk fixture note {index}: {error}"));
        }
    }

    fn path(&self) -> &PathBuf {
        &self.root
    }
}

struct ProfileOverride {
    root: PathBuf,
    previous: Option<OsString>,
}

impl ProfileOverride {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-settings-profile-{stamp}"));
        fs::create_dir_all(&root).expect("create preferences profile");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }

    fn preferences_path(&self) -> PathBuf {
        self.root.join("preferences.json")
    }
}

impl Drop for ProfileOverride {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => std::env::set_var("ELEPHANT_FREYA_PROFILE", previous),
            None => std::env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_library_card(runner: &mut TestingRunner, label: &str) {
    let node = library_card_node(runner, label);
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
    // NoteCard.vue intentionally defers activation by 220 ms so a title
    // double-click can enter rename without opening the card. Wait only for
    // that production contract after the click; never before it.
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    if matches!(
        label,
        "Settings" | "Show notes as list" | "Show notes as grid"
    ) {
        let node = accessible_nodes(runner, label)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
        let area = node.layout().area;
        runner.click_cursor((
            f64::from((area.min_x() + area.max_x()) / 2.0),
            f64::from((area.min_y() + area.max_y()) / 2.0),
        ));
        return;
    }
    for delta in [500., -500.] {
        for _ in 0..16 {
            let node = accessible_nodes(runner, label)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
            let area = node.layout().visible_area();
            if area.origin.x >= 0.
                && area.origin.y >= 0.
                && area.origin.x + area.size.width <= 1280.
                && area.origin.y + area.size.height <= 840.
            {
                runner.click_cursor((
                    f64::from(area.origin.x + area.size.width / 2.0),
                    f64::from(area.origin.y + area.size.height / 2.0),
                ));
                return;
            }
            runner.scroll((900., 700.), (0., delta));
            runner.sync_and_update();
        }
    }
    panic!("Freya node {label:?} exists but cannot be scrolled into view");
}

fn click_action_for_card(runner: &mut TestingRunner, card_label: &str, action_label: &str) {
    let card_area = library_card_node(runner, card_label).layout().area;
    let node = accessible_nodes(runner, action_label)
        .into_iter()
        .filter(|node| {
            let area = node.layout().area;
            let center_x = (area.min_x() + area.max_x()) / 2.;
            let center_y = (area.min_y() + area.max_y()) / 2.;
            center_x >= card_area.min_x()
                && center_x <= card_area.max_x()
                && center_y >= card_area.min_y()
                && center_y <= card_area.max_y()
        })
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("card action areas must be ordered")
        })
        .unwrap_or_else(|| {
            panic!("no Freya action {action_label:?} is contained by library card {card_label:?}")
        });
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn click_smallest_label(runner: &mut TestingRunner, label: &str) {
    let nodes = accessible_nodes(runner, label);
    let node = nodes
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("accessible node areas must be ordered")
        })
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn library_card_node(runner: &TestingRunner, label: &str) -> TestingNode {
    accessible_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            let left_area = left.layout().area.size.area();
            let right_area = right.layout().area.size.area();
            left_area
                .partial_cmp(&right_area)
                .expect("library card areas must be ordered")
                .then_with(|| {
                    left.layout()
                        .area
                        .min_x()
                        .partial_cmp(&right.layout().area.min_x())
                        .expect("library card coordinates must be ordered")
                })
        })
        .unwrap_or_else(|| panic!("no Freya library card has accessible label {label:?}"))
}

fn hover_library_card(runner: &mut TestingRunner, label: &str) {
    let area = library_card_node(runner, label).layout().area;
    runner.move_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
    runner.sync_and_update();
}

fn focused_rename_input(runner: &TestingRunner, current_title: &str) -> TestingNode {
    accessible_nodes(runner, current_title)
        .into_iter()
        .filter(|node| {
            let area = node.layout().area;
            area.max_y() - area.min_y() < 50.
        })
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("rename input areas must be ordered")
        })
        .unwrap_or_else(|| panic!("rename input is not visible for {current_title:?}"))
}

fn replace_prefilled_rename(runner: &mut TestingRunner, current_title: &str, next_title: &str) {
    let input = focused_rename_input(runner, current_title);
    let area = input.layout().area;
    runner.click_cursor((
        (area.max_x() - 3.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
    runner.press_key(Key::Named(NamedKey::End));
    for _ in current_title.chars() {
        runner.press_key(Key::Named(NamedKey::Backspace));
    }
    runner.write_text(next_title);
}

#[test]
fn clicking_a_note_that_disappears_surfaces_the_real_open_error() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(accessible_nodes(&runner, "Broken").len() >= 1);
    fs::remove_file(fixture.path().join("Broken.md")).expect("remove note after render");

    click_library_card(&mut runner, "Broken");
    runner.sync_and_update();

    assert!(
        accessible_nodes(&runner, "Library error").len() >= 1,
        "the real card callback must surface the failed file load"
    );
}

#[test]
fn opening_existing_note_records_shell_history_and_back_restores_library() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(!accessible_nodes(&runner, "Show notes as list").is_empty());
    click_library_card(&mut runner, "A");
    runner.sync_and_update();
    assert!(
        accessible_nodes(&runner, "Show notes as list").is_empty(),
        "opening an existing note must replace the library surface with the editor"
    );

    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "A").is_empty());
    assert!(
        !accessible_nodes(&runner, "Show notes as list").is_empty(),
        "Back must return from an existing note to the recorded library directory"
    );
}

#[test]
fn card_actions_are_overlayed_and_rename_and_delete_use_the_real_vault() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let before = library_card_node(&runner, "Rename").layout().area;
    hover_library_card(&mut runner, "Rename");
    assert!(
        accessible_nodes(&runner, "Note actions").len() > 1,
        "each visible note card keeps its own Tauri-parity actions trigger"
    );
    click_action_for_card(&mut runner, "Rename", "Note actions");
    runner.sync_and_update();
    assert!(accessible_nodes(&runner, "Note actions").len() >= 1);
    assert_eq!(library_card_node(&runner, "Rename").layout().area, before);

    click_smallest_label(&mut runner, "Rename");
    runner.sync_and_update();
    let prefilled_input = focused_rename_input(&runner, "Rename");
    assert!(
        prefilled_input.layout().area.size.area() < before.size.area(),
        "the Tauri-parity rename input must be inline inside the existing card"
    );
    runner.press_key(Key::Named(NamedKey::Escape));
    runner.sync_and_update();
    assert!(fixture.path().join("Rename.md").is_file());
    assert_eq!(library_card_node(&runner, "Rename").layout().area, before);

    hover_library_card(&mut runner, "Rename");
    click_action_for_card(&mut runner, "Rename", "Note actions");
    runner.sync_and_update();
    click_smallest_label(&mut runner, "Rename");
    runner.sync_and_update();
    replace_prefilled_rename(&mut runner, "Rename", "Renamed");
    assert!(
        accessible_nodes(&runner, "Renamed").len() >= 1,
        "the focused prefilled rename input must receive replacement text"
    );
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();
    assert!(fixture.path().join("Renamed.md").is_file());
    assert!(!fixture.path().join("Rename.md").exists());

    hover_library_card(&mut runner, "Delete");
    click_action_for_card(&mut runner, "Delete", "Note actions");
    runner.sync_and_update();
    click_smallest_label(&mut runner, "Delete");
    runner.sync_and_update();
    assert!(!fixture.path().join("Delete.md").exists());
    assert!(accessible_nodes(&runner, "Delete").is_empty());

    fs::remove_file(fixture.path().join("MissingAction.md")).expect("remove action target");
    hover_library_card(&mut runner, "MissingAction");
    click_action_for_card(&mut runner, "MissingAction", "Note actions");
    runner.sync_and_update();
    click_smallest_label(&mut runner, "Delete");
    runner.sync_and_update();
    assert_eq!(
        accessible_nodes(&runner, "Library error").len(),
        1,
        "a real vault delete failure must remain visible in the library surface"
    );
}

#[test]
fn grid_list_nested_folder_back_and_empty_states_use_real_fixture_content() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert!(!accessible_nodes(&runner, "A").is_empty());
    assert!(!accessible_nodes(&runner, LONG_NOTE_TITLE).is_empty());
    assert!(!accessible_nodes(&runner, "Folder").is_empty());
    assert!(!accessible_nodes(&runner, "Empty Folder").is_empty());

    click_label(&mut runner, "Show notes as list");
    runner.sync_and_update();
    let list_area = library_card_node(&runner, LONG_NOTE_TITLE).layout().area;
    assert!(
        ((list_area.max_y() - list_area.min_y()) - 58.).abs() < 0.5,
        "list cards must keep the Tauri 58px compact height"
    );
    click_label(&mut runner, "Show notes as grid");
    runner.sync_and_update();

    click_library_card(&mut runner, "Folder");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Inside").is_empty());
    assert!(!accessible_nodes(&runner, "Subfolder").is_empty());

    click_library_card(&mut runner, "Subfolder");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Nested").is_empty());
    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Inside").is_empty());
    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Folder").is_empty());

    click_library_card(&mut runner, "Empty Folder");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "Empty library").len(), 1);
    click_label(&mut runner, "Retour");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, LONG_NOTE_TITLE).is_empty());
}

#[test]
fn dragging_a_library_entry_into_a_folder_moves_the_real_file() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let source = library_card_node(&runner, "A")
        .layout()
        .area
        .center()
        .to_f64();
    let target = library_card_node(&runner, "Folder")
        .layout()
        .area
        .center()
        .to_f64();
    runner.move_cursor(source);
    runner.press_cursor(source);
    runner.move_cursor(target);
    runner.sync_and_update();
    runner.release_cursor(target);
    runner.sync_and_update();

    assert!(!fixture.path().join("A.md").exists());
    assert!(fixture.path().join("Folder").join("A.md").exists());
}

#[test]
fn scrolling_reveals_buffered_entries_and_fetches_beyond_first_real_page() {
    let fixture = FixtureVault::new();
    fixture.add_bulk_notes(250);
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let initial_note_actions = accessible_nodes(&runner, "Note actions").len();
    assert!(
        initial_note_actions <= 72,
        "the initial Freya render window must stay bounded like Tauri; got {initial_note_actions} note cards"
    );
    assert!(initial_note_actions > 0);

    // Exercise repeated real scroll transitions until the third backend page
    // is actually observable. A fixed number of wheel events is not a page
    // contract: viewport/content heights change after each append.
    for _ in 0..28 {
        if !accessible_nodes(&runner, "Bulk 249").is_empty() {
            break;
        }
        runner.scroll((640., 420.), (0., -1200.));
        runner.sync_and_update();
        runner.scroll((640., 420.), (0., 180.));
        runner.sync_and_update();
    }

    let paged_note_actions = accessible_nodes(&runner, "Note actions").len();
    assert!(
        paged_note_actions > 120,
        "scrolling must fetch beyond the first 120-entry backend page; got {paged_note_actions} note cards"
    );
    assert!(
        accessible_nodes(&runner, "Bulk 249").len() >= 1,
        "a note from the final page of the 250-note fixture must become reachable after repeated real page continuation"
    );
}

#[test]
fn settings_control_persists_canonical_preferences_and_restores_on_restart() {
    let profile = ProfileOverride::new();
    fs::write(
        profile.preferences_path(),
        r#"{"autoSave":false,"unknownLeadKey":{"keep":true}}"#,
    )
    .expect("seed canonical preferences");
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    click_label(&mut runner, "Select Editor settings");
    runner.sync_and_update();
    let before = Rect::try_downcast(
        accessible_nodes(&runner, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("autosave control must be a real rectangle")
    .style
    .background;
    click_label(&mut runner, "Enable autosave");
    runner.sync_and_update();
    let after = Rect::try_downcast(
        accessible_nodes(&runner, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("autosave control must remain a real rectangle")
    .style
    .background;
    assert_ne!(
        before, after,
        "the visible setting control must change on click"
    );

    let saved: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(profile.preferences_path()).expect("canonical preferences are saved"),
    )
    .expect("canonical preferences remain valid JSON");
    assert_eq!(saved["autoSave"], serde_json::Value::Bool(true));
    assert_eq!(
        saved["unknownLeadKey"]["keep"],
        serde_json::Value::Bool(true)
    );
    assert!(!fixture.path().join("preferences.json").exists());

    let restart_root = fixture.path().clone();
    let (mut restarted, ()) = TestingRunner::new(
        move || app_with_vault(restart_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    click_label(&mut restarted, "Settings");
    restarted.sync_and_update();
    click_label(&mut restarted, "Select Editor settings");
    restarted.sync_and_update();
    let restored = Rect::try_downcast(
        accessible_nodes(&restarted, "Enable autosave")[0]
            .element()
            .as_ref(),
    )
    .expect("restored autosave control must be a real rectangle")
    .style
    .background;
    assert_eq!(
        restored, after,
        "restart must restore the persisted visual state"
    );
}

#[test]
fn malformed_canonical_preferences_are_visible_through_the_settings_error_surface() {
    let profile = ProfileOverride::new();
    fs::write(profile.preferences_path(), "{ not valid preferences")
        .expect("write malformed canonical preferences");
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "Settings error").len(), 1);
    assert_eq!(
        accessible_nodes(&runner, "Settings could not be loaded").len(),
        0,
        "the visible error must be represented by the surface state, not a hidden fallback"
    );
    assert_eq!(
        fs::read_to_string(profile.preferences_path()).unwrap(),
        "{ not valid preferences"
    );
}

#[test]
fn folder_sidebar_visibility_round_trips_through_the_real_workspace_metadata() {
    let fixture = FixtureVault::new();
    let workspace_dir = fixture.path().join(".elephantnote/config");
    fs::create_dir_all(&workspace_dir).expect("create workspace metadata directory");
    fs::write(
        workspace_dir.join("workspace.json"),
        r#"{
          "version": 1,
          "sidebar": [{"path": "Folder", "title": "Folder", "type": "folder"}]
        }"#,
    )
    .expect("seed workspace sidebar metadata");

    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    let sidebar_nodes = |runner: &TestingRunner, label: &str| {
        let nodes = accessible_nodes(runner, label);
        nodes
            .into_iter()
            .filter(|node| {
                let size = node.layout().area.size;
                size.width >= 200. && size.height <= 40.
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(sidebar_nodes(&runner, "Folder").len(), 1);
    assert_eq!(sidebar_nodes(&runner, "Empty Folder").len(), 0);

    hover_library_card(&mut runner, "Folder");
    click_action_for_card(&mut runner, "Folder", "Folder actions");
    runner.sync_and_update();
    assert_eq!(accessible_nodes(&runner, "Hide from sidebar").len(), 1);
    click_action_for_card(&mut runner, "Folder", "Hide from sidebar");
    runner.sync_and_update();
    runner.sync_and_update();

    let workspace: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(workspace_dir.join("workspace.json")).expect("read workspace"),
    )
    .expect("workspace remains valid JSON");
    assert!(
        workspace["sidebar"].as_array().unwrap().is_empty(),
        "the real Hide from sidebar action must remove Folder from canonical workspace metadata"
    );
    assert_eq!(sidebar_nodes(&runner, "Folder").len(), 0);
}
