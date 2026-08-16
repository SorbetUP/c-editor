use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault(PathBuf);

impl FixtureVault {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-library-pin-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Alpha.md"), "# Alpha\n").unwrap();
        Self(root)
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn label(runner: &TestingRunner, value: &str) -> TestingNode {
    runner
        .find_many(|node, element| {
            (element.accessibility().builder.label() == Some(value)).then_some(node)
        })
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing accessibility label {value:?}"))
}

fn click(runner: &mut TestingRunner, value: &str) {
    runner.click_cursor(label(runner, value).layout().area.center().to_f64());
    runner.sync_and_update();
}

#[test]
fn library_card_pin_action_persists_and_can_be_reversed() {
    let fixture = FixtureVault::new();
    let root = fixture.0.clone();
    let workspace = root.join(".elephantnote/config/workspace.json");
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click(&mut runner, "Note actions");
    click(&mut runner, "Pin");
    click(&mut runner, "Note actions");
    assert!(
        runner
            .find_many(|node, element| {
                (element.accessibility().builder.label() == Some("Unpin")).then_some(node)
            })
            .len()
            == 1
    );
    let persisted: Value = serde_json::from_str(&fs::read_to_string(&workspace).unwrap()).unwrap();
    assert_eq!(persisted["freyaShell"]["pinnedPaths"][0], "Alpha.md");

    click(&mut runner, "Unpin");
    let persisted: Value = serde_json::from_str(&fs::read_to_string(&workspace).unwrap()).unwrap();
    assert!(persisted["freyaShell"]["pinnedPaths"]
        .as_array()
        .unwrap()
        .is_empty());
}
