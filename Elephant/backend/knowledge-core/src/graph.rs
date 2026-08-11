use crate::relations::{
    relation_type_name, KnowledgeNodeKind, KnowledgeRelation, RelationOrigin, RelationStatus,
    RelationType,
};
use crate::storage::KnowledgeStore;
use crate::wiki_graph_projection::project_wiki_territories;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeGraphNode>,
    pub edges: Vec<KnowledgeGraphEdge>,
    pub clusters: Vec<KnowledgeGraphCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeGraphNode {
    pub id: String,
    pub path: String,
    pub relative_path: String,
    pub title: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub source_count: usize,
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub relation_type: String,
    pub reason: String,
    pub weight: f32,
    pub origin: String,
    pub status: String,
    pub evidence_chunk_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeGraphCluster {
    pub id: String,
    pub label: String,
    pub paths: Vec<String>,
    pub node_count: usize,
    pub tags: Vec<String>,
}

impl KnowledgeStore {
    pub fn graph_projection(&self, include_suggestions: bool) -> Result<KnowledgeGraph, String> {
        let started_at = Instant::now();
        eprintln!(
            "[Knowledge][Graph] projection:start include_suggestions={} database={}",
            include_suggestions,
            self.database_path().display()
        );

        self.initialize_relations()?;
        self.initialize_taxonomy()?;
        self.initialize_wikis()?;
        let conn = Connection::open(self.database_path()).map_err(|error| {
            eprintln!("[Knowledge][Graph] projection:error stage=open_database error={error}");
            error.to_string()
        })?;
        let documents = load_documents(&conn)?;
        let tags = load_document_tags(&conn)?;
        let relations = self.list_relations(None, 50_000)?;
        let tag_assignments = tags.values().map(Vec::len).sum::<usize>();
        eprintln!(
            "[Knowledge][Graph] data:loaded documents={} tagged_documents={} tag_assignments={} relations={}",
            documents.len(),
            tags.len(),
            tag_assignments,
            relations.len()
        );

        let resolver = DocumentResolver::new(&documents);
        let mut nodes = documents
            .iter()
            .map(|document| KnowledgeGraphNode {
                id: document.relative_path.clone(),
                path: document.relative_path.clone(),
                relative_path: document.relative_path.clone(),
                title: document.title.clone(),
                kind: "note".into(),
                node_type: "note".into(),
                summary: document.summary.clone(),
                tags: tags
                    .get(&document.relative_path)
                    .cloned()
                    .unwrap_or_default(),
                source_count: 0,
                chunk_count: document.chunk_count,
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.id.cmp(&right.id));
        let note_node_count = nodes.len();

        let node_ids = nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<HashSet<_>>();
        let mut edges = Vec::new();
        let mut seen_edges = HashSet::new();
        let mut rejected_count = 0usize;
        let mut suggested_hidden_count = 0usize;
        let mut non_document_count = 0usize;
        let mut unresolved_count = 0usize;
        let mut duplicate_count = 0usize;

        for relation in relations {
            if matches!(relation.status, RelationStatus::Rejected) {
                rejected_count += 1;
                continue;
            }
            if !include_suggestions && matches!(relation.status, RelationStatus::Suggested) {
                suggested_hidden_count += 1;
                continue;
            }
            if !matches!(relation.source.kind, KnowledgeNodeKind::Document)
                || !matches!(relation.target.kind, KnowledgeNodeKind::Document)
            {
                non_document_count += 1;
                continue;
            }

            let Some(source) = resolver.resolve(&relation.source.id) else {
                unresolved_count += 1;
                eprintln!(
                    "[Knowledge][Graph] relation:unresolved side=source relation_id={} reference={}",
                    relation.id, relation.source.id
                );
                continue;
            };
            let Some(target) = resolver.resolve(&relation.target.id) else {
                unresolved_count += 1;
                eprintln!(
                    "[Knowledge][Graph] relation:unresolved side=target relation_id={} reference={}",
                    relation.id, relation.target.id
                );
                continue;
            };
            if source == target || !node_ids.contains(source) || !node_ids.contains(target) {
                unresolved_count += 1;
                continue;
            }
            let edge_key = format!(
                "{}::{}::{}::{}",
                source,
                target,
                relation_type_name(&relation.relation_type),
                relation_origin_label(&relation.origin)
            );
            if !seen_edges.insert(edge_key) {
                duplicate_count += 1;
                continue;
            }
            edges.push(to_graph_edge(&relation, source, target));
        }
        edges.sort_by(|left, right| left.id.cmp(&right.id));
        eprintln!(
            "[Knowledge][Graph] relations:projected edges={} rejected={} suggestions_hidden={} non_document={} unresolved={} duplicates={}",
            edges.len(),
            rejected_count,
            suggested_hidden_count,
            non_document_count,
            unresolved_count,
            duplicate_count
        );

        let wiki_projection = project_wiki_territories(self, &node_ids, include_suggestions)?;
        let wiki_node_count = wiki_projection.nodes.len();
        let wiki_edge_count = wiki_projection.edges.len();
        let assigned_document_count = wiki_projection.assigned_document_ids.len();
        nodes.extend(wiki_projection.nodes);
        edges.extend(wiki_projection.edges);
        nodes.sort_by(|left, right| left.id.cmp(&right.id));
        edges.sort_by(|left, right| left.id.cmp(&right.id));

        let mut clusters = if wiki_projection.clusters.is_empty() {
            eprintln!("[Knowledge][Graph] territories:fallback mode=folders");
            build_folder_clusters(&nodes)
        } else {
            let mut territories = wiki_projection.clusters;
            if let Some(unassigned) =
                build_unassigned_cluster(&nodes, &wiki_projection.assigned_document_ids)
            {
                eprintln!(
                    "[Knowledge][Graph] territories:unassigned notes={}",
                    unassigned.node_count
                );
                territories.push(unassigned);
            }
            territories
        };
        clusters.sort_by(|left, right| left.id.cmp(&right.id));

        let mut edge_types = BTreeMap::<String, usize>::new();
        for edge in &edges {
            *edge_types.entry(edge.edge_type.clone()).or_default() += 1;
        }
        eprintln!(
            "[Knowledge][Graph] projection:complete note_nodes={} wiki_nodes={} edges={} wiki_edges={} territories={} assigned_notes={} edge_types={:?} duration_ms={}",
            note_node_count,
            wiki_node_count,
            edges.len(),
            wiki_edge_count,
            clusters.len(),
            assigned_document_count,
            edge_types,
            started_at.elapsed().as_millis()
        );

        Ok(KnowledgeGraph {
            nodes,
            edges,
            clusters,
        })
    }
}

#[derive(Debug, Clone)]
struct DocumentRow {
    relative_path: String,
    title: String,
    summary: String,
    chunk_count: usize,
}

fn load_documents(conn: &Connection) -> Result<Vec<DocumentRow>, String> {
    let mut statement = conn
        .prepare(
            "SELECT d.relative_path, d.title,
                    COALESCE((SELECT substr(c.text, 1, 320)
                              FROM chunks c
                              WHERE c.document_path=d.relative_path
                              ORDER BY c.ordinal LIMIT 1), ''),
                    (SELECT COUNT(*) FROM chunks c WHERE c.document_path=d.relative_path)
             FROM documents d ORDER BY d.relative_path",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(DocumentRow {
                relative_path: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
                chunk_count: row.get::<_, i64>(3)? as usize,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_document_tags(conn: &Connection) -> Result<HashMap<String, Vec<String>>, String> {
    let mut statement = conn
        .prepare(
            "SELECT dt.document_path, t.display_name
             FROM document_tags dt
             JOIN tags t ON t.id=dt.tag_id
             WHERE dt.status IN ('accepted', 'manual') AND t.status='active'
             ORDER BY dt.document_path, t.display_name",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;
    let mut output = HashMap::<String, Vec<String>>::new();
    for row in rows {
        let (path, tag) = row.map_err(|error| error.to_string())?;
        output.entry(path).or_default().push(tag);
    }
    Ok(output)
}

fn to_graph_edge(relation: &KnowledgeRelation, source: &str, target: &str) -> KnowledgeGraphEdge {
    let explicit = matches!(relation.relation_type, RelationType::ExplicitLink);
    KnowledgeGraphEdge {
        id: relation.id.clone(),
        source: source.to_string(),
        target: target.to_string(),
        edge_type: if explicit {
            "explicit-link".into()
        } else {
            "semantic".into()
        },
        relation_type: relation_type_name(&relation.relation_type).into(),
        reason: relation.reason.clone(),
        weight: relation
            .confidence
            .unwrap_or(if explicit { 1.0 } else { 0.5 }),
        origin: relation_origin_label(&relation.origin).into(),
        status: relation_status_label(&relation.status).into(),
        evidence_chunk_ids: relation.evidence_chunk_ids.clone(),
    }
}

fn relation_origin_label(origin: &RelationOrigin) -> &'static str {
    match origin {
        RelationOrigin::User => "user",
        RelationOrigin::Markdown => "markdown",
        RelationOrigin::Model => "model",
        RelationOrigin::System => "system",
    }
}

fn relation_status_label(status: &RelationStatus) -> &'static str {
    match status {
        RelationStatus::Suggested => "suggested",
        RelationStatus::Accepted => "accepted",
        RelationStatus::Rejected => "rejected",
        RelationStatus::Explicit => "explicit",
    }
}

fn build_folder_clusters(nodes: &[KnowledgeGraphNode]) -> Vec<KnowledgeGraphCluster> {
    let mut clusters = BTreeMap::<String, Vec<String>>::new();
    for node in nodes {
        if node.kind != "note" {
            continue;
        }
        let folder = node
            .relative_path
            .rsplit_once('/')
            .map(|(folder, _)| folder)
            .filter(|folder| !folder.is_empty())
            .unwrap_or("root");
        clusters
            .entry(folder.to_string())
            .or_default()
            .push(node.id.clone());
    }
    clusters
        .into_iter()
        .map(|(id, paths)| KnowledgeGraphCluster {
            label: if id == "root" {
                "Root".into()
            } else {
                id.rsplit('/').next().unwrap_or(&id).to_string()
            },
            node_count: paths.len(),
            id,
            paths,
            tags: vec!["folder-cluster".into()],
        })
        .collect()
}

fn build_unassigned_cluster(
    nodes: &[KnowledgeGraphNode],
    assigned_document_ids: &HashSet<String>,
) -> Option<KnowledgeGraphCluster> {
    let paths = nodes
        .iter()
        .filter(|node| node.kind == "note" && !assigned_document_ids.contains(&node.id))
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return None;
    }
    Some(KnowledgeGraphCluster {
        id: "unassigned".into(),
        label: "Unassigned notes".into(),
        node_count: paths.len(),
        paths,
        tags: vec!["unassigned-territory".into()],
    })
}

struct DocumentResolver {
    exact: HashSet<String>,
    aliases: HashMap<String, Option<String>>,
}

impl DocumentResolver {
    fn new(documents: &[DocumentRow]) -> Self {
        let exact = documents
            .iter()
            .map(|document| document.relative_path.clone())
            .collect::<HashSet<_>>();
        let mut aliases = HashMap::<String, Option<String>>::new();
        for document in documents {
            for alias in document_aliases(document) {
                match aliases.get(&alias) {
                    None => {
                        aliases.insert(alias, Some(document.relative_path.clone()));
                    }
                    Some(Some(existing)) if existing != &document.relative_path => {
                        aliases.insert(alias, None);
                    }
                    _ => {}
                }
            }
        }
        Self { exact, aliases }
    }

    fn resolve<'a>(&'a self, value: &'a str) -> Option<&'a str> {
        let normalized = normalize_document_reference(value);
        if self.exact.contains(&normalized) {
            return self.exact.get(&normalized).map(String::as_str);
        }
        self.aliases.get(&normalized).and_then(Option::as_deref)
    }
}

fn document_aliases(document: &DocumentRow) -> Vec<String> {
    let path = normalize_document_reference(&document.relative_path);
    let without_extension = path.strip_suffix(".md").unwrap_or(&path).to_string();
    let basename = without_extension
        .rsplit('/')
        .next()
        .unwrap_or(&without_extension)
        .to_string();
    let title = normalize_document_reference(&document.title);
    [path, without_extension, basename, title]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect()
}

fn normalize_document_reference(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;
    use crate::relations::{KnowledgeRelation, RelationOrigin, RelationStatus, RelationType};
    use crate::wiki_core::{WikiCitation, WikiDraft, WikiDraftStatus};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "elephant-graph-{name}-{}-{stamp}",
            std::process::id()
        ))
    }

    fn document_node(path: &str) -> crate::relations::KnowledgeNodeRef {
        crate::relations::KnowledgeNodeRef {
            kind: KnowledgeNodeKind::Document,
            id: path.into(),
        }
    }

    fn accepted_wiki(source_path: &str) -> WikiDraft {
        WikiDraft {
            id: "wiki-a".into(),
            topic: "Topic A".into(),
            title: "Topic A".into(),
            slug: "topic-a".into(),
            markdown: "# Topic A\n".into(),
            citations: vec![WikiCitation {
                key: "source-1".into(),
                document_path: source_path.into(),
                document_title: "A".into(),
                chunk_id: "chunk-a".into(),
                heading: "A".into(),
                start_offset: 0,
                end_offset: 10,
            }],
            source_paths: vec![source_path.into()],
            source_hash: "hash".into(),
            model_id: "test-model".into(),
            status: WikiDraftStatus::Accepted,
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn projects_only_resolved_document_relations() {
        let root = temp_vault("projection");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let a = analyze_markdown("Notes/A.md", "# A\nSee [[B]].", 1);
        let b = analyze_markdown("Notes/B.md", "# B\nTarget", 1);
        store.upsert_document(&a).unwrap();
        store.upsert_document(&b).unwrap();
        store.sync_markdown_relations(&a).unwrap();

        let graph = store.graph_projection(false).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "Notes/A.md");
        assert_eq!(graph.edges[0].target, "Notes/B.md");
        assert_eq!(graph.edges[0].edge_type, "explicit-link");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn graph_projection_includes_wiki_node_source_edge_and_territory() {
        let root = temp_vault("wiki-territory");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        store
            .upsert_document(&analyze_markdown("Notes/A.md", "# A\nBody", 1))
            .unwrap();
        store.save_wiki_draft(&accepted_wiki("Notes/A.md")).unwrap();

        let graph = store.graph_projection(false).unwrap();
        assert!(graph.nodes.iter().any(|node| node.kind == "wiki"));
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.edge_type == "wiki-source"));
        assert!(graph
            .clusters
            .iter()
            .any(|cluster| cluster.tags.iter().any(|tag| tag == "wiki-territory")));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn notes_outside_wikis_are_kept_in_an_unassigned_territory() {
        let root = temp_vault("unassigned");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        store
            .upsert_document(&analyze_markdown("A.md", "# A\nBody", 1))
            .unwrap();
        store
            .upsert_document(&analyze_markdown("B.md", "# B\nBody", 1))
            .unwrap();
        store.save_wiki_draft(&accepted_wiki("A.md")).unwrap();

        let graph = store.graph_projection(false).unwrap();
        let unassigned = graph
            .clusters
            .iter()
            .find(|cluster| cluster.id == "unassigned")
            .unwrap();
        assert_eq!(unassigned.paths, vec!["B.md"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn hides_suggestions_until_requested() {
        let root = temp_vault("suggestions");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        let a = analyze_markdown("A.md", "# A\nA supports B.", 1);
        let b = analyze_markdown("B.md", "# B\nTarget", 1);
        store.upsert_document(&a).unwrap();
        store.upsert_document(&b).unwrap();
        store
            .save_relation(&KnowledgeRelation::new(
                document_node("A.md"),
                document_node("B.md"),
                RelationType::Supports,
                RelationOrigin::Model,
                RelationStatus::Suggested,
                Some(0.9),
                vec![a.chunks[0].id.clone()],
                "Direct support.",
                Some("model".into()),
            ))
            .unwrap();

        assert!(store.graph_projection(false).unwrap().edges.is_empty());
        let graph = store.graph_projection(true).unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].relation_type, "supports");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ambiguous_basename_does_not_create_false_edge() {
        let root = temp_vault("ambiguous");
        fs::create_dir_all(&root).unwrap();
        let mut store = KnowledgeStore::open(&root).unwrap();
        for (path, markdown) in [
            ("Source.md", "# Source\nSee [[Same]]."),
            ("One/Same.md", "# Same\nOne"),
            ("Two/Same.md", "# Same\nTwo"),
        ] {
            let document = analyze_markdown(path, markdown, 1);
            store.upsert_document(&document).unwrap();
            store.sync_markdown_relations(&document).unwrap();
        }
        assert!(store.graph_projection(false).unwrap().edges.is_empty());
        fs::remove_dir_all(root).ok();
    }
}
