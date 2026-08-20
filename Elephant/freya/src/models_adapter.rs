//! Native Freya client for the official `elephant.open-models` package service.
//!
//! This is a transport adapter only. Model discovery, downloads, activation,
//! deletion and llama.cpp lifecycle remain owned by the pinned official addon
//! service. UI calls are executed on worker threads so Hugging Face and
//! llama-server operations cannot block Freya's render thread.

use crate::resource_locator;
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, String>;
const PROTOCOL: &str = "elephant-addon-service-v1";
const ADDON_ID: &str = "elephant.open-models";
const POLL_CANCEL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct ModelsClient {
    root: PathBuf,
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<Result<String>>,
    next_id: u64,
    healthy: bool,
}

impl ModelsClient {
    fn spawn(root: &Path) -> Result<Self> {
        let executable = resource_locator::native_service(
            "ELEPHANT_OPEN_MODELS_SERVICE",
            "open-models",
            "elephant-open-models-service",
        )?;
        let data_dir = root.join(".elephantnote/addons/data/elephant.open-models");
        fs::create_dir_all(&data_dir)
            .map_err(|error| format!("Create Open Models data directory: {error}"))?;
        let mut command = Command::new(&executable);
        command
            .env("ELEPHANT_VAULT_DIR", root)
            .env("ELEPHANT_ADDON_DATA_DIR", &data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if std::env::var_os("ELEPHANT_LLAMA_SERVER_PATH").is_none() {
            if let Ok(llama) = resource_locator::llama_server() {
                command.env("ELEPHANT_LLAMA_SERVER_PATH", llama);
            }
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("Start Open Models service {}: {error}", executable.display()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Open Models service stdin is unavailable".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Open Models service stdout is unavailable".to_owned())?;
        let (sender, responses) = mpsc::channel();
        thread::Builder::new()
            .name("elephant-open-models-stdout".to_owned())
            .spawn(move || {
                let mut stdout = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match stdout.read_line(&mut line) {
                        Ok(0) => {
                            let _ = sender.send(Err("Open Models service closed its stdout".to_owned()));
                            break;
                        }
                        Ok(_) => {
                            if sender.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.send(Err(format!("Read Open Models response: {error}")));
                            break;
                        }
                    }
                }
            })
            .map_err(|error| format!("Start Open Models response reader: {error}"))?;
        eprintln!(
            "[freya][models] action=service-start executable={} vault={}",
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
        self.request_inner(method, params, None)
    }

    fn request_cancellable(
        &mut self,
        method: &str,
        params: Value,
        generation: &AtomicU64,
        expected_generation: u64,
    ) -> Result<Value> {
        self.request_inner(method, params, Some((generation, expected_generation)))
    }

    fn request_inner(
        &mut self,
        method: &str,
        params: Value,
        cancellation: Option<(&AtomicU64, u64)>,
    ) -> Result<Value> {
        if !self.healthy {
            return Err("Open Models service is unhealthy".to_owned());
        }
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
            self.terminate("write-error");
            return Err(format!("Write Open Models request {method}: {error}"));
        }

        let deadline = Instant::now() + request_timeout(method);
        let line = loop {
            if cancellation.is_some_and(|(generation, expected)| {
                generation.load(Ordering::Acquire) != expected
            }) {
                self.terminate("cancelled");
                return Err(format!("Open Models request {method} was cancelled"));
            }
            let now = Instant::now();
            if now >= deadline {
                self.terminate("timeout");
                return Err(format!(
                    "Open Models request {method} timed out after {}s",
                    request_timeout(method).as_secs()
                ));
            }
            let remaining = deadline.saturating_duration_since(now);
            let wait = remaining.min(POLL_CANCEL_INTERVAL);
            match self.responses.recv_timeout(wait) {
                Ok(Ok(line)) => break line,
                Ok(Err(error)) => {
                    self.terminate("reader-error");
                    return Err(format!("Open Models request {method}: {error}"));
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    self.terminate("reader-disconnected");
                    return Err(format!(
                        "Open Models response reader disconnected during {method}"
                    ));
                }
            }
        };

        let envelope: Value = match serde_json::from_str(line.trim()) {
            Ok(value) => value,
            Err(error) => {
                self.terminate("invalid-json");
                return Err(format!("Parse Open Models response {method}: {error}"));
            }
        };
        if envelope.get("protocol").and_then(Value::as_str) != Some(PROTOCOL) {
            self.terminate("protocol-mismatch");
            return Err(format!(
                "Open Models response {method} used an unexpected protocol"
            ));
        }
        if envelope.get("id").and_then(Value::as_u64) != Some(id) {
            self.terminate("id-mismatch");
            return Err(format!(
                "Open Models response {method} did not match request {id}"
            ));
        }
        if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(envelope
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Open Models service returned an error")
                .to_owned());
        }
        Ok(envelope.get("result").cloned().unwrap_or(Value::Null))
    }

    fn terminate(&mut self, reason: &str) {
        if !self.healthy {
            return;
        }
        self.healthy = false;
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[freya][models] action=service-terminate reason={reason}");
    }
}

impl Drop for ModelsClient {
    fn drop(&mut self) {
        if self.healthy {
            let _ = self.request("service.stop", json!({}));
        }
        self.terminate("drop");
    }
}

fn request_timeout(method: &str) -> Duration {
    match method {
        // The pinned Open Models service streams the whole GGUF download before
        // replying, so this RPC legitimately needs a long but finite deadline.
        "models.download" => Duration::from_secs(30 * 60),
        // Hugging Face requests inside the service use a 180s HTTP timeout.
        "models.search" => Duration::from_secs(200),
        _ => Duration::from_secs(30),
    }
}

#[derive(Debug)]
struct AsyncOutcome {
    generation: u64,
    method: String,
    result: Result<Value>,
}

#[derive(Clone)]
pub(crate) struct ModelsService {
    connection: Arc<Mutex<Option<ModelsClient>>>,
    outcome: Arc<Mutex<Option<AsyncOutcome>>>,
    generation: Arc<AtomicU64>,
}

impl Default for ModelsService {
    fn default() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            outcome: Arc::new(Mutex::new(None)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl std::fmt::Debug for ModelsService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ModelsService")
    }
}

impl PartialEq for ModelsService {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.connection, &other.connection)
    }
}

impl Eq for ModelsService {}

impl ModelsService {
    pub(crate) fn call(&self, root: &Path, method: &str, params: Value) -> Result<Value> {
        call_connection(&self.connection, root, method, params, None)
    }

    pub(crate) fn begin(
        &self,
        root: PathBuf,
        method: impl Into<String>,
        params: Value,
    ) -> Result<u64> {
        let method = method.into();
        let generation = self.generation.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = None;
        }
        let connection = Arc::clone(&self.connection);
        let outcome = Arc::clone(&self.outcome);
        let generation_state = Arc::clone(&self.generation);
        thread::Builder::new()
            .name(format!("elephant-models-{generation}"))
            .spawn(move || {
                let result = call_connection(
                    &connection,
                    &root,
                    &method,
                    params,
                    Some((&generation_state, generation)),
                );
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(AsyncOutcome {
                        generation,
                        method,
                        result,
                    });
                }
            })
            .map_err(|error| format!("Unable to start Open Models worker: {error}"))?;
        Ok(generation)
    }

    pub(crate) fn poll(&self) -> Option<(String, Result<Value>)> {
        let outcome = self.outcome.lock().ok()?.take()?;
        let generation = self.generation.load(Ordering::Acquire);
        (outcome.generation == generation).then_some((outcome.method, outcome.result))
    }

    pub(crate) fn cancel_ui(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = None;
        }
    }

    pub(crate) fn reset_for_vault_switch(&self) {
        self.cancel_ui();
        if let Ok(mut connection) = self.connection.lock() {
            *connection = None;
        }
    }
}

fn call_connection(
    connection: &Arc<Mutex<Option<ModelsClient>>>,
    root: &Path,
    method: &str,
    params: Value,
    cancellation: Option<(&AtomicU64, u64)>,
) -> Result<Value> {
    let mut connection = connection
        .lock()
        .map_err(|_| "Open Models service lock is poisoned".to_owned())?;
    let replace = connection
        .as_ref()
        .map(|client| client.root != root || !client.healthy)
        .unwrap_or(true);
    if replace {
        *connection = Some(ModelsClient::spawn(root)?);
    }
    let result = {
        let client = connection
            .as_mut()
            .ok_or_else(|| "Open Models service connection is unavailable".to_owned())?;
        match cancellation {
            Some((generation, expected)) => {
                client.request_cancellable(method, params, generation, expected)
            }
            None => client.request(method, params),
        }
    };
    let unhealthy = connection
        .as_ref()
        .map(|client| !client.healthy)
        .unwrap_or(false);
    if method == "service.stop" || unhealthy || result.is_err() && client_exited(connection.as_mut()) {
        *connection = None;
    }
    result
}

fn client_exited(client: Option<&mut ModelsClient>) -> bool {
    client
        .and_then(|client| client.child.try_wait().ok().flatten())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn request_deadlines_are_finite() {
        assert_eq!(request_timeout("models.list"), Duration::from_secs(30));
        assert_eq!(request_timeout("models.search"), Duration::from_secs(200));
        assert_eq!(request_timeout("models.download"), Duration::from_secs(30 * 60));
    }

    #[test]
    fn official_service_lists_and_selects_real_gguf_files() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-open-models-{stamp}"));
        let models = root.join(".elephantnote/addons/data/elephant.open-models/models");
        fs::create_dir_all(&models).unwrap();
        fs::write(models.join("tiny.gguf"), b"GGUF-fixture").unwrap();
        let service = ModelsService::default();
        let list = service.call(&root, "models.list", json!({})).unwrap();
        assert_eq!(list["models"].as_array().unwrap().len(), 1);
        let selected = service
            .call(&root, "models.activate", json!({"id":"tiny.gguf"}))
            .unwrap();
        assert_eq!(selected["fileName"], "tiny.gguf");
        let active = service.call(&root, "models.active", json!({})).unwrap();
        assert_eq!(active["fileName"], "tiny.gguf");
        let _ = service.call(&root, "models.deactivate", json!({}));
        let _ = service.call(&root, "service.stop", json!({}));
        let _ = fs::remove_dir_all(root);
    }
}