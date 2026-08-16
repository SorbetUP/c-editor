//! Functional adapter for the official code-execution addon service.
//!
//! The editor view only supplies a language and source text.  Process
//! discovery, the versioned service protocol, bounded execution and result
//! normalization stay here so the renderer never becomes an interpreter.

use serde_json::{json, Value};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PROTOCOL: &str = "elephant-addon-service-v1";
const ADDON_ID: &str = "elephant.code-execution";
const OUTPUT_LINE_LIMIT: u64 = 200;
const TIMEOUT_MS: u64 = 15_000;
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_WAIT: Duration = Duration::from_secs(125);

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
    stdout: BufReader<ChildStdout>,
    next_id: u64,
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
        eprintln!(
            "[freya][code-execution] action=service-start executable={} vault={}",
            executable.display(),
            root.display()
        );
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 0,
        })
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id = self.next_id.saturating_add(1);
        let request = json!({
            "protocol": PROTOCOL,
            "id": self.next_id,
            "addonId": ADDON_ID,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{request}")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("Write code execution request {method}: {error}"))?;
        let mut line = String::new();
        if self
            .stdout
            .read_line(&mut line)
            .map_err(|error| format!("Read code execution response {method}: {error}"))?
            == 0
        {
            let status = self
                .child
                .try_wait()
                .ok()
                .flatten()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown exit".to_owned());
            return Err(format!(
                "Code execution service exited before responding ({status})"
            ));
        }
        let envelope: Value = serde_json::from_str(line.trim())
            .map_err(|error| format!("Parse code execution response {method}: {error}"))?;
        if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(envelope
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Code execution service returned an error")
                .to_owned());
        }
        Ok(envelope.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl Drop for ServiceClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[freya][code-execution] action=service-stop");
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
    if let Some(path) = env::var_os("ELEPHANT_CODE_EXECUTION_SERVICE") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path.clone()).ok_or_else(|| {
            format!(
                "ELEPHANT_CODE_EXECUTION_SERVICE is not a file: {}",
                path.display()
            )
        });
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
        return Err("No packaged Code execution service is available for this platform".to_owned());
    };
    let file = if cfg!(target_os = "windows") {
        "elephant-code-execution.exe"
    } else {
        "elephant-code-execution"
    };
    [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../addons/official/code-execution/native")
            .join(platform)
            .join(file),
        PathBuf::from("resources/official-addons/official/code-execution/native")
            .join(platform)
            .join(file),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| "Packaged Elephant Code execution service executable was not found".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
