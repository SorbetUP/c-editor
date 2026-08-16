//! Native Add-ons registry controls for the Settings surface.

use freya::prelude::*;

use crate::theme;

use crate::app::ShellState;

use super::super::super::SettingsViewState;

pub(crate) fn addons_settings(
    settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let snapshot = settings_state.read().clone();
    let addons = snapshot.addons;
    let refresh_state = settings_state;
    let refresh_shell = shell_state;
    let refresh = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt("Refresh addons")
        .on_press(move |_| refresh_addons(refresh_state, refresh_shell))
        .child(label().text("Refresh"));

    let mut body = rect().width(Size::fill()).spacing(10.).child(
        rect()
            .width(Size::fill())
            .horizontal()
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text(if addons.loading {
                        "Reading installed addons…"
                    } else {
                        "Installed addons"
                    }),
            )
            .child(refresh),
    );
    if let Some(error) = addons.error {
        body = body.child(
            label()
                .a11y_alt(format!("Addon lifecycle error: {error}"))
                .color(theme::token_color(palette, theme::ThemeToken::Danger))
                .text(error),
        );
    }
    if addons.loaded && addons.items.is_empty() {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("No addon is installed in this vault."),
        );
    }
    for addon in addons.items {
        body = body.child(addon_row(addon, settings_state, shell_state, palette));
    }
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
        .child(body)
        .into_element()
}

fn addon_row(
    addon: crate::addon_adapter::InstalledAddon,
    settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let id = addon.manifest.id.clone();
    let name = addon.manifest.name.clone();
    let enabled = addon.enabled;
    let runtime_status = match id.as_str() {
        "elephant.graph" | "elephant.wiki" | "elephant.calendar" | "elephant.dashboard"
        | "elephant.sync" => "Native Freya surface available",
        _ => "JavaScript worker runtime is not connected in Freya",
    };
    let runtime_status_label = format!("Addon runtime status {name}: {runtime_status}");
    let toggle_state = settings_state;
    let toggle_shell = shell_state;
    let toggle = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt(format!(
            "{} {} addon",
            if enabled { "Disable" } else { "Enable" },
            name
        ))
        .on_press(move |_| toggle_addon(toggle_state, toggle_shell, id.clone(), !enabled))
        .child(label().text(if enabled {
            "Registry enabled"
        } else {
            "Registry disabled"
        }));
    let uninstall_state = settings_state;
    let uninstall_shell = shell_state;
    let uninstall_id = addon.manifest.id.clone();
    let uninstall = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt(format!("Uninstall {name} addon"))
        .on_press(move |_| uninstall_addon(uninstall_state, uninstall_shell, uninstall_id.clone()))
        .child(label().text("Uninstall"));
    let controls = rect()
        .horizontal()
        .spacing(6.)
        .child(toggle)
        .child(uninstall);
    rect()
        .width(Size::fill())
        .padding(Gaps::new(9., 0., 9., 0.))
        .horizontal()
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .expanded()
                .spacing(2.)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .text(addon.manifest.name),
                )
                .child(
                    label()
                        .font_size(11.)
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(format!(
                            "{} · {}",
                            addon.manifest.id, addon.manifest.version
                        )),
                )
                .child(
                    label()
                        .a11y_alt(runtime_status_label)
                        .font_size(10.)
                        .color(theme::token_color(
                            palette,
                            if runtime_status.starts_with("JavaScript") {
                                theme::ThemeToken::Danger
                            } else {
                                theme::ThemeToken::Muted
                            },
                        ))
                        .text(runtime_status),
                ),
        )
        .child(controls)
        .into_element()
}

fn refresh_addons(mut settings_state: State<SettingsViewState>, shell_state: State<ShellState>) {
    let Some(vault) = shell_state.read().vault.clone() else {
        settings_state
            .write()
            .apply_addons_result(Err("No vault selected.".to_owned()));
        return;
    };
    settings_state.write().begin_addons_load();
    let result = crate::addon_adapter::list(vault.root());
    settings_state.write().apply_addons_result(result);
}

fn toggle_addon(
    mut settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    addon_id: String,
    enabled: bool,
) {
    let Some(vault) = shell_state.read().vault.clone() else {
        settings_state
            .write()
            .finish_addon_action(Err("No vault selected.".to_owned()));
        return;
    };
    settings_state.write().begin_addon_action(addon_id.clone());
    let result = crate::addon_adapter::set_enabled(vault.root(), &addon_id, enabled).map(|_| ());
    finish_addon_action(settings_state, shell_state, result);
}

fn uninstall_addon(
    mut settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    addon_id: String,
) {
    let Some(vault) = shell_state.read().vault.clone() else {
        settings_state
            .write()
            .finish_addon_action(Err("No vault selected.".to_owned()));
        return;
    };
    settings_state.write().begin_addon_action(addon_id.clone());
    let result = crate::addon_adapter::uninstall(vault.root(), &addon_id);
    finish_addon_action(settings_state, shell_state, result);
}

fn finish_addon_action(
    mut settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    result: Result<(), String>,
) {
    let should_refresh = result.is_ok();
    settings_state.write().finish_addon_action(result);
    if should_refresh {
        refresh_addons(settings_state, shell_state);
    }
}
