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
use std::{fmt, path::Path, time::Instant};

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
    let max_links_per_note = nodes
        .iter()
        .map(|node| {
            edges
                .iter()
                .filter(|edge| edge.source == node.id || edge.target == node.id)
                .count()
        })
        .max()
        .unwrap_or(0);

    GraphSnapshot {
        generated_at: String::new(),
        vault_root: root.to_string_lossy().replace('\\', "/"),
        source: GraphDataSource::KnowledgeCore,
        nodes,
        edges: edges.clone(),
        clusters,
        stats: GraphStats {
            total_notes,
            total_candidate_edges: edges.len(),
            rendered_edges: edges.len(),
            max_links_per_note,
            max_edges: edges.len(),
        },
        index_path: store.database_path().to_string_lossy().replace('\\', "/"),
        search_status: None,
    }
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
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn physical_205_note_vault_has_no_silent_200_node_ceiling() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-freya-graph-205-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        for index in 0..205usize {
            let previous = index.saturating_sub(1);
            let next = (index + 1).min(204);
            fs::write(
                root.join(format!("Note-{index:03}.md")),
                format!(
                    "# Note {index:03}\n\n#graph-test\n\n[[Note-{previous:03}]] [[Note-{next:03}]]\n"
                ),
            )
            .unwrap();
        }

        let execution = refresh_root(&root, false).expect("build real 205-note graph");
        assert!(
            execution.snapshot.nodes.len() >= 205,
            "graph must retain every physical note, got {} nodes",
            execution.snapshot.nodes.len()
        );
        for expected in [0usize, 199, 200, 204] {
            let path = format!("Note-{expected:03}.md");
            assert!(
                execution
                    .snapshot
                    .nodes
                    .iter()
                    .any(|node| node.relative_path == path),
                "graph must retain {path}"
            );
        }
        assert!(
            execution
                .snapshot
                .edges
                .iter()
                .any(|edge| edge.reason.contains("Note-204") || edge.reason.contains("Note-203"))
                || !execution.snapshot.edges.is_empty(),
            "graph should expose real relationships for the fixture"
        );
        let _ = fs::remove_dir_all(root);
    }
}
