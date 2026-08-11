use crate::graph::{KnowledgeGraphCluster, KnowledgeGraphEdge, KnowledgeGraphNode};
use crate::wiki_core::{WikiDraft, WikiDraftStatus};
use crate::KnowledgeStore;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Default)]
pub(crate) struct WikiGraphProjection {
    pub nodes: Vec<KnowledgeGraphNode>,
    pub edges: Vec<KnowledgeGraphEdge>,
    pub clusters: Vec<KnowledgeGraphCluster>,
    pub assigned_document_ids: HashSet<String>,
}

pub(crate) fn project_wiki_territories(
    store: &KnowledgeStore,
    document_ids: &HashSet<String>,
    include_suggestions: bool,
) -> Result<WikiGraphProjection, String> {
    let started_at = Instant::now();
    eprintln!(
        "[Knowledge][Graph][Wiki] projection:start include_suggestions={} indexed_documents={}",
        include_suggestions,
        document_ids.len()
    );

    let drafts = store.list_wiki_drafts(None, 10_000)?;
    let visible = drafts
        .into_iter()
        .filter(|draft| wiki_is_visible(&draft.status, include_suggestions))
        .collect::<Vec<_>>();
    eprintln!(
        "[Knowledge][Graph][Wiki] drafts:loaded visible={} include_suggestions={}",
        visible.len(),
        include_suggestions
    );

    let wiki_by_reference = build_wiki_reference_index(&visible);
    let mut projection = WikiGraphProjection::default();
    let mut seen_edges = HashSet::new();

    for draft in &visible {
        let wiki_node_id = wiki_node_id(draft);
        let status = wiki_status_name(&draft.status);
        let mut resolved_sources = Vec::new();
        let mut missing_sources = Vec::new();
        for source_path in &draft.source_paths {
            if document_ids.contains(source_path) {
                resolved_sources.push(source_path.clone());
                projection.assigned_document_ids.insert(source_path.clone());
            } else {
                missing_sources.push(source_path.clone());
            }
        }
        resolved_sources.sort();
        resolved_sources.dedup();
        missing_sources.sort();
        missing_sources.dedup();

        projection.nodes.push(KnowledgeGraphNode {
            id: wiki_node_id.clone(),
            path: format!(".elephantnote/wiki/{}.md", draft.slug),
            relative_path: format!(".elephantnote/wiki/{}.md", draft.slug),
            title: draft.title.clone(),
            kind: "wiki".into(),
            node_type: "wiki".into(),
            summary: draft.topic.clone(),
            tags: vec![format!("wiki-status:{status}")],
            source_count: resolved_sources.len(),
            chunk_count: draft.citations.len(),
        });

        let mut cluster_paths = vec![wiki_node_id.clone()];
        for source_path in &resolved_sources {
            cluster_paths.push(source_path.clone());
            let edge_id = format!(
                "wiki-source:{}:{}",
                draft.id,
                blake3::hash(source_path.as_bytes()).to_hex()
            );
            if !seen_edges.insert(edge_id.clone()) {
                continue;
            }
            projection.edges.push(KnowledgeGraphEdge {
                id: edge_id,
                source: wiki_node_id.clone(),
                target: source_path.clone(),
                edge_type: "wiki-source".into(),
                relation_type: "wiki_source".into(),
                reason: "Cited source of this Wiki".into(),
                weight: 1.0,
                origin: "wiki".into(),
                status: status.into(),
                evidence_chunk_ids: draft
                    .citations
                    .iter()
                    .filter(|citation| citation.document_path == *source_path)
                    .map(|citation| citation.chunk_id.clone())
                    .collect(),
            });
        }

        projection.clusters.push(KnowledgeGraphCluster {
            id: wiki_node_id.clone(),
            label: draft.title.clone(),
            node_count: cluster_paths.len(),
            paths: cluster_paths,
            tags: vec!["wiki-territory".into(), format!("status:{status}")],
        });

        eprintln!(
            "[Knowledge][Graph][Wiki] territory:projected id={} slug={} status={} resolved_sources={} missing_sources={} citations={}",
            draft.id,
            draft.slug,
            status,
            resolved_sources.len(),
            missing_sources.len(),
            draft.citations.len()
        );
        for path in missing_sources {
            eprintln!(
                "[Knowledge][Graph][Wiki] source:missing wiki_id={} path={}",
                draft.id, path
            );
        }
    }

    for draft in &visible {
        let source_id = wiki_node_id(draft);
        let related = parse_related_wiki_references(&draft.markdown);
        let mut resolved = 0usize;
        let mut unresolved = 0usize;
        for reference in related {
            let normalized = normalize_wiki_reference(&reference);
            let Some(target_id) = wiki_by_reference.get(&normalized) else {
                unresolved += 1;
                eprintln!(
                    "[Knowledge][Graph][Wiki] related:unresolved wiki_id={} reference={}",
                    draft.id, reference
                );
                continue;
            };
            if target_id == &source_id {
                continue;
            }
            let edge_id = format!("wiki-link:{}:{}", source_id, target_id);
            if !seen_edges.insert(edge_id.clone()) {
                continue;
            }
            projection.edges.push(KnowledgeGraphEdge {
                id: edge_id,
                source: source_id.clone(),
                target: target_id.clone(),
                edge_type: "wiki-link".into(),
                relation_type: "wiki_cites_wiki".into(),
                reason: format!("Related Wiki: {reference}"),
                weight: 1.0,
                origin: "wiki".into(),
                status: wiki_status_name(&draft.status).into(),
                evidence_chunk_ids: Vec::new(),
            });
            resolved += 1;
        }
        if resolved > 0 || unresolved > 0 {
            eprintln!(
                "[Knowledge][Graph][Wiki] related:projected wiki_id={} resolved={} unresolved={}",
                draft.id, resolved, unresolved
            );
        }
    }

    projection
        .nodes
        .sort_by(|left, right| left.id.cmp(&right.id));
    projection
        .edges
        .sort_by(|left, right| left.id.cmp(&right.id));
    projection
        .clusters
        .sort_by(|left, right| left.id.cmp(&right.id));
    eprintln!(
        "[Knowledge][Graph][Wiki] projection:complete wiki_nodes={} wiki_edges={} territories={} assigned_documents={} duration_ms={}",
        projection.nodes.len(),
        projection.edges.len(),
        projection.clusters.len(),
        projection.assigned_document_ids.len(),
        started_at.elapsed().as_millis()
    );
    Ok(projection)
}

fn wiki_is_visible(status: &WikiDraftStatus, include_suggestions: bool) -> bool {
    matches!(
        status,
        WikiDraftStatus::Accepted | WikiDraftStatus::Outdated
    ) || (include_suggestions && matches!(status, WikiDraftStatus::Proposed))
}

fn wiki_node_id(draft: &WikiDraft) -> String {
    format!("wiki:{}", draft.id)
}

fn wiki_status_name(status: &WikiDraftStatus) -> &'static str {
    match status {
        WikiDraftStatus::Proposed => "proposed",
        WikiDraftStatus::Accepted => "accepted",
        WikiDraftStatus::Rejected => "rejected",
        WikiDraftStatus::Outdated => "outdated",
    }
}

fn build_wiki_reference_index(drafts: &[WikiDraft]) -> HashMap<String, String> {
    let mut output = HashMap::new();
    for draft in drafts {
        let node_id = wiki_node_id(draft);
        for reference in [&draft.title, &draft.slug, &draft.topic] {
            let normalized = normalize_wiki_reference(reference);
            if !normalized.is_empty() {
                output.entry(normalized).or_insert_with(|| node_id.clone());
            }
        }
    }
    output
}

fn parse_related_wiki_references(markdown: &str) -> Vec<String> {
    let mut in_related_section = false;
    let mut references = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_related_section = heading.trim().eq_ignore_ascii_case("related wikis");
            continue;
        }
        if !in_related_section {
            continue;
        }
        let mut remainder = trimmed;
        while let Some(start) = remainder.find("[[") {
            let after_start = &remainder[start + 2..];
            let Some(end) = after_start.find("]]") else {
                break;
            };
            let raw = &after_start[..end];
            let target = raw
                .split('|')
                .next()
                .unwrap_or(raw)
                .split('#')
                .next()
                .unwrap_or(raw)
                .trim();
            if !target.is_empty() {
                references.push(target.to_string());
            }
            remainder = &after_start[end + 2..];
        }
    }
    references.sort();
    references.dedup();
    references
}

fn normalize_wiki_reference(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".md")
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;
    use crate::wiki_core::{WikiCitation, WikiDraft};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "elephant-wiki-graph-{name}-{}-{stamp}",
            std::process::id()
        ))
    }

    fn draft(id: &str, title: &str, status: WikiDraftStatus, sources: Vec<&str>) -> WikiDraft {
        WikiDraft {
            id: id.into(),
            topic: title.into(),
            title: title.into(),
            slug: normalize_wiki_reference(title),
            markdown: format!("# {title}\n\n## Related wikis\n"),
            citations: sources
                .iter()
                .enumerate()
                .map(|(index, path)| WikiCitation {
                    key: format!("source-{}", index + 1),
                    document_path: (*path).into(),
                    document_title: (*path).into(),
                    chunk_id: format!("chunk-{index}"),
                    heading: title.into(),
                    start_offset: 0,
                    end_offset: 10,
                })
                .collect(),
            source_paths: sources.into_iter().map(String::from).collect(),
            source_hash: "hash".into(),
            model_id: "test-model".into(),
            status,
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn accepted_wiki_becomes_a_territory_with_source_edges() {
        let root = temp_vault("accepted");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        store
            .upsert_document(&analyze_markdown("Notes/A.md", "# A\nBody", 1))
            .unwrap();
        store
            .save_wiki_draft(&draft(
                "wiki-a",
                "Topic A",
                WikiDraftStatus::Accepted,
                vec!["Notes/A.md"],
            ))
            .unwrap();

        let ids = HashSet::from(["Notes/A.md".to_string()]);
        let projection = project_wiki_territories(&store, &ids, false).unwrap();
        assert_eq!(projection.nodes.len(), 1);
        assert_eq!(projection.nodes[0].kind, "wiki");
        assert_eq!(projection.edges.len(), 1);
        assert_eq!(projection.edges[0].edge_type, "wiki-source");
        assert_eq!(projection.clusters[0].paths.len(), 2);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn proposed_wikis_are_only_visible_with_suggestions() {
        let root = temp_vault("proposed");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        store
            .upsert_document(&analyze_markdown("A.md", "# A\nBody", 1))
            .unwrap();
        store
            .save_wiki_draft(&draft(
                "wiki-a",
                "Topic A",
                WikiDraftStatus::Proposed,
                vec!["A.md"],
            ))
            .unwrap();
        let ids = HashSet::from(["A.md".to_string()]);
        assert!(project_wiki_territories(&store, &ids, false)
            .unwrap()
            .nodes
            .is_empty());
        assert_eq!(
            project_wiki_territories(&store, &ids, true)
                .unwrap()
                .nodes
                .len(),
            1
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn related_wikis_create_wiki_to_wiki_edges() {
        let root = temp_vault("related");
        fs::create_dir_all(&root).unwrap();
        let store = KnowledgeStore::open(&root).unwrap();
        let mut first = draft("wiki-a", "Topic A", WikiDraftStatus::Accepted, vec!["A.md"]);
        first.markdown = "# Topic A\n\n## Related wikis\n\n- [[Topic B]]\n".into();
        store.save_wiki_draft(&first).unwrap();
        store
            .save_wiki_draft(&draft(
                "wiki-b",
                "Topic B",
                WikiDraftStatus::Accepted,
                vec!["B.md"],
            ))
            .unwrap();
        let ids = HashSet::from(["A.md".to_string(), "B.md".to_string()]);
        let projection = project_wiki_territories(&store, &ids, false).unwrap();
        assert!(projection
            .edges
            .iter()
            .any(|edge| edge.edge_type == "wiki-link"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn parses_only_related_wiki_section_links() {
        let markdown = "# A\n\n[[Not related]]\n\n## Related wikis\n\n- [[B|Bee]]\n- [[C#Part]]\n\n## Sources\n\n[[Ignored source]]";
        assert_eq!(parse_related_wiki_references(markdown), vec!["B", "C"]);
    }
}
