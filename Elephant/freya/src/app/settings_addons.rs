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
    let addon_runtime = snapshot.addon_runtime;
    let addons = snapshot.addons;
    let refresh_state = settings_state;
    let refresh_shell = shell_state;
    let refresh = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt("Refresh addons")
        .on_press(move |_| refresh_addons(refresh_state, refresh_shell))
        .child(label().text("Refresh"));
    let install_state = settings_state;
    let install_shell = shell_state;
    let install = rect()
        .padding(Gaps::new(6., 9., 6., 9.))
        .with_corner_radius(7.)
        .a11y_alt("Install addon package")
        .on_press(move |_| install_addon(install_state, install_shell))
        .child(label().text("Install package"));
    let actions = rect()
        .horizontal()
        .spacing(6.)
        .child(install)
        .child(refresh);

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
            .child(actions),
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
        let runtime_active = addon_runtime.is_active(&addon.manifest.id);
        body = body.child(addon_row(
            addon,
            runtime_active,
            settings_state,
            shell_state,
            palette,
        ));
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
    runtime_active: bool,
    settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let id = addon.manifest.id.clone();
    let name = addon.manifest.name.clone();
    let enabled = addon.enabled;
    let runtime_status = if crate::addon_adapter::has_native_view(&id) {
        "Native Freya surface available"
    } else if runtime_active {
        "JavaScript worker active"
    } else {
        "JavaScript worker runtime is not connected in Freya"
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
    let runtime = settings_state.read().addon_runtime.clone();
    let runtime_result = runtime.reconcile_enabled(vault.root());
    let addons_result = crate::addon_adapter::list(vault.root());
    let mut state = settings_state.write();
    state.apply_addons_result(addons_result);
    state.finish_addon_action(runtime_result);
}

fn install_addon(mut settings_state: State<SettingsViewState>, shell_state: State<ShellState>) {
    let Some(package_path) = rfd::FileDialog::new()
        .add_filter("Elephant add-on", &["enaddon", "zip"])
        .pick_file()
    else {
        return;
    };
    let Some(vault) = shell_state.read().vault.clone() else {
        settings_state
            .write()
            .finish_addon_action(Err("No vault selected.".to_owned()));
        return;
    };
    let action = package_path.display().to_string();
    settings_state.write().begin_addon_action(action);
    let result = crate::addon_adapter::install(vault.root(), &package_path).map(|_| ());
    finish_addon_action(settings_state, shell_state, result);
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
    let runtime = settings_state.read().addon_runtime.clone();
    let result = runtime.set_enabled(vault.root(), &addon_id, enabled);
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
    let runtime = settings_state.read().addon_runtime.clone();
    let result = runtime.uninstall(vault.root(), &addon_id);
    finish_addon_action(settings_state, shell_state, result);
}

fn finish_addon_action(
    mut settings_state: State<SettingsViewState>,
    shell_state: State<ShellState>,
    result: Result<(), String>,
) {
    let refresh = shell_state
        .read()
        .vault
        .clone()
        .ok_or_else(|| "No vault selected.".to_owned())
        .and_then(|vault| crate::addon_adapter::list(vault.root()));
    let mut state = settings_state.write();
    state.apply_addons_result(refresh);
    state.finish_addon_action(result);
}
