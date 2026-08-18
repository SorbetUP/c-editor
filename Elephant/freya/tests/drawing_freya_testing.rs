use elephant_freya::app::app_with_vault;
use freya::prelude::{Code, Key, Modifiers, ModifiersExt};
use freya_testing::prelude::{KeyboardEventName, PlatformEvent};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-drawing-{label}-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture vault");
        Self { root }
    }

    fn seed_scene(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        let scene = json!({
            "type": "excalidraw",
            "version": 2,
            "source": "elephant-freya-drawing-test",
            "elements": [],
            "appState": {},
            "files": {}
        });
        fs::write(
            &path,
            serde_json::to_vec_pretty(&scene).expect("serialize scene"),
        )
        .expect("write Excalidraw scene");
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../frontend/src/muya/lib/assets/pngicon/image/2.png"),
            path.with_extension("png"),
        )
        .expect("copy real Excalidraw preview fixture");
        path
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = labeled_nodes(runner, label)
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("accessible node areas must be ordered")
        })
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
    runner.poll(Duration::from_millis(10), Duration::from_millis(260));
}

fn press_key(runner: &mut TestingRunner, key: Key, modifiers: Modifiers) {
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key,
        code: Code::Unidentified,
        modifiers,
    });
    runner.sync_and_update();
}

#[test]
fn drawing_tool_svg_sources_use_valid_raw_string_quotes() {
    let source = include_str!("../src/app/drawing.rs");
    assert!(!source.contains(r#"viewBox=\""#));
    assert_eq!(source.matches("viewBox=\"0 0 24 24\"").count(), 11);
}

#[test]
fn clicking_a_real_drawing_does_not_open_the_markdown_editor() {
    let fixture = FixtureVault::new("open");
    let scene_path = fixture.seed_scene("Sketch.excalidraw");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Sketch");
    runner.sync_and_update();

    assert!(
        labeled_nodes(&runner, "NoteEditorHost").is_empty(),
        "a real Excalidraw entry must not be routed into the Markdown editor"
    );
    assert_eq!(
        labeled_nodes(&runner, "DrawingCanvas").len(),
        1,
        "Freya must mount the native drawing canvas for the real scene"
    );
    assert_eq!(labeled_nodes(&runner, "Back to library").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Save drawing").len(), 1);
    assert_eq!(labeled_nodes(&runner, "Close drawing").len(), 1);
    assert!(
        scene_path.is_file(),
        "opening must not delete the source scene"
    );
    assert!(
        scene_path.with_extension("png").is_file(),
        "the persisted Excalidraw preview companion must remain available"
    );
    assert!(
        fs::read_to_string(scene_path)
            .expect("read source scene")
            .contains("elephant-freya-drawing-test"),
        "the real Excalidraw JSON must remain the source of truth"
    );

    click_label(&mut runner, "Close drawing");
    assert!(labeled_nodes(&runner, "DrawingCanvas").is_empty());
}

#[test]
fn drawing_toolbar_selects_tools_and_supports_save_and_escape_shortcuts() {
    let fixture = FixtureVault::new("toolbar-shortcuts");
    let scene_path = fixture.seed_scene("Shortcuts.excalidraw");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Shortcuts");
    runner.sync_and_update();

    assert_eq!(
        labeled_nodes(&runner, "Drawing toolbar active tool: Selection").len(),
        1
    );
    assert_eq!(labeled_nodes(&runner, "Rectangle tool (2)").len(), 1);
    click_label(&mut runner, "Rectangle tool (2)");
    assert_eq!(
        labeled_nodes(&runner, "Drawing toolbar active tool: Rectangle").len(),
        1
    );

    press_key(
        &mut runner,
        Key::Character("7".to_owned()),
        Modifiers::default(),
    );
    assert_eq!(
        labeled_nodes(&runner, "Drawing toolbar active tool: Freedraw").len(),
        1
    );
    press_key(
        &mut runner,
        Key::Character("s".to_owned()),
        Modifiers::ctrl_or_meta(),
    );
    assert!(scene_path.is_file());
    assert_eq!(labeled_nodes(&runner, "Library error").len(), 0);

    press_key(
        &mut runner,
        Key::Named(freya::prelude::NamedKey::Escape),
        Modifiers::default(),
    );
    assert!(labeled_nodes(&runner, "DrawingCanvas").is_empty());
}

#[test]
fn clicking_a_markdown_drawing_reads_its_real_scene_and_preview_paths() {
    let fixture = FixtureVault::new("markdown");
    fs::create_dir_all(fixture.root.join(".assets")).expect("create asset directory");
    let scene_path = fixture.seed_scene(".assets/visual.excalidraw");
    fs::write(
        fixture.root.join("Visual drawing.md"),
        "---\ntitle: \"Visual drawing\"\ntype: \"drawing\"\n---\n\n# Visual drawing\n\n![Excalidraw: Visual drawing](.assets/visual.png)\n",
    )
    .expect("write drawing note");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Visual drawing");
    runner.sync_and_update();

    assert!(labeled_nodes(&runner, "NoteEditorHost").is_empty());
    assert_eq!(
        labeled_nodes(&runner, "DrawingCanvas").len(),
        1,
        "Markdown drawing cards must resolve and mount the persisted native scene"
    );
    assert!(scene_path.is_file());
    assert!(scene_path.with_extension("png").is_file());
}

#[test]
fn missing_persisted_preview_still_opens_from_the_real_scene() {
    let fixture = FixtureVault::new("missing-preview");
    let scene_path = fixture.seed_scene("Missing preview.excalidraw");
    fs::remove_file(scene_path.with_extension("png")).expect("remove preview fixture");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Missing preview");
    runner.sync_and_update();

    assert!(labeled_nodes(&runner, "NoteEditorHost").is_empty());
    assert_eq!(
        labeled_nodes(&runner, "DrawingCanvas").len(),
        1,
        "the native renderer must use the real scene even without a preview companion"
    );
}

#[test]
fn closing_a_drawing_keeps_it_open_when_persisting_fails() {
    let fixture = FixtureVault::new("close-save-error");
    let scene_path = fixture.seed_scene("Unsaved.excalidraw");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Unsaved");
    runner.sync_and_update();
    fs::remove_file(&scene_path).expect("remove scene before close");
    fs::create_dir(&scene_path).expect("block scene path before close");

    click_label(&mut runner, "Close drawing");
    runner.sync_and_update();

    assert_eq!(
        labeled_nodes(&runner, "DrawingCanvas").len(),
        1,
        "a failed save must not discard the open drawing"
    );
    assert_eq!(labeled_nodes(&runner, "Library error").len(), 1);
}

#[test]
fn create_drawing_action_persists_real_scene_and_mounts_native_renderer() {
    let fixture = FixtureVault::new("create");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Create");
    click_label(&mut runner, "Drawing");
    runner.sync_and_update();

    assert_eq!(
        labeled_nodes(&runner, "DrawingCanvas").len(),
        1,
        "the source Drawing action must mount the native renderer"
    );
    let created_scene = fixture
        .root
        .join(".assets")
        .join("Untitled Drawing.excalidraw");
    assert!(
        created_scene.is_file(),
        "the native creation layer must persist the real scene format"
    );
    let scene: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&created_scene).expect("read created scene"))
            .expect("created scene must be valid JSON");
    assert_eq!(scene["type"], "excalidraw");
    assert_eq!(scene["version"], 1);
    assert!(scene["elements"].is_array());
    assert!(scene["files"].is_object());
    assert!(
        !created_scene.with_extension("png").exists(),
        "the native layer must not invent a PNG without the real renderer"
    );
    let created_scenes = fs::read_dir(&fixture.root.join(".assets"))
        .expect("read Excalidraw asset directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|value| value.to_str()) == Some("excalidraw")
        })
        .count();
    assert_eq!(
        created_scenes, 1,
        "the native layer must create one real scene"
    );
}

#[test]
fn drawing_toolbar_exposes_excalidraw_tools_and_keyboard_route_controls() {
    let fixture = FixtureVault::new("toolbar");
    fixture.seed_scene("Toolbar.excalidraw");
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Toolbar");
    runner.sync_and_update();

    assert_eq!(labeled_nodes(&runner, "Drawing tools").len(), 1);
    for (tool, shortcut) in [
        ("Selection", "1"),
        ("Hand", "H"),
        ("Rectangle", "2"),
        ("Ellipse", "4"),
        ("Diamond", "3"),
        ("Arrow", "5"),
        ("Line", "6"),
        ("Freedraw", "7"),
        ("Text", "8"),
        ("Image", "9"),
        ("Eraser", "0"),
    ] {
        assert_eq!(
            labeled_nodes(&runner, &format!("{tool} tool ({shortcut})")).len(),
            1,
            "toolbar must expose the {tool} tool and its shortcut"
        );
    }

    click_label(&mut runner, "Rectangle tool (2)");
    click_label(&mut runner, "Close drawing");
    runner.sync_and_update();
    assert!(labeled_nodes(&runner, "DrawingCanvas").is_empty());
}
