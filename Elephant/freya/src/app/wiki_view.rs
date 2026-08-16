//! Native Wiki workspace backed by the existing knowledge-core store.
//!
//! This renderer deliberately does not synthesize a catalogue.  It reads the
//! persisted draft records produced by the production Wiki pipeline and
//! exposes only actions that the native store can complete: refresh, accept a
//! proposed draft, and open an accepted draft from the vault.

use elephantnote_knowledge_core::{KnowledgeStore, WikiDraft, WikiDraftStatus};
use freya::prelude::*;

use crate::theme;

use super::ShellState;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct WikiViewState {
    revision: u64,
    error: Option<String>,
}

impl WikiViewState {
    fn refresh(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.error = None;
    }

    fn fail(&mut self, error: String) {
        self.error = Some(error);
    }
}

pub(super) fn wiki_workspace(
    shell_state: State<ShellState>,
    view_state: State<WikiViewState>,
    palette: theme::ThemePalette,
) -> Element {
    let snapshot = shell_state.read().clone();
    let Some(vault) = snapshot.vault.as_ref() else {
        return super::route_notice("Wiki", "No active vault.");
    };

    let (drafts, load_error) = match KnowledgeStore::open(vault.root())
        .and_then(|store| store.list_wiki_drafts(None, 200))
    {
        Ok(drafts) => (drafts, None),
        Err(error) => (Vec::new(), Some(format!("Wiki store unavailable: {error}"))),
    };
    let error = load_error.or_else(|| view_state.read().error.clone());
    let mut refresh_state = view_state;
    let refresh = rect()
        .padding(Gaps::new(7., 11., 7., 11.))
        .with_corner_radius(8.)
        .background(theme::token_color(palette, theme::ThemeToken::Soft))
        .on_press(move |_| refresh_state.write().refresh())
        .a11y_alt("Refresh Wiki drafts")
        .child(label().text("Refresh"));

    let mut surface = rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(26., 30., 36., 30.))
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .spacing(16.)
        .a11y_alt("Wiki workspace")
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .spacing(4.)
                        .child(
                            label()
                                .font_size(24.)
                                .font_weight(FontWeight::BOLD)
                                .text("Wiki"),
                        )
                        .child(
                            label()
                                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                                .text("Persisted knowledge drafts from the active vault"),
                        ),
                )
                .child(refresh),
        );

    if let Some(error) = error {
        surface = surface.child(
            rect()
                .padding(Gaps::new_all(10.))
                .background(theme::token_color(palette, theme::ThemeToken::Soft))
                .a11y_alt("Wiki error")
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Danger))
                        .text(error),
                ),
        );
    }

    if drafts.is_empty() {
        return surface
            .child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(24.))
                    .background(theme::token_color(palette, theme::ThemeToken::Surface))
                    .with_corner_radius(12.)
                    .a11y_alt("Wiki empty state")
                    .child(label().text("No Wiki drafts are stored in this vault.")),
            )
            .into_element();
    }

    let cards = drafts
        .into_iter()
        .map(|draft| wiki_card(shell_state, view_state, draft, palette))
        .collect::<Vec<_>>();
    surface
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .show_scrollbar(true)
                .child(rect().width(Size::fill()).spacing(10.).children(cards)),
        )
        .into_element()
}

fn wiki_card(
    shell_state: State<ShellState>,
    view_state: State<WikiViewState>,
    draft: WikiDraft,
    palette: theme::ThemePalette,
) -> Element {
    let status = status_label(&draft.status);
    let accept_id = draft.id.clone();
    let accept = if matches!(
        draft.status,
        WikiDraftStatus::Proposed | WikiDraftStatus::Outdated
    ) {
        let accept_shell = shell_state;
        let accept_view = view_state;
        Some(
            rect()
                .padding(Gaps::new(6., 9., 6., 9.))
                .with_corner_radius(7.)
                .a11y_alt(format!("Accept Wiki {}", draft.title))
                .on_press(move |_| accept_wiki(accept_shell, accept_view, accept_id.clone()))
                .child(label().text("Accept")),
        )
    } else {
        None
    };
    let dismiss = if draft.status == WikiDraftStatus::Proposed {
        let dismiss_shell = shell_state;
        let dismiss_view = view_state;
        let dismiss_id = draft.id.clone();
        Some(
            rect()
                .padding(Gaps::new(6., 9., 6., 9.))
                .with_corner_radius(7.)
                .a11y_alt(format!("Dismiss Wiki {}", draft.title))
                .on_press(move |_| {
                    dismiss_wiki(dismiss_shell, dismiss_view, dismiss_id.clone())
                })
                .child(label().text("Dismiss")),
        )
    } else {
        None
    };
    let open = if draft.status == WikiDraftStatus::Accepted {
        let mut open_shell = shell_state;
        let path = format!(".elephantnote/wiki/{}.md", draft.slug);
        Some(
            rect()
                .padding(Gaps::new(6., 9., 6., 9.))
                .with_corner_radius(7.)
                .a11y_alt(format!("Open Wiki {}", draft.title))
                .on_press(move |_| open_shell.write().open_note_path(&path))
                .child(label().text("Open")),
        )
    } else {
        None
    };
    rect()
        .width(Size::fill())
        .padding(Gaps::new_all(14.))
        .background(theme::token_color(palette, theme::ThemeToken::Surface))
        .border(
            Border::new()
                .fill(theme::token_color(palette, theme::ThemeToken::Border))
                .width(1.),
        )
        .with_corner_radius(12.)
        .spacing(7.)
        .a11y_alt(format!("Wiki draft {}", draft.title))
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .font_size(16.)
                        .font_weight(FontWeight::BOLD)
                        .text(draft.title),
                )
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(status),
                ),
        )
        .child(
            label()
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(draft.topic),
        )
        .child(label().font_size(11.).text(format!(
            "{} citations · {}",
            draft.citations.len(),
            draft.slug
        )))
        .child(
            rect()
                .horizontal()
                .spacing(7.)
                .maybe_child(accept)
                .maybe_child(dismiss)
                .maybe_child(open),
        )
        .into_element()
}

fn accept_wiki(
    shell_state: State<ShellState>,
    mut view_state: State<WikiViewState>,
    draft_id: String,
) {
    let Some(vault) = shell_state.read().vault.clone() else {
        view_state.write().fail("No active vault.".to_owned());
        return;
    };
    let result = KnowledgeStore::open(vault.root())
        .and_then(|store| store.accept_wiki_draft(vault.root(), &draft_id).map(|_| ()));
    match result {
        Ok(()) => {
            eprintln!("[freya][wiki] action:accept-complete draft={draft_id}");
            view_state.write().refresh();
        }
        Err(error) => {
            eprintln!("[freya][wiki] action:accept-failure draft={draft_id} error={error}");
            view_state.write().fail(error);
        }
    }
}

fn dismiss_wiki(
    shell_state: State<ShellState>,
    mut view_state: State<WikiViewState>,
    draft_id: String,
) {
    let Some(vault) = shell_state.read().vault.clone() else {
        view_state.write().fail("No active vault.".to_owned());
        return;
    };
    let result = KnowledgeStore::open(vault.root()).and_then(|store| {
        store
            .set_wiki_draft_status(&draft_id, WikiDraftStatus::Rejected)
            .map(|_| ())
    });
    match result {
        Ok(()) => {
            eprintln!("[freya][wiki] action:dismiss-complete draft={draft_id}");
            view_state.write().refresh();
        }
        Err(error) => {
            eprintln!("[freya][wiki] action:dismiss-failure draft={draft_id} error={error}");
            view_state.write().fail(error);
        }
    }
}


fn status_label(status: &WikiDraftStatus) -> &'static str {
    match status {
        WikiDraftStatus::Proposed => "Proposed",
        WikiDraftStatus::Accepted => "Accepted",
        WikiDraftStatus::Rejected => "Dismissed",
        WikiDraftStatus::Outdated => "Outdated",
    }
}
