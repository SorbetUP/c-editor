//! Native Chat workspace using the existing OpenAI-compatible PI runtime.

use freya::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::Path};

use crate::{pi_adapter, theme};

use super::ShellState;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8317/v1";
const CHAT_FILE: &str = "chat.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub(super) struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub base_url: String,
    pub busy: bool,
    pub error: Option<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            model: String::new(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            busy: false,
            error: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ChatDocument {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    model: String,
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default)]
    messages: Vec<ChatMessage>,
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.to_owned()
}

impl ChatState {
    pub(super) fn load_for(&mut self, root: &Path) {
        let path = crate::vault_adapter::vault_layout::hidden_dir(root, CHAT_FILE);
        self.error = None;
        match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<ChatDocument>(&raw) {
                Ok(document) => {
                    self.model = document.model;
                    self.base_url = document.base_url;
                    self.messages = document.messages;
                }
                Err(error) => self.error = Some(format!("Chat history is invalid: {error}")),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => self.error = Some(format!("Chat history cannot be read: {error}")),
        }
    }

    fn save_for(&self, root: &Path) -> Result<(), String> {
        let path = crate::vault_adapter::vault_layout::hidden_dir(root, CHAT_FILE);
        let parent = path
            .parent()
            .ok_or_else(|| "Chat history has no parent directory".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let document = ChatDocument {
            version: 1,
            model: self.model.clone(),
            base_url: self.base_url.clone(),
            messages: self.messages.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?;
        fs::write(&path, bytes).map_err(|error| error.to_string())
    }
}

#[derive(PartialEq)]
struct ChatWorkspace {
    shell: State<ShellState>,
    palette: theme::ThemePalette,
}

impl Component for ChatWorkspace {
    fn render(&self) -> impl IntoElement {
        let snapshot = self.shell.read().clone();
        let model = use_state(|| snapshot.chat.model.clone());
        let base_url = use_state(|| snapshot.chat.base_url.clone());
        let input = use_state(String::new);
        let mut save_shell = self.shell;
        let save_model = model;
        let save_base_url = base_url;
        let save = rect()
            .width(Size::px(132.))
            .padding(Gaps::new(6., 9., 6., 9.))
            .with_corner_radius(7.)
            .a11y_alt("Save Chat provider settings")
            .on_press(move |_| {
                let mut shell = save_shell.write();
                shell.chat.model = save_model.read().trim().to_owned();
                shell.chat.base_url = save_base_url.read().trim().to_owned();
                if let Some(vault) = shell.vault.as_ref() {
                    if let Err(error) = shell.chat.save_for(vault.root()) {
                        shell.chat.error =
                            Some(format!("Chat settings could not be saved: {error}"));
                    }
                }
            })
            .child(label().text("Save provider"));

        let send_shell = self.shell;
        let send_input = input;
        let send_model = model;
        let send_base_url = base_url;
        let send = rect()
            .width(Size::px(92.))
            .padding(Gaps::new(8., 12., 8., 12.))
            .with_corner_radius(8.)
            .a11y_alt(if snapshot.chat.busy {
                "Sending chat message"
            } else {
                "Send chat message"
            })
            .on_press(move |_| {
                send_message(
                    send_shell,
                    send_input.read().clone(),
                    send_model.read().clone(),
                    send_base_url.read().clone(),
                    send_input,
                )
            })
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text(if snapshot.chat.busy {
                        "Sending…"
                    } else {
                        "Send"
                    }),
            );

        let mut message_input = Input::new(input);
        message_input = message_input
            .width(Size::fill())
            .placeholder("Ask your configured provider…")
            .on_submit({
                let submit_shell = self.shell;
                let submit_model = model;
                let submit_base_url = base_url;
                let submit_input = input;
                move |value: String| {
                    send_message(
                        submit_shell,
                        value,
                        submit_model.read().clone(),
                        submit_base_url.read().clone(),
                        submit_input,
                    )
                }
            });

        let mut body = rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new_all(22.))
            .spacing(12.)
            .a11y_alt("Chat workspace")
            .child(
                label()
                    .font_size(24.)
                    .font_weight(FontWeight::BOLD)
                    .text("Chat"),
            )
            .child(
                label()
                    .color(theme::token_color(self.palette, theme::ThemeToken::Muted))
                    .text("Provider-backed conversation persisted in this vault."),
            );

        let settings = rect()
            .width(Size::fill())
            .spacing(8.)
            .child(Input::new(model).width(Size::fill()).placeholder("Model"))
            .child(
                Input::new(base_url)
                    .width(Size::fill())
                    .placeholder("Provider base URL"),
            )
            .child(save);
        body = body.child(settings);
        for message in snapshot.chat.messages {
            body = body.child(message_row(message, self.palette));
        }
        if let Some(error) = snapshot.chat.error {
            body = body.child(
                label()
                    .color(theme::token_color(self.palette, theme::ThemeToken::Danger))
                    .a11y_alt("Chat error")
                    .text(error),
            );
        }
        body = body.child(
            rect()
                .width(Size::fill())
                .spacing(8.)
                .child(message_input)
                .child(send),
        );
        body.into_element()
    }
}

pub(super) fn workspace(shell: State<ShellState>, palette: theme::ThemePalette) -> Element {
    ChatWorkspace { shell, palette }.into_element()
}

fn message_row(message: ChatMessage, palette: theme::ThemePalette) -> Element {
    let is_user = message.role == "user";
    rect()
        .width(Size::fill())
        .padding(Gaps::new(10., 12., 10., 12.))
        .background(theme::token_color(
            palette,
            if is_user {
                theme::ThemeToken::Soft
            } else {
                theme::ThemeToken::Surface
            },
        ))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(10.)
        .a11y_alt(format!("Chat {} message", message.role))
        .child(label().font_weight(FontWeight::BOLD).text(if is_user {
            "You"
        } else {
            "Assistant"
        }))
        .child(label().text(message.content))
        .into_element()
}

fn send_message(
    mut shell: State<ShellState>,
    input: String,
    model: String,
    base_url: String,
    mut input_state: State<String>,
) {
    let prompt = input.trim().to_owned();
    if prompt.is_empty() || shell.read().chat.busy {
        return;
    }
    let model = model.trim().to_owned();
    let base_url = base_url.trim().to_owned();
    if model.is_empty() {
        shell.write().chat.error = Some("Choose a model before sending a message.".to_owned());
        return;
    }
    input_state.set(String::new());
    let root = shell
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let messages = {
        let mut state = shell.write();
        state.chat.model = model.clone();
        state.chat.base_url = base_url.clone();
        state.chat.error = None;
        state.chat.busy = true;
        state.chat.messages.push(ChatMessage {
            role: "user".to_owned(),
            content: prompt.clone(),
        });
        state
            .chat
            .messages
            .iter()
            .map(|message| json!({ "role": message.role, "content": message.content }))
            .collect::<Vec<_>>()
    };
    let payload = json!({ "baseUrl": base_url });
    spawn(async move {
        let result = pi_adapter::complete(&model, &messages, &payload).await;
        let mut state = shell.write();
        state.chat.busy = false;
        match result {
            Ok(answer) => state.chat.messages.push(ChatMessage {
                role: "assistant".to_owned(),
                content: answer.answer,
            }),
            Err(error) => {
                eprintln!("[freya][chat] action=send-failure error={error}");
                state.chat.error = Some(format!("Chat request failed: {error}"));
            }
        }
        if let Some(root) = root.as_deref() {
            if let Err(error) = state.chat.save_for(root) {
                state.chat.error = Some(format!("Chat history could not be saved: {error}"));
            }
        }
    });
}
