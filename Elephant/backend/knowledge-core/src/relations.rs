use crate::model::DocumentSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNodeKind {
    Document,
    Tag,
    Concept,
    Wiki,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KnowledgeNodeRef {
    pub kind: KnowledgeNodeKind,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    ExplicitLink,
    TaggedWith,
    PartOf,
    DependsOn,
    Supports,
    Contradicts,
    ExampleOf,
    Mentions,
    Supersedes,
    DerivedFrom,
    WikiSource,
    RelatedTo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationOrigin {
    User,
    Markdown,
    Model,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationStatus {
    Suggested,
    Accepted,
    Rejected,
    Explicit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeRelation {
    pub id: String,
    pub source: KnowledgeNodeRef,
    pub target: KnowledgeNodeRef,
    pub relation_type: RelationType,
    pub origin: RelationOrigin,
    pub status: RelationStatus,
    pub confidence: Option<f32>,
    #[serde(default)]
    pub evidence_chunk_ids: Vec<String>,
    pub reason: String,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl KnowledgeRelation {
    pub fn new(
        source: KnowledgeNodeRef,
        target: KnowledgeNodeRef,
        relation_type: RelationType,
        origin: RelationOrigin,
        status: RelationStatus,
        confidence: Option<f32>,
        evidence_chunk_ids: Vec<String>,
        reason: impl Into<String>,
        model_id: Option<String>,
    ) -> Self {
        let id = stable_relation_id(&source, &target, &relation_type, &origin);
        Self {
            id,
            source,
            target,
            relation_type,
            origin,
            status,
            confidence,
            evidence_chunk_ids,
            reason: reason.into(),
            model_id,
        }
    }

    pub fn validate(&self, source_document: Option<&DocumentSnapshot>) -> RelationValidation {
        let mut errors = Vec::new();
        validate_node(&self.source, "source", &mut errors);
        validate_node(&self.target, "target", &mut errors);

        if self.source == self.target {
            errors.push("A relation cannot connect a node to itself.".into());
        }

        if let Some(confidence) = self.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                errors.push("Relation confidence must be between 0 and 1.".into());
            }
        }

        if matches!(self.origin, RelationOrigin::Model) {
            if self.evidence_chunk_ids.is_empty() {
                errors.push("Model relations require at least one evidence chunk.".into());
            }
            if self.reason.trim().is_empty() {
                errors.push("Model relations require an explanation.".into());
            }
            if self.model_id.as_deref().unwrap_or("").trim().is_empty() {
                errors.push("Model relations must record the model ID.".into());
            }
            if self.confidence.is_none() {
                errors.push("Model relations must include a confidence score.".into());
            }
        }

        if matches!(self.origin, RelationOrigin::Markdown)
            && !matches!(self.relation_type, RelationType::ExplicitLink)
        {
            errors.push("Markdown-origin relations must use explicit_link.".into());
        }

        if let Some(document) = source_document {
            let known_chunks = document
                .chunks
                .iter()
                .map(|chunk| chunk.id.as_str())
                .collect::<HashSet<_>>();
            for chunk_id in &self.evidence_chunk_ids {
                if !known_chunks.contains(chunk_id.as_str()) {
                    errors.push(format!(
                        "Relation references an unknown evidence chunk: {chunk_id}."
                    ));
                }
            }
        }

        RelationValidation {
            valid: errors.is_empty(),
            errors,
        }
    }
}

pub fn stable_relation_id(
    source: &KnowledgeNodeRef,
    target: &KnowledgeNodeRef,
    relation_type: &RelationType,
    origin: &RelationOrigin,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"relation");
    for part in [
        node_kind_name(&source.kind),
        source.id.as_str(),
        relation_type_name(relation_type),
        node_kind_name(&target.kind),
        target.id.as_str(),
        relation_origin_name(origin),
    ] {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    let hex = hasher.finalize().to_hex().to_string();
    format!("rel-{}", &hex[..24])
}

pub(crate) fn node_kind_name(kind: &KnowledgeNodeKind) -> &'static str {
    match kind {
        KnowledgeNodeKind::Document => "document",
        KnowledgeNodeKind::Tag => "tag",
        KnowledgeNodeKind::Concept => "concept",
        KnowledgeNodeKind::Wiki => "wiki",
        KnowledgeNodeKind::Folder => "folder",
    }
}

pub(crate) fn parse_node_kind(value: &str) -> KnowledgeNodeKind {
    match value {
        "tag" => KnowledgeNodeKind::Tag,
        "concept" => KnowledgeNodeKind::Concept,
        "wiki" => KnowledgeNodeKind::Wiki,
        "folder" => KnowledgeNodeKind::Folder,
        _ => KnowledgeNodeKind::Document,
    }
}

pub(crate) fn relation_type_name(relation_type: &RelationType) -> &'static str {
    match relation_type {
        RelationType::ExplicitLink => "explicit_link",
        RelationType::TaggedWith => "tagged_with",
        RelationType::PartOf => "part_of",
        RelationType::DependsOn => "depends_on",
        RelationType::Supports => "supports",
        RelationType::Contradicts => "contradicts",
        RelationType::ExampleOf => "example_of",
        RelationType::Mentions => "mentions",
        RelationType::Supersedes => "supersedes",
        RelationType::DerivedFrom => "derived_from",
        RelationType::WikiSource => "wiki_source",
        RelationType::RelatedTo => "related_to",
    }
}

pub(crate) fn parse_relation_type(value: &str) -> RelationType {
    match value {
        "explicit_link" => RelationType::ExplicitLink,
        "tagged_with" => RelationType::TaggedWith,
        "part_of" => RelationType::PartOf,
        "depends_on" => RelationType::DependsOn,
        "supports" => RelationType::Supports,
        "contradicts" => RelationType::Contradicts,
        "example_of" => RelationType::ExampleOf,
        "mentions" => RelationType::Mentions,
        "supersedes" => RelationType::Supersedes,
        "derived_from" => RelationType::DerivedFrom,
        "wiki_source" => RelationType::WikiSource,
        _ => RelationType::RelatedTo,
    }
}

pub(crate) fn relation_origin_name(origin: &RelationOrigin) -> &'static str {
    match origin {
        RelationOrigin::User => "user",
        RelationOrigin::Markdown => "markdown",
        RelationOrigin::Model => "model",
        RelationOrigin::System => "system",
    }
}

pub(crate) fn parse_relation_origin(value: &str) -> RelationOrigin {
    match value {
        "user" => RelationOrigin::User,
        "markdown" => RelationOrigin::Markdown,
        "model" => RelationOrigin::Model,
        _ => RelationOrigin::System,
    }
}

pub(crate) fn relation_status_name(status: &RelationStatus) -> &'static str {
    match status {
        RelationStatus::Suggested => "suggested",
        RelationStatus::Accepted => "accepted",
        RelationStatus::Rejected => "rejected",
        RelationStatus::Explicit => "explicit",
    }
}

pub(crate) fn parse_relation_status(value: &str) -> RelationStatus {
    match value {
        "accepted" => RelationStatus::Accepted,
        "rejected" => RelationStatus::Rejected,
        "explicit" => RelationStatus::Explicit,
        _ => RelationStatus::Suggested,
    }
}

fn validate_node(node: &KnowledgeNodeRef, label: &str, errors: &mut Vec<String>) {
    if node.id.trim().is_empty() {
        errors.push(format!("Relation {label} ID cannot be empty."));
    }
    if node.id.chars().count() > 1_024 {
        errors.push(format!("Relation {label} ID is too long."));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;

    fn document_node(path: &str) -> KnowledgeNodeRef {
        KnowledgeNodeRef {
            kind: KnowledgeNodeKind::Document,
            id: path.into(),
        }
    }

    #[test]
    fn model_relation_requires_evidence_model_and_confidence() {
        let relation = KnowledgeRelation::new(
            document_node("A.md"),
            document_node("B.md"),
            RelationType::Supports,
            RelationOrigin::Model,
            RelationStatus::Suggested,
            None,
            Vec::new(),
            "",
            None,
        );
        let validation = relation.validate(None);
        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("evidence")));
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("model ID")));
    }

    #[test]
    fn validates_evidence_against_source_document() {
        let document = analyze_markdown("A.md", "# A\nA supports B.", 1);
        let relation = KnowledgeRelation::new(
            document_node("A.md"),
            document_node("B.md"),
            RelationType::Supports,
            RelationOrigin::Model,
            RelationStatus::Suggested,
            Some(0.91),
            vec![document.chunks[0].id.clone()],
            "The source explicitly supports B.",
            Some("small-model".into()),
        );
        assert!(relation.validate(Some(&document)).valid);
    }

    #[test]
    fn relation_ids_are_stable_and_typed() {
        let source = document_node("A.md");
        let target = document_node("B.md");
        let supports = stable_relation_id(
            &source,
            &target,
            &RelationType::Supports,
            &RelationOrigin::Model,
        );
        assert_eq!(
            supports,
            stable_relation_id(
                &source,
                &target,
                &RelationType::Supports,
                &RelationOrigin::Model,
            )
        );
        assert_ne!(
            supports,
            stable_relation_id(
                &source,
                &target,
                &RelationType::Contradicts,
                &RelationOrigin::Model,
            )
        );
    }

    #[test]
    fn markdown_cannot_claim_semantic_relations() {
        let relation = KnowledgeRelation::new(
            document_node("A.md"),
            document_node("B.md"),
            RelationType::Supports,
            RelationOrigin::Markdown,
            RelationStatus::Explicit,
            None,
            Vec::new(),
            "",
            None,
        );
        assert!(!relation.validate(None).valid);
    }
}
