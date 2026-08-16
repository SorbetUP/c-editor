//! Native Vault and trash controls for the Settings surface.

use freya::prelude::*;

use crate::theme;

use crate::app::{choose_vault, ShellState};

use super::super::super::SettingsViewState;

pub(crate) fn vault_settings(
    settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let shell = shell_state.read().clone();
    let settings = settings_state.read().clone();
    let active_vault = shell.vault.as_ref().map(|vault| {
        (
            vault.descriptor().name.clone(),
            vault.root().display().to_string(),
        )
    });
    let trash = settings.trash;
    let has_items = !trash.items.is_empty();
    let settings_for_refresh = settings_state;
    let shell_for_refresh = shell_state;
    let refresh = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt("Refresh vault trash")
        .on_press(move |_| refresh_trash(settings_for_refresh, shell_for_refresh))
        .child(label().text("Refresh trash"));

    let choose_state = shell_state;
    let choose = rect()
        .padding(Gaps::new(8., 12., 8., 12.))
        .with_corner_radius(8.)
        .a11y_alt("Open another vault")
        .on_press(move |_| choose_vault(choose_state))
        .child(label().text("Open another vault"));

    let mut registry_rows = rect()
        .width(Size::fill())
        .height(Size::px(shell.vault_registry.vaults.len() as f32 * 58.))
        .spacing(8.)
        .a11y_alt("Registered vaults");
    for vault in &shell.vault_registry.vaults {
        let id = vault.id.clone();
        let name = vault.name.clone();
        let is_active = shell.vault_registry.active_vault_id.as_deref() == Some(vault.id.as_str());
        let mut activate_state = shell_state;
        let mut remove_state = shell_state;
        let activate_name = name.clone();
        let activate = (!is_active).then(|| {
            rect()
                .width(Size::px(88.))
                .padding(Gaps::new(5., 8., 5., 8.))
                .with_corner_radius(7.)
                .a11y_alt(format!("Activate {activate_name}"))
                .on_press(move |_| activate_state.write().activate_vault(&id))
                .child(label().text("Activate"))
                .into_element()
        });
        let remove_id = vault.id.clone();
        let remove_name = vault.name.clone();
        let remove = rect()
            .width(Size::px(88.))
            .padding(Gaps::new(5., 8., 5., 8.))
            .with_corner_radius(7.)
            .a11y_alt(format!("Remove {remove_name} from list"))
            .on_press(move |_| remove_state.write().remove_vault(&remove_id))
            .child(label().text("Remove"));
        registry_rows = registry_rows.child(
            rect()
                .width(Size::fill())
                .height(Size::px(58.))
                .padding(Gaps::new(8., 10., 8., 10.))
                .horizontal()
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .background(theme::token_color(
                    palette,
                    if is_active {
                        theme::ThemeToken::Soft
                    } else {
                        theme::ThemeToken::Surface
                    },
                ))
                .with_corner_radius(8.)
                .a11y_alt(format!("Vault {name}"))
                .child(
                    rect()
                        .expanded()
                        .spacing(2.)
                        .child(label().font_weight(FontWeight::BOLD).text(name))
                        .child(
                            label()
                                .font_size(10.)
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text(vault.path.clone()),
                        ),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(5.)
                        .maybe_child(activate)
                        .child(remove),
                ),
        );
    }

    let trash_summary = if trash.loading {
        "Reading trash…".to_owned()
    } else if let Some(error) = trash.error.as_deref() {
        format!("Trash unavailable: {error}")
    } else if has_items {
        format!(
            "{} deleted {}",
            trash.items.len(),
            if trash.items.len() == 1 {
                "item"
            } else {
                "items"
            }
        )
    } else if trash.loaded {
        "Trash is empty".to_owned()
    } else {
        "Trash has not been read yet".to_owned()
    };

    let mut trash_panel =
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(14.))
            .background(theme::token_color(palette, theme::ThemeToken::Soft))
            .border(
                Border::new()
                    .fill(theme::token_color(palette, theme::ThemeToken::Border))
                    .width(1.),
            )
            .with_corner_radius(12.)
            .spacing(9.)
            .a11y_alt("Vault trash")
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .child(label().font_weight(FontWeight::BOLD).text(trash_summary))
                    .child(rect().horizontal().spacing(7.).child(refresh).maybe_child(
                        has_items.then(|| {
                            let mut toggle_state = settings_state;
                            rect()
                                .padding(Gaps::new(5., 8., 5., 8.))
                                .with_corner_radius(7.)
                                .a11y_alt(if settings.settings.trash_expanded {
                                    "Collapse vault trash"
                                } else {
                                    "Expand vault trash"
                                })
                                .on_press(move |_| {
                                    toggle_state.write().toggle_trash_expanded();
                                })
                                .child(label().text(if settings.settings.trash_expanded {
                                    "Collapse"
                                } else {
                                    "Expand"
                                }))
                                .into_element()
                        }),
                    )),
            )
            .maybe_child(trash.error.clone().map(|error| {
                label()
                    .color(theme::token_color(palette, theme::ThemeToken::Danger))
                    .text(error)
                    .into_element()
            }));

    if settings.settings.trash_expanded && has_items {
        let rows = trash
            .items
            .iter()
            .map(|item| trash_row(item, settings_state, shell_state, palette))
            .collect::<Vec<_>>();
        trash_panel = trash_panel.children(rows);

        let mut action_state = settings_state;
        let action = if trash.empty_confirmation {
            rect()
                .padding(Gaps::new(7., 10., 7., 10.))
                .with_corner_radius(7.)
                .a11y_alt("Confirm empty vault trash")
                .on_press(move |_| empty_trash(action_state, shell_state))
                .child(label().text("Confirm empty trash"))
        } else {
            rect()
                .padding(Gaps::new(7., 10., 7., 10.))
                .with_corner_radius(7.)
                .a11y_alt("Empty vault trash")
                .on_press(move |_| action_state.write().request_empty_trash())
                .child(label().text("Empty trash"))
        };
        trash_panel = trash_panel.child(action);
    }

    let body = if let Some((name, path)) = active_vault {
        rect()
            .width(Size::fill())
            .spacing(5.)
            .child(label().font_weight(FontWeight::BOLD).text(name))
            .child(
                label()
                    .font_size(11.)
                    .color(theme::token_color(palette, theme::ThemeToken::Muted))
                    .text(path),
            )
            .into_element()
    } else {
        rect()
            .width(Size::fill())
            .spacing(5.)
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text("No vault selected"),
            )
            .child(
                label()
                    .color(theme::token_color(palette, theme::ThemeToken::Muted))
                    .text("Choose a local folder to start using Elephant."),
            )
            .into_element()
    };

    rect()
        .width(Size::fill())
        .spacing(14.)
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(16.))
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .border(
                    Border::new()
                        .fill(theme::token_color(palette, theme::ThemeToken::Border))
                        .width(1.),
                )
                .with_corner_radius(12.)
                .spacing(11.)
                .child(registry_rows)
                .child(body)
                .child(choose),
        )
        .child(trash_panel)
        .into_element()
}

fn trash_row(
    item: &crate::vault_adapter::TrashEntry,
    settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let trash_path = item.trash_path.clone();
    let label_text = if item.name.is_empty() {
        item.original_path.clone()
    } else {
        item.name.clone()
    };
    let restore_state = settings_state;
    rect()
        .width(Size::fill())
        .padding(Gaps::new(7., 0., 7., 0.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .child(
            rect()
                .expanded()
                .spacing(2.)
                .child(label().font_weight(FontWeight::BOLD).text(label_text))
                .child(
                    label()
                        .font_size(10.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(item.original_path.clone()),
                ),
        )
        .child(
            rect()
                .padding(Gaps::new(6., 9., 6., 9.))
                .with_corner_radius(7.)
                .a11y_alt(format!("Restore {}", item.original_path))
                .on_press(move |_| restore_trash(restore_state, shell_state, trash_path.clone()))
                .child(label().text("Restore")),
        )
        .into_element()
}

fn refresh_trash(mut settings_state: State<SettingsViewState>, shell_state: State<ShellState>) {
    let vault = shell_state.read().vault.clone();
    settings_state.write().begin_trash_load();
    let result = vault
        .map(|vault| vault.list_trash().map_err(|error| error.to_string()))
        .unwrap_or_else(|| Err("No vault selected.".to_owned()));
    settings_state.write().apply_trash_result(result);
}

fn restore_trash(
    mut settings_state: State<SettingsViewState>,
    mut shell_state: State<ShellState>,
    trash_path: String,
) {
    let vault = shell_state.read().vault.clone();
    settings_state
        .write()
        .begin_trash_action(trash_path.clone());
    let result = vault
        .map(|vault| {
            vault
                .restore_trash(&trash_path)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .unwrap_or_else(|| Err("No vault selected.".to_owned()));
    let succeeded = result.is_ok();
    settings_state.write().finish_trash_action(result);
    if succeeded {
        refresh_trash(settings_state, shell_state);
        let directory = shell_state.read().library.current_path.as_str().to_owned();
        shell_state.write().reload_directory(&directory);
    }
}

fn empty_trash(mut settings_state: State<SettingsViewState>, shell_state: State<ShellState>) {
    let vault = shell_state.read().vault.clone();
    settings_state.write().begin_trash_action("empty");
    let result = vault
        .map(|vault| {
            vault
                .empty_trash()
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .unwrap_or_else(|| Err("No vault selected.".to_owned()));
    settings_state.write().finish_trash_action(result);
    refresh_trash(settings_state, shell_state);
}
