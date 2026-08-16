//! Freya adapter for the package-owned Elephant Sync service.
//!
//! The service speaks the versioned `elephant-addon-service-v1` protocol over
//! stdin/stdout.  Freya keeps one explicit connection per shell state so the
//! Iroh identity and endpoint survive individual UI actions.

use serde_json::{json, Value};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

type Result<T> = std::result::Result<T, String>;
const PROTOCOL: &str = "elephant-addon-service-v1";
const ADDON_ID: &str = "elephant.sync";

#[derive(Debug)]
struct SyncClient {
    root: PathBuf,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
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
        eprintln!(
            "[freya][sync] action=service-start executable={} vault={}",
            executable.display(),
            root.display()
        );
        Ok(Self {
            root: root.to_path_buf(),
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 0,
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
        writeln!(self.stdin, "{request}")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("Write Sync request {method}: {error}"))?;
        let mut line = String::new();
        let read = self
            .stdout
            .read_line(&mut line)
            .map_err(|error| format!("Read Sync response {method}: {error}"))?;
        if read == 0 {
            let status = self
                .child
                .try_wait()
                .ok()
                .flatten()
                .map(|status| status.to_string())
                .unwrap_or_else(|| "unknown exit".to_owned());
            return Err(format!("Sync service exited before responding ({status})"));
        }
        let envelope: Value = serde_json::from_str(line.trim())
            .map_err(|error| format!("Parse Sync response {method}: {error}"))?;
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

impl Drop for SyncClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[freya][sync] action=service-stop");
    }
}

#[derive(Clone, Debug)]
pub(super) struct SyncState {
    pub status: Option<Value>,
    pub invitation: String,
    pub message: Option<String>,
    pub error: Option<String>,
    pub busy: bool,
    connection: Arc<Mutex<Option<SyncClient>>>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            status: None,
            invitation: String::new(),
            message: None,
            error: None,
            busy: false,
            connection: Arc::new(Mutex::new(None)),
        }
    }
}

impl SyncState {
    pub(super) fn call(&self, root: &Path, method: &str, params: Value) -> Result<Value> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "Sync service lock is poisoned".to_owned())?;
        let replace = connection
            .as_ref()
            .map(|client| client.root != root)
            .unwrap_or(true);
        if replace {
            *connection = Some(SyncClient::spawn(root)?);
        }
        connection
            .as_mut()
            .ok_or_else(|| "Sync service connection is unavailable".to_owned())?
            .request(method, params)
    }
}

fn service_executable() -> Result<PathBuf> {
    if let Some(path) = env::var_os("ELEPHANT_SYNC_SERVICE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "ELEPHANT_SYNC_SERVICE does not point to a file: {}",
            path.display()
        ));
    }

    let platform = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-aarch64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x86_64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x86_64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x86_64"
    } else {
        return Err("No packaged Sync service is available for this platform".to_owned());
    };
    let file_name = if cfg!(target_os = "windows") {
        "elephant-sync-service.exe"
    } else {
        "elephant-sync-service"
    };
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../addons/official/sync/native")
            .join(platform)
            .join(file_name),
        PathBuf::from("resources/official-addons/official/sync/native")
            .join(platform)
            .join(file_name),
    ];
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "Packaged Elephant Sync service executable was not found".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
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
}
