//! Native Freya surface for the official Open Models package service.
//!
//! The view does not manipulate GGUF files or `active-model.json` directly.
//! Every lifecycle operation crosses the package-owned
//! `elephant-addon-service-v1` service, matching the official addon contract.

use freya::{prelude::*, sdk::use_timeout};
use serde_json::{json, Value};
use std::{path::PathBuf, time::Duration};

use crate::{models_adapter::ModelsService, theme};

use super::ShellState;

const POLL_INTERVAL: Duration = Duration::from_millis(75);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ModelsState {
    pub files: Vec<ModelFile>,
    /// Selected model according to the Open Models service. Selection alone
    /// does not claim that llama-server has loaded the model.
    pub active: Option<String>,
    pub loaded: bool,
    pub busy: bool,
    pub status: Option<Value>,
    pub error: Option<String>,
    service: ModelsService,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModelFile {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub path: String,
    pub size: u64,
    pub active: bool,
}

impl ModelsState {
    fn begin(&mut self, root: PathBuf, method: &str, params: Value) -> Result<(), String> {
        if self.busy {
            return Err("An Open Models operation is already running.".to_owned());
        }
        self.error = None;
        self.busy = true;
        if let Err(error) = self.service.begin(root, method.to_owned(), params) {
            self.busy = false;
            return Err(error);
        }
        eprintln!("[freya][models] action=start method={method}");
        Ok(())
    }

    fn poll(&mut self, root: &PathBuf) {
        let Some((method, result)) = self.service.poll() else {
            return;
        };
        self.busy = false;
        match result {
            Ok(value) => {
                eprintln!("[freya][models] action=complete method={method}");
                self.error = None;
                match method.as_str() {
                    "service.start" | "models.status" => {
                        self.status = Some(value);
                        self.loaded = true;
                        // Status and list are separate service contracts. Chain
                        // the list without allowing a stale UI snapshot to
                        // claim model availability from filesystem inspection.
                        if let Err(error) = self.begin(root.clone(), "models.list", json!({})) {
                            self.error = Some(error);
                        }
                    }
                    "models.list" | "models.list-local" => {
                        self.apply_list(&value);
                        self.loaded = true;
                    }
                    "models.activate" => {
                        self.active = model_identity(&value);
                        if let Err(error) = self.begin(root.clone(), "models.list", json!({})) {
                            self.error = Some(error);
                        }
                    }
                    "models.deactivate" => {
                        self.active = None;
                        self.status = self.status.take().map(|mut status| {
                            if let Some(object) = status.as_object_mut() {
                                object.insert("serverRunning".to_owned(), Value::Bool(false));
                                object.insert("serverModelPath".to_owned(), Value::String(String::new()));
                            }
                            status
                        });
                        if let Err(error) = self.begin(root.clone(), "models.list", json!({})) {
                            self.error = Some(error);
                        }
                    }
                    "models.delete" | "models.download" => {
                        if let Err(error) = self.begin(root.clone(), "models.list", json!({})) {
                            self.error = Some(error);
                        }
                    }
                    "service.stop" => {
                        self.status = Some(json!({
                            "running": false,
                            "serverRunning": false,
                            "owner": "elephant.open-models"
                        }));
                    }
                    _ => {}
                }
            }
            Err(error) => {
                eprintln!("[freya][models] action=failure method={method} error={error}");
                self.error = Some(error);
            }
        }
    }

    fn apply_list(&mut self, value: &Value) {
        let items = value
            .get("models")
            .and_then(Value::as_array)
            .or_else(|| value.as_array())
            .cloned()
            .unwrap_or_default();
        self.files = items
            .iter()
            .filter_map(parse_model)
            .collect::<Vec<_>>();
        self.files.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        self.active = self
            .files
            .iter()
            .find(|model| model.active)
            .map(|model| model.id.clone());
    }

    fn cancel_ui(&mut self) {
        self.service.cancel_ui();
        self.busy = false;
        self.error = None;
    }
}

pub(super) fn workspace(shell: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let download_input = use_state(String::new);
    let timeout = use_timeout(|| POLL_INTERVAL);
    let snapshot = shell.read().clone();
    let root = snapshot
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let scope = root
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut initial_shell = shell;
    let root_for_start = root.clone();
    use_side_effect_with_deps(&scope, move |_| {
        let Some(root) = root_for_start.clone() else {
            return;
        };
        let should_start = {
            let state = initial_shell.read();
            !state.models.loaded && !state.models.busy
        };
        if should_start {
            if let Err(error) = initial_shell
                .write()
                .models
                .begin(root, "service.start", json!({}))
            {
                initial_shell.write().models.error = Some(error);
            }
        }
    });

    let mut poll_timeout = timeout;
    let mut poll_shell = shell;
    let root_for_poll = root.clone();
    use_side_effect(move || {
        if !poll_timeout.elapsed() {
            return;
        }
        poll_timeout.reset();
        if let Some(root) = root_for_poll.as_ref() {
            poll_shell.write().models.poll(root);
        }
    });

    let snapshot = shell.read().clone();
    let running = snapshot
        .models
        .status
        .as_ref()
        .and_then(|status| status.get("running"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let server_running = snapshot
        .models
        .status
        .as_ref()
        .and_then(|status| status.get("serverRunning"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let server_model_path = snapshot
        .models
        .status
        .as_ref()
        .and_then(|status| status.get("serverModelPath"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();

    let mut refresh_shell = shell;
    let refresh = button(
        "Refresh Open Models",
        "Refresh",
        snapshot.models.busy,
        move || {
            start_action(&mut refresh_shell, "models.status", json!({}));
        },
    );
    let mut lifecycle_shell = shell;
    let lifecycle_method = if running { "service.stop" } else { "service.start" };
    let lifecycle_text = if running { "Stop service" } else { "Start service" };
    let lifecycle = button(
        if running {
            "Stop Open Models service"
        } else {
            "Start Open Models service"
        },
        lifecycle_text,
        snapshot.models.busy,
        move || {
            start_action(&mut lifecycle_shell, lifecycle_method, json!({}));
        },
    );
    let cancel = snapshot.models.busy.then(|| {
        let mut cancel_shell = shell;
        button(
            "Cancel Open Models UI request",
            "Cancel",
            false,
            move || cancel_shell.write().models.cancel_ui(),
        )
    });

    let input_for_download = download_input;
    let mut download_shell = shell;
    let download = button(
        "Download GGUF through Open Models",
        "Download",
        snapshot.models.busy,
        move || {
            let id = input_for_download.read().trim().to_owned();
            if id.is_empty() {
                download_shell.write().models.error = Some(
                    "Enter a Hugging Face repository or direct GGUF URL first.".to_owned(),
                );
                return;
            }
            start_action(&mut download_shell, "models.download", json!({ "id": id }));
        },
    );

    let mut body = rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new_all(22.))
        .spacing(12.)
        .a11y_alt("Models workspace")
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    rect()
                        .spacing(3.)
                        .child(
                            label()
                                .font_size(24.)
                                .font_weight(FontWeight::BOLD)
                                .text("Models"),
                        )
                        .child(
                            label()
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text("Official Open Models service · GGUF + llama.cpp"),
                        ),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .child(lifecycle)
                        .child(refresh)
                        .maybe_child(cancel),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(12.))
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .with_corner_radius(10.)
                .spacing(5.)
                .child(label().font_weight(FontWeight::BOLD).text(format!(
                    "Service: {} · llama-server: {}",
                    if running { "running" } else { "stopped" },
                    if server_running { "running" } else { "not loaded" }
                )))
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(if server_running && !server_model_path.is_empty() {
                            format!("Runtime model: {server_model_path}")
                        } else {
                            "Selecting a model does not claim it is loaded; llama-server starts when the provider is actually used.".to_owned()
                        }),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .spacing(8.)
                .child(
                    Input::new(download_input)
                        .width(Size::fill())
                        .placeholder("Hugging Face repository or direct .gguf URL"),
                )
                .child(download),
        );

    if snapshot.models.busy {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("Open Models operation running in background…"),
        );
    }
    if let Some(error) = snapshot.models.error {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Danger))
                .a11y_alt("Models error")
                .text(error),
        );
    }

    for file in &snapshot.models.files {
        let id = file.id.clone();
        let delete_id = id.clone();
        let selected = snapshot.models.active.as_deref() == Some(id.as_str()) || file.active;
        let runtime_loaded = server_running && server_model_path == file.path;
        let mut activate_shell = shell;
        let activate_method = if selected {
            "models.deactivate"
        } else {
            "models.activate"
        };
        let activate = button(
            if selected {
                "Deselect local model"
            } else {
                "Select local model"
            },
            if selected { "Deselect" } else { "Select" },
            snapshot.models.busy,
            move || {
                start_action(
                    &mut activate_shell,
                    activate_method,
                    if selected { json!({}) } else { json!({ "id": id }) },
                );
            },
        );
        let mut remove_shell = shell;
        let remove = button(
            "Remove local model",
            "Remove",
            snapshot.models.busy,
            move || {
                start_action(
                    &mut remove_shell,
                    "models.delete",
                    json!({ "id": delete_id }),
                );
            },
        );
        body = body.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(10., 12., 10., 12.))
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(10.)
                .a11y_alt(format!("Local model {}", file.name))
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .text(file.name.clone()),
                )
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(format!(
                            "{} · {}{}",
                            file.file_name,
                            format_bytes(file.size),
                            if runtime_loaded {
                                " · llama-server loaded"
                            } else if selected {
                                " · selected"
                            } else {
                                ""
                            }
                        )),
                )
                .child(rect().horizontal().spacing(8.).child(activate).child(remove)),
        );
    }

    if snapshot.models.loaded && snapshot.models.files.is_empty() && !snapshot.models.busy {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("No GGUF model is installed by Open Models for this vault."),
        );
    }
    body.into_element()
}

fn start_action(shell: &mut State<ShellState>, method: &str, params: Value) {
    let root = shell
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let Some(root) = root else {
        shell.write().models.error = Some("No vault selected.".to_owned());
        return;
    };
    if let Err(error) = shell.write().models.begin(root, method, params) {
        shell.write().models.error = Some(error);
    }
}

fn button(
    alt: &'static str,
    text: &'static str,
    disabled: bool,
    mut action: impl FnMut() + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(7., 10., 7., 10.))
        .with_corner_radius(8.)
        .a11y_alt(alt)
        .on_press(move |_| {
            if !disabled {
                action();
            }
        })
        .child(label().font_weight(FontWeight::BOLD).text(text))
        .into_element()
}

fn parse_model(value: &Value) -> Option<ModelFile> {
    let path = value
        .get("path")
        .or_else(|| value.get("modelPath"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let file_name = value
        .get("fileName")
        .and_then(Value::as_str)
        .or_else(|| path.rsplit(['/', '\\']).next())
        .unwrap_or("model.gguf")
        .to_owned();
    if file_name.is_empty() {
        return None;
    }
    let id = value
        .get("id")
        .or_else(|| value.get("repoId"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(&file_name)
        .to_owned();
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(&file_name)
        .to_owned();
    Some(ModelFile {
        id,
        name,
        file_name,
        path,
        size: value.get("size").and_then(Value::as_u64).unwrap_or_default(),
        active: value.get("active").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn model_identity(value: &Value) -> Option<String> {
    value
        .get("id")
        .or_else(|| value.get("repoId"))
        .or_else(|| value.get("fileName"))
        .or_else(|| value.get("path"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn format_bytes(size: u64) -> String {
    if size >= 1_073_741_824 {
        format!("{:.1} GiB", size as f64 / 1_073_741_824.)
    } else if size >= 1_048_576 {
        format!("{:.1} MiB", size as f64 / 1_048_576.)
    } else if size >= 1_024 {
        format!("{:.1} KiB", size as f64 / 1_024.)
    } else {
        format!("{size} bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_official_service_model_shape_without_claiming_runtime_loaded() {
        let model = parse_model(&json!({
            "id": "org/model",
            "name": "Model",
            "fileName": "model-Q4_K_M.gguf",
            "path": "/vault/.elephantnote/addons/data/elephant.open-models/models/model.gguf",
            "size": 4096,
            "active": true
        }))
        .unwrap();
        assert_eq!(model.id, "org/model");
        assert!(model.active);
        assert_eq!(model.size, 4096);
    }
}
