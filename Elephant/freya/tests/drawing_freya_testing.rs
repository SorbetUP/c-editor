use elephant_freya::app::app_with_vault;
use freya::prelude::{Key, NamedKey};
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

    fn seed_scene(&self, name: &str, preview: bool) -> PathBuf {
        let path = self.root.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create scene parent");
        }
        let title = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Drawing");
        let scene = json!({
            "type": "excalidraw",
            "version": 2,
            "source": "elephant-freya-drawing-test",
            "title": title,
            "elements": [
                {"id":"fixture-rect","type":"rectangle","x":20,"y":20,"width":80,"height":50,"strokeColor":"#1b1b1f","backgroundColor":"transparent","strokeWidth":2,"opacity":100}
            ],
            "appState": {"viewBackgroundColor":"#ffffff"},
            "files": {}
        });
        fs::write(
            &path,
            serde_json::to_vec_pretty(&scene).expect("serialize scene"),
        )
        .expect("write Excalidraw scene");
        if preview {
            fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../frontend/src/muya/lib/assets/pngicon/image/2.png"),
                path.with_extension("png"),
            )
            .expect("copy real Excalidraw preview fixture");
        }
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

fn drawing_canvas_nodes(runner: &TestingRunner) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        element
            .accessibility()
            .builder
            .label()
            .filter(|label| label.starts_with("Drawing canvas ·"))
            .map(|_| node)
    })
}

#[test]
fn clicking_a_real_drawing_opens_the_native_renderer_not_markdown() {
    let fixture = FixtureVault::new("open");
    let scene_path = fixture.seed_scene("Sketch.excalidraw", true);
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
        labeled_nodes(&runner, "Drawing editor Sketch").len(),
        1,
        "the native drawing shell must own the opened scene"
    );
    assert_eq!(drawing_canvas_nodes(&runner).len(), 1);
    assert_eq!(
        labeled_nodes(&runner, "Drawing element fixture-rect rectangle").len(),
        1,
        "the persisted Excalidraw element must be rendered natively"
    );
    assert!(scene_path.is_file());
    assert!(scene_path.with_extension("png").is_file());
    assert!(
        fs::read_to_string(scene_path)
            .expect("read source scene")
            .contains("elephant-freya-drawing-test"),
        "opening must preserve the real Excalidraw JSON source"
    );
}

#[test]
fn clicking_a_markdown_drawing_resolves_its_real_sidecar_into_native_canvas() {
    let fixture = FixtureVault::new("markdown");
    let scene_path = fixture.seed_scene(".assets/visual.excalidraw", true);
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
    assert_eq!(drawing_canvas_nodes(&runner).len(), 1);
    assert_eq!(
        labeled_nodes(&runner, "Drawing element fixture-rect rectangle").len(),
        1
    );
    assert!(scene_path.is_file());
    assert!(scene_path.with_extension("png").is_file());
}

#[test]
fn missing_png_preview_does_not_block_the_canonical_scene_editor() {
    let fixture = FixtureVault::new("missing-preview");
    let scene_path = fixture.seed_scene("Missing preview.excalidraw", false);
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
    assert_eq!(drawing_canvas_nodes(&runner).len(), 1);
    assert!(
        labeled_nodes(&runner, "Drawing preview unavailable").is_empty(),
        "PNG is a derived preview and must not gate editing the canonical scene"
    );
    assert!(scene_path.is_file());
}

#[test]
fn create_rename_close_and_reopen_keeps_a_visible_real_scene() {
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
        labeled_nodes(&runner, "Drawing editor Untitled Drawing").len(),
        1,
        "the source Drawing action must enter the real native renderer"
    );
    assert_eq!(drawing_canvas_nodes(&runner).len(), 1);

    let created_scene = fixture.root.join("Untitled Drawing.excalidraw");
    assert!(
        created_scene.is_file(),
        "new native drawings must be visible library entries, not hidden .assets orphans"
    );
    assert!(
        !fixture.root.join(".assets/Untitled Drawing.excalidraw").exists(),
        "standalone library creation must not strand the drawing under .assets"
    );
    let scene: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&created_scene).expect("read created scene"))
            .expect("created scene must be valid JSON");
    assert_eq!(scene["type"], "excalidraw");
    assert_eq!(scene["version"], 2);
    assert_eq!(scene["title"], "Untitled Drawing");
    assert!(scene["elements"].is_array());
    assert!(scene["files"].is_object());
    assert!(!created_scene.with_extension("png").exists());

    // The standalone title is a real Freya Input. Append a suffix and submit;
    // this must rename the canonical file and update its JSON title.
    click_label(&mut runner, "Untitled Drawing");
    runner.write_text(" Renamed");
    runner.press_key(Key::Named(NamedKey::Enter));
    runner.sync_and_update();

    let renamed_scene = fixture.root.join("Untitled Drawing Renamed.excalidraw");
    assert!(renamed_scene.is_file(), "Enter must commit the visible rename");
    assert!(!created_scene.exists(), "the old canonical path must be gone");
    let renamed_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&renamed_scene).expect("read renamed drawing"),
    )
    .expect("renamed drawing JSON");
    assert_eq!(renamed_json["title"], "Untitled Drawing Renamed");

    click_label(&mut runner, "Close drawing · Esc");
    runner.sync_and_update();
    assert!(drawing_canvas_nodes(&runner).is_empty());
    assert_eq!(
        labeled_nodes(&runner, "Untitled Drawing Renamed").len(),
        1,
        "after close, the renamed drawing must remain discoverable in the library"
    );

    click_label(&mut runner, "Untitled Drawing Renamed");
    runner.sync_and_update();
    assert_eq!(
        labeled_nodes(&runner, "Drawing editor Untitled Drawing Renamed").len(),
        1,
        "the newly created and renamed canonical scene must reopen"
    );
}
