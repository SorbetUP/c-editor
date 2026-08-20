//! Functional adapter for the official code-execution addon service.
//!
//! The editor view only supplies a language and source text. Process
//! discovery, the versioned service protocol, bounded execution and result
//! normalization stay here so the renderer never becomes an interpreter.

use crate::resource_locator;
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

const PROTOCOL: &str = "elephant-addon-service-v1";
const ADDON_ID: &str = "elephant.code-execution";
const OUTPUT_LINE_LIMIT: u64 = 200;
const TIMEOUT_MS: u64 = 15_000;
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_WAIT: Duration = Duration::from_secs(125);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CodeExecutionState {
    pub block_id: Option<muya_core::NodeId>,
    pub language: String,
    pub running: bool,
    pub output: String,
    pub exit_code: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CodeExecutionResult {
    pub output: String,
    pub exit_code: String,
}

struct ServiceClient {
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<Result<String, String>>,
    next_id: u64,
    healthy: bool,
}

impl ServiceClient {
    fn start(root: &Path) -> Result<Self, String> {
        let executable = service_executable()?;
        let data_dir = root.join(".elephantnote/addons/data/elephant.code-execution");
        fs::create_dir_all(&data_dir)
            .map_err(|error| format!("Create code execution data directory: {error}"))?;
        let mut child = Command::new(&executable)
            .env("ELEPHANT_VAULT_DIR", root)
            .env("ELEPHANT_ADDON_DATA_DIR", &data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                format!(
                    "Start code execution service {}: {error}",
                    executable.display()
                )
            })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Code execution stdin is unavailable".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Code execution stdout is unavailable".to_owned())?;
        let (sender, responses) = mpsc::channel();
        thread::Builder::new()
            .name("elephant-code-execution-stdout".to_owned())
            .spawn(move || {
                let mut stdout = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match stdout.read_line(&mut line) {
                        Ok(0) => {
                            let _ = sender.send(Err(
                                "Code execution service closed its stdout".to_owned(),
                            ));
                            break;
                        }
                        Ok(_) => {
                            if sender.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.send(Err(format!(
                                "Read code execution response: {error}"
                            )));
                            break;
                        }
                    }
                }
            })
            .map_err(|error| format!("Start code execution response reader: {error}"))?;
        eprintln!(
            "[freya][code-execution] action=service-start executable={} vault={}",
            executable.display(),
            root.display()
        );
        Ok(Self {
            child,
            stdin,
            responses,
            next_id: 0,
            healthy: true,
        })
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        if !self.healthy {
            return Err("Code execution service is unhealthy".to_owned());
        }
        self.next_id = self.next_id.saturating_add(1);
        let request_id = self.next_id;
        let request = json!({
            "protocol": PROTOCOL,
            "id": request_id,
            "addonId": ADDON_ID,
            "method": method,
            "params": params,
        });
        if let Err(error) = writeln!(self.stdin, "{request}").and_then(|_| self.stdin.flush()) {
            self.terminate();
            return Err(format!("Write code execution request {method}: {error}"));
        }
        let line = match self.responses.recv_timeout(REQUEST_TIMEOUT) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => {
                self.terminate();
                return Err(format!("Code execution request {method}: {error}"));
            }
            Err(RecvTimeoutError::Timeout) => {
                self.terminate();
                return Err(format!(
                    "Code execution request {method} timed out after {}s",
                    REQUEST_TIMEOUT.as_secs()
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.terminate();
                return Err(format!(
                    "Code execution response reader disconnected during {method}"
                ));
            }
        };
        let envelope: Value = match serde_json::from_str(line.trim()) {
            Ok(value) => value,
            Err(error) => {
                self.terminate();
                return Err(format!("Parse code execution response {method}: {error}"));
            }
        };
        if envelope.get("protocol").and_then(Value::as_str) != Some(PROTOCOL) {
            self.terminate();
            return Err(format!(
                "Code execution response {method} used an unexpected protocol"
            ));
        }
        if envelope.get("id").and_then(Value::as_u64) != Some(request_id) {
            self.terminate();
            return Err(format!(
                "Code execution response {method} did not match request {request_id}"
            ));
        }
        if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(envelope
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Code execution service returned an error")
                .to_owned());
        }
        Ok(envelope.get("result").cloned().unwrap_or(Value::Null))
    }

    fn terminate(&mut self) {
        if !self.healthy {
            return;
        }
        self.healthy = false;
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[freya][code-execution] action=service-terminate");
    }
}

impl Drop for ServiceClient {
    fn drop(&mut self) {
        self.terminate();
    }
}

pub(super) fn execute(
    root: &Path,
    language: &str,
    code: &str,
) -> Result<CodeExecutionResult, String> {
    let interpreter = interpreter(language);
    let mut client = ServiceClient::start(root)?;
    let status = client.request(
        "interpreter.status",
        json!({
            "executable": interpreter.executable,
            "args": interpreter.args,
        }),
    )?;
    if status.get("available").and_then(Value::as_bool) != Some(true) {
        return Err(status
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("The selected interpreter is unavailable")
            .to_owned());
    }
    let started = client.request(
        "execute",
        json!({
            "executable": interpreter.executable,
            "args": interpreter.args,
            "code": code,
            "cwd": "",
            "outputLineLimit": OUTPUT_LINE_LIMIT,
            "timeoutMs": TIMEOUT_MS,
        }),
    )?;
    let execution_id = started
        .get("executionId")
        .and_then(Value::as_str)
        .ok_or_else(|| "Code execution service returned no execution id".to_owned())?
        .to_owned();
    let deadline = Instant::now() + MAX_WAIT;
    let snapshot = loop {
        if Instant::now() >= deadline {
            let _ = client.request("execution.cancel", json!({ "executionId": execution_id }));
            return Err("Code execution did not finish before the safety deadline".to_owned());
        }
        let snapshot =
            client.request("execution.status", json!({ "executionId": execution_id }))?;
        if snapshot.get("running").and_then(Value::as_bool) != Some(true) {
            break snapshot;
        }
        thread::sleep(POLL_INTERVAL);
    };
    let _ = client.request("execution.forget", json!({ "executionId": execution_id }));
    if let Some(error) = snapshot
        .get("error")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        return Err(error.to_owned());
    }
    let result = snapshot.get("result").cloned().unwrap_or(Value::Null);
    let stdout = result
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let stderr = result
        .get("stderr")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let output = [stdout, stderr]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let exit_code = if result.get("interrupted").and_then(Value::as_bool) == Some(true) {
        "interrupted".to_owned()
    } else if result.get("timedOut").and_then(Value::as_bool) == Some(true) {
        "timeout".to_owned()
    } else {
        result
            .get("code")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .to_string()
    };
    Ok(CodeExecutionResult { output, exit_code })
}

struct Interpreter {
    executable: &'static str,
    args: &'static [&'static str],
}

fn interpreter(language: &str) -> Interpreter {
    match language.to_ascii_lowercase().as_str() {
        "javascript" | "js" | "jsx" | "node" | "mjs" | "cjs" => Interpreter {
            executable: "node",
            args: &["-"],
        },
        "shell" | "sh" | "bash" | "zsh" => Interpreter {
            executable: "bash",
            args: &["-s"],
        },
        _ => Interpreter {
            executable: "python3",
            args: &["-"],
        },
    }
}

fn service_executable() -> Result<PathBuf, String> {
    resource_locator::native_service(
        "ELEPHANT_CODE_EXECUTION_SERVICE",
        "code-execution",
        "elephant-code-execution",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn language_aliases_use_the_official_interpreter_contract() {
        assert_eq!(interpreter("py").executable, "python3");
        assert_eq!(interpreter("js").executable, "node");
        assert_eq!(interpreter("bash").args, &["-s"]);
    }

    #[test]
    fn real_service_executes_a_python_block() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = env::temp_dir().join(format!("elephant-freya-code-{stamp}"));
        fs::create_dir_all(&root).expect("temporary vault");
        let result = execute(&root, "python", "print('freya code execution')")
            .expect("real service execution");
        assert_eq!(result.exit_code, "0");
        assert!(result.output.contains("freya code execution"));
        let _ = fs::remove_dir_all(root);
    }
}