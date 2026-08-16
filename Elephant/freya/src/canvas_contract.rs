//! Renderer-independent Canvas domain contracts.
//!
//! The Canvas view receives its graph from the existing graph/search contract.
//! This module owns validation, viewport limits, node movement, and the small
//! persisted position document.  It deliberately contains no Freya elements,
//! filesystem calls, or graph discovery logic.

use crate::search_graph_contract::{GraphEdge, GraphNode, GraphPosition, GraphSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const CANVAS_SCHEMA_VERSION: u32 = 1;
pub const CANVAS_RELATIVE_PATH: &str = ".elephantnote/canvas.json";
pub const CANVAS_WORLD_WIDTH: f32 = 1_800.0;
pub const CANVAS_WORLD_HEIGHT: f32 = 1_200.0;
pub const CANVAS_MIN_ZOOM: f32 = 0.5;
pub const CANVAS_MAX_ZOOM: f32 = 1.8;
pub const CANVAS_ZOOM_STEP: f32 = 0.1;
pub const CANVAS_MAX_ID_LENGTH: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub enum CanvasError {
    InvalidId { kind: &'static str, id: String },
    DuplicateId { kind: &'static str, id: String },
    UnknownNode { id: String },
    InvalidEdgeEndpoint { edge: String, node: String },
    InvalidPosition { id: String },
    InvalidZoom { value: f32 },
    UnsupportedSchema { version: u32 },
}

impl fmt::Display for CanvasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId { kind, id } => write!(formatter, "Invalid {kind} id: {id:?}"),
            Self::DuplicateId { kind, id } => write!(formatter, "Duplicate {kind} id: {id:?}"),
            Self::UnknownNode { id } => write!(formatter, "Unknown Canvas node: {id:?}"),
            Self::InvalidEdgeEndpoint { edge, node } => {
                write!(formatter, "Edge {edge:?} references unknown node {node:?}")
            }
            Self::InvalidPosition { id } => write!(formatter, "Invalid position for node {id:?}"),
            Self::InvalidZoom { value } => write!(formatter, "Invalid Canvas zoom: {value}"),
            Self::UnsupportedSchema { version } => {
                write!(formatter, "Unsupported Canvas schema version: {version}")
            }
        }
    }
}

impl std::error::Error for CanvasError {}

/// A finite Canvas coordinate.  JSON cannot represent NaN or infinity, so the
/// domain rejects those values before they reach persistence or rendering.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanvasPosition {
    pub x: f32,
    pub y: f32,
}

impl CanvasPosition {
    pub fn new(x: f32, y: f32) -> Result<Self, CanvasError> {
        let position = Self { x, y };
        position
            .is_valid()
            .then_some(position)
            .ok_or_else(|| CanvasError::InvalidPosition {
                id: "<coordinate>".to_owned(),
            })
    }

    pub fn from_graph(id: &str, position: &GraphPosition) -> Result<Self, CanvasError> {
        Self::new(position.x, position.y)
            .map_err(|_| CanvasError::InvalidPosition { id: id.to_owned() })
    }

    pub fn translated(self, dx: f32, dy: f32) -> Result<Self, CanvasError> {
        Self::new(self.x + dx, self.y + dy)
    }

    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// The only state written to `.elephantnote/canvas.json`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasDocument {
    pub version: u32,
    pub positions: BTreeMap<String, CanvasPosition>,
}

impl CanvasDocument {
    pub fn empty() -> Self {
        Self {
            version: CANVAS_SCHEMA_VERSION,
            positions: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), CanvasError> {
        if self.version != CANVAS_SCHEMA_VERSION {
            return Err(CanvasError::UnsupportedSchema {
                version: self.version,
            });
        }
        for (id, position) in &self.positions {
            validate_id("node", id)?;
            if !position.is_valid() {
                return Err(CanvasError::InvalidPosition { id: id.clone() });
            }
        }
        Ok(())
    }
}

/// Pure graph state consumed later by a renderer/runtime adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasGraph {
    pub snapshot: GraphSnapshot,
    saved_positions: BTreeMap<String, CanvasPosition>,
}

impl CanvasGraph {
    pub fn from_snapshot(
        snapshot: GraphSnapshot,
        saved_positions: BTreeMap<String, CanvasPosition>,
    ) -> Result<Self, CanvasError> {
        validate_snapshot(&snapshot)?;
        for (id, position) in &saved_positions {
            validate_id("node", id)?;
            if !position.is_valid() {
                return Err(CanvasError::InvalidPosition { id: id.clone() });
            }
        }
        Ok(Self {
            snapshot,
            saved_positions,
        })
    }

    pub fn nodes(&self) -> &[GraphNode] {
        &self.snapshot.nodes
    }

    pub fn edges(&self) -> &[GraphEdge] {
        &self.snapshot.edges
    }

    pub fn saved_positions(&self) -> &BTreeMap<String, CanvasPosition> {
        &self.saved_positions
    }

    /// Returns a saved position, a position supplied by the graph service, or
    /// a deterministic fallback matching the Canvas world dimensions.
    pub fn position_for(&self, id: &str) -> Result<CanvasPosition, CanvasError> {
        validate_id("node", id)?;
        let node_index = self
            .snapshot
            .nodes
            .iter()
            .position(|node| node.id == id)
            .ok_or_else(|| CanvasError::UnknownNode { id: id.to_owned() })?;
        if let Some(position) = self.saved_positions.get(id).copied() {
            return Ok(position);
        }
        if let Some(position) = self.snapshot.nodes[node_index].position.as_ref() {
            return CanvasPosition::from_graph(id, position);
        }
        Ok(default_position(node_index, self.snapshot.nodes.len()))
    }

    pub fn set_position(&mut self, id: &str, position: CanvasPosition) -> Result<(), CanvasError> {
        self.require_node(id)?;
        if !position.is_valid() {
            return Err(CanvasError::InvalidPosition { id: id.to_owned() });
        }
        self.saved_positions.insert(id.to_owned(), position);
        Ok(())
    }

    pub fn move_node(&mut self, id: &str, dx: f32, dy: f32) -> Result<(), CanvasError> {
        let current = self.position_for(id)?;
        self.set_position(
            id,
            current
                .translated(dx, dy)
                .map_err(|_| CanvasError::InvalidPosition { id: id.to_owned() })?,
        )
    }

    pub fn persisted_document(&self) -> CanvasDocument {
        let node_ids = self
            .snapshot
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<BTreeSet<_>>();
        CanvasDocument {
            version: CANVAS_SCHEMA_VERSION,
            positions: self
                .saved_positions
                .iter()
                .filter(|(id, _)| node_ids.contains(id.as_str()))
                .map(|(id, position)| (id.clone(), *position))
                .collect(),
        }
    }

    pub fn replace_snapshot(&mut self, snapshot: GraphSnapshot) -> Result<(), CanvasError> {
        validate_snapshot(&snapshot)?;
        self.saved_positions
            .retain(|id, _| snapshot.nodes.iter().any(|node| node.id == *id));
        self.snapshot = snapshot;
        Ok(())
    }

    fn require_node(&self, id: &str) -> Result<(), CanvasError> {
        validate_id("node", id)?;
        self.snapshot
            .nodes
            .iter()
            .any(|node| node.id == id)
            .then_some(())
            .ok_or_else(|| CanvasError::UnknownNode { id: id.to_owned() })
    }
}

pub fn validate_id(kind: &'static str, id: &str) -> Result<(), CanvasError> {
    if id.is_empty()
        || id.trim() != id
        || id.len() > CANVAS_MAX_ID_LENGTH
        || id.chars().any(char::is_control)
        || id == "."
        || id == ".."
        || id.starts_with('/')
        || id.split('/').any(|segment| segment == "..")
    {
        return Err(CanvasError::InvalidId {
            kind,
            id: id.to_owned(),
        });
    }
    Ok(())
}

pub fn clamp_zoom(value: f32) -> f32 {
    if !value.is_finite() {
        return CANVAS_MIN_ZOOM;
    }
    value.clamp(CANVAS_MIN_ZOOM, CANVAS_MAX_ZOOM)
}

pub fn validate_snapshot(snapshot: &GraphSnapshot) -> Result<(), CanvasError> {
    let mut node_ids = BTreeSet::new();
    for node in &snapshot.nodes {
        validate_id("node", &node.id)?;
        if !node_ids.insert(node.id.as_str()) {
            return Err(CanvasError::DuplicateId {
                kind: "node",
                id: node.id.clone(),
            });
        }
        if let Some(position) = node.position.as_ref() {
            CanvasPosition::from_graph(&node.id, position)?;
        }
    }

    let mut edge_ids = BTreeSet::new();
    for edge in &snapshot.edges {
        validate_id("edge", &edge.id)?;
        if !edge_ids.insert(edge.id.as_str()) {
            return Err(CanvasError::DuplicateId {
                kind: "edge",
                id: edge.id.clone(),
            });
        }
        for endpoint in [&edge.source, &edge.target] {
            if !node_ids.contains(endpoint.as_str()) {
                return Err(CanvasError::InvalidEdgeEndpoint {
                    edge: edge.id.clone(),
                    node: endpoint.clone(),
                });
            }
        }
    }
    Ok(())
}

fn default_position(index: usize, count: usize) -> CanvasPosition {
    if count == 0 {
        return CanvasPosition { x: 0.0, y: 0.0 };
    }
    let angle = index as f32 * std::f32::consts::TAU / count as f32;
    CanvasPosition {
        x: CANVAS_WORLD_WIDTH / 2.0 + 320.0 * angle.cos(),
        y: CANVAS_WORLD_HEIGHT / 2.0 + 220.0 * angle.sin(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_graph_contract::{
        GraphCluster, GraphDataSource, GraphEdgeType, GraphNodeKind, GraphStats,
    };

    fn snapshot(nodes: &[&str], edges: &[(&str, &str, &str)]) -> GraphSnapshot {
        GraphSnapshot {
            generated_at: String::new(),
            vault_root: "vault".to_owned(),
            source: GraphDataSource::KnowledgeCore,
            nodes: nodes
                .iter()
                .map(|id| GraphNode {
                    id: (*id).to_owned(),
                    relative_path: format!("{id}.md"),
                    title: (*id).to_owned(),
                    kind: GraphNodeKind::Note,
                    folder: String::new(),
                    tags: Vec::new(),
                    summary: String::new(),
                    headings: Vec::new(),
                    key_terms: Vec::new(),
                    updated_at: String::new(),
                    weak_title: false,
                    source_count: 0,
                    chunk_count: 0,
                    position: None,
                })
                .collect(),
            edges: edges
                .iter()
                .map(|(id, source, target)| GraphEdge {
                    id: (*id).to_owned(),
                    source: (*source).to_owned(),
                    target: (*target).to_owned(),
                    edge_type: GraphEdgeType::ExplicitLink,
                    reason: String::new(),
                    weight: 1.0,
                })
                .collect(),
            clusters: Vec::<GraphCluster>::new(),
            stats: GraphStats {
                total_notes: nodes.len(),
                total_candidate_edges: edges.len(),
                rendered_edges: edges.len(),
                max_links_per_note: 0,
                max_edges: edges.len(),
            },
            index_path: String::new(),
            search_status: None,
        }
    }

    #[test]
    fn validates_node_and_edge_ids_before_canvas_uses_them() {
        let duplicate = snapshot(&["Alpha", "Alpha"], &[]);
        assert!(matches!(
            validate_snapshot(&duplicate),
            Err(CanvasError::DuplicateId { kind: "node", .. })
        ));

        let dangling = snapshot(&["Alpha"], &[("edge", "Alpha", "Missing")]);
        assert!(matches!(
            validate_snapshot(&dangling),
            Err(CanvasError::InvalidEdgeEndpoint { .. })
        ));
        assert!(validate_id("node", "Notes/Alpha.md").is_ok());
        assert!(validate_id("node", "../outside.md").is_err());
    }

    #[test]
    fn movement_uses_finite_coordinates_and_keeps_zoom_bounded() {
        let mut graph = CanvasGraph::from_snapshot(snapshot(&["Alpha"], &[]), BTreeMap::new())
            .expect("valid graph");
        let initial = graph.position_for("Alpha").expect("default position");
        graph.move_node("Alpha", 20.0, -10.0).expect("move node");
        assert_eq!(
            graph.position_for("Alpha").expect("saved position"),
            CanvasPosition::new(initial.x + 20.0, initial.y - 10.0).unwrap()
        );
        assert_eq!(clamp_zoom(-1.0), CANVAS_MIN_ZOOM);
        assert_eq!(clamp_zoom(99.0), CANVAS_MAX_ZOOM);
        assert_eq!(clamp_zoom(f32::NAN), CANVAS_MIN_ZOOM);
        assert!(CanvasPosition::new(f32::INFINITY, 0.0).is_err());
    }

    #[test]
    fn persisted_document_contains_only_current_graph_nodes() {
        let mut saved = BTreeMap::new();
        saved.insert("Alpha".to_owned(), CanvasPosition::new(12.0, 24.0).unwrap());
        saved.insert("Deleted".to_owned(), CanvasPosition::new(1.0, 2.0).unwrap());
        let graph =
            CanvasGraph::from_snapshot(snapshot(&["Alpha"], &[]), saved).expect("valid graph");
        let document = graph.persisted_document();
        assert_eq!(document.positions.len(), 1);
        assert_eq!(document.positions["Alpha"].x, 12.0);
        assert_eq!(document.version, CANVAS_SCHEMA_VERSION);
    }
}
