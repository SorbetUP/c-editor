#[path = "../src/addon_worker.rs"]
mod addon_worker;

use addon_worker::{AddonWorker, AddonWorkerConfig, AddonWorkerError, WORKER_PROTOCOL};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const WORKING_ENTRY: &str = r#"
self.elephantAddon = {
  activate(api) {
    api.log.info('fixture activated');
    api.commands.register({
      id: 'fixture.worker.ping',
      title: 'Ping',
      run(payload) { return { pong: payload?.value ?? null }; }
    });
    return () => api.log.info('fixture disposed');
  },
  deactivate(api) { api.log.info('fixture deactivated'); }
};
"#;

const FAILING_ENTRY: &str = r#"
self.elephantAddon = {
  activate() { throw new Error('fixture activation failed'); }
};
"#;

struct Fixture {
    root: PathBuf,
    entry: PathBuf,
}

impl Fixture {
    fn new(source: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-addon-worker-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create worker fixture directory");
        let entry = root.join("main.js");
        fs::write(&entry, source).expect("write worker fixture entry");
        Self { root, entry }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn worker(fixture: &Fixture) -> AddonWorker {
    AddonWorker::spawn(AddonWorkerConfig::new(
        "node",
        &fixture.entry,
        "fixture.worker",
        json!({ "apiVersion": 1, "permissions": {}, "contributes": {} }),
    ))
    .expect("start real node worker")
}

fn has_event(events: &[Value], kind: &str, predicate: impl Fn(&Value) -> bool) -> bool {
    events
        .iter()
        .any(|event| event.get("type").and_then(Value::as_str) == Some(kind) && predicate(event))
}

#[test]
fn real_node_worker_activates_speaks_ndjson_and_stops_cleanly() {
    let fixture = Fixture::new(WORKING_ENTRY);
    let mut worker = worker(&fixture);

    assert!(worker.pid() > 0, "the API must own a real child process");
    assert_eq!(worker.ready_message()["protocol"], WORKER_PROTOCOL);
    assert_eq!(worker.ready_message()["runtime"], "node");

    let activation = worker.activate().expect("activate fixture addon");
    assert_eq!(activation["type"], "activation-result");
    assert_eq!(activation["ok"], true);
    assert!(has_event(worker.events(), "register-action", |event| {
        event["action"]["id"] == "fixture.worker.ping"
    }));
    assert!(has_event(worker.events(), "log", |event| {
        event["args"][0] == "fixture activated"
    }));

    let command = worker
        .request(
            "run-command",
            json!({ "commandId": "fixture.worker.ping", "payload": { "value": 7 } }),
        )
        .expect("run registered command");
    assert_eq!(command["type"], "command-result");
    assert_eq!(command["result"]["pong"], 7);

    let deactivation = worker.deactivate().expect("deactivate fixture addon");
    assert_eq!(deactivation["type"], "deactivation-result");
    assert_eq!(deactivation["ok"], true);
    assert!(!worker.is_running().expect("inspect stopped child"));
    assert!(has_event(worker.events(), "unregister-action", |event| {
        event["id"] == "fixture.worker.ping"
    }));
    assert!(has_event(worker.events(), "log", |event| {
        event["args"][0] == "fixture deactivated"
    }));
    assert!(has_event(worker.events(), "log", |event| {
        event["args"][0] == "fixture disposed"
    }));
}

#[test]
fn real_node_worker_returns_activation_errors_and_can_still_be_stopped() {
    let fixture = Fixture::new(FAILING_ENTRY);
    let mut worker = worker(&fixture);

    let error = worker
        .activate()
        .expect_err("activation failure must cross the protocol");
    match error {
        AddonWorkerError::Remote { operation, error } => {
            assert_eq!(operation, "activate");
            assert_eq!(error["message"], "fixture activation failed");
        }
        other => panic!("expected a remote activation error, got {other:?}"),
    }
    let deactivation = worker
        .deactivate()
        .expect("failed worker must still cleanly stop");
    assert_eq!(deactivation["ok"], true);
    assert!(!worker.is_running().expect("inspect stopped failed worker"));
}

#[test]
fn real_node_worker_brokers_rpc_without_faking_a_result() {
    let fixture = Fixture::new(
        r#"
    self.elephantAddon = { activate: async (api) => {
      const info = await api.app.info();
      api.log.info(info.name);
    }};
  "#,
    );
    let mut worker = worker(&fixture);
    let mut broker = |method: &str, params: &Value| -> Result<Value, String> {
        assert_eq!(method, "app.info");
        assert_eq!(params, &json!({}));
        Ok(json!({ "name": "ElephantNote", "addonApiVersion": 1 }))
    };
    let activation = worker
        .activate_with_broker(&mut broker)
        .expect("brokered activation");
    assert_eq!(activation["ok"], true);
    assert!(has_event(worker.events(), "rpc", |event| {
        event["method"] == "app.info"
    }));
    assert!(has_event(worker.events(), "log", |event| {
        event["args"][0] == "ElephantNote"
    }));
    worker
        .deactivate_with_broker(&mut broker)
        .expect("stop brokered worker");
}
