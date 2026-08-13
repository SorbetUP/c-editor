use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(env::var(name).unwrap_or_else(|_| panic!("{name} is required")))
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
                .unwrap()
        })
        .unwrap_or_else(|| panic!("missing Freya accessibility label {label:?}"));
    runner.click_cursor(node.layout().area.center().to_f64());
    runner.sync_and_update();
}

fn new_runner(vault: PathBuf) -> TestingRunner {
    let (runner, ()) = TestingRunner::new(
        move || app_with_vault(vault.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    runner
}

fn capture(runner: &mut TestingRunner, output: &Path, id: &str) {
    runner.sync_and_update();
    let dir = output.join(id);
    fs::create_dir_all(&dir).expect("create parity surface output");
    runner.render_to_file(dir.join("static.png"));
    runner.poll_n(Duration::from_millis(250), 1);
    runner.sync_and_update();
    runner.render_to_file(dir.join("stability.png"));
}

#[test]
fn capture_settings_and_graph_parity_surfaces() {
    let fixture_root = required_path("FREYA_PARITY_FIXTURE_ROOT");
    let output = required_path("FREYA_PARITY_SURFACE_OUTPUT");
    let vault = fixture_root.join("vault");
    assert!(vault.is_dir(), "shared parity fixture vault is missing");
    fs::create_dir_all(&output).expect("create Freya parity surface root");

    let profile = output.join("profile");
    fs::create_dir_all(&profile).expect("create isolated Freya profile");
    env::set_var("ELEPHANT_FREYA_PROFILE", &profile);

    let mut settings = new_runner(vault.clone());
    click_label(&mut settings, "Settings");
    capture(&mut settings, &output, "settings-general");
    click_label(&mut settings, "Select Editor settings");
    capture(&mut settings, &output, "settings-editor");

    let mut graph = new_runner(vault.clone());
    click_label(&mut graph, "Search");
    click_label(&mut graph, "Graph workspace");
    click_label(&mut graph, "Refresh graph");
    capture(&mut graph, &output, "graph-default");

    fs::write(
        output.join("surface-manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "runtime": "freya",
            "viewport": {"width": 1280, "height": 840, "scaleFactor": 1, "deviceScaleFactor": 1},
            "captureMethod": "TestingRunner.render_to_file",
            "checkpoints": ["settings-general", "settings-editor", "graph-default"]
        }))
        .expect("serialize parity surface manifest"),
    )
    .expect("write parity surface manifest");
}
