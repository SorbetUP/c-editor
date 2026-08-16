//! Filesystem/runtime boundary for the Freya Canvas domain.
//!
//! The graph is never fabricated here: callers must provide the real
//! `GraphSnapshot` produced by the existing graph/search runtime.  This
//! boundary only reads/writes the per-vault Canvas document and delegates graph
//! validation and interaction semantics to `canvas_contract`.

use crate::canvas_contract::{
    clamp_zoom, CanvasDocument, CanvasError, CanvasGraph, CanvasPosition, CANVAS_RELATIVE_PATH,
};
use crate::search_graph_contract::{GraphPosition, GraphSnapshot};
use serde_json::Error as JsonError;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub enum CanvasRuntimeError {
    Io { path: PathBuf, source: io::Error },
    Json { path: PathBuf, source: JsonError },
    Domain(CanvasError),
}

impl fmt::Display for CanvasRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(
                formatter,
                "Canvas I/O failed at {}: {source}",
                path.display()
            ),
            Self::Json { path, source } => write!(
                formatter,
                "Canvas JSON is invalid at {}: {source}",
                path.display()
            ),
            Self::Domain(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CanvasRuntimeError {}

impl From<CanvasError> for CanvasRuntimeError {
    fn from(value: CanvasError) -> Self {
        Self::Domain(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasRuntime {
    vault_root: PathBuf,
    graph: CanvasGraph,
    zoom: f32,
}

impl CanvasRuntime {
    /// Opens Canvas with a real graph snapshot supplied by the caller.
    ///
    /// Requiring the snapshot as an argument keeps graph discovery explicit:
    /// this module cannot silently invent nodes or switch to a fallback source.
    pub fn open(
        vault_root: impl Into<PathBuf>,
        snapshot: GraphSnapshot,
    ) -> Result<Self, CanvasRuntimeError> {
        let vault_root = vault_root.into();
        let persisted = CanvasPersistence::load(&vault_root)?;
        let graph = CanvasGraph::from_snapshot(snapshot, persisted.positions)?;
        Ok(Self {
            vault_root,
            graph,
            zoom: 1.0,
        })
    }

    pub fn vault_root(&self) -> &Path {
        &self.vault_root
    }

    pub fn canvas_path(&self) -> PathBuf {
        CanvasPersistence::path(&self.vault_root)
    }

    pub fn graph(&self) -> &CanvasGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut CanvasGraph {
        &mut self.graph
    }

    pub fn nodes(&self) -> &[crate::search_graph_contract::GraphNode] {
        self.graph.nodes()
    }

    pub fn edges(&self) -> &[crate::search_graph_contract::GraphEdge] {
        self.graph.edges()
    }

    /// Projects persisted Canvas positions back into the renderer contract.
    /// The graph service remains the source of nodes and edges; only the
    /// user-owned viewport coordinates are overlaid here.
    pub fn snapshot_for_render(&self) -> GraphSnapshot {
        let mut snapshot = self.graph.snapshot.clone();
        for node in &mut snapshot.nodes {
            if let Ok(position) = self.position_for(&node.id) {
                node.position = Some(GraphPosition {
                    x: position.x,
                    y: position.y,
                });
            }
        }
        snapshot
    }

    pub fn position_for(&self, id: &str) -> Result<CanvasPosition, CanvasRuntimeError> {
        Ok(self.graph.position_for(id)?)
    }

    pub fn set_node_position(
        &mut self,
        id: &str,
        position: CanvasPosition,
    ) -> Result<(), CanvasRuntimeError> {
        self.graph.set_position(id, position)?;
        Ok(())
    }

    pub fn move_node(&mut self, id: &str, dx: f32, dy: f32) -> Result<(), CanvasRuntimeError> {
        self.graph.move_node(id, dx, dy)?;
        Ok(())
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Mirrors `CanvasView.vue`: the stored scale is always within 50–180%.
    pub fn set_zoom(&mut self, value: f32) -> f32 {
        self.zoom = clamp_zoom(value);
        self.zoom
    }

    pub fn save(&self) -> Result<(), CanvasRuntimeError> {
        CanvasPersistence::save(&self.vault_root, &self.graph.persisted_document())
    }

    /// Replaces the graph with another explicit snapshot while retaining only
    /// positions whose IDs still exist in the new snapshot.
    pub fn replace_snapshot(&mut self, snapshot: GraphSnapshot) -> Result<(), CanvasRuntimeError> {
        self.graph.replace_snapshot(snapshot)?;
        Ok(())
    }
}

pub struct CanvasPersistence;

impl CanvasPersistence {
    pub fn path(vault_root: impl AsRef<Path>) -> PathBuf {
        vault_root.as_ref().join(CANVAS_RELATIVE_PATH)
    }

    pub fn load(vault_root: impl AsRef<Path>) -> Result<CanvasDocument, CanvasRuntimeError> {
        let path = Self::path(vault_root);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(CanvasDocument::empty());
            }
            Err(source) => return Err(CanvasRuntimeError::Io { path, source }),
        };
        let document = serde_json::from_slice::<CanvasDocument>(&bytes).map_err(|source| {
            CanvasRuntimeError::Json {
                path: path.clone(),
                source,
            }
        })?;
        document.validate()?;
        Ok(document)
    }

    pub fn save(
        vault_root: impl AsRef<Path>,
        document: &CanvasDocument,
    ) -> Result<(), CanvasRuntimeError> {
        document.validate()?;
        let path = Self::path(vault_root);
        let bytes =
            serde_json::to_vec_pretty(document).map_err(|source| CanvasRuntimeError::Json {
                path: path.clone(),
                source,
            })?;
        atomic_write(&path, &bytes)
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CanvasRuntimeError> {
    let parent = path.parent().ok_or_else(|| CanvasRuntimeError::Io {
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidInput, "Canvas path has no parent"),
    })?;
    fs::create_dir_all(parent).map_err(|source| CanvasRuntimeError::Io {
        path: parent.to_path_buf(),
        source,
    })?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temporary = parent.join(format!(".canvas.json.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| {
        let mut file = File::create(&temporary).map_err(|source| CanvasRuntimeError::Io {
            path: temporary.clone(),
            source,
        })?;
        file.write_all(bytes)
            .map_err(|source| CanvasRuntimeError::Io {
                path: temporary.clone(),
                source,
            })?;
        file.sync_all().map_err(|source| CanvasRuntimeError::Io {
            path: temporary.clone(),
            source,
        })?;
        fs::rename(&temporary, path).map_err(|source| CanvasRuntimeError::Io {
            path: path.to_path_buf(),
            source,
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas_contract::{CANVAS_MAX_ZOOM, CANVAS_SCHEMA_VERSION};
    use crate::search_graph_contract::{
        GraphCluster, GraphDataSource, GraphEdge, GraphNode, GraphNodeKind, GraphStats,
    };
    use std::collections::BTreeMap;

    fn snapshot() -> GraphSnapshot {
        GraphSnapshot {
            generated_at: String::new(),
            vault_root: "vault".to_owned(),
            source: GraphDataSource::KnowledgeCore,
            nodes: vec![GraphNode {
                id: "Notes/Alpha.md".to_owned(),
                relative_path: "Notes/Alpha.md".to_owned(),
                title: "Alpha".to_owned(),
                kind: GraphNodeKind::Note,
                folder: "Notes".to_owned(),
                tags: Vec::new(),
                summary: String::new(),
                headings: Vec::new(),
                key_terms: Vec::new(),
                updated_at: String::new(),
                weak_title: false,
                source_count: 0,
                chunk_count: 0,
                position: None,
            }],
            edges: Vec::<GraphEdge>::new(),
            clusters: Vec::<GraphCluster>::new(),
            stats: GraphStats {
                total_notes: 1,
                total_candidate_edges: 0,
                rendered_edges: 0,
                max_links_per_note: 0,
                max_edges: 0,
            },
            index_path: String::new(),
            search_status: None,
        }
    }

    fn temporary_vault(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "elephant-canvas-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary vault");
        path
    }

    #[test]
    fn real_snapshot_positions_roundtrip_per_vault_atomically() {
        let vault = temporary_vault("roundtrip");
        let mut runtime = CanvasRuntime::open(&vault, snapshot()).expect("open Canvas");
        runtime
            .set_node_position("Notes/Alpha.md", CanvasPosition::new(120.0, 240.0).unwrap())
            .expect("position");
        runtime
            .move_node("Notes/Alpha.md", 10.0, -20.0)
            .expect("move");
        runtime.set_zoom(99.0);
        runtime.save().expect("persist Canvas");

        let raw = fs::read_to_string(CanvasPersistence::path(&vault)).expect("canvas file");
        assert!(raw.contains("Notes/Alpha.md"));
        assert!(!raw.contains(".tmp"));
        let reopened = CanvasRuntime::open(&vault, snapshot()).expect("reopen Canvas");
        assert_eq!(reopened.zoom(), 1.0);
        assert_eq!(reopened.position_for("Notes/Alpha.md").unwrap().x, 130.0);
        assert_eq!(reopened.position_for("Notes/Alpha.md").unwrap().y, 220.0);
        assert_eq!(runtime.zoom(), CANVAS_MAX_ZOOM);

        fs::remove_dir_all(vault).expect("cleanup");
    }

    #[test]
    fn missing_document_is_empty_but_malformed_document_is_visible() {
        let vault = temporary_vault("malformed");
        let empty = CanvasPersistence::load(&vault).expect("missing file is empty");
        assert_eq!(empty, CanvasDocument::empty());

        let path = CanvasPersistence::path(&vault);
        fs::create_dir_all(path.parent().unwrap()).expect("metadata directory");
        fs::write(&path, b"{not-json").expect("malformed file");
        assert!(matches!(
            CanvasPersistence::load(&vault),
            Err(CanvasRuntimeError::Json { .. })
        ));

        fs::remove_dir_all(vault).expect("cleanup");
    }

    #[test]
    fn stale_positions_are_not_serialized_for_a_replaced_snapshot() {
        let vault = temporary_vault("stale");
        let mut positions = BTreeMap::new();
        positions.insert(
            "Deleted.md".to_owned(),
            CanvasPosition::new(1.0, 2.0).unwrap(),
        );
        CanvasPersistence::save(
            &vault,
            &CanvasDocument {
                version: CANVAS_SCHEMA_VERSION,
                positions,
            },
        )
        .expect("seed positions");
        let runtime = CanvasRuntime::open(&vault, snapshot()).expect("open Canvas");
        runtime.save().expect("rewrite positions");
        let reloaded = CanvasPersistence::load(&vault).expect("reload positions");
        assert!(reloaded.positions.is_empty());
        fs::remove_dir_all(vault).expect("cleanup");
    }
}
