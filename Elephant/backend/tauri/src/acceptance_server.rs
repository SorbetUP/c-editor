use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
pub struct AcceptanceState {
    pending: Mutex<HashMap<String, mpsc::Sender<Result<Value, String>>>>,
    ready: AtomicBool,
}

#[derive(Debug, Deserialize)]
struct CommandRequest {
    command: String,
    #[serde(default)]
    args: Vec<Value>,
}

#[derive(Clone, Debug, Serialize)]
struct CommandEvent<'a> {
    request_id: &'a str,
    command: &'a str,
    args: &'a [Value],
}

fn response(stream: &mut TcpStream, status: &str, body: Value) {
    let bytes = body.to_string().into_bytes();
    let header = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&bytes);
}

fn read_request(stream: &mut TcpStream) -> Result<(String, Vec<u8>), String> {
    let mut data = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end;
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("connection closed before request headers".into());
        }
        data.extend_from_slice(&buffer[..read]);
        if let Some(index) = data.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = index + 4;
            break;
        }
        if data.len() > 64 * 1024 {
            return Err("request headers too large".into());
        }
    }
    let header = String::from_utf8_lossy(&data[..header_end]);
    let request_line = header.lines().next().unwrap_or_default().to_string();
    let content_length = header
        .lines()
        .find_map(|line| {
            line.strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))
        })
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    while data.len() - header_end < content_length {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("connection closed before request body".into());
        }
        data.extend_from_slice(&buffer[..read]);
    }
    Ok((
        request_line,
        data[header_end..header_end + content_length].to_vec(),
    ))
}

fn handle_connection(
    mut stream: TcpStream,
    app: AppHandle,
    state: State<'_, AcceptanceState>,
    sequence: u64,
) {
    let Ok((request_line, body)) = read_request(&mut stream) else {
        response(
            &mut stream,
            "400 Bad Request",
            serde_json::json!({"ok": false, "error": "invalid HTTP request"}),
        );
        return;
    };
    if request_line.starts_with("GET /health") {
        response(
            &mut stream,
            "200 OK",
            serde_json::json!({"ok": true, "transport": "tauri"}),
        );
        return;
    }
    if !request_line.starts_with("POST /command") {
        response(
            &mut stream,
            "404 Not Found",
            serde_json::json!({"ok": false, "error": "unknown endpoint"}),
        );
        return;
    }
    let request: CommandRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            response(
                &mut stream,
                "400 Bad Request",
                serde_json::json!({"ok": false, "error": error.to_string()}),
            );
            return;
        }
    };
    let request_id = format!("tauri-{sequence}");
    println!(
        "[acceptance-tauri] command:start request_id={request_id} command={} args={}",
        request.command,
        request.args.len()
    );
    let (sender, receiver) = mpsc::channel();
    state
        .pending
        .lock()
        .expect("acceptance state poisoned")
        .insert(request_id.clone(), sender);
    for _ in 0..600 {
        if state.ready.load(Ordering::Acquire) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if !state.ready.load(Ordering::Acquire) {
        state
            .pending
            .lock()
            .expect("acceptance state poisoned")
            .remove(&request_id);
        response(
            &mut stream,
            "503 Service Unavailable",
            serde_json::json!({"ok": false, "requestId": request_id, "error": "renderer acceptance bridge is not ready"}),
        );
        return;
    }
    let event = CommandEvent {
        request_id: &request_id,
        command: &request.command,
        args: &request.args,
    };
    if let Err(error) = app.emit("elephant:acceptance:command", event) {
        state
            .pending
            .lock()
            .expect("acceptance state poisoned")
            .remove(&request_id);
        response(
            &mut stream,
            "500 Internal Server Error",
            serde_json::json!({"ok": false, "requestId": request_id, "error": error.to_string()}),
        );
        return;
    }
    let result = receiver
        .recv_timeout(Duration::from_secs(60))
        .unwrap_or_else(|_| Err("renderer command timed out".into()));
    state
        .pending
        .lock()
        .expect("acceptance state poisoned")
        .remove(&request_id);
    match result {
        Ok(value) => {
            println!(
                "[acceptance-tauri] command:done request_id={request_id} command={}",
                request.command
            );
            response(
                &mut stream,
                "200 OK",
                serde_json::json!({"ok": true, "requestId": request_id, "result": value}),
            );
        }
        Err(error) => {
            eprintln!(
                "[acceptance-tauri] command:error request_id={request_id} command={} error={error}",
                request.command
            );
            response(
                &mut stream,
                "500 Internal Server Error",
                serde_json::json!({"ok": false, "requestId": request_id, "error": error}),
            );
        }
    }
}

#[tauri::command]
pub fn tauri_acceptance_result(
    request_id: String,
    result: Option<Value>,
    error: Option<String>,
    state: State<'_, AcceptanceState>,
) -> bool {
    let sender = state
        .pending
        .lock()
        .expect("acceptance state poisoned")
        .remove(&request_id);
    match sender {
        Some(sender) => sender
            .send(error.map_or_else(|| Ok(result.unwrap_or(Value::Null)), Err))
            .is_ok(),
        None => false,
    }
}

#[tauri::command]
pub fn tauri_acceptance_ready(state: State<'_, AcceptanceState>) -> bool {
    state.ready.store(true, Ordering::Release);
    println!("[acceptance-tauri] renderer:ready");
    true
}

#[tauri::command]
pub fn tauri_acceptance_enabled() -> bool {
    std::env::var_os("ELEPHANT_ACCEPTANCE_TAURI_PORT").is_some()
}

pub fn start(app: &AppHandle) {
    let Ok(raw_port) = std::env::var("ELEPHANT_ACCEPTANCE_TAURI_PORT") else {
        return;
    };
    let port = raw_port.parse::<u16>().unwrap_or(0);
    let listener = std::net::TcpListener::bind(("127.0.0.1", port))
        .expect("failed to bind Tauri acceptance server");
    let actual_port = listener
        .local_addr()
        .expect("failed to read Tauri acceptance server address")
        .port();
    println!("ELEPHANT_ACCEPTANCE_TAURI_PORT={actual_port}");
    let handle = app.clone();
    std::thread::Builder::new()
        .name("acceptance-tauri-server".into())
        .spawn(move || {
            for (sequence, connection) in listener.incoming().enumerate() {
                let Ok(stream) = connection else {
                    continue;
                };
                let app = handle.clone();
                std::thread::spawn(move || {
                    handle_connection(
                        stream,
                        app.clone(),
                        app.state::<AcceptanceState>(),
                        sequence as u64,
                    )
                });
            }
        })
        .expect("failed to start Tauri acceptance server thread");
}
