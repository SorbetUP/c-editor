use elephant_freya::{
    app::app_with_vault_view,
    navigation_contract::WorkspaceView,
};
use freya::prelude::{Color, Rect};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-graph-canvas-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create graph canvas fixture");
        fs::write(
            root.join("Alpha.md"),
            "# Alpha\n\nA real graph fixture linking [[Beta]].\n",
        )
        .expect("write Alpha fixture");
        fs::write(root.join("Beta.md"), "# Beta\n\nThe linked destination.\n")
            .expect("write Beta fixture");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn labeled_node(runner: &TestingRunner, label: &str) -> TestingNode {
    runner
        .find(|node, element| {
            (element.accessibility().builder.label() == Some(label)).then_some(node)
        })
        .unwrap_or_else(|| panic!("missing Freya accessibility target {label:?}"))
}

fn canvas_node(runner: &TestingRunner) -> TestingNode {
    runner
        .find(|node, element| {
            element
                .accessibility()
                .builder
                .label()
                .filter(|label| label.starts_with("Graph canvas viewport"))
                .map(|_| node)
        })
        .expect("real graph canvas must be visible")
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let area = labeled_node(runner, label).layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

fn viewport_label(runner: &TestingRunner) -> String {
    canvas_node(runner)
        .element()
        .accessibility()
        .builder
        .label()
        .expect("canvas viewport label")
        .to_string()
}

#[test]
fn graph_canvas_captures_real_drag_pan_zoom_and_recenter_motion() {
    let fixture = FixtureVault::new();
    let root = fixture.path().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault_view(root.clone(), WorkspaceView::Graph),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Graph workspace");
    click_label(&mut runner, "Refresh graph");
    runner.sync_and_update();

    let output = std::env::temp_dir().join("elephant-freya-graph-canvas-motion");
    fs::create_dir_all(&output).expect("create motion evidence directory");
    let start_path = output.join("start.png");
    let drag_path = output.join("during-node-drag.png");
    let final_path = output.join("after-pan-zoom-recenter.png");
    runner.render_to_file(&start_path);
    assert!(fs::metadata(&start_path).expect("start capture").len() > 0);

    let canvas = canvas_node(&runner);
    assert!(canvas.layout().area.size.width > 0.);
    assert!(canvas.layout().area.size.height > 0.);
    let initial_viewport = viewport_label(&runner);
    let alpha = labeled_node(&runner, "Select graph node Alpha");
    let initial_alpha_area = alpha.layout().area;
    let beta = labeled_node(&runner, "Select graph node Beta");
    let beta_area = beta.layout().area;
    let normal_beta = Rect::try_downcast(beta.element().as_ref())
        .expect("graph node is a native Freya rect")
        .style
        .background
        .as_color();
    runner.move_cursor((
        ((beta_area.min_x() + beta_area.max_x()) / 2.) as f64,
        ((beta_area.min_y() + beta_area.max_y()) / 2.) as f64,
    ));
    runner.sync_and_update();
    let hovered_beta = Rect::try_downcast(
        labeled_node(&runner, "Select graph node Beta")
            .element()
            .as_ref(),
    )
    .expect("hovered graph node is a native Freya rect")
    .style
    .background
    .as_color();
    assert_eq!(normal_beta, Some(Color::from_rgb(255, 255, 255)));
    assert_eq!(hovered_beta, Some(Color::from_rgb(233, 239, 247)));
    assert_ne!(normal_beta, hovered_beta, "hover must change node styling");
    let alpha_point = (
        ((initial_alpha_area.min_x() + initial_alpha_area.max_x()) / 2.) as f64,
        ((initial_alpha_area.min_y() + initial_alpha_area.max_y()) / 2.) as f64,
    );

    // This is a real pointer sequence against the rendered node. It must
    // move the persisted view override, not merely change an accessibility
    // label or a test-side coordinate.
    runner.press_cursor(alpha_point);
    runner.move_cursor((alpha_point.0 + 80., alpha_point.1 + 42.));
    runner.sync_and_update();
    let dragged_alpha_area = labeled_node(&runner, "Select graph node Alpha")
        .layout()
        .area;
    assert_ne!(
        initial_alpha_area.origin, dragged_alpha_area.origin,
        "node drag must change the rendered node position"
    );
    runner.render_to_file(&drag_path);
    assert!(fs::metadata(&drag_path).expect("drag capture").len() > 0);
    runner.release_cursor((alpha_point.0 + 80., alpha_point.1 + 42.));

    let before_zoom = viewport_label(&runner);
    let canvas_area = canvas_node(&runner).layout().area;
    runner.scroll(canvas_area.center().to_f64(), (0., -120.));
    let after_zoom = viewport_label(&runner);
    assert_ne!(
        before_zoom, after_zoom,
        "wheel action must change the real camera"
    );

    let blank = (
        canvas_area.max_x() as f64 - 28.,
        canvas_area.max_y() as f64 - 28.,
    );
    runner.press_cursor(blank);
    runner.move_cursor((blank.0 - 45., blank.1 - 24.));
    runner.release_cursor((blank.0 - 45., blank.1 - 24.));
    let after_pan = viewport_label(&runner);
    assert_ne!(
        after_zoom, after_pan,
        "blank drag must pan the real graph camera"
    );

    click_label(&mut runner, "Select graph node Alpha");
    click_label(&mut runner, "Recenter graph");
    runner.sync_and_update();
    let after_recenter = viewport_label(&runner);
    assert_ne!(
        after_pan, after_recenter,
        "recenter must change the real camera"
    );
    assert!(
        labeled_node(&runner, "Graph viewport centered on Alpha")
            .layout()
            .area
            .size
            .width
            > 0.
    );
    runner.render_to_file(&final_path);
    assert!(fs::metadata(&final_path).expect("final capture").len() > 0);

    assert_eq!(
        initial_viewport, after_recenter,
        "recenter must return the camera to the deterministic fitted viewport"
    );
}
