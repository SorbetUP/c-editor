//! Freya adapter for the historical native knowledge graph service.
//!
//! Provenance is the unchanged `elephantnote-knowledge-core` implementation
//! transplanted from `feature/rust-knowledge-core`:
//! `pipeline::rebuild_vault` and `storage::KnowledgeStore::graph_projection`.
//! This module only executes that service for the active Freya vault and maps
//! its serialized graph into the renderer contract. It never creates fallback
//! nodes, edges, clusters, or positions.

use crate::{
    search_graph_contract::{
        GraphCluster, GraphClusterKind, GraphDataSource, GraphEdge, GraphNode, GraphNodeKind,
        GraphSnapshot, GraphStats,
    },
    vault_adapter::VaultAdapter,
};
use elephantnote_knowledge_core::{rebuild_vault, KnowledgeGraph, KnowledgeStore};
use std::{collections::BTreeMap, fmt, path::Path, time::Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GraphRuntimeError(String);

impl GraphRuntimeError {
    pub(super) fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GraphRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for GraphRuntimeError {}

pub(super) struct GraphExecution {
    pub snapshot: GraphSnapshot,
}

pub(super) fn refresh(
    vault: Option<&VaultAdapter>,
    include_suggestions: bool,
) -> Result<GraphExecution, GraphRuntimeError> {
    let vault = vault.ok_or_else(|| GraphRuntimeError("No active vault.".to_string()))?;
    refresh_root(vault.root(), include_suggestions)
}

fn refresh_root(
    root: &Path,
    include_suggestions: bool,
) -> Result<GraphExecution, GraphRuntimeError> {
    let started_at = Instant::now();
    eprintln!(
        "[freya][graph] action=start service=elephantnote-knowledge-core root={} include_suggestions={include_suggestions}",
        root.display()
    );

    let rebuild = rebuild_vault(root).map_err(|error| {
        eprintln!(
            "[freya][graph] action=rebuild:error root={} error={error}",
            root.display()
        );
        GraphRuntimeError(format!("Knowledge graph rebuild failed: {error}"))
    })?;
    eprintln!(
        "[freya][graph] action=rebuild:complete root={} scanned={} indexed={} unchanged={} removed={} failed={}",
        root.display(),
        rebuild.scanned,
        rebuild.indexed,
        rebuild.unchanged,
        rebuild.removed,
        rebuild.failed.len()
    );

    let store = KnowledgeStore::open(root).map_err(|error| {
        eprintln!(
            "[freya][graph] action=open:error root={} error={error}",
            root.display()
        );
        GraphRuntimeError(format!("Knowledge graph store failed: {error}"))
    })?;
    let graph = store
        .graph_projection(include_suggestions)
        .map_err(|error| {
            eprintln!(
                "[freya][graph] action=projection:error database={} error={error}",
                store.database_path().display()
            );
            GraphRuntimeError(format!("Knowledge graph projection failed: {error}"))
        })?;
    let snapshot = map_graph(root, &store, graph);
    eprintln!(
        "[freya][graph] action=complete root={} nodes={} edges={} clusters={} database={} duration_ms={}",
        root.display(),
        snapshot.nodes.len(),
        snapshot.edges.len(),
        snapshot.clusters.len(),
        snapshot.index_path,
        started_at.elapsed().as_millis()
    );
    Ok(GraphExecution { snapshot })
}

fn map_graph(root: &Path, store: &KnowledgeStore, graph: KnowledgeGraph) -> GraphSnapshot {
    let nodes = graph.nodes.into_iter().map(map_node).collect::<Vec<_>>();
    let edges = graph.edges.into_iter().map(map_edge).collect::<Vec<_>>();
    let clusters = graph
        .clusters
        .into_iter()
        .map(map_cluster)
        .collect::<Vec<_>>();
    let total_notes = nodes
        .iter()
        .filter(|node| node.kind == GraphNodeKind::Note)
        .count();
    let max_links_per_note = max_incident_edges(&nodes, &edges);
    let edge_count = edges.len();

    GraphSnapshot {
        // The historical graph payload has no generatedAt field. Keep this
        // field empty instead of manufacturing a timestamp in the adapter.
        generated_at: String::new(),
        vault_root: root.to_string_lossy().replace('\\', "/"),
        // This is the historical Rust service itself, not the search
        // inspection or Atomic JS service. Keep the provenance observable in
        // the contract so callers cannot mistake the adapter for a fallback.
        source: GraphDataSource::KnowledgeCore,
        nodes,
        edges,
        clusters,
        stats: GraphStats {
            total_notes,
            total_candidate_edges: edge_count,
            rendered_edges: edge_count,
            max_links_per_note,
            max_edges: edge_count,
        },
        index_path: store.database_path().to_string_lossy().replace('\\', "/"),
        search_status: None,
    }
}

/// Computes the same incident-edge maximum as the previous per-node scan, but
/// in O(V + E) instead of O(V * E). This matters for the production contract,
/// which explicitly allows thousands of nodes and edges.
fn max_incident_edges(nodes: &[GraphNode], edges: &[GraphEdge]) -> usize {
    let mut degree_by_id = BTreeMap::<&str, usize>::new();
    for edge in edges {
        *degree_by_id.entry(edge.source.as_str()).or_default() += 1;
        *degree_by_id.entry(edge.target.as_str()).or_default() += 1;
    }
    nodes
        .iter()
        .map(|node| degree_by_id.get(node.id.as_str()).copied().unwrap_or(0))
        .max()
        .unwrap_or(0)
}

fn map_node(node: elephantnote_knowledge_core::KnowledgeGraphNode) -> GraphNode {
    GraphNode {
        id: node.id,
        relative_path: node.relative_path,
        title: node.title,
        kind: GraphNodeKind::parse(&node.kind),
        folder: String::new(),
        tags: node.tags,
        summary: node.summary,
        headings: Vec::new(),
        key_terms: Vec::new(),
        updated_at: String::new(),
        weak_title: false,
        source_count: node.source_count,
        chunk_count: node.chunk_count,
        position: None,
    }
}

fn map_edge(edge: elephantnote_knowledge_core::KnowledgeGraphEdge) -> GraphEdge {
    GraphEdge {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        edge_type: crate::search_graph_contract::GraphEdgeType::parse(&edge.edge_type),
        reason: edge.reason,
        weight: edge.weight,
    }
}

fn map_cluster(cluster: elephantnote_knowledge_core::KnowledgeGraphCluster) -> GraphCluster {
    let kind = if cluster.tags.iter().any(|tag| tag == "wiki-territory") {
        GraphClusterKind::Semantic
    } else if cluster.tags.iter().any(|tag| tag == "folder-cluster") {
        GraphClusterKind::Folder
    } else {
        GraphClusterKind::Other
    };
    GraphCluster {
        id: cluster.id,
        label: cluster.label,
        node_count: cluster.node_count,
        paths: cluster.paths,
        tags: cluster.tags,
        cohesion: 0.0,
        edge_count: 0,
        key_terms: Vec::new(),
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_graph_contract::GraphEdgeType;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TempVault {
        root: PathBuf,
    }

    impl TempVault {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "elephant-freya-graph-{label}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create temp graph vault");
            Self { root }
        }
    }

    impl Drop for TempVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_note(root: &Path, index: usize, body: &str) {
        fs::write(
            root.join(format!("note-{index:03}.md")),
            format!("# Note {index:03}\n\n{body}\n"),
        )
        .expect("write markdown note");
    }

    fn contract_note(id: &str) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            relative_path: id.to_string(),
            title: id.to_string(),
            kind: GraphNodeKind::Note,
            folder: String::new(),
            tags: Vec::new(),
            summary: String::new(),
            headings: Vec::new(),
            key_terms: Vec::new(),
            updated_at: String::new(),
            weak_title: false,
            source_count: 0,
            chunk_count: 1,
            position: None,
        }
    }

    fn contract_edge(id: &str, source: &str, target: &str) -> GraphEdge {
        GraphEdge {
            id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            edge_type: GraphEdgeType::ExplicitLink,
            reason: "test".to_string(),
            weight: 1.0,
        }
    }

    #[test]
    fn incident_edge_stats_are_linear_and_preserve_existing_semantics() {
        let nodes = vec![
            contract_note("a.md"),
            contract_note("b.md"),
            contract_note("c.md"),
            contract_note("isolated.md"),
        ];
        let edges = vec![
            contract_edge("ab", "a.md", "b.md"),
            contract_edge("ac", "a.md", "c.md"),
            contract_edge("bc", "b.md", "c.md"),
        ];
        assert_eq!(max_incident_edges(&nodes, &edges), 2);
        assert_eq!(max_incident_edges(&nodes, &[]), 0);
    }

    #[test]
    fn production_projection_handles_small_real_markdown_graph() {
        let vault = TempVault::new("small");
        write_note(&vault.root, 0, "[[Note 001]] #alpha");
        write_note(&vault.root, 1, "[[Note 002]] #alpha #shared");
        write_note(&vault.root, 2, "#isolated");

        let execution = refresh_root(&vault.root, false).expect("project small graph");
        let snapshot = execution.snapshot;

        assert_eq!(snapshot.stats.total_notes, 3);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|node| node.kind == GraphNodeKind::Note)
                .count(),
            3
        );
        assert!(snapshot.edges.iter().any(|edge| {
            edge.edge_type == GraphEdgeType::ExplicitLink
                && edge.reason.to_ascii_lowercase().contains("wiki")
        }));
        assert!(snapshot
            .nodes
            .iter()
            .any(|node| node.relative_path.ends_with("note-002.md")));
    }

    #[test]
    fn production_projection_does_not_truncate_at_two_hundred_notes() {
        const NOTE_COUNT: usize = 205;
        let vault = TempVault::new("205");

        for index in 0..NOTE_COUNT {
            let body = match index {
                // Multiple outgoing wiki links exercise fan-out and edge mapping.
                203 => "[[Note 000]] [[Note 050]] [[Note 150]] #fanout".to_string(),
                // Keep the final note intentionally isolated.
                204 => "#isolated long-label-regression-fixture".to_string(),
                // Leave 202 -> 203 connected while 204 remains isolated.
                _ if index < 203 => format!("[[Note {:03}]] #cluster-{}", index + 1, index % 7),
                _ => "#tail".to_string(),
            };
            write_note(&vault.root, index, &body);
        }

        let execution = refresh_root(&vault.root, false).expect("project 205-note graph");
        let snapshot = execution.snapshot;

        assert_eq!(snapshot.stats.total_notes, NOTE_COUNT);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|node| node.kind == GraphNodeKind::Note)
                .count(),
            NOTE_COUNT,
            "the production projection must never silently stop at 200 notes"
        );
        for index in 0..NOTE_COUNT {
            let suffix = format!("note-{index:03}.md");
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|node| node.relative_path.ends_with(&suffix)),
                "missing projected node {suffix}"
            );
        }

        let explicit_links = snapshot
            .edges
            .iter()
            .filter(|edge| edge.edge_type == GraphEdgeType::ExplicitLink)
            .count();
        assert!(
            explicit_links >= 205,
            "expected the chain plus fan-out wiki links, got {explicit_links} explicit edges"
        );

        let isolated = snapshot
            .nodes
            .iter()
            .find(|node| node.relative_path.ends_with("note-204.md"))
            .expect("isolated node is present");
        assert!(
            snapshot
                .edges
                .iter()
                .all(|edge| edge.source != isolated.id && edge.target != isolated.id),
            "the deliberately isolated note must remain an isolated graph node"
        );
    }
}
