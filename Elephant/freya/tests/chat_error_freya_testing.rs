use elephant_freya::{app::app_with_vault_view, navigation_contract::WorkspaceView};
use freya::prelude::*;
use freya_testing::{prelude::*, TestingNode, TestingRunner};
use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const PROMPT: &str = "Explain the fixture error";
const PROVIDER_ERROR: &str = "Provider fixture rejected request";

struct ProviderFixture {
    base_url: String,
    received: Arc<Mutex<Option<String>>>,
    stop: Option<mpsc::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl ProviderFixture {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind provider fixture");
        listener
            .set_nonblocking(true)
            .expect("configure provider fixture");
        let address = listener.local_addr().expect("provider address");
        let received = Arc::new(Mutex::new(None));
        let thread_received = Arc::clone(&received);
        let (stop, stop_rx) = mpsc::channel();
        let thread = thread::spawn(move || loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("configure provider stream");
                    let body = read_request(&mut stream);
                    *thread_received.lock().expect("provider lock") = Some(body);
                    let response_body =
                        format!("{{\"error\":{{\"message\":\"{PROVIDER_ERROR}\"}}}}");
                    let response = format!(
                        "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response_body.len(), response_body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write provider response");
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("provider fixture failed: {error}"),
            }
        });
        Self {
            base_url: format!("http://{address}/v1"),
            received,
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    fn received(&self) -> Option<String> {
        self.received.lock().expect("provider lock").clone()
    }
}

impl Drop for ProviderFixture {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            thread.join().expect("provider thread");
        }
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("provider read timeout");
    loop {
        let mut chunk = [0_u8; 4096];
        let read = stream.read(&mut chunk).expect("read provider request");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let length = String::from_utf8_lossy(&bytes[..end])
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or_default();
        if bytes.len() >= end + 4 + length {
            return String::from_utf8_lossy(&bytes[end + 4..end + 4 + length]).to_string();
        }
    }
    String::new()
}

struct FixtureVault(PathBuf);

impl FixtureVault {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-chat-error-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("create Chat fixture");
        fs::write(root.join("Welcome.md"), "# Chat fixture\n").expect("write Chat fixture");
        Self(root)
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn labeled(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click(runner: &mut TestingRunner, label: &str) {
    let node = labeled(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing Chat target {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        f64::from((area.min_x() + area.max_x()) / 2.),
        f64::from((area.min_y() + area.max_y()) / 2.),
    ));
}

fn replace_input(runner: &mut TestingRunner, label: &str, value: &str) {
    click(runner, label);
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("a".to_owned()),
        code: Code::Unidentified,
        modifiers: Modifiers::ctrl_or_meta(),
    });
    runner.sync_and_update();
    runner.write_text(value);
}

fn error_visible(runner: &TestingRunner) -> bool {
    labeled(runner, "Chat error").len() == 1
}

#[test]
fn chat_sends_through_provider_shows_error_and_persists_history() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime")
        .block_on(async {
            let fixture = FixtureVault::new();
            let provider = ProviderFixture::start();
            let root = fixture.0.clone();
            let (mut runner, ()) = TestingRunner::new(
                move || app_with_vault_view(root.clone(), WorkspaceView::Chat),
                (1280., 840.).into(),
                |_| (),
                1.,
            );
            replace_input(&mut runner, "Model", "fixture-model");
            replace_input(&mut runner, "http://127.0.0.1:8317/v1", &provider.base_url);
            click(&mut runner, "Save Chat provider settings");
            click(&mut runner, "Ask your configured provider…");
            runner.write_text(PROMPT);
            runner.press_key(Key::Named(NamedKey::Enter));
            assert_eq!(
                labeled(&runner, "Chat user message").len(),
                1,
                "sending must append the user message before the provider call"
            );

            for _ in 0..200 {
                runner.handle_events_immediately();
                runner.sync_and_update();
                if error_visible(&runner) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            assert!(
                error_visible(&runner),
                "provider error must be visible in Chat"
            );
            let request: Value = serde_json::from_str(
                &provider
                    .received()
                    .expect("provider must receive Chat request"),
            )
            .expect("provider request JSON");
            assert_eq!(request["model"], "fixture-model");
            assert_eq!(request["messages"][0]["content"], PROMPT);
            let persisted: Value = serde_json::from_str(
                &fs::read_to_string(fixture.0.join(".elephantnote/chat.json"))
                    .expect("Chat history must be persisted"),
            )
            .expect("Chat history JSON");
            assert_eq!(persisted["messages"][0]["content"], PROMPT);
        });
}
