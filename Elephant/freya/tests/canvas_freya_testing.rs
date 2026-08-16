use elephant_freya::{app::app_with_vault_view, navigation_contract::WorkspaceView};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault(PathBuf);

impl FixtureVault {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-canvas-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&root).expect("create canvas fixture");
        fs::write(root.join("Alpha.md"), "# Alpha\n\n[[Beta]]\n").expect("write Alpha");
        fs::write(root.join("Beta.md"), "# Beta\n\nTarget\n").expect("write Beta");
        Self(root)
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing Canvas target {label:?}"))
}

fn click(runner: &mut TestingRunner, label: &str) {
    let area = node(runner, label).layout().area;
    runner.click_cursor((
        f64::from((area.min_x() + area.max_x()) / 2.),
        f64::from((area.min_y() + area.max_y()) / 2.),
    ));
}

#[test]
fn canvas_route_loads_real_graph_and_opens_a_selected_note() {
    let fixture = FixtureVault::new();
    let root = fixture.0.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(root.clone(), WorkspaceView::Canvas),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    runner.sync_and_update();
    click(&mut runner, "Refresh Canvas graph");
    runner.sync_and_update();
    assert_eq!(node(&runner, "Semantic Canvas").layout().area.size.width > 0., true);
    let zoom_before = runner
        .find(|_, element| {
            element
                .accessibility()
                .builder
                .label()
                .filter(|label| label.starts_with("Canvas zoom "))
                .map(str::to_owned)
        })
        .expect("Canvas zoom indicator");
    click(&mut runner, "Zoom out Canvas");
    runner.sync_and_update();
    let zoom_after = runner
        .find(|_, element| {
            element
                .accessibility()
                .builder
                .label()
                .filter(|label| label.starts_with("Canvas zoom "))
                .map(str::to_owned)
        })
        .expect("Canvas zoom indicator after update");
    assert_ne!(zoom_before, zoom_after);
    click(&mut runner, "Select graph node Alpha");
    runner.sync_and_update();
    let area = node(&runner, "Select graph node Alpha").layout().area;
    let start = (
        f64::from((area.min_x() + area.max_x()) / 2.),
        f64::from((area.min_y() + area.max_y()) / 2.),
    );
    runner.press_cursor(start);
    runner.move_cursor((start.0 + 24., start.1 + 12.));
    runner.release_cursor((start.0 + 24., start.1 + 12.));
    click(&mut runner, "Save Canvas positions");
    runner.sync_and_update();
    let canvas_file = fixture.0.join(".elephantnote/canvas.json");
    let canvas: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(canvas_file).expect("Canvas positions file"))
            .expect("Canvas JSON");
    assert!(canvas["positions"]["Alpha.md"].is_object());
    click(&mut runner, "Open selected note");
    runner.sync_and_update();
    assert_eq!(node(&runner, "NoteEditorHost").layout().area.size.width > 0., true);
}
