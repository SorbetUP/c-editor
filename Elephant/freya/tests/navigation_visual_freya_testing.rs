//! Source-derived geometry and state proof for the native navigation shell.
//!
//! The expected numbers below come from develop@62a606c1:
//! `TopVaultBar.vue`, `NavigationBar.vue`, `IconRail.vue`, `SidebarNav.vue`,
//! and `AppShell.vue` plus their scoped shell styles.  The assertions inspect
//! the real Freya accessibility/layout tree and drive the same pointer path a
//! user drives; no state is mutated directly by this test.

use elephant_freya::{app::app_with_vault, theme};
use freya::{
    elements::image::Image,
    prelude::{Color, Fill, Rect, Size2D},
};
use freya_testing::{TestingNode, TestingRunner};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const VIEWPORT: (f32, f32) = (1280., 840.);
const TOPBAR_HEIGHT: f32 = 28.;
const NAV_BUTTON_SIZE: f32 = 24.;
const NAV_BUTTON_TOP: f32 = 4.;
const NAV_GAP: f32 = 2.;
const RAIL_WIDTH: f32 = 56.;
const RAIL_ACTION_SIZE: f32 = 34.;
const RAIL_GAP: f32 = 2.;
const SIDEBAR_WIDTH: f32 = 232.;
const SIDEBAR_RESIZER_HIT_WIDTH: f32 = 12.;
const SIDEBAR_ALL_NOTES_HEIGHT: f32 = 38.;

#[cfg(target_os = "macos")]
const NAV_LEFT: f32 = 84.;
#[cfg(not(target_os = "macos"))]
const NAV_LEFT: f32 = 56.;

#[cfg(target_os = "macos")]
const RAIL_PADDING_TOP: f32 = 36.;
#[cfg(not(target_os = "macos"))]
const RAIL_PADDING_TOP: f32 = 8.;

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-navigation-visual-{stamp}"));
        fs::create_dir_all(root.join("Projects")).expect("create visual fixture vault");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nA fixture note\n")
            .expect("write visual fixture note");
        fs::write(
            root.join("Projects/Plan.md"),
            "# Plan\n\nA nested fixture note\n",
        )
        .expect("write nested visual fixture note");
        Self { root }
    }

    fn path(&self) -> &PathBuf {
        &self.root
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Freya label {label:?}"))
}

fn require_label_matching<F>(runner: &TestingRunner, predicate: F) -> TestingNode
where
    F: Fn(&str) -> bool,
{
    runner
        .find(|node, element| {
            element
                .accessibility()
                .builder
                .label()
                .filter(|label| predicate(label))
                .map(|_| node)
        })
        .unwrap_or_else(|| panic!("missing Freya label matching predicate"))
}

fn area(node: &TestingNode) -> Size2D {
    node.layout().area.size
}

fn origin(node: &TestingNode) -> (f32, f32) {
    let area = node.layout().area;
    (area.origin.x, area.origin.y)
}

fn center(node: &TestingNode) -> (f64, f64) {
    let area = node.layout().area;
    (
        f64::from(area.origin.x + area.size.width / 2.),
        f64::from(area.origin.y + area.size.height / 2.),
    )
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    runner.click_cursor(center(&require_label(runner, label)));
}

fn background(runner: &TestingRunner, label: &str) -> freya::prelude::Fill {
    Rect::try_downcast(require_label(runner, label).element().as_ref())
        .expect("navigation surface must be a rect")
        .style
        .background
}

fn assert_svg_icon(node: &TestingNode, expected_pixels: i32) {
    let dimensions = node.children().into_iter().find_map(|child| {
        Image::try_downcast(child.element().as_ref())
            .map(|image| image.image_handle.image.dimensions())
    });
    assert_eq!(
        dimensions.map(|size| (size.width, size.height)),
        Some((expected_pixels, expected_pixels)),
        "Lucide SVG must rasterize to a pixel image at its visible size"
    );
}

fn has_background(runner: &TestingRunner, label: &str, expected: Fill) -> bool {
    labeled_nodes(runner, label).into_iter().any(|node| {
        Rect::try_downcast(node.element().as_ref())
            .map(|rect| rect.style.background == expected)
            .unwrap_or(false)
    })
}

fn evidence_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("freya-navigation-visual");
    fs::create_dir_all(&dir).expect("create navigation visual evidence directory");
    dir.join(name)
}

fn capture(runner: &mut TestingRunner, name: &str) {
    let path = evidence_path(name);
    runner.sync_and_update();
    runner.render_to_file(&path);
    let size = fs::metadata(&path)
        .unwrap_or_else(|error| panic!("missing PNG evidence {}: {error}", path.display()))
        .len();
    assert!(size > 0, "empty PNG evidence {}", path.display());
}

#[test]
fn source_navigation_geometry_labels_and_order_are_exact_at_shared_viewport() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        VIEWPORT.into(),
        |_| (),
        1.,
    );

    let topbar = require_label(&runner, "TopVaultBar");
    assert_eq!(origin(&topbar), (0., 0.));
    assert_eq!(area(&topbar), Size2D::new(VIEWPORT.0, TOPBAR_HEIGHT));

    let back = require_label(&runner, "Retour");
    let forward = require_label(&runner, "Avancer");
    assert_svg_icon(&back, 18);
    assert_svg_icon(&forward, 18);
    assert_eq!(area(&back), Size2D::new(NAV_BUTTON_SIZE, NAV_BUTTON_SIZE));
    assert_eq!(
        area(&forward),
        Size2D::new(NAV_BUTTON_SIZE, NAV_BUTTON_SIZE)
    );
    assert_eq!(origin(&back), (NAV_LEFT, NAV_BUTTON_TOP));
    assert_eq!(
        origin(&forward),
        (NAV_LEFT + NAV_BUTTON_SIZE + NAV_GAP, NAV_BUTTON_TOP)
    );
    assert!(origin(&back).0 < origin(&forward).0);

    let rail = require_label(&runner, "Workspace navigation");
    assert_eq!(origin(&rail), (0., TOPBAR_HEIGHT));
    assert_eq!(
        area(&rail),
        Size2D::new(RAIL_WIDTH, VIEWPORT.1 - TOPBAR_HEIGHT)
    );
    assert_eq!(
        background(&mut runner, "Workspace navigation"),
        Fill::Color(Color::from_rgb(237, 242, 247)),
        "light rail must use appearance.js activeThemeTokens.sidebar (#edf2f7)"
    );

    let hide_sidebar = require_label(&runner, "Hide sidebar");
    let search = require_label(&runner, "Search");
    let settings = require_label(&runner, "Settings");
    let vault = require_label_matching(&runner, |label| label.ends_with("- open vault switcher"));
    assert_svg_icon(&hide_sidebar, 18);
    assert_svg_icon(&search, 18);
    assert_svg_icon(&settings, 18);
    assert_svg_icon(&vault, 19);
    for node in [&hide_sidebar, &search, &vault, &settings] {
        assert_eq!(area(node), Size2D::new(RAIL_ACTION_SIZE, RAIL_ACTION_SIZE));
        assert_eq!(origin(node).0, (RAIL_WIDTH - RAIL_ACTION_SIZE) / 2.);
    }
    assert_eq!(origin(&hide_sidebar).1, TOPBAR_HEIGHT + RAIL_PADDING_TOP);
    assert_eq!(
        origin(&search).1,
        origin(&hide_sidebar).1 + RAIL_ACTION_SIZE + RAIL_GAP
    );
    assert!(origin(&search).1 < origin(&vault).1);
    assert!(origin(&vault).1 < origin(&settings).1);

    let sidebar = require_label(&runner, "Sidebar");
    assert_eq!(origin(&sidebar), (RAIL_WIDTH, TOPBAR_HEIGHT));
    assert_eq!(
        area(&sidebar),
        Size2D::new(SIDEBAR_WIDTH, VIEWPORT.1 - TOPBAR_HEIGHT)
    );
    assert_eq!(
        background(&mut runner, "Sidebar"),
        Fill::Color(Color::from_rgb(237, 242, 247)),
        "light sidebar must use appearance.js activeThemeTokens.sidebar (#edf2f7)"
    );
    assert!(
        has_background(
            &runner,
            "Alpha",
            Fill::Color(theme::color(theme::card_background())),
        ),
        "the real Library card must use the source color-mix surface token"
    );

    let all_notes = require_label(&runner, "All notes");
    assert_svg_icon(&all_notes, 18);
    assert_eq!(origin(&all_notes), (RAIL_WIDTH + 8., TOPBAR_HEIGHT + 8.));
    assert_eq!(area(&all_notes).height, SIDEBAR_ALL_NOTES_HEIGHT);
    assert_eq!(area(&all_notes).width, SIDEBAR_WIDTH - 16.);
    assert!(origin(&all_notes).1 < origin(&require_label(&runner, "Notes")).1);
    let search_notes = require_label(&runner, "Search notes");
    assert!(origin(&require_label(&runner, "Notes")).0 < origin(&search_notes).0);

    let resizer = require_label(&runner, "Resize sidebar");
    assert_eq!(origin(&resizer).0, RAIL_WIDTH + SIDEBAR_WIDTH);
    assert_eq!(area(&resizer).width, SIDEBAR_RESIZER_HIT_WIDTH);
    assert_eq!(area(&resizer).height, VIEWPORT.1 - TOPBAR_HEIGHT);

    capture(&mut runner, "navigation-static.png");
}

#[test]
fn source_navigation_pointer_motion_proves_hover_drop_and_disabled_frames() {
    let fixture = FixtureVault::new();
    let root = fixture.path().clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        VIEWPORT.into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Projects");
    runner.sync_and_update();
    let back = require_label(&runner, "Retour");
    let before_back = background(&runner, "Retour");
    runner.move_cursor(center(&back));
    runner.sync_and_update();
    let after_back = background(&runner, "Retour");
    assert_ne!(
        before_back, after_back,
        "enabled nav hover must change background"
    );
    capture(&mut runner, "navigation-hover-back.png");

    let search = require_label(&runner, "Search");
    let toggle = require_label(&runner, "Hide sidebar");
    let before_toggle = background(&runner, "Hide sidebar");
    let search_point = center(&search);
    let toggle_point = center(&toggle);
    runner.press_cursor(search_point);
    runner.move_cursor(toggle_point);
    runner.sync_and_update();
    let drop_background = background(&runner, "Hide sidebar");
    assert_ne!(drop_background, background(&runner, "Search"));
    assert_ne!(
        before_toggle, drop_background,
        "rail drop target must be visible"
    );
    capture(&mut runner, "navigation-drop-target.png");
    runner.release_cursor(toggle_point);
    runner.sync_and_update();

    let back = require_label(&runner, "Retour");
    assert_eq!(background(&runner, "Retour"), before_back);
    assert_eq!(area(&back), Size2D::new(NAV_BUTTON_SIZE, NAV_BUTTON_SIZE));
    assert_ne!(origin(&back), (0., 0.));

    let forward = require_label(&runner, "Avancer");
    let opacity = Rect::try_downcast(forward.element().as_ref())
        .expect("disabled forward control must be a rect")
        .effect
        .and_then(|effect| effect.opacity);
    assert_eq!(opacity, Some(0.3));
    capture(&mut runner, "navigation-disabled-forward.png");
}
