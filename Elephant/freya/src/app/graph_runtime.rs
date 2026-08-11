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
        // The historical graph payload has no generatedAt field. Keep this
        // field empty instead of manufacturing a timestamp in the adapter.
        generated_at: String::new(),
        vault_root: root.to_string_lossy().replace('\\', "/"),
        // This is the historical Rust service itself, not the search
        // inspection or Atomic JS service. Keep the provenance observable in
        // the contract so callers cannot mistake the adapter for a fallback.
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
