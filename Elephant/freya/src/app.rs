use crate::application::{load_from_environment, VaultEntry, VaultSnapshot};
use crate::theme;
use crate::vault::VaultService;
use freya::prelude::*;
use std::fs;

#[derive(Clone, Copy, Debug, PartialEq)]
enum WorkspaceView {
    Notes,
    Search,
    Graph,
    Settings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LibraryUi {
    sort_ascending: bool,
    list: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct OpenNote {
    entry: VaultEntry,
    content: Result<String, String>,
}

impl WorkspaceView {
    fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Search,
            2 => Self::Graph,
            3 => Self::Settings,
            _ => Self::Notes,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Notes => 0,
            Self::Search => 1,
            Self::Graph => 2,
            Self::Settings => 3,
        }
    }
}

pub fn app() -> impl IntoElement {
    let view = use_state(|| 0usize);
    let sidebar_open = use_state(|| true);
    let snapshot = use_state(load_from_environment);
    let library_ui = use_state(LibraryUi::default);
    let create_menu_open = use_state(|| false);
    let opened_note = use_state(|| None::<OpenNote>);
    shell(
        view,
        sidebar_open,
        snapshot,
        library_ui,
        create_menu_open,
        opened_note,
    )
}

pub fn app_with_shared_view() -> impl IntoElement {
    let view = use_consume::<State<usize>>();
    let sidebar_open = use_state(|| true);
    let snapshot = use_state(load_from_environment);
    let library_ui = use_state(LibraryUi::default);
    let create_menu_open = use_state(|| false);
    let opened_note = use_state(|| None::<OpenNote>);
    shell(
        view,
        sidebar_open,
        snapshot,
        library_ui,
        create_menu_open,
        opened_note,
    )
}

pub fn app_with_test_context() -> impl IntoElement {
    let view = use_consume::<State<usize>>();
    let snapshot = use_consume::<State<Result<VaultSnapshot, String>>>();
    let sidebar_open = use_state(|| true);
    let library_ui = use_state(LibraryUi::default);
    let create_menu_open = use_state(|| false);
    let opened_note = use_state(|| None::<OpenNote>);
    shell(
        view,
        sidebar_open,
        snapshot,
        library_ui,
        create_menu_open,
        opened_note,
    )
}

fn shell(
    view: State<usize>,
    mut sidebar_open: State<bool>,
    snapshot: State<Result<VaultSnapshot, String>>,
    library_ui: State<LibraryUi>,
    create_menu_open: State<bool>,
    opened_note: State<Option<OpenNote>>,
) -> impl IntoElement {
    let selected_view = WorkspaceView::from_index(*view.read());
    let current_snapshot = snapshot.read().clone();
    let create_root = current_snapshot
        .as_ref()
        .ok()
        .map(|snapshot| snapshot.root.clone());

    let rail = rect()
        .width(Size::px(72.))
        .height(Size::fill())
        .background(theme::RAIL)
        .padding(Gaps::new_all(8.))
        .spacing(8.)
        .child(
            label()
                .font_size(22.)
                .font_weight(FontWeight::BOLD)
                .color(theme::PRIMARY_TEXT)
                .text("E"),
        )
        .child(nav_item(
            "All notes",
            "▣",
            WorkspaceView::Notes,
            selected_view,
            view,
        ))
        .child(nav_item(
            "Search",
            "⌕",
            WorkspaceView::Search,
            selected_view,
            view,
        ))
        .child(nav_item(
            "Graph",
            "◌",
            WorkspaceView::Graph,
            selected_view,
            view,
        ))
        .child(nav_item(
            "Settings",
            "⚙",
            WorkspaceView::Settings,
            selected_view,
            view,
        ))
        .child(rect().expanded().on_mouse_up(move |_| {
            *sidebar_open.write() = !*sidebar_open.read();
        }));

    let sidebar = if *sidebar_open.read() {
        rect()
            .width(Size::px(246.))
            .height(Size::fill())
            .background(theme::SIDEBAR)
            .padding(Gaps::new_all(18.))
            .spacing(12.)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(38.))
                    .padding(Gaps::new_all(8.))
                    .background(theme::SELECTED)
                    .with_corner_radius(8.)
                    .horizontal()
                    .spacing(10.)
                    .child(label().font_size(18.).text("▣"))
                    .child(
                        label()
                            .font_size(16.)
                            .font_weight(FontWeight::BOLD)
                            .color(theme::PRIMARY_TEXT)
                            .text("All notes"),
                    ),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(24.))
                    .horizontal()
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        label()
                            .font_size(12.)
                            .font_weight(FontWeight::BOLD)
                            .color(theme::SECONDARY_TEXT)
                            .text("NOTES"),
                    )
                    .child(
                        label()
                            .font_size(18.)
                            .color(theme::SECONDARY_TEXT)
                            .text("⌕"),
                    ),
            )
            .child(sidebar_entries(current_snapshot.as_ref().ok()))
    } else {
        rect().width(Size::px(0.)).height(Size::fill())
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::CANVAS)
        .color(theme::PRIMARY_TEXT)
        .horizontal()
        .child(rail)
        .child(sidebar)
        .child(main_content(
            selected_view,
            current_snapshot,
            library_ui,
            opened_note,
        ))
        .child(create_menu_open_overlay(
            create_root,
            snapshot,
            create_menu_open,
        ))
        .child(create_fab(create_menu_open))
}

fn nav_item(
    label_text: &'static str,
    icon: &'static str,
    target: WorkspaceView,
    selected: WorkspaceView,
    mut view: State<usize>,
) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .height(Size::px(44.))
        .padding(Gaps::new_all(8.))
        .background(if target == selected {
            theme::SELECTED
        } else {
            theme::RAIL
        })
        .on_mouse_up(move |_| *view.write() = target.index())
        .a11y_alt(label_text)
        .child(label().font_size(22.).color(theme::PRIMARY_TEXT).text(icon))
}

fn sidebar_entries(snapshot: Option<&VaultSnapshot>) -> impl IntoElement {
    match snapshot {
        Some(snapshot) if snapshot.entries.is_empty() => label()
            .color(theme::SECONDARY_TEXT)
            .text("No visible notes")
            .into_element(),
        Some(snapshot) => rect()
            .children(
                snapshot
                    .entries
                    .iter()
                    .take(80)
                    .map(sidebar_entry)
                    .collect::<Vec<_>>(),
            )
            .into_element(),
        None => label()
            .color(theme::SECONDARY_TEXT)
            .text("Select a vault to begin")
            .into_element(),
    }
}

fn sidebar_entry(entry: &VaultEntry) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(34.))
        .padding(Gaps::new_all(6.))
        .background(theme::SIDEBAR)
        .child(
            label()
                .color(theme::SECONDARY_TEXT)
                .text(if entry.is_folder {
                    format!("▸  {}", entry.title)
                } else {
                    format!("   {}", entry.title)
                }),
        )
        .into_element()
}

fn main_content(
    view: WorkspaceView,
    snapshot: Result<VaultSnapshot, String>,
    library_ui: State<LibraryUi>,
    opened_note: State<Option<OpenNote>>,
) -> Element {
    let content = match view {
        WorkspaceView::Notes => match opened_note.read().clone() {
            Some(note) => editor_page(note, opened_note),
            None => notes_page(snapshot, library_ui, opened_note),
        },
        WorkspaceView::Search => {
            placeholder_page("Search", "Search remains backed by Rust services.")
        }
        WorkspaceView::Graph => placeholder_page(
            "Graph",
            "Graph rendering is still an explicit migration surface.",
        ),
        WorkspaceView::Settings => placeholder_page(
            "Settings",
            "Preferences will be moved after shell parity is proven.",
        ),
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::CANVAS)
        .padding(Gaps::new_all(28.))
        .spacing(18.)
        .child(library_toolbar(view, library_ui))
        .child(content)
        .into_element()
}

fn library_toolbar(view: WorkspaceView, mut library_ui: State<LibraryUi>) -> Element {
    if view != WorkspaceView::Notes {
        return rect().height(Size::px(0.)).into_element();
    }

    let sort_button = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .padding(Gaps::new_all(8.))
        .background(theme::PANEL)
        .with_corner_radius(12.)
        .on_mouse_up(move |_| {
            let mut state = library_ui.write();
            state.sort_ascending = !state.sort_ascending;
        })
        .child(label().font_size(24.).color(theme::PRIMARY_TEXT).text("↕"));
    let view_button = rect()
        .width(Size::px(52.))
        .height(Size::px(52.))
        .padding(Gaps::new_all(8.))
        .background(theme::PANEL)
        .with_corner_radius(12.)
        .on_mouse_up(move |_| {
            let list = library_ui.read().list;
            library_ui.write().list = !list;
        })
        .child(label().font_size(24.).color(theme::PRIMARY_TEXT).text("☷"));

    rect()
        .height(Size::px(52.))
        .width(Size::fill())
        .horizontal()
        .main_align(Alignment::End)
        .spacing(14.)
        .child(sort_button)
        .child(view_button)
        .into_element()
}

fn create_menu_open_overlay(
    root: Option<std::path::PathBuf>,
    snapshot_state: State<Result<VaultSnapshot, String>>,
    menu_open: State<bool>,
) -> Element {
    if !*menu_open.read() {
        return rect().height(Size::px(0.)).into_element();
    }

    rect()
        .position(Position::new_absolute().right(20.).bottom(86.))
        .width(Size::px(280.))
        .height(Size::px(190.))
        .padding(Gaps::new_all(8.))
        .background(theme::PANEL)
        .border(Border::new().fill(theme::BORDER).width(1.))
        .with_corner_radius(14.)
        .layer(Layer::OverlayLevel(10))
        .spacing(4.)
        .child(
            label()
                .padding(Gaps::new_all(8.))
                .font_size(12.)
                .font_weight(FontWeight::BOLD)
                .color(theme::SECONDARY_TEXT)
                .text("CREATE"),
        )
        .child(create_menu_item(
            "▤",
            "Note",
            "Create a new note",
            root.clone(),
            snapshot_state,
            menu_open,
            false,
        ))
        .child(create_menu_item(
            "▱",
            "Folder",
            "Organize notes in a folder",
            root,
            snapshot_state,
            menu_open,
            true,
        ))
        .into_element()
}

fn create_menu_item(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    root: Option<std::path::PathBuf>,
    mut snapshot_state: State<Result<VaultSnapshot, String>>,
    mut menu_open: State<bool>,
    create_folder: bool,
) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(64.))
        .padding(Gaps::new_all(10.))
        .background(theme::PANEL)
        .with_corner_radius(10.)
        .layer(Layer::OverlayLevel(11))
        .horizontal()
        .spacing(12.)
        .on_mouse_up(move |_| {
            *menu_open.write() = false;
            let Some(root) = root.clone() else {
                *snapshot_state.write() = Err("Select a vault to begin".to_string());
                return;
            };
            let action = if create_folder {
                "create-folder"
            } else {
                "create-note"
            };
            eprintln!("[freya][vault] action:start name={action} source=create-menu");
            match create_entry(&root, create_folder) {
                Ok(snapshot) => {
                    eprintln!("[freya][vault] action:done name={action} source=create-menu");
                    *snapshot_state.write() = Ok(snapshot);
                }
                Err(error) => {
                    eprintln!(
                        "[freya][vault] action:error name={action} source=create-menu error={error}"
                    );
                    *snapshot_state.write() = Err(error);
                }
            }
        })
        .a11y_alt(title)
        .child(label().font_size(24.).text(icon))
        .child(
            rect()
                .spacing(2.)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .color(theme::PRIMARY_TEXT)
                        .text(title),
                )
                .child(label().color(theme::SECONDARY_TEXT).text(description)),
        )
        .into_element()
}

fn create_fab(mut menu_open: State<bool>) -> Element {
    rect()
        .position(Position::new_absolute().right(20.).bottom(20.))
        .width(Size::px(56.))
        .height(Size::px(56.))
        .background(theme::PRIMARY)
        .with_corner_radius(12.)
        .center()
        .on_mouse_up(move |_| {
            let open = *menu_open.read();
            *menu_open.write() = !open;
        })
        .a11y_alt("Create")
        .child(label().font_size(32.).color(theme::PRIMARY_TEXT).text("+"))
        .into_element()
}

fn create_entry(root: &std::path::Path, create_folder: bool) -> Result<VaultSnapshot, String> {
    let service = VaultService::open(root)?;
    if create_folder {
        service.create_folder(None)?;
    } else {
        service.create_note(None, None, Some("Untitled".to_string()))?;
    }
    VaultService::open(root)?.snapshot()
}

fn notes_page(
    snapshot: Result<VaultSnapshot, String>,
    library_ui: State<LibraryUi>,
    opened_note: State<Option<OpenNote>>,
) -> Element {
    match snapshot {
        Ok(mut snapshot) => {
            let ui = *library_ui.read();
            snapshot.entries.sort_by(|left, right| {
                let order = left
                    .title
                    .to_ascii_lowercase()
                    .cmp(&right.title.to_ascii_lowercase());
                if ui.sort_ascending {
                    order
                } else {
                    order.reverse()
                }
            });
            let cards = snapshot
                .entries
                .iter()
                .take(12)
                .map(|entry| note_card(entry, ui.list, &snapshot.root, opened_note))
                .collect::<Vec<_>>();
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .spacing(12.)
                .child(
                    label()
                        .color(theme::SECONDARY_TEXT)
                        .text(format!("{} visible entries", snapshot.entries.len())),
                )
                .child(if ui.list {
                    rect()
                        .width(Size::fill())
                        .spacing(8.)
                        .children(cards)
                        .into_element()
                } else {
                    rect()
                        .width(Size::fill())
                        .horizontal()
                        .spacing(10.)
                        .children(cards)
                        .into_element()
                })
                .into_element()
        }
        Err(error) => rect()
            .spacing(12.)
            .child(label().color(theme::ERROR).text("Vault unavailable"))
            .child(label().color(theme::SECONDARY_TEXT).text(error))
            .into_element(),
    }
}

fn note_card(
    entry: &VaultEntry,
    list: bool,
    root: &std::path::Path,
    mut opened_note: State<Option<OpenNote>>,
) -> Element {
    let height = if list { 58. } else { 176. };
    let icon = if entry.is_folder { "▱" } else { "▤" };
    let mut card = rect()
        .width(if list { Size::fill() } else { Size::px(344.) })
        .height(Size::px(height))
        .padding(Gaps::new_all(12.))
        .background(theme::PANEL)
        .border(Border::new().fill(theme::BORDER).width(1.))
        .with_corner_radius(10.)
        .spacing(10.)
        .child(
            label()
                .font_size(if list { 19. } else { 25. })
                .font_weight(FontWeight::BOLD)
                .color(theme::PRIMARY_TEXT)
                .text(format!("{icon}  {}", entry.title)),
        )
        .child(if list {
            rect().height(Size::px(0.)).into_element()
        } else {
            let preview = if entry.is_folder && !entry.preview.is_empty() {
                entry.preview.join("  ·  ")
            } else if entry.is_folder {
                "No items yet".to_string()
            } else {
                "Markdown note".to_string()
            };
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .background(theme::CANVAS)
                .with_corner_radius(8.)
                .padding(Gaps::new_all(10.))
                .child(label().color(theme::SECONDARY_TEXT).text(preview))
                .into_element()
        });
    if !entry.is_folder {
        let event_entry = entry.clone();
        let path = root.join(&event_entry.relative_path);
        card = card.on_mouse_up(move |_| {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("Unable to open {}: {error}", event_entry.relative_path));
            *opened_note.write() = Some(OpenNote {
                entry: event_entry.clone(),
                content,
            });
        });
    }
    card.into_element()
}

fn editor_page(note: OpenNote, mut opened_note: State<Option<OpenNote>>) -> Element {
    let body = note.content.clone().unwrap_or_else(|error| error);
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::CANVAS)
        .spacing(18.)
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(52.))
                .background(theme::PANEL)
                .padding(Gaps::new_all(12.))
                .on_mouse_up(move |_| *opened_note.write() = None)
                .child(
                    label()
                        .font_weight(FontWeight::BOLD)
                        .color(theme::PRIMARY_TEXT)
                        .text(format!("←  {}", note.entry.title)),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .background(theme::PANEL)
                .padding(Gaps::new_all(18.))
                .with_corner_radius(10.)
                .child(label().color(theme::PRIMARY_TEXT).text(body)),
        )
        .into_element()
}

fn placeholder_page(title: &'static str, body: &'static str) -> Element {
    rect()
        .background(theme::PANEL)
        .padding(Gaps::new_all(18.))
        .spacing(10.)
        .child(label().font_size(18.).text(title))
        .child(label().color(theme::SECONDARY_TEXT).text(body))
        .into_element()
}
