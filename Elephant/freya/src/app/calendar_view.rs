//! Native Calendar workspace backed by the vault's persisted calendar file.

use freya::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::{theme, vault_adapter::VaultEntry};

use super::ShellState;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub starts_at: String,
    pub ends_at: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CalendarState {
    pub events: Vec<CalendarEvent>,
    pub loaded: bool,
    pub importing: bool,
    pub error: Option<String>,
}

impl CalendarState {
    pub(super) fn load_for(&mut self, root: &Path) {
        let path = calendar_path(root);
        self.error = None;
        self.events = match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<CalendarDocument>(&raw) {
                Ok(document) => normalize_events(document.events),
                Err(error) => {
                    self.error = Some(format!("Calendar data is invalid: {error}"));
                    Vec::new()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                self.error = Some(format!("Calendar data cannot be read: {error}"));
                Vec::new()
            }
        };
        self.loaded = true;
        eprintln!(
            "[freya][calendar] action=load path={} events={} error={}",
            path.display(),
            self.events.len(),
            self.error.is_some()
        );
    }
}

#[derive(Deserialize, Serialize)]
struct CalendarDocument {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    events: Vec<CalendarEvent>,
}

fn calendar_path(root: &Path) -> std::path::PathBuf {
    crate::vault_adapter::vault_layout::hidden_dir(
        root,
        crate::vault_adapter::vault_layout::CALENDAR_FILE,
    )
}

fn normalize_events(events: Vec<CalendarEvent>) -> Vec<CalendarEvent> {
    let mut events = events
        .into_iter()
        .filter(|event| !event.title.trim().is_empty())
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.starts_at
            .cmp(&right.starts_at)
            .then_with(|| left.title.cmp(&right.title))
    });
    events
}

pub(super) fn workspace(shell: State<ShellState>, palette: theme::ThemePalette) -> Element {
    let snapshot = shell.read().clone();
    let calendar = snapshot.calendar;
    let import_shell = shell;
    let import = rect()
        .padding(Gaps::new(8., 12., 8., 12.))
        .with_corner_radius(8.)
        .a11y_alt("Import ICS calendar")
        .on_press(move |_| import_ics(import_shell))
        .child(label().font_weight(FontWeight::BOLD).text("Import ICS"));
    let clear_shell = shell;
    let clear = rect()
        .padding(Gaps::new(8., 12., 8., 12.))
        .with_corner_radius(8.)
        .a11y_alt("Clear calendar events")
        .on_press(move |_| clear_events(clear_shell))
        .child(label().font_weight(FontWeight::BOLD).text("Clear events"));
    let actions = rect().horizontal().spacing(8.).child(import).child(clear);

    let mut body = rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new_all(22.))
        .spacing(12.)
        .a11y_alt("Calendar workspace")
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .spacing(3.)
                        .child(
                            label()
                                .font_size(24.)
                                .font_weight(FontWeight::BOLD)
                                .text("Calendar"),
                        )
                        .child(
                            label()
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text("Offline events and notes grouped by date."),
                        ),
                )
                .child(actions),
        );

    if let Some(error) = calendar.error {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Danger))
                .a11y_alt("Calendar error")
                .text(error),
        );
    }
    if calendar.importing {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text("Importing calendar…"),
        );
    }
    let has_events = !calendar.events.is_empty();
    for event in calendar.events {
        body = body.child(event_row(event, palette));
    }

    let notes = snapshot
        .page
        .map(|page| page.entries)
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| !entry.is_directory && entry.path.to_ascii_lowercase().ends_with(".md"))
        .take(8)
        .collect::<Vec<_>>();
    let has_notes = !notes.is_empty();
    if has_notes {
        body = body.child(label().font_weight(FontWeight::BOLD).text("Recent notes"));
        for entry in notes {
            body = body.child(note_row(entry, shell, palette));
        }
    }
    if !calendar.loaded || (!has_events && !has_notes) {
        body = body.child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .a11y_alt("Calendar empty")
                .text("No calendar event or note is available yet."),
        );
    }
    body.into_element()
}

fn event_row(event: CalendarEvent, palette: theme::ThemePalette) -> Element {
    let date = event.starts_at.get(..10).unwrap_or("No date").to_owned();
    let time = event.starts_at.get(11..16).unwrap_or("All day").to_owned();
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
        .spacing(3.)
        .a11y_alt(format!("Calendar event {}", event.title))
        .child(label().font_weight(FontWeight::BOLD).text(event.title))
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(format!("{date} · {time}")),
        )
        .maybe_child((!event.location.is_empty()).then(|| {
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(event.location)
        }))
        .into_element()
}

fn note_row(
    entry: VaultEntry,
    mut shell: State<ShellState>,
    palette: theme::ThemePalette,
) -> Element {
    let title = entry.title.clone();
    rect()
        .width(Size::fill())
        .padding(Gaps::new(8., 12., 8., 12.))
        .background(theme::token_color(palette, theme::ThemeToken::Soft))
        .with_corner_radius(8.)
        .a11y_alt(format!("Open calendar note {title}"))
        .on_press(move |_| shell.write().open_note(&entry))
        .child(label().text(title))
        .into_element()
}

fn import_ics(mut shell: State<ShellState>) {
    let Some(vault) = shell.read().vault.clone() else {
        shell.write().calendar.error = Some("No vault selected.".to_owned());
        return;
    };
    let Some(files) = rfd::FileDialog::new()
        .add_filter("Calendar", &["ics", "ical"])
        .pick_files()
    else {
        return;
    };
    let mut state = shell.write();
    state.calendar.importing = true;
    let mut imported = state.calendar.events.clone();
    let result = files.into_iter().try_for_each(|file| {
        let source = fs::read_to_string(&file).map_err(|error| error.to_string())?;
        imported.extend(parse_ics(&source));
        Ok::<(), String>(())
    });
    match result {
        Ok(()) => {
            imported = normalize_events(imported);
            let path = calendar_path(vault.root());
            let document = CalendarDocument {
                version: 1,
                updated_at: String::new(),
                events: imported.clone(),
            };
            match serde_json::to_vec_pretty(&document)
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    fs::create_dir_all(path.parent().expect("calendar parent"))
                        .map_err(|error| error.to_string())
                        .and_then(|_| fs::write(&path, bytes).map_err(|error| error.to_string()))
                }) {
                Ok(()) => {
                    state.calendar.events = imported;
                    state.calendar.error = None;
                    eprintln!(
                        "[freya][calendar] action=import-complete path={}",
                        path.display()
                    );
                }
                Err(error) => {
                    state.calendar.error = Some(format!("Calendar import failed: {error}"))
                }
            }
        }
        Err(error) => state.calendar.error = Some(format!("Calendar import failed: {error}")),
    }
    state.calendar.importing = false;
}

fn clear_events(mut shell: State<ShellState>) {
    let Some(vault) = shell.read().vault.clone() else {
        shell.write().calendar.error = Some("No vault selected.".to_owned());
        return;
    };
    let path = calendar_path(vault.root());
    let result = match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Calendar data cannot be cleared: {error}")),
    };
    let mut state = shell.write();
    match result {
        Ok(()) => {
            state.calendar.events.clear();
            state.calendar.loaded = true;
            state.calendar.error = None;
            eprintln!(
                "[freya][calendar] action=clear-complete path={}",
                path.display()
            );
        }
        Err(error) => {
            eprintln!(
                "[freya][calendar] action=clear-failure path={} error={error}",
                path.display()
            );
            state.calendar.error = Some(error);
        }
    }
}

fn parse_ics(source: &str) -> Vec<CalendarEvent> {
    let mut events = Vec::new();
    let mut current = None;
    for line in source.replace("\r\n", "\n").lines() {
        match line.trim() {
            "BEGIN:VEVENT" => current = Some(CalendarEvent::default()),
            "END:VEVENT" => {
                if let Some(mut event) = current.take() {
                    if event.id.is_empty() {
                        event.id = format!("{}-{}", event.starts_at, event.title);
                    }
                    event.source = "ics".to_owned();
                    events.push(event);
                }
            }
            line if current.is_some() => {
                let Some((raw_key, value)) = line.split_once(':') else {
                    continue;
                };
                let key = raw_key
                    .split(';')
                    .next()
                    .unwrap_or(raw_key)
                    .to_ascii_uppercase();
                let value = value
                    .replace("\\n", "\n")
                    .replace("\\,", ",")
                    .replace("\\;", ";");
                let event = current.as_mut().expect("current event");
                match key.as_str() {
                    "UID" => event.id = value,
                    "SUMMARY" => event.title = value,
                    "DTSTART" => event.starts_at = value,
                    "DTEND" => event.ends_at = value,
                    "LOCATION" => event.location = value,
                    "DESCRIPTION" => event.description = value,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    events
}
