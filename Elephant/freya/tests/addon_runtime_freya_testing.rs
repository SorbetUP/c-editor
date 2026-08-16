use elephant_freya::{addon_runtime::RuntimeManager, app::app_with_vault};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const ADDON_ID: &str = "fixture.external";
const WORKING_ENTRY: &str = r#"
self.elephantAddon = {
  activate(api) {
    api.log.info('fixture active');
    return () => api.log.info('fixture disposed');
  },
  deactivate(api) { api.log.info('fixture stopped'); }
};
"#;
const FAILING_ENTRY: &str = r#"
self.elephantAddon = {
  activate() { throw new Error('fixture activation failed'); }
};
"#;

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new(entry: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-addon-runtime-{}-{stamp}-{}",
            std::process::id(),
            entry.len()
        ));
        let package = root.join(".elephantnote/addons/packages").join(ADDON_ID);
        fs::create_dir_all(&package).expect("create addon package directory");
        fs::write(package.join("main.js"), entry).expect("write addon entry");
        fs::write(
            root.join(".elephantnote/addons/registry.json"),
            serde_json::to_vec_pretty(&json!({
                "version": 1,
                "addons": {
                    ADDON_ID: {
                        "manifest": {
                            "id": ADDON_ID,
                            "name": "Fixture External",
                            "version": "1.0.0",
                            "apiVersion": 1,
                            "runtime": { "type": "javascript-worker", "entry": "main.js" },
                            "permissions": {},
                            "contributes": {},
                            "activationEvents": []
                        },
                        "enabled": false,
                        "packageHash": "fixture",
                        "installedAt": "0",
                        "source": "external"
                    }
                }
            }))
            .expect("serialize addon registry"),
        )
        .expect("write addon registry");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn registry_enabled(&self) -> bool {
        self.registry()["addons"][ADDON_ID]["enabled"] == true
    }

    fn set_registry_enabled(&self, enabled: bool) {
        let mut value = self.registry();
        value["addons"][ADDON_ID]["enabled"] = Value::Bool(enabled);
        fs::write(
            self.root.join(".elephantnote/addons/registry.json"),
            serde_json::to_vec_pretty(&value).expect("serialize addon registry"),
        )
        .expect("update addon registry");
    }

    fn registry(&self) -> Value {
        serde_json::from_slice(
            &fs::read(self.root.join(".elephantnote/addons/registry.json"))
                .expect("read addon registry"),
        )
        .expect("parse addon registry")
    }

    fn addon_installed(&self) -> bool {
        !self.registry()["addons"][ADDON_ID].is_null()
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn process_is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no Freya node has accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

#[test]
fn enable_restart_disable_and_uninstall_own_the_real_worker_lifecycle() {
    let fixture = FixtureVault::new(WORKING_ENTRY);
    let mut manager = RuntimeManager::default();

    manager
        .set_enabled(fixture.path(), ADDON_ID, true)
        .expect("enable external addon through runtime manager");
    let pid = manager.active_pid(ADDON_ID).expect("active worker pid");
    assert!(
        process_is_alive(pid),
        "the managed Node worker must be alive"
    );
    assert!(
        fixture.registry_enabled(),
        "enable must persist after activation"
    );

    drop(manager);
    assert!(
        !process_is_alive(pid),
        "dropping Freya state must stop its worker"
    );
    assert!(
        fixture.registry_enabled(),
        "shutdown must preserve the enabled preference for restart"
    );

    let mut manager = RuntimeManager::default();
    manager
        .reconcile_enabled(fixture.path())
        .expect("reactivate persisted enabled addon after restart");
    let restarted_pid = manager
        .active_pid(ADDON_ID)
        .expect("reactivated worker pid");
    assert!(
        process_is_alive(restarted_pid),
        "settings refresh must restore a real Node worker"
    );

    manager
        .set_enabled(fixture.path(), ADDON_ID, false)
        .expect("disable external addon through runtime manager");
    assert!(!manager.is_active(ADDON_ID).expect("inspect runtime"));
    assert!(
        !process_is_alive(restarted_pid),
        "disable must stop the Node process"
    );
    assert!(
        !fixture.registry_enabled(),
        "disable must persist after shutdown"
    );

    manager
        .set_enabled(fixture.path(), ADDON_ID, true)
        .expect("re-enable before uninstall");
    let uninstall_pid = manager
        .active_pid(ADDON_ID)
        .expect("worker before uninstall");
    manager
        .uninstall(fixture.path(), ADDON_ID)
        .expect("uninstall through runtime manager");
    assert!(!manager
        .is_active(ADDON_ID)
        .expect("inspect uninstalled runtime"));
    assert!(
        !process_is_alive(uninstall_pid),
        "uninstall must stop the Node process"
    );
    assert!(
        !fixture.addon_installed(),
        "uninstall must remove the registry record"
    );
    assert!(
        !fixture
            .path()
            .join(".elephantnote/addons/packages")
            .join(ADDON_ID)
            .exists(),
        "uninstall must remove the package"
    );
}

#[test]
fn activation_error_is_returned_and_visible_in_settings() {
    let fixture = FixtureVault::new(FAILING_ENTRY);
    let mut manager = RuntimeManager::default();
    let error = manager
        .set_enabled(fixture.path(), ADDON_ID, true)
        .expect_err("activation failure must be returned");
    assert!(error.contains("fixture activation failed"));
    assert!(
        !fixture.registry_enabled(),
        "failed activation stays disabled"
    );
    assert!(!manager.is_active(ADDON_ID).expect("inspect failed runtime"));
    fixture.set_registry_enabled(true);

    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    click_label(&mut runner, "Settings");
    runner.sync_and_update();
    click_label(&mut runner, "Select Addons settings");
    runner.sync_and_update();
    click_label(&mut runner, "Refresh addons");
    runner.sync_and_update();

    let errors = runner.find_many(|node, element| {
        element
            .accessibility()
            .builder
            .label()
            .filter(|label| {
                label.starts_with("Addon lifecycle error:")
                    && label.contains("fixture activation failed")
            })
            .map(|_| node)
    });
    assert_eq!(errors.len(), 1, "activation error must be visible once");
    assert!(
        !fixture.registry_enabled(),
        "refresh activation failure must roll the persisted flag back"
    );
}
