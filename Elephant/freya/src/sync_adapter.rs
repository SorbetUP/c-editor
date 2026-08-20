//! Freya adapter for the package-owned Elephant Sync service.
//!
//! The service speaks the versioned `elephant-addon-service-v1` protocol over
//! stdin/stdout. Freya keeps one explicit connection per shell state so the
//! Iroh identity and endpoint survive individual UI actions. UI-triggered
//! requests are executed on a worker thread. A dedicated stdout reader feeds a
//! timed channel so a hung sidecar cannot block a worker forever; unhealthy
//! connections are dropped and recreated on the next action.

use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::resource_locator;

type Result<T> = std::result::Result<T, String>;
const PROTOCOL: &str = "elephant-addon-service-v1";
const ADDON_ID: &str = "elephant.sync";
const DEFAULT_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const SYNC_RUN_RESPONSE_TIMEOUT: Duration = Duration::from_secs(180);

type ResponseLine = std::result::Result<String, String>;

#[derive(Debug)]
struct SyncClient {
    root: PathBuf,
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<ResponseLine>,
    next_id: u64,
    healthy: bool,
}

impl SyncClient {
    fn spawn(root: &Path) -> Result<Self> {
        let executable = service_executable()?;
        let data_dir = root.join(".elephantnote/addons/data/elephant.sync");
        fs::create_dir_all(&data_dir).map_err(|error| {
            format!("Create Sync data directory {}: {error}", data_dir.display())
        })?;
        let mut child = Command::new(&executable)
            .env("ELEPHANT_VAULT_DIR", root)
            .env("ELEPHANT_ADDON_DATA_DIR", &data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("Start Sync service {}: {error}", executable.display()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Sync service stdin is unavailable".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Sync service stdout is unavailable".to_owned())?;
        let responses = spawn_stdout_reader(stdout)?;
        eprintln!(
            "[freya][sync] action=service-start executable={} vault={}",
            executable.display(),
            root.display()
        );
        Ok(Self {
            root: root.to_path_buf(),
            child,
            stdin,
            responses,
            next_id: 0,
            healthy: true,
        })
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        let request = json!({
            "protocol": PROTOCOL,
            "id": id,
            "addonId": ADDON_ID,
            "method": method,
            "params": params,
        });
        if let Err(error) = writeln!(self.stdin, "{request}").and_then(|_| self.stdin.flush()) {
            self.healthy = false;
            return Err(format!("Write Sync request {method}: {error}"));
        }

        let timeout = if method == "sync.run" {
            SYNC_RUN_RESPONSE_TIMEOUT
        } else {
            DEFAULT_RESPONSE_TIMEOUT
        };
        let line = match self.responses.recv_timeout(timeout) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => {
                self.healthy = false;
                return Err(format!("Read Sync response {method}: {error}"));
            }
            Err(RecvTimeoutError::Timeout) => {
                self.healthy = false;
                let _ = self.child.kill();
                let _ = self.child.wait();
                return Err(format!(
                    "Sync service timed out while waiting for {method} after {} seconds",
                    timeout.as_secs()
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.healthy = false;
                let status = self
                    .child
                    .try_wait()
                    .ok()
                    .flatten()
                    .map(|status| status.to_string())
                    .unwrap_or_else(|| "unknown exit".to_owned());
                return Err(format!(
                    "Sync service exited before responding to {method} ({status})"
                ));
            }
        };
        let envelope: Value = match serde_json::from_str(line.trim()) {
            Ok(value) => value,
            Err(error) => {
                self.healthy = false;
                return Err(format!("Parse Sync response {method}: {error}"));
            }
        };
        let response_id = envelope.get("id").and_then(Value::as_u64);
        if response_id.is_some() && response_id != Some(id) {
            self.healthy = false;
            return Err(format!(
                "Sync service response id mismatch for {method}: expected {id}, got {response_id:?}"
            ));
        }
        if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(envelope
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Sync service returned an error")
                .to_owned());
        }
        Ok(envelope.get("result").cloned().unwrap_or(Value::Null))
    }
}

fn spawn_stdout_reader(stdout: ChildStdout) -> Result<Receiver<ResponseLine>> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("elephant-sync-stdout".to_owned())
        .spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if sender.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(error.to_string()));
                        break;
                    }
                }
            }
        })
        .map_err(|error| format!("Start Sync stdout reader: {error}"))?;
    Ok(receiver)
}

impl Drop for SyncClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[freya][sync] action=service-stop");
    }
}

#[derive(Debug)]
struct AsyncOutcome {
    generation: u64,
    method: String,
    result: Result<Value>,
}

#[derive(Clone, Debug)]
pub(super) struct SyncState {
    pub status: Option<Value>,
    pub invitation: String,
    pub message: Option<String>,
    pub error: Option<String>,
    pub busy: bool,
    generation: u64,
    connection: Arc<Mutex<Option<SyncClient>>>,
    outcome: Arc<Mutex<Option<AsyncOutcome>>>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            status: None,
            invitation: String::new(),
            message: None,
            error: None,
            busy: false,
            generation: 0,
            connection: Arc::new(Mutex::new(None)),
            outcome: Arc::new(Mutex::new(None)),
        }
    }
}

impl SyncState {
    /// Blocking boundary retained for focused adapter tests and non-UI callers.
    /// The transport itself is bounded by `recv_timeout`, so this cannot wait
    /// forever even outside the UI worker path.
    pub(super) fn call(&self, root: &Path, method: &str, params: Value) -> Result<Value> {
        call_connection(&self.connection, root, method, params)
    }

    pub(super) fn begin_call(
        &mut self,
        root: PathBuf,
        method: impl Into<String>,
        params: Value,
    ) -> Result<u64> {
        if self.busy {
            return Err("A Sync action is already running.".to_owned());
        }
        self.generation = self.generation.saturating_add(1);
        let generation = self.generation;
        let method = method.into();
        self.busy = true;
        self.error = None;
        self.message = None;
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = None;
        }

        let connection = Arc::clone(&self.connection);
        let outcome = Arc::clone(&self.outcome);
        let method_for_worker = method.clone();
        thread::Builder::new()
            .name(format!("elephant-sync-{generation}"))
            .spawn(move || {
                let result = call_connection(&connection, &root, &method_for_worker, params);
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(AsyncOutcome {
                        generation,
                        method: method_for_worker,
                        result,
                    });
                }
            })
            .map_err(|error| {
                self.busy = false;
                format!("Unable to start Sync worker: {error}")
            })?;
        Ok(generation)
    }

    pub(super) fn poll_call(&mut self) -> Option<(String, Result<Value>)> {
        let outcome = self.outcome.lock().ok()?.take()?;
        if outcome.generation != self.generation {
            return None;
        }
        self.busy = false;
        Some((outcome.method, outcome.result))
    }

    /// Cancel the renderer's interest in the current request. The transport is
    /// independently bounded by a timeout, and a timed-out connection is killed
    /// and discarded before the next action.
    pub(super) fn cancel_pending(&mut self) {
        self.generation = self.generation.saturating_add(1);
        self.busy = false;
        self.message = Some("Sync action cancelled in the UI.".to_owned());
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = None;
        }
    }
}

fn call_connection(
    connection: &Arc<Mutex<Option<SyncClient>>>,
    root: &Path,
    method: &str,
    params: Value,
) -> Result<Value> {
    let mut connection = connection
        .lock()
        .map_err(|_| "Sync service lock is poisoned".to_owned())?;
    let replace = connection
        .as_ref()
        .map(|client| client.root != root || !client.healthy)
        .unwrap_or(true);
    if replace {
        *connection = Some(SyncClient::spawn(root)?);
    }
    let (result, healthy) = {
        let client = connection
            .as_mut()
            .ok_or_else(|| "Sync service connection is unavailable".to_owned())?;
        let result = client.request(method, params);
        (result, client.healthy)
    };
    if !healthy {
        *connection = None;
    }
    result
}

fn service_executable() -> Result<PathBuf> {
    resource_locator::native_service(
        "ELEPHANT_SYNC_SERVICE",
        "sync",
        "elephant-sync-service",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn service_protocol_scan_reads_the_real_vault() {
        let root = env::temp_dir().join(format!(
            "elephant-freya-sync-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("vault root");
        fs::write(root.join("Alpha.md"), "# Alpha\n").expect("note");
        let state = SyncState::default();
        let result = state
            .call(&root, "sync.scan", json!({}))
            .expect("real Sync service scan");
        assert_eq!(result["owner"], ADDON_ID);
        assert_eq!(result["files"], 1);
        drop(state);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn async_call_does_not_block_caller() {
        let root = env::temp_dir().join(format!(
            "elephant-freya-sync-async-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("vault root");
        let mut state = SyncState::default();
        state
            .begin_call(root.clone(), "sync.scan", json!({}))
            .unwrap();
        assert!(state.busy);
        for _ in 0..200 {
            if let Some((_method, result)) = state.poll_call() {
                assert!(result.is_ok());
                let _ = fs::remove_dir_all(root);
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("async Sync worker did not complete in time");
    }
}
