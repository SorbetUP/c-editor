use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
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
