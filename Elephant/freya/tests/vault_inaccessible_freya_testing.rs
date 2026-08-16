//! Freya proof for recovering from an inaccessible active vault.
//!
//! The registry remains the source of truth when the last active path has
//! disappeared. The user must be able to see the failure and open another
//! registered vault without selecting its directory again.

use elephant_freya::app::app;
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
    previous_vault: Option<std::ffi::OsString>,
}

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = env::temp_dir().join(format!("elephant-freya-inaccessible-vault-{stamp}"));
        let profile = root.join("profile");
        let available = root.join("Available Vault");
        let missing = root.join("Missing Vault");
        fs::create_dir_all(&available).expect("create available vault");
        fs::create_dir_all(&profile).expect("create registry profile");
        fs::write(available.join("Available.md"), "# Available\n").expect("write note");
        fs::write(
            profile.join("elephantnote.json"),
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": 1,
                "vaults": [
                    {"id": "missing-vault", "name": "Missing Vault", "path": missing, "icon": "", "lastOpenedAt": "2", "enabled": true},
                    {"id": "available-vault", "name": "Available Vault", "path": fs::canonicalize(&available).expect("canonical available vault"), "icon": "", "lastOpenedAt": "1", "enabled": true}
                ],
                "activeVaultId": "missing-vault"
            }))
            .expect("encode registry"),
        )
        .expect("write registry");

        let previous_profile = env::var_os("ELEPHANT_FREYA_PROFILE");
        let previous_vault = env::var_os("ELEPHANT_FREYA_VAULT");
        env::set_var("ELEPHANT_FREYA_PROFILE", &profile);
        env::remove_var("ELEPHANT_FREYA_VAULT");
        Self {
            root,
            profile,
            previous_profile,
            previous_vault,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        match self.previous_profile.take() {
            Some(value) => env::set_var("ELEPHANT_FREYA_PROFILE", value),
            None => env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        match self.previous_vault.take() {
            Some(value) => env::set_var("ELEPHANT_FREYA_VAULT", value),
            None => env::remove_var("ELEPHANT_FREYA_VAULT"),
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

#[test]
fn inaccessible_active_vault_keeps_registered_recovery_target() {
    let fixture = Fixture::new();
    let (mut runner, ()) = TestingRunner::new(app, (1280., 840.).into(), |_| (), 1.);

    assert!(node(&runner, "Vault unavailable").layout().area.size.width > 0.);
    node(&runner, "Open registered vault Available Vault");
    let target = node(&runner, "Open registered vault Available Vault")
        .layout()
        .area
        .center()
        .to_f64();
    runner.click_cursor(target);
    runner.sync_and_update();

    assert!(node(&runner, "Available").layout().area.size.width > 0.);
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.profile.join("elephantnote.json")).expect("read registry"),
    )
    .expect("parse registry");
    assert_eq!(registry["activeVaultId"], "available-vault");
}
