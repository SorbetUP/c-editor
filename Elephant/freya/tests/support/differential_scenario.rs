use crate::capture_support::snapshot_files;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

pub const SCENARIO_PATH: &str = "migration/freya/differential-scenarios.json";
pub const EXPECTED_ACTION_COUNT: usize = 14;

#[derive(Clone, Debug, Deserialize)]
pub struct Scenario {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub id: String,
    pub fixture: ScenarioFixture,
    pub viewport: Viewport,
    pub actions: Vec<ScenarioAction>,
    pub checkpoints: Vec<ScenarioCheckpoint>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScenarioFixture {
    pub id: String,
    pub roots: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    #[serde(rename = "scaleFactor")]
    pub scale_factor: f32,
    #[serde(rename = "deviceScaleFactor")]
    pub device_scale_factor: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScenarioAction {
    pub id: String,
    pub logical: String,
    pub checkpoint: String,
    pub frames: Vec<u64>,
    pub input: Option<String>,
    pub text: Option<String>,
    pub repeat: Option<u32>,
    #[serde(rename = "keysBeforeText")]
    pub keys_before_text: Option<Vec<String>>,
    #[serde(rename = "pointerPath")]
    pub pointer_path: Option<Vec<String>>,
    pub delta: Option<ScrollDelta>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScrollDelta {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScenarioCheckpoint {
    pub id: String,
    #[serde(rename = "afterAction")]
    pub after_action: String,
}

pub struct ProfileGuard {
    previous: Option<std::ffi::OsString>,
    path: PathBuf,
}

impl ProfileGuard {
    pub fn new(output: &Path) -> Self {
        let path = output.join("freya-profile");
        fs::create_dir_all(&path).expect("create isolated Freya capture profile");
        fs::write(
            path.join("preferences.json"),
            r#"{"autoSave":true,"autoSaveDelay":0}"#,
        )
        .expect("write isolated Freya capture preferences");
        let previous = env::var_os("ELEPHANT_FREYA_PROFILE");
        env::set_var("ELEPHANT_FREYA_PROFILE", &path);
        Self { previous, path }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => env::set_var("ELEPHANT_FREYA_PROFILE", value),
            None => env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = &self.path;
    }
}

pub fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| {
        panic!("Freya differential capture requires {name}; run it through the shared orchestrator")
    })
}

pub fn required_path_env(name: &str) -> PathBuf {
    let path = PathBuf::from(required_env(name));
    assert!(path.is_absolute(), "{name} must be an absolute path");
    path
}

pub fn load_scenario(path: &Path) -> Scenario {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read shared scenario {}: {error}", path.display()));
    let scenario: Scenario = serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse shared scenario {}: {error}", path.display()));
    assert_eq!(scenario.schema_version, 1, "shared scenario schema changed");
    assert_eq!(scenario.actions.len(), EXPECTED_ACTION_COUNT);
    assert_eq!(scenario.checkpoints.len(), EXPECTED_ACTION_COUNT);
    for (index, action) in scenario.actions.iter().enumerate() {
        assert!(!action.id.is_empty(), "shared action {index} has no id");
        assert!(
            !action.frames.is_empty(),
            "shared action {} has no frames",
            action.id
        );
        assert_eq!(
            scenario.checkpoints[index].id, action.checkpoint,
            "checkpoint order must remain aligned with action order"
        );
        assert_eq!(
            scenario.checkpoints[index].after_action, action.id,
            "checkpoint {} must remain attached to action {}",
            scenario.checkpoints[index].id, action.id
        );
    }
    scenario
}

pub fn vault_root(scenario: &Scenario, fixture_root: &Path) -> PathBuf {
    fixture_root.join(
        scenario
            .fixture
            .roots
            .get("vault")
            .expect("fixture vault root"),
    )
}

pub fn fixture_record(scenario: &Scenario, fixture_root: &Path) -> Value {
    let vault_root = vault_root(scenario, fixture_root);
    assert!(vault_root.is_dir(), "orchestrator fixture vault is missing");
    json!({
        "id": scenario.fixture.id,
        "roots": scenario.fixture.roots,
        "files": snapshot_files(&vault_root),
    })
}
