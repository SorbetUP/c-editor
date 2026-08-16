use elephant_freya::{app::app_with_vault_view, navigation_contract::WorkspaceView};
use freya_testing::{TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing native target {label:?}"))
}

fn click(runner: &mut TestingRunner, label: &str) {
    let area = node(runner, label).layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn fixture_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("elephant-freya-models-activation-{stamp}"));
    fs::create_dir_all(root.join(".elephantnote/models")).expect("create models directory");
    fs::write(root.join(".elephantnote/models/tiny.gguf"), b"GGUF fixture")
        .expect("write GGUF fixture");
    root
}

#[test]
fn activating_a_local_gguf_persists_the_selection_without_claiming_runtime() {
    let root = fixture_root();
    let active_path = root.join(".elephantnote/models/active-model.json");
    let app_root = root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(app_root.clone(), WorkspaceView::Models),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click(&mut runner, "Refresh local models");
    runner.sync_and_update();
    assert!(
        node(&runner, "Local model tiny.gguf")
            .layout()
            .area
            .size
            .width
            > 0.
    );
    click(&mut runner, "Activate local model tiny.gguf");
    runner.sync_and_update();

    assert!(
        node(&runner, "Active local model tiny.gguf")
            .layout()
            .area
            .size
            .width
            > 0.
    );
    let active: Value = serde_json::from_slice(&fs::read(&active_path).expect("active model file"))
        .expect("valid active model JSON");
    assert_eq!(active["fileName"], "tiny.gguf");
    let expected_path = fs::canonicalize(root.join(".elephantnote/models/tiny.gguf"))
        .expect("canonical model path");
    assert_eq!(
        active["path"].as_str(),
        Some(expected_path.to_string_lossy().as_ref())
    );
    assert!(runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some("Local model active-model.json"))
                .then_some(node)
        })
        .is_none());

    drop(runner);
    let restart_root = root.clone();
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault_view(restart_root.clone(), WorkspaceView::Models),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    let mut runner = runner;
    click(&mut runner, "Refresh local models");
    runner.sync_and_update();
    assert!(
        node(&runner, "Active local model tiny.gguf")
            .layout()
            .area
            .size
            .width
            > 0.
    );

    fs::remove_dir_all(root).expect("remove model fixture");
}
