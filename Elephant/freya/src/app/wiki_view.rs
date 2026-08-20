//! Native Wiki workspace backed by the production knowledge-core store.
//!
//! Source search, model-backed proposal generation, strict citation validation,
//! accept/reject and direct creation all reuse `elephantnote-knowledge-core`.
//! Freya never fabricates uncited Wiki Markdown.

use elephantnote_knowledge_core::{
    build_wiki_synthesis_request, parse_and_render_wiki, rebuild_vault,
    wiki_draft_from_rendered, KnowledgeSearchHit, KnowledgeStore, WikiDraft, WikiDraftStatus,
    WikiSourceChunk,
};
use freya::prelude::*;
use serde_json::json;
use std::{collections::BTreeSet, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use crate::{background, pi_adapter, theme};

use super::ShellState;

const SOURCE_SEARCH_LIMIT: usize = 24;
const MAX_WIKI_SECTIONS: usize = 6;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct WikiViewState {
    revision: u64,
    error: Option<String>,
    busy: bool,
    source_results: Vec<KnowledgeSearchHit>,
}

impl WikiViewState {
    fn refresh(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.error = None;
    }

    fn fail(&mut self, error: String) {
        self.busy = false;
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
    let topic_input = use_state(String::new);

    let (drafts, load_error) = match KnowledgeStore::open(vault.root())
        .and_then(|store| store.list_wiki_drafts(None, 200))
    {
        Ok(drafts) => (drafts, None),
        Err(error) => (Vec::new(), Some(format!("Wiki store unavailable: {error}"))),
    };
    let view_snapshot = view_state.read().clone();
    let error = load_error.or_else(|| view_snapshot.error.clone());
    let mut refresh_state = view_state;
    let refresh = action_button(
        "Refresh Wiki drafts",
        "Refresh",
        false,
        move || refresh_state.write().refresh(),
    );

    let mut search_shell = shell_state;
    let search_view = view_state;
    let search_topic = topic_input;
    let search_sources_button = action_button(
        "Search Wiki source notes",
        "Search sources",
        view_snapshot.busy,
        move || {
            search_sources(
                search_shell,
                search_view,
                search_topic.read().trim().to_owned(),
            )
        },
    );

    let mut propose_shell = shell_state;
    let propose_view = view_state;
    let propose_topic = topic_input;
    let propose = action_button(
        "Generate cited Wiki proposal",
        "Propose Wiki",
        view_snapshot.busy,
        move || {
            generate_wiki(
                propose_shell,
                propose_view,
                propose_topic.read().trim().to_owned(),
                None,
                false,
            )
        },
    );

    let mut direct_shell = shell_state;
    let direct_view = view_state;
    let direct_topic = topic_input;
    let direct = action_button(
        "Generate and directly accept cited Wiki",
        "Create directly",
        view_snapshot.busy,
        move || {
            generate_wiki(
                direct_shell,
                direct_view,
                direct_topic.read().trim().to_owned(),
                None,
                true,
            )
        },
    );

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
                                .text("Cited knowledge drafts generated from the active vault"),
                        ),
                )
                .child(refresh),
        )
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(12.))
                .background(theme::token_color(palette, theme::ThemeToken::Surface))
                .with_corner_radius(12.)
                .spacing(8.)
                .child(
                    Input::new(topic_input)
                        .width(Size::fill())
                        .placeholder("Wiki topic"),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .child(search_sources_button)
                        .child(propose)
                        .child(direct),
                )
                .child(
                    label()
                        .color(theme::token_color(palette, theme::ThemeToken::Muted))
                        .text(if view_snapshot.busy {
                            "Knowledge/model operation running in background…"
                        } else {
                            "Generation is accepted only after knowledge-core validates every factual claim against supplied chunk IDs."
                        }),
                ),
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

    if !view_snapshot.source_results.is_empty() {
        let mut source_list = rect()
            .width(Size::fill())
            .spacing(6.)
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text(format!("Sources · {} matches", view_snapshot.source_results.len())),
            );
        for hit in view_snapshot.source_results.iter().take(12) {
            let path = hit.relative_path.clone();
            let mut open_shell = shell_state;
            source_list = source_list.child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(9.))
                    .background(theme::token_color(palette, theme::ThemeToken::Soft))
                    .with_corner_radius(8.)
                    .a11y_alt(format!("Wiki source {}", hit.title))
                    .on_press(move |_| open_shell.write().open_note_path(&path))
                    .child(label().font_weight(FontWeight::BOLD).text(hit.title.clone()))
                    .child(
                        label()
                            .color(theme::token_color(palette, theme::ThemeToken::Muted))
                            .text(format!("{} · {}", hit.relative_path, hit.heading)),
                    )
                    .child(label().text(hit.excerpt.clone())),
            );
        }
        surface = surface.child(source_list);
    }

    let cards = drafts
        .into_iter()
        .map(|draft| wiki_card(shell_state, view_state, draft, palette))
        .collect::<Vec<_>>();
    let draft_surface = if cards.is_empty() {
        rect()
            .width(Size::fill())
            .padding(Gaps::new_all(24.))
            .background(theme::token_color(palette, theme::ThemeToken::Surface))
            .with_corner_radius(12.)
            .a11y_alt("Wiki empty state")
            .child(label().text("No Wiki drafts are stored in this vault."))
            .into_element()
    } else {
        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .show_scrollbar(true)
            .child(rect().width(Size::fill()).spacing(10.).children(cards))
            .into_element()
    };
    surface.child(draft_surface).into_element()
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
        Some(action_button(
            "Accept Wiki draft",
            "Accept",
            view_state.read().busy,
            move || accept_wiki(accept_shell, accept_view, accept_id.clone()),
        ))
    } else {
        None
    };
    let dismiss = if matches!(draft.status, WikiDraftStatus::Proposed | WikiDraftStatus::Outdated) {
        let dismiss_shell = shell_state;
        let dismiss_view = view_state;
        let dismiss_id = draft.id.clone();
        Some(action_button(
            "Dismiss Wiki draft",
            "Dismiss",
            view_state.read().busy,
            move || dismiss_wiki(dismiss_shell, dismiss_view, dismiss_id.clone()),
        ))
    } else {
        None
    };
    let regenerate = if draft.status == WikiDraftStatus::Outdated {
        let regenerate_shell = shell_state;
        let regenerate_view = view_state;
        let topic = draft.topic.clone();
        let title = draft.title.clone();
        Some(action_button(
            "Regenerate outdated Wiki from current sources",
            "Regenerate",
            view_state.read().busy,
            move || {
                generate_wiki(
                    regenerate_shell,
                    regenerate_view,
                    topic.clone(),
                    Some(title.clone()),
                    false,
                )
            },
        ))
    } else {
        None
    };
    let open = if draft.status == WikiDraftStatus::Accepted {
        let mut open_shell = shell_state;
        let path = format!(".elephantnote/wiki/{}.md", draft.slug);
        Some(action_button(
            "Open accepted Wiki",
            "Open",
            false,
            move || open_shell.write().open_note_path(&path),
        ))
    } else {
        None
    };

    let mut citations = rect().width(Size::fill()).spacing(3.);
    for citation in draft.citations.iter().take(4) {
        citations = citations.child(
            label()
                .font_size(11.)
                .color(theme::token_color(palette, theme::ThemeToken::Muted))
                .text(format!(
                    "{} · {} · bytes {}–{}",
                    citation.document_path,
                    citation.heading,
                    citation.start_offset,
                    citation.end_offset
                )),
        );
    }

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
            "{} citations · {} sources · model {}",
            draft.citations.len(),
            draft.source_paths.len(),
            draft.model_id
        )))
        .child(citations)
        .child(
            rect()
                .horizontal()
                .spacing(7.)
                .maybe_child(accept)
                .maybe_child(dismiss)
                .maybe_child(regenerate)
                .maybe_child(open),
        )
        .into_element()
}

fn search_sources(
    shell_state: State<ShellState>,
    mut view_state: State<WikiViewState>,
    query: String,
) {
    let query = query.trim().to_owned();
    if query.is_empty() {
        view_state.write().fail("Enter a Wiki topic before searching sources.".to_owned());
        return;
    }
    let Some(vault) = shell_state.read().vault.clone() else {
        view_state.write().fail("No active vault.".to_owned());
        return;
    };
    if view_state.read().busy {
        return;
    }
    view_state.write().busy = true;
    let root = vault.root().to_path_buf();
    let expected_vault = vault.descriptor().path.clone();
    let shell_for_apply = shell_state;
    let mut view_for_apply = view_state;
    background::run(
        "wiki-source-search",
        move || -> Result<Vec<KnowledgeSearchHit>, String> {
            rebuild_vault(&root).map_err(|error| error.to_string())?;
            KnowledgeStore::open(&root)?.search(&query, SOURCE_SEARCH_LIMIT)
        },
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            let mut state = view_for_apply.write();
            state.busy = false;
            match outcome {
                Ok(Ok(results)) => {
                    state.source_results = results;
                    state.error = None;
                    state.revision = state.revision.saturating_add(1);
                }
                Ok(Err(error)) => state.error = Some(format!("Wiki source search failed: {error}")),
                Err(error) => state.error = Some(format!("Wiki source worker failed: {error}")),
            }
        },
    );
}

fn generate_wiki(
    shell_state: State<ShellState>,
    mut view_state: State<WikiViewState>,
    topic: String,
    requested_title: Option<String>,
    auto_accept: bool,
) {
    let topic = topic.trim().to_owned();
    if topic.is_empty() {
        view_state.write().fail("Enter a Wiki topic before generation.".to_owned());
        return;
    }
    let snapshot = shell_state.read().clone();
    let Some(vault) = snapshot.vault.as_ref() else {
        view_state.write().fail("No active vault.".to_owned());
        return;
    };
    let model = snapshot.chat.model.trim().to_owned();
    let base_url = snapshot.chat.base_url.trim().to_owned();
    if model.is_empty() {
        view_state.write().fail(
            "Configure a Chat/PI model before generating a Wiki. Wiki generation never falls back to uncited local text."
                .to_owned(),
        );
        return;
    }
    if view_state.read().busy {
        return;
    }

    view_state.write().busy = true;
    view_state.write().error = None;
    let root = vault.root().to_path_buf();
    let expected_vault = vault.descriptor().path.clone();
    let topic_for_worker = topic.clone();
    let title_for_worker = requested_title.clone();
    let shell_for_prepare = shell_state;
    let view_for_prepare = view_state;
    background::run(
        "wiki-prepare",
        move || prepare_wiki_request(&root, &topic_for_worker, title_for_worker.as_deref()),
        move |outcome| {
            if !same_active_vault(shell_for_prepare, &expected_vault) {
                return;
            }
            let (sources, request, root) = match outcome {
                Ok(Ok(value)) => value,
                Ok(Err(error)) => {
                    view_for_prepare
                        .write()
                        .fail(format!("Wiki preparation failed: {error}"));
                    return;
                }
                Err(error) => {
                    view_for_prepare
                        .write()
                        .fail(format!("Wiki preparation worker failed: {error}"));
                    return;
                }
            };

            let messages = vec![
                json!({ "role": "system", "content": request.system_prompt }),
                json!({ "role": "user", "content": request.user_prompt }),
            ];
            let payload = json!({
                "baseUrl": base_url,
                "maxTokens": request.max_output_tokens,
                "temperature": 0.1
            });
            let mut view_after_model = view_for_prepare;
            let shell_after_model = shell_for_prepare;
            let expected_after_model = expected_vault.clone();
            let topic_after_model = topic.clone();
            let model_after_model = model.clone();
            spawn(async move {
                let result = pi_adapter::complete(&model_after_model, &messages, &payload).await;
                if !same_active_vault(shell_after_model, &expected_after_model) {
                    return;
                }
                let result = result.and_then(|answer| {
                    let rendered = parse_and_render_wiki(
                        &answer.answer,
                        &topic_after_model,
                        &sources,
                        MAX_WIKI_SECTIONS,
                    )?;
                    let draft = wiki_draft_from_rendered(
                        &topic_after_model,
                        rendered,
                        &model_after_model,
                        unix_timestamp(),
                    );
                    let store = KnowledgeStore::open(&root)?;
                    store.save_wiki_draft(&draft)?;
                    if auto_accept {
                        store.accept_wiki_draft(&root, &draft.id).map(|_| ())?;
                    }
                    Ok::<_, String>(draft)
                });
                let mut state = view_after_model.write();
                state.busy = false;
                match result {
                    Ok(draft) => {
                        eprintln!(
                            "[freya][wiki] action=generation-complete draft={} auto_accept={} citations={}",
                            draft.id,
                            auto_accept,
                            draft.citations.len()
                        );
                        state.error = None;
                        state.revision = state.revision.saturating_add(1);
                    }
                    Err(error) => {
                        eprintln!("[freya][wiki] action=generation-failure error={error}");
                        state.error = Some(format!("Wiki generation failed: {error}"));
                    }
                }
            });
        },
    );
}

fn prepare_wiki_request(
    root: &Path,
    topic: &str,
    requested_title: Option<&str>,
) -> Result<(
    Vec<WikiSourceChunk>,
    elephantnote_knowledge_core::StructuredModelRequest,
    PathBuf,
), String> {
    rebuild_vault(root).map_err(|error| error.to_string())?;
    let store = KnowledgeStore::open(root)?;
    let hits = store.search(topic, SOURCE_SEARCH_LIMIT)?;
    if hits.is_empty() {
        return Err("No indexed note chunks match this Wiki topic.".to_owned());
    }

    let mut source_ids = BTreeSet::new();
    let mut sources = Vec::new();
    for hit in hits {
        if !source_ids.insert(hit.chunk_id.clone()) {
            continue;
        }
        let Some(document) = store.inspect_document(&hit.relative_path)? else {
            continue;
        };
        let Some(chunk) = document.chunks.iter().find(|chunk| chunk.id == hit.chunk_id) else {
            continue;
        };
        sources.push(WikiSourceChunk {
            document_path: document.relative_path.clone(),
            document_title: document.title.clone(),
            chunk_id: chunk.id.clone(),
            heading: hit.heading,
            start_offset: chunk.start_offset,
            end_offset: chunk.end_offset,
            text: chunk.text.clone(),
        });
    }
    if sources.is_empty() {
        return Err("Knowledge search returned no readable source chunks.".to_owned());
    }
    let request = build_wiki_synthesis_request(
        topic,
        requested_title,
        &sources,
        MAX_WIKI_SECTIONS,
    );
    Ok((sources, request, root.to_path_buf()))
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
    let root = vault.root().to_path_buf();
    let expected_vault = vault.descriptor().path.clone();
    view_state.write().busy = true;
    let shell_for_apply = shell_state;
    let mut view_for_apply = view_state;
    background::run(
        "wiki-accept",
        move || {
            KnowledgeStore::open(&root)?
                .accept_wiki_draft(&root, &draft_id)
                .map(|_| ())
        },
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            let mut state = view_for_apply.write();
            state.busy = false;
            match outcome {
                Ok(Ok(())) => state.refresh(),
                Ok(Err(error)) => state.error = Some(error),
                Err(error) => state.error = Some(format!("Wiki accept worker failed: {error}")),
            }
        },
    );
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
    let root = vault.root().to_path_buf();
    let expected_vault = vault.descriptor().path.clone();
    view_state.write().busy = true;
    let shell_for_apply = shell_state;
    let mut view_for_apply = view_state;
    background::run(
        "wiki-dismiss",
        move || {
            KnowledgeStore::open(&root)?
                .set_wiki_draft_status(&draft_id, WikiDraftStatus::Rejected)
                .map(|_| ())
        },
        move |outcome| {
            if !same_active_vault(shell_for_apply, &expected_vault) {
                return;
            }
            let mut state = view_for_apply.write();
            state.busy = false;
            match outcome {
                Ok(Ok(())) => state.refresh(),
                Ok(Err(error)) => state.error = Some(error),
                Err(error) => state.error = Some(format!("Wiki dismiss worker failed: {error}")),
            }
        },
    );
}

fn same_active_vault(shell: State<ShellState>, expected_path: &str) -> bool {
    shell
        .read()
        .vault
        .as_ref()
        .is_some_and(|vault| vault.descriptor().path == expected_path)
}

fn action_button(
    alt: &'static str,
    text: &'static str,
    disabled: bool,
    mut action: impl FnMut() + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(7., 10., 7., 10.))
        .with_corner_radius(8.)
        .background(theme::color(theme::SURFACE))
        .a11y_alt(alt)
        .on_press(move |_| {
            if !disabled {
                action();
            }
        })
        .child(label().font_weight(FontWeight::BOLD).text(text))
        .into_element()
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or_default()
}

fn status_label(status: &WikiDraftStatus) -> &'static str {
    match status {
        WikiDraftStatus::Proposed => "Proposed",
        WikiDraftStatus::Accepted => "Accepted",
        WikiDraftStatus::Rejected => "Dismissed",
        WikiDraftStatus::Outdated => "Outdated",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn source_preparation_uses_real_knowledge_chunks() {
        let root = std::env::temp_dir().join(format!(
            "elephant-wiki-source-{}",
            unix_timestamp()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("Iroh.md"),
            "# Iroh\n\nIroh synchronizes files peer to peer.\n",
        )
        .unwrap();
        let (sources, request, _) = prepare_wiki_request(&root, "Iroh", None).unwrap();
        assert!(!sources.is_empty());
        assert!(request.user_prompt.contains("Iroh"));
        assert!(request.user_prompt.contains(&sources[0].chunk_id));
        let _ = fs::remove_dir_all(root);
    }
}
