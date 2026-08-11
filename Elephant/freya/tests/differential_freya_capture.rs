//! Differential evidence capture for the real Freya shell.
//!
//! The Tauri/Playwright runner consumes the PNGs and JSON files emitted here.
//! Every pointer action is resolved from the rendered accessibility tree; a
//! missing target is an assertion failure, never a skipped step.
//!
//! Optional fixture schema (`migration/freya/differential-scenarios.json`):
//! `{ "fixture": { "files": [{"root":"vault","path":"Alpha.md",
//! "content":"# Alpha\n"}] } }`.
//! When that file is absent, `fallback_fixture` below is used deliberately and
//! this fallback is recorded in `manifest.json`.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const SIZE: (f32, f32) = (1280.0, 840.0);
const SCENARIO_PATH: &str = "migration/freya/differential-scenarios.json";
const MOTION_STAGES: &[&str] = &[
    "startup",
    "pointer-over-create",
    "create-menu",
    "note-created",
    "editor-open",
    "editor-scroll",
    "settings-open",
    "settings-editor",
];

#[derive(Debug, Clone)]
struct FixtureSpec {
    folders: Vec<String>,
    files: Vec<(String, String)>,
    source: &'static str,
}

#[derive(Debug, Deserialize)]
struct ScenarioFile {
    fixture: Value,
}

#[derive(Debug, Serialize)]
struct LayoutSnapshot {
    index: usize,
    label: Option<String>,
    description: Option<String>,
    role: String,
    visible: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn evidence_dir() -> PathBuf {
    let path = std::env::var_os("ELEPHANT_FREYA_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/differential-evidence")
        });
    fs::create_dir_all(&path).expect("create Freya evidence directory");
    for stage in MOTION_STAGES {
        let stage_path = path.join(stage);
        if stage_path.exists() {
            fs::remove_dir_all(stage_path).expect("remove stale Freya checkpoint directory");
        }
    }
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let is_stale_root_capture = MOTION_STAGES.iter().any(|stage| {
                name == *stage
                    || name == format!("{stage}.png")
                    || name == format!("{stage}.accessibility-layout.json")
                    || name.starts_with(&format!("{stage}-frame-"))
            });
            if is_stale_root_capture && entry.path().is_file() {
                fs::remove_file(entry.path()).expect("remove stale Freya root capture");
            }
        }
    }
    path
}

fn scenario_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SCENARIO_PATH)
}

fn fallback_fixture() -> FixtureSpec {
    FixtureSpec {
        folders: vec!["Projects".into()],
        files: vec![
            (
                "Alpha.md".into(),
                "# Alpha\n\nA deterministic fixture note.\n".into(),
            ),
            (
                "Projects/Plan.md".into(),
                "# Plan\n\nA nested deterministic fixture note.\n".into(),
            ),
        ],
        source: "embedded-fallback: migration/freya/differential-scenarios.json was absent",
    }
}

fn scenario_fixture() -> FixtureSpec {
    let path = scenario_path();
    if !path.is_file() {
        return fallback_fixture();
    }

    let raw = fs::read_to_string(&path).expect("read differential scenario fixture");
    let scenario: ScenarioFile =
        serde_json::from_str(&raw).expect("parse differential-scenarios.json");
    let fixture = scenario.fixture;
    let mut folders = fixture
        .get("folders")
        .or_else(|| fixture.get("directories"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_owned)
                        .expect("fixture folder entries must be strings")
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let files_value = fixture
        .get("files")
        .and_then(Value::as_array)
        .expect("fixture must define a files array");
    let files = files_value
        .iter()
        .filter_map(|item| {
            let object = item
                .as_object()
                .expect("fixture file entries must be objects");
            if object.get("root").and_then(Value::as_str) != Some("vault") {
                return None;
            }
            let path = object
                .get("path")
                .and_then(Value::as_str)
                .expect("fixture file object needs path")
                .to_owned();
            let content = object
                .get("content")
                .map(render_fixture_content)
                .or_else(|| object.get("json").map(render_json_content))
                .unwrap_or_else(|| panic!("fixture vault file {path:?} needs content or json"));
            if let Some(parent) = Path::new(&path).parent().and_then(|path| path.to_str()) {
                if !parent.is_empty() && parent != "." && !folders.iter().any(|item| item == parent)
                {
                    folders.push(parent.to_owned());
                }
            }
            Some((path, content))
        })
        .collect::<Vec<_>>();
    assert!(
        files.iter().any(|(path, _)| path.ends_with(".md")),
        "differential fixture must contain a markdown file"
    );

    FixtureSpec {
        folders,
        files,
        source: "migration/freya/differential-scenarios.json",
    }
}

fn render_json_content(value: &Value) -> String {
    serde_json::to_string_pretty(value).expect("serialize fixture JSON") + "\n"
}

fn render_fixture_content(value: &Value) -> String {
    let Some(text) = value.as_str() else {
        let object = value
            .as_object()
            .expect("fixture content must be a string or object");
        if let Some(prefix) = object.get("prefix").and_then(Value::as_str) {
            let from = object
                .get("generatedLines")
                .and_then(Value::as_object)
                .and_then(|lines| lines.get("from"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let to = object
                .get("generatedLines")
                .and_then(Value::as_object)
                .and_then(|lines| lines.get("to"))
                .and_then(Value::as_u64)
                .unwrap_or(from);
            let template = object
                .get("generatedLines")
                .and_then(Value::as_object)
                .and_then(|lines| lines.get("template"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let generated = (from..=to)
                .map(|index| template.replace("{index}", &index.to_string()))
                .collect::<Vec<_>>()
                .join("\n");
            return format!(
                "{prefix}{generated}{}",
                object.get("suffix").and_then(Value::as_str).unwrap_or("")
            );
        }
        return render_json_content(value);
    };
    text.to_owned()
}

fn make_vault(spec: &FixtureSpec, evidence: &Path) -> PathBuf {
    let root = evidence.join("fixture-vault");
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale differential fixture vault");
    }
    for folder in &spec.folders {
        fs::create_dir_all(root.join(folder)).expect("create differential fixture folder");
    }
    for (path, content) in &spec.files {
        let file = root.join(path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).expect("create differential fixture note parent");
        }
        fs::write(file, content).expect("write differential fixture note");
    }
    root
}

fn center_of_label(runner: &TestingRunner, label: &str) -> (f64, f64) {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then(|| {
                let area = node.layout().visible_area();
                (
                    f64::from(area.origin.x + area.size.width / 2.0),
                    f64::from(area.origin.y + area.size.height / 2.0),
                )
            })
        })
        .unwrap_or_else(|| panic!("Freya action target is missing: accessible label {label:?}"))
}

fn capture_tree(runner: &TestingRunner) -> Vec<LayoutSnapshot> {
    runner
        .find_many(|node: TestingNode, element| {
            let accessibility = element.accessibility();
            let area = node.layout().area;
            Some(LayoutSnapshot {
                index: 0,
                label: accessibility.builder.label().map(str::to_owned),
                description: accessibility.builder.description().map(str::to_owned),
                role: format!("{:?}", accessibility.builder.role()),
                // `TestingNode::is_visible()` unwraps an effect entry that Freya
                // does not create for every node in 0.4.1. Keep the snapshot
                // honest by reporting the observable layout visibility instead of
                // turning a valid tree capture into an internal runner panic.
                visible: area.size.width > 0.0 && area.size.height > 0.0,
                x: area.origin.x,
                y: area.origin.y,
                width: area.size.width,
                height: area.size.height,
            })
        })
        .into_iter()
        .enumerate()
        .map(|(index, mut node)| {
            node.index = index;
            node
        })
        .collect()
}

fn capture_to_paths(runner: &mut TestingRunner, png: &Path, tree_path: &Path, stage: &str) {
    runner.sync_and_update();
    runner.render_to_file(png);
    let size = fs::metadata(&png)
        .unwrap_or_else(|error| panic!("Freya capture {stage:?} has no metadata: {error}"))
        .len();
    assert!(size > 0, "Freya capture {stage:?} is empty");

    let tree = capture_tree(runner);
    fs::write(
        tree_path,
        serde_json::to_vec_pretty(&json!({
            "stage": stage,
            "viewport": {"width": SIZE.0, "height": SIZE.1},
            "nodes": tree,
        }))
        .expect("serialize Freya accessibility/layout snapshot"),
    )
    .expect("write Freya accessibility/layout snapshot");
}

fn capture_stage(runner: &mut TestingRunner, dir: &Path, stage: &str) {
    let stage_dir = dir.join(stage);
    fs::create_dir_all(&stage_dir).expect("create Freya checkpoint directory");
    capture_to_paths(
        runner,
        &stage_dir.join("static.png"),
        &stage_dir.join("static.accessibility-layout.json"),
        stage,
    );
}

fn capture_motion(runner: &mut TestingRunner, dir: &Path, stage: &str) {
    runner.animation_clock().set_speed(1.0);
    let stage_dir = dir.join(stage);
    let frames_dir = stage_dir.join("frames");
    fs::create_dir_all(&frames_dir).expect("create Freya temporal frame directory");
    for frame in 0..9 {
        capture_to_paths(
            runner,
            &frames_dir.join(format!("frame-{frame:03}.png")),
            &frames_dir.join(format!("frame-{frame:03}.accessibility-layout.json")),
            &format!("{stage} frame {frame}"),
        );
        runner.poll_n(Duration::from_millis(50), 1);
    }
}

fn move_and_capture(runner: &mut TestingRunner, dir: &Path, label: &str, stage: &str) {
    let point = center_of_label(runner, label);
    runner.move_cursor(point);
    capture_motion(runner, dir, stage);
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let point = center_of_label(runner, label);
    runner.move_cursor(point);
    runner.click_cursor(point);
    runner.sync_and_update();
}

#[test]
fn capture_deterministic_freya_differential_journey() {
    let evidence = evidence_dir();
    let spec = scenario_fixture();
    let vault = make_vault(&spec, &evidence);
    fs::write(
        evidence.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "runtime": "freya-testing",
            "scenario": "deterministic-shell-journey",
            "fixtureSource": spec.source,
            "evidenceDirectorySource": if std::env::var_os("ELEPHANT_FREYA_EVIDENCE_DIR").is_some() { "ELEPHANT_FREYA_EVIDENCE_DIR" } else { "default-target-directory" },
            "viewport": {"width": SIZE.0, "height": SIZE.1},
            "motionFramesPerStage": 9,
            "pollStepMilliseconds": 50,
            "scope": "partial-native-shell",
            "unprovenRequiredActions": [
                "edit-alpha-note",
                "scroll-alpha-note",
                "drag-search-rail-item",
                "close-alpha-note"
            ],
        }))
        .expect("serialize Freya evidence manifest"),
    )
    .expect("write Freya evidence manifest");

    let app_vault = vault.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_vault.clone()),
        SIZE.into(),
        |_| (),
        1.0,
    );

    capture_stage(&mut runner, &evidence, "startup");
    move_and_capture(&mut runner, &evidence, "Create", "pointer-over-create");
    click_label(&mut runner, "Create");
    capture_motion(&mut runner, &evidence, "create-menu");
    click_label(&mut runner, "Note");
    assert!(
        vault.join("Untitled.md").is_file(),
        "Note action did not persist"
    );
    capture_motion(&mut runner, &evidence, "note-created");

    click_label(&mut runner, "Alpha note");
    capture_motion(&mut runner, &evidence, "editor-open");
    let editor_point = center_of_label(&runner, "Heading 1");
    runner.move_cursor(editor_point);
    runner.scroll(editor_point, (0.0, -480.0));
    capture_motion(&mut runner, &evidence, "editor-scroll");

    click_label(&mut runner, "Settings");
    capture_motion(&mut runner, &evidence, "settings-open");
    click_label(&mut runner, "Select Editor settings");
    capture_motion(&mut runner, &evidence, "settings-editor");
}
