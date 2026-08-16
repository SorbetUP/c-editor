//! Native view of model files stored in the active vault.

use freya::prelude::*;
use std::{fs, path::Path};

use crate::theme;

use super::ShellState;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ModelsState {
    pub files: Vec<ModelFile>,
    pub loaded: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModelFile {
    pub name: String,
    pub size: u64,
}

impl ModelsState {
    pub(super) fn load_for(&mut self, root: &Path) {
        let directory = crate::vault_adapter::vault_layout::hidden_dir(
            root,
            crate::vault_adapter::vault_layout::MODELS_DIR,
        );
        self.error = None;
        self.files = match fs::read_dir(&directory) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let metadata = entry.metadata().ok()?;
                    metadata.is_file().then(|| ModelFile {
                        name: entry.file_name().to_string_lossy().into_owned(),
                        size: metadata.len(),
                    })
                })
                .collect(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                self.error = Some(format!("Models directory cannot be read: {error}"));
                Vec::new()
            }
        };
        self.files.sort_by(|left, right| left.name.cmp(&right.name));
        self.loaded = true;
        eprintln!(
            "[freya][models] action=load path={} files={} error={}",
            directory.display(),
            self.files.len(),
            self.error.is_some()
        );
    }

    fn remove_for(&mut self, root: &Path, name: &str) -> Result<(), String> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
        {
            return Err("Model file name is invalid.".to_owned());
        }
        let directory = crate::vault_adapter::vault_layout::hidden_dir(
            root,
            crate::vault_adapter::vault_layout::MODELS_DIR,
        );
        let path = directory.join(name);
        if !path.is_file() {
            return Err(format!("Local model is not present: {name}"));
        }
        fs::remove_file(&path).map_err(|error| format!("Model cannot be removed: {error}"))?;
        self.load_for(root);
        eprintln!(
            "[freya][models] action=remove-complete path={}",
            path.display()
        );
        Ok(())
    }
}

pub(super) fn workspace(shell: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let snapshot = shell.read().clone();
    let mut refresh_shell = shell;
    let refresh = rect()
        .padding(Gaps::new(8., 12., 8., 12.))
        .with_corner_radius(8.)
        .a11y_alt("Refresh local models")
        .on_press(move |_| {
            let root = refresh_shell
                .read()
                .vault
                .as_ref()
                .map(|vault| vault.root().to_path_buf());
            if let Some(root) = root {
                refresh_shell.write().models.load_for(&root);
            }
        })
        .child(label().font_weight(FontWeight::BOLD).text("Refresh"));
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
                                .text("Local model files available to native runtimes."),
                        ),
                )
                .child(refresh),
        );
    if let Some(error) = snapshot.models.error {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Danger))
                .a11y_alt("Models error")
                .text(error),
        );
    }
    for file in &snapshot.models.files {
        let name = file.name.clone();
        let mut remove_shell = shell;
        let remove = rect()
            .padding(Gaps::new(6., 9., 6., 9.))
            .with_corner_radius(7.)
            .a11y_alt(format!("Remove local model {name}"))
            .on_press(move |_| {
                let Some(root) = remove_shell
                    .read()
                    .vault
                    .as_ref()
                    .map(|vault| vault.root().to_path_buf())
                else {
                    remove_shell.write().models.error = Some("No vault selected.".to_owned());
                    return;
                };
                let result = remove_shell.write().models.remove_for(&root, &name);
                if let Err(error) = result {
                    eprintln!("[freya][models] action=remove-failure name={name} error={error}");
                    remove_shell.write().models.error = Some(error);
                }
            })
            .child(label().text("Remove"));
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
                        .text(format_bytes(file.size)),
                )
                .child(remove),
        );
    }
    if snapshot.models.loaded && snapshot.models.files.is_empty() {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("No local model file is installed in this vault."),
        );
    }
    body.into_element()
}

fn format_bytes(size: u64) -> String {
    if size >= 1_073_741_824 {
        format!("{:.1} GiB", size as f64 / 1_073_741_824.)
    } else if size >= 1_048_576 {
        format!("{:.1} MiB", size as f64 / 1_048_576.)
    } else {
        format!("{} bytes", size)
    }
}
