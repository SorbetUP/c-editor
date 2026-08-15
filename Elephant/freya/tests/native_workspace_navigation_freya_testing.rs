use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-native-workspaces-{stamp}"));
        fs::create_dir_all(root.join(".elephantnote/addons/packages/elephant.graph"))
            .expect("create addon package directory");
        fs::create_dir_all(root.join(".elephantnote/addons/packages/elephant.calendar"))
            .expect("create addon package directory");
        fs::create_dir_all(root.join(".elephantnote/addons/packages/elephant.dashboard"))
            .expect("create addon package directory");
        fs::create_dir_all(root.join(".elephantnote/addons/packages/elephant.ai-chat"))
            .expect("create addon package directory");
        fs::create_dir_all(root.join(".elephantnote/addons/packages/elephant.open-models"))
            .expect("create addon package directory");
        fs::create_dir_all(root.join(".elephantnote/models")).expect("create models directory");
        fs::write(
            root.join(".elephantnote/models/tiny.gguf"),
            b"native model fixture",
        )
        .expect("write model fixture");
        fs::write(
            root.join(".elephantnote/addons/registry.json"),
            serde_json::to_vec_pretty(&json!({
                "version": 1,
                "addons": {
                    "elephant.graph": {
                        "manifest": {
                            "id": "elephant.graph",
                            "name": "Graph",
                            "version": "1.3.0",
                            "runtime": { "type": "javascript-worker", "entry": "main.v2.js" },
                            "contributes": { "views": true }
                        },
                        "enabled": true
                    },
                    "elephant.calendar": {
                        "manifest": {
                            "id": "elephant.calendar",
                            "name": "Calendar",
                            "version": "1.3.0",
                            "runtime": { "type": "javascript-worker", "entry": "main.js" },
                            "contributes": { "views": true }
                        },
                        "enabled": true
                    },
                    "elephant.dashboard": {
                        "manifest": {
                            "id": "elephant.dashboard",
                            "name": "Dashboard",
                            "version": "1.0.1",
                            "runtime": { "type": "javascript-worker", "entry": "main.js" }
                        },
                        "enabled": true
                    },
                    "elephant.ai-chat": {
                        "manifest": {
                            "id": "elephant.ai-chat",
                            "name": "Chat",
                            "version": "1.0.0",
                            "runtime": { "type": "javascript-worker", "entry": "main.js" }
                        },
                        "enabled": true
                    },
                    "elephant.open-models": {
                        "manifest": {
                            "id": "elephant.open-models",
                            "name": "Models",
                            "version": "1.0.0",
                            "runtime": { "type": "javascript-worker", "entry": "main.js" }
                        },
                        "enabled": true
                    }
                }
            }))
            .expect("serialize addon registry"),
        )
        .expect("write addon registry");
        fs::write(
            root.join(".elephantnote/calendar.json"),
            br#"{
                "version": 1,
                "updatedAt": "2026-08-16T09:00:00Z",
                "events": [{
                    "id": "event-1",
                    "title": "Team sync",
                    "startsAt": "2026-08-16T10:30:00Z",
                    "endsAt": "2026-08-16T11:00:00Z",
                    "source": "fixture"
                }]
            }"#,
        )
        .expect("write calendar fixture");
        fs::write(
            root.join("Alpha.md"),
            "# Alpha\n\nA native workspace fixture.\n",
        )
        .expect("write fixture note");
        Self { root }
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
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
    let area = node(runner, label).layout().area;
    runner.click_cursor((
        ((area.min_x() + area.max_x()) / 2.) as f64,
        ((area.min_y() + area.max_y()) / 2.) as f64,
    ));
}

#[test]
fn enabled_official_workspace_views_are_real_clickable_native_routes() {
    let fixture = FixtureVault::new();
    let root = fixture.root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );
    let evidence = std::env::temp_dir().join("freya-native-workspaces-evidence");
    fs::create_dir_all(&evidence).expect("create evidence directory");

    click(&mut runner, "Graph");
    runner.sync_and_update();
    assert_eq!(
        node(&runner, "Graph workspace").layout().area.size.width > 0.,
        true
    );
    assert_eq!(
        node(&runner, "Refresh graph").layout().area.size.width > 0.,
        true
    );
    runner.render_to_file(&evidence.join("graph.png"));

    click(&mut runner, "Calendar");
    runner.sync_and_update();
    assert_eq!(
        node(&runner, "Calendar workspace").layout().area.size.width > 0.,
        true
    );
    assert_eq!(
        node(&runner, "Calendar event Team sync")
            .layout()
            .area
            .size
            .width
            > 0.,
        true
    );
    runner.render_to_file(&evidence.join("calendar.png"));

    click(&mut runner, "Dashboard");
    runner.sync_and_update();
    assert!(fixture.root.join(".elephantnote/Dashboard.md").is_file());
    assert!(node(&runner, "Dashboard").layout().area.size.width > 0.);
    runner.render_to_file(&evidence.join("dashboard.png"));

    click(&mut runner, "Chat");
    runner.sync_and_update();
    assert!(node(&runner, "Chat workspace").layout().area.size.width > 0.);
    assert!(
        node(&runner, "Save Chat provider settings")
            .layout()
            .area
            .size
            .width
            > 0.
    );
    runner.render_to_file(&evidence.join("chat.png"));

    click(&mut runner, "Models");
    runner.sync_and_update();
    assert!(node(&runner, "Models workspace").layout().area.size.width > 0.);
    assert!(
        node(&runner, "Local model tiny.gguf")
            .layout()
            .area
            .size
            .width
            > 0.
    );
    runner.render_to_file(&evidence.join("models.png"));
}
