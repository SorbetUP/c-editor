//! Native Freya surface for the package-owned Iroh Sync service.

use freya::prelude::*;
use serde_json::{json, Value};

use crate::theme;

use super::ShellState;

#[derive(PartialEq)]
struct SyncWorkspace {
    shell: State<ShellState>,
    palette: theme::ThemePalette,
}

impl Component for SyncWorkspace {
    fn render(&self) -> impl IntoElement {
        let snapshot = self.shell.read().clone();
        let invite_input = use_state(String::new);
        let refresh = action_button(
            "Refresh Sync status",
            "Refresh status",
            self.shell,
            "sync.status",
            json!({}),
        );
        let scan = action_button(
            "Scan the active vault for Sync",
            "Scan vault",
            self.shell,
            "sync.scan",
            json!({}),
        );
        let create = action_button(
            "Create a Sync invitation",
            "Create invitation",
            self.shell,
            "sync.create-invite",
            json!({}),
        );
        let mut accept_shell = self.shell;
        let accept_input = invite_input;
        let accept = rect()
            .width(Size::fill())
            .padding(Gaps::new(8., 12., 8., 12.))
            .with_corner_radius(8.)
            .a11y_alt("Pair this device with Sync invitation")
            .on_press(move |_| {
                let value = accept_input.read().trim().to_owned();
                if value.is_empty() {
                    accept_shell.write().sync.error =
                        Some("Paste a Sync invitation before pairing this device.".to_owned());
                    return;
                }
                invoke(
                    accept_shell,
                    "sync.accept-invite",
                    json!({ "manualCode": value }),
                );
            })
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text("Pair this device"),
            );
        let run = action_button(
            "Run a real Iroh Sync session",
            "Sync now",
            self.shell,
            "sync.run",
            json!({}),
        );

        let status_text = status_text(snapshot.sync.status.as_ref());
        let mut body = rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new_all(22.))
            .spacing(14.)
            .a11y_alt("Sync workspace")
            .child(
                label()
                    .font_size(24.)
                    .font_weight(FontWeight::BOLD)
                    .text("Sync"),
            )
            .child(
                label()
                    .color(theme::token_color(self.palette, theme::ThemeToken::Muted))
                    .text("Package-owned Iroh synchronization for this vault."),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(14.))
                    .background(theme::token_color(self.palette, theme::ThemeToken::Surface))
                    .border(
                        Border::new()
                            .fill(theme::token_color(self.palette, theme::ThemeToken::Border))
                            .width(1.),
                    )
                    .with_corner_radius(12.)
                    .spacing(8.)
                    .child(label().font_weight(FontWeight::BOLD).text(status_text))
                    .child(label().text(if snapshot.sync.busy {
                        "Sync service is processing an action…"
                    } else {
                        "The service is idle."
                    }))
                    .child(rect().horizontal().spacing(8.).child(refresh).child(scan)),
            );

        if let Some(message) = snapshot.sync.message {
            body = body.child(
                label()
                    .color(theme::token_color(self.palette, theme::ThemeToken::Muted))
                    .a11y_alt("Sync result")
                    .text(message),
            );
        }
        if let Some(error) = snapshot.sync.error {
            body = body.child(
                label()
                    .color(theme::token_color(self.palette, theme::ThemeToken::Danger))
                    .a11y_alt("Sync error")
                    .text(error),
            );
        }

        body = body.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(14.))
                .background(theme::token_color(self.palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(self.palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(12.)
                .spacing(8.)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .text("Invite another device"),
                )
                .child(label().text("Create a short-lived encrypted invitation."))
                .child(create)
                .child(label().a11y_alt("Sync invitation").text(
                    if snapshot.sync.invitation.is_empty() {
                        "No invitation created yet.".to_owned()
                    } else {
                        snapshot.sync.invitation.clone()
                    },
                )),
        );

        body = body.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(14.))
                .background(theme::token_color(self.palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(self.palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(12.)
                .spacing(8.)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .text("Join an existing vault"),
                )
                .child(label().text("Paste a manual code or JSON invitation."))
                .child(
                    Input::new(invite_input)
                        .width(Size::fill())
                        .placeholder("Sync invitation"),
                )
                .child(accept),
        );

        body = body.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(14.))
                .background(theme::token_color(self.palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(self.palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(12.)
                .spacing(8.)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .text("Run synchronization"),
                )
                .child(label().text("This calls the physical Sync service; errors stay visible."))
                .child(run),
        );
        body.into_element()
    }
}

pub(super) fn workspace(shell: State<ShellState>, palette: theme::ThemePalette) -> Element {
    SyncWorkspace { shell, palette }.into_element()
}

fn action_button(
    alt: &'static str,
    text: &'static str,
    shell: State<ShellState>,
    method: &'static str,
    params: Value,
) -> Element {
    rect()
        .padding(Gaps::new(8., 12., 8., 12.))
        .with_corner_radius(8.)
        .a11y_alt(alt)
        .on_press(move |_| invoke(shell, method, params.clone()))
        .child(label().font_weight(FontWeight::BOLD).text(text))
        .into_element()
}

fn invoke(mut shell: State<ShellState>, method: &str, params: Value) {
    let root = shell
        .read()
        .vault
        .as_ref()
        .map(|vault| vault.root().to_path_buf());
    let Some(root) = root else {
        shell.write().sync.error = Some("No vault selected.".to_owned());
        return;
    };
    let connection = shell.read().sync.clone();
    {
        let mut state = shell.write();
        state.sync.busy = true;
        state.sync.error = None;
        state.sync.message = None;
    }
    let result = connection.call(&root, method, params);
    let mut state = shell.write();
    state.sync.busy = false;
    match result {
        Ok(value) => {
            if method == "sync.create-invite" {
                state.sync.invitation = value
                    .get("qrPayload")
                    .or_else(|| value.get("manualCode"))
                    .or_else(|| value.get("invite"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
            }
            state.sync.status = Some(value.clone());
            state.sync.message = Some(action_message(method, &value));
            eprintln!("[freya][sync] action=complete method={method}");
        }
        Err(error) => {
            eprintln!("[freya][sync] action=failure method={method} error={error}");
            state.sync.error = Some(error);
        }
    }
}

fn action_message(method: &str, value: &Value) -> String {
    match method {
        "sync.scan" => format!(
            "Vault scan complete: {} file(s), {} director{}.",
            value
                .get("files")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            value
                .get("directories")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            if value.get("directories").and_then(Value::as_u64) == Some(1) {
                "y"
            } else {
                "ies"
            }
        ),
        "sync.create-invite" => "Invitation created by the physical Sync service.".to_owned(),
        "sync.accept-invite" => "Device pairing completed.".to_owned(),
        "sync.run" => format!(
            "Synchronization complete: {} file(s) transferred.",
            value
                .get("transferredFiles")
                .and_then(Value::as_u64)
                .unwrap_or_default()
        ),
        _ => "Sync status refreshed.".to_owned(),
    }
}

fn status_text(value: Option<&Value>) -> String {
    let state = value
        .and_then(|value| value.get("pairingState").or_else(|| value.get("state")))
        .and_then(Value::as_str)
        .unwrap_or("not connected");
    format!("Sync service: {state}")
}
