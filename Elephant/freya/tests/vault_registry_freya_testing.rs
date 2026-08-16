//! Real UI proof for the native multi-vault shell path.
//!
//! The fixture drives the same registry file and vault switches as Tauri. It
//! asserts that switching changes the production library, not just a label in
//! the vault popover, and that removing an inactive vault persists.

use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    profile: PathBuf,
    previous_profile: Option<std::ffi::OsString>,
}

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-vault-registry-{stamp}"));
        let profile = root.join("profile");
        let first = root.join("First Vault");
        let second = root.join("Second Vault");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::write(first.join("First.md"), "# First\n").unwrap();
        fs::write(second.join("Second.md"), "# Second\n").unwrap();
        fs::create_dir_all(&profile).unwrap();
        let first_path = fs::canonicalize(&first).unwrap();
        let second_path = fs::canonicalize(&second).unwrap();
        fs::write(
            profile.join("elephantnote.json"),
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": 1,
                "vaults": [
                    {"id": "first-vault", "name": "First Vault", "path": first_path, "icon": "", "lastOpenedAt": "1", "enabled": true},
                    {"id": "second-vault", "name": "Second Vault", "path": second_path, "icon": "", "lastOpenedAt": "2", "enabled": true}
                ],
                "activeVaultId": "first-vault"
            }))
            .unwrap(),
        )
        .unwrap();
        let previous_profile = env::var_os("ELEPHANT_FREYA_PROFILE");
        env::set_var("ELEPHANT_FREYA_PROFILE", &profile);
        Self {
            root,
            profile,
            previous_profile,
        }
    }

    fn first(&self) -> PathBuf {
        self.root.join("First Vault")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        match self.previous_profile.take() {
            Some(value) => env::set_var("ELEPHANT_FREYA_PROFILE", value),
            None => env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing native target {label:?}"))
}

fn click(runner: &mut TestingRunner, label: &str) {
    runner.click_cursor(node(runner, label).layout().area.center().to_f64());
    runner.sync_and_update();
}

#[test]
fn vault_switcher_activates_second_vault_and_remove_persists_registry() {
    let fixture = Fixture::new();
    let root = fixture.first();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click(&mut runner, "First Vault - open vault switcher");
    assert!(node(&runner, "First Vault").layout().area.size.width > 0.);
    assert!(node(&runner, "Second Vault").layout().area.size.width > 0.);
    let evidence = std::env::temp_dir().join("freya-vanilla-parity-evidence");
    fs::create_dir_all(&evidence).unwrap();
    runner.render_to_file(&evidence.join("vault-switcher.png"));
    click(&mut runner, "Second Vault");
    assert!(node(&runner, "Second").layout().area.size.width > 0.);

    click(&mut runner, "Second Vault - open vault switcher");
    click(&mut runner, "Manage vaults");
    assert!(node(&runner, "Registered vaults").layout().area.size.width > 0.);
    runner.render_to_file(&evidence.join("vault-settings.png"));
    click(&mut runner, "Remove First Vault from list");
    runner.sync_and_update();

    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.profile.join("elephantnote.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(registry["vaults"].as_array().unwrap().len(), 1);
    assert_eq!(registry["vaults"][0]["name"], "Second Vault");
}
