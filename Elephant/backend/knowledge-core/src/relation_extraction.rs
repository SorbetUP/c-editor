use crate::extraction::{StructuredModelRequest, StructuredTask};
use crate::model::DocumentSnapshot;
use crate::relations::{
    KnowledgeNodeKind, KnowledgeNodeRef, KnowledgeRelation, RelationOrigin, RelationStatus,
    RelationType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const DEFAULT_MAX_RELATIONS: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationExtraction {
    #[serde(default)]
    pub relations: Vec<ExtractedRelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedRelation {
    pub target: KnowledgeNodeRef,
    pub relation_type: RelationType,
    pub confidence: f32,
    #[serde(default)]
    pub evidence_chunk_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationExtractionValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl RelationExtraction {
    pub fn validate(
        &self,
        document: &DocumentSnapshot,
        allowed_targets: &[KnowledgeNodeRef],
    ) -> RelationExtractionValidation {
        self.validate_with_limit(document, allowed_targets, DEFAULT_MAX_RELATIONS)
    }

    pub fn validate_with_limit(
        &self,
        document: &DocumentSnapshot,
        allowed_targets: &[KnowledgeNodeRef],
        max_relations: usize,
    ) -> RelationExtractionValidation {
        let mut errors = Vec::new();
        if self.relations.len() > max_relations {
            errors.push(format!(
                "Model returned {} relations; the configured maximum is {max_relations}.",
                self.relations.len()
            ));
        }

        let known_chunks = document
            .chunks
            .iter()
            .map(|chunk| chunk.id.as_str())
            .collect::<HashSet<_>>();
        let allowed = allowed_targets.iter().cloned().collect::<HashSet<_>>();
        let mut seen = HashSet::new();

        for (index, relation) in self.relations.iter().enumerate() {
            let label = format!("Relation suggestion {index}");
            if !allowed.contains(&relation.target) {
                errors.push(format!(
                    "{label} targets a node that was not supplied as a candidate: {:?}:{}.",
                    relation.target.kind, relation.target.id
                ));
            }
            if relation.target.id.trim().is_empty() {
                errors.push(format!("{label} target ID cannot be empty."));
            }
            if !relation.confidence.is_finite() || !(0.0..=1.0).contains(&relation.confidence) {
                errors.push(format!("{label} confidence must be between 0 and 1."));
            }
            if relation.reason.trim().is_empty() {
                errors.push(format!("{label} must explain why the relation applies."));
            }
            if relation.evidence_chunk_ids.is_empty() {
                errors.push(format!("{label} must cite at least one evidence chunk."));
            }
            for chunk_id in &relation.evidence_chunk_ids {
                if !known_chunks.contains(chunk_id.as_str()) {
                    errors.push(format!(
                        "{label} references an unknown evidence chunk: {chunk_id}."
                    ));
                }
            }

            let relation_key = format!(
                "{:?}:{}:{:?}",
                relation.target.kind, relation.target.id, relation.relation_type
            );
            if !seen.insert(relation_key) {
                errors.push(format!("{label} duplicates another relation suggestion."));
            }
        }

        RelationExtractionValidation {
            valid: errors.is_empty(),
            errors,
        }
    }

    pub fn into_relations(
        self,
        document: &DocumentSnapshot,
        allowed_targets: &[KnowledgeNodeRef],
        model_id: &str,
        max_relations: usize,
    ) -> Result<Vec<KnowledgeRelation>, String> {
        let validation = self.validate_with_limit(document, allowed_targets, max_relations);
        if !validation.valid {
            return Err(validation.errors.join(" "));
        }
        let source = KnowledgeNodeRef {
            kind: KnowledgeNodeKind::Document,
            id: document.relative_path.clone(),
        };
        Ok(self
            .relations
            .into_iter()
            .map(|relation| {
                KnowledgeRelation::new(
                    source.clone(),
                    relation.target,
                    relation.relation_type,
                    RelationOrigin::Model,
                    RelationStatus::Suggested,
                    Some(relation.confidence),
                    relation.evidence_chunk_ids,
                    relation.reason,
                    Some(model_id.to_string()),
                )
            })
            .collect())
    }
}

pub fn build_relation_extraction_request(
    document: &DocumentSnapshot,
    allowed_targets: &[KnowledgeNodeRef],
    target_descriptions_json: &str,
    max_relations: usize,
) -> StructuredModelRequest {
    let chunks = document
        .chunks
        .iter()
        .map(|chunk| format!("<chunk id=\"{}\">\n{}\n</chunk>", chunk.id, chunk.text))
        .collect::<Vec<_>>()
        .join("\n\n");
    let allowed_target_ids = serde_json::to_string(allowed_targets).unwrap_or_else(|_| "[]".into());

    StructuredModelRequest {
        task: StructuredTask::ExtractRelations,
        system_prompt: format!(
            "You extract typed relations from one source note. Return only data matching the requested JSON schema. You may target only nodes in allowed_targets. Return at most {max_relations} high-confidence relations. Every relation must cite one or more supplied source chunk IDs. Do not create a relation based only on broad topical similarity. Prefer precise types such as depends_on, supports, contradicts, example_of, supersedes, part_of, mentions, or related_to."
        ),
        user_prompt: format!(
            "Source document path: {}\nSource title: {}\n\nAllowed targets:\n{}\n\nTarget descriptions:\n{}\n\nSource chunks:\n{}",
            document.relative_path,
            document.title,
            allowed_target_ids,
            target_descriptions_json,
            chunks
        ),
        json_schema_name: "elephantnote_relation_extraction_v1".into(),
        max_output_tokens: 3_072,
    }
}

pub fn parse_relation_extraction_response(
    json: &str,
    document: &DocumentSnapshot,
    allowed_targets: &[KnowledgeNodeRef],
    model_id: &str,
    max_relations: usize,
) -> Result<Vec<KnowledgeRelation>, String> {
    let extraction: RelationExtraction = serde_json::from_str(json)
        .map_err(|error| format!("Invalid relation extraction JSON: {error}"))?;
    extraction.into_relations(document, allowed_targets, model_id, max_relations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;

    fn document() -> DocumentSnapshot {
        analyze_markdown(
            "Notes/Iroh.md",
            "# Iroh\n\nElephantNote uses Iroh for peer-to-peer synchronization.",
            1,
        )
    }

    fn target(path: &str) -> KnowledgeNodeRef {
        KnowledgeNodeRef {
            kind: KnowledgeNodeKind::Document,
            id: path.into(),
        }
    }

    #[test]
    fn rejects_targets_not_in_candidate_set() {
        let document = document();
        let extraction = RelationExtraction {
            relations: vec![ExtractedRelation {
                target: target("Unknown.md"),
                relation_type: RelationType::DependsOn,
                confidence: 0.9,
                evidence_chunk_ids: vec![document.chunks[0].id.clone()],
                reason: "Uses it.".into(),
            }],
        };
        assert!(!extraction.validate(&document, &[target("Iroh.md")]).valid);
    }

    #[test]
    fn converts_valid_output_to_model_relation() {
        let document = document();
        let allowed = vec![target("Iroh.md")];
        let extraction = RelationExtraction {
            relations: vec![ExtractedRelation {
                target: allowed[0].clone(),
                relation_type: RelationType::DependsOn,
                confidence: 0.95,
                evidence_chunk_ids: vec![document.chunks[0].id.clone()],
                reason: "The note states that ElephantNote uses Iroh.".into(),
            }],
        };
        let relations = extraction
            .into_relations(&document, &allowed, "small-model", 12)
            .unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].source.id, "Notes/Iroh.md");
        assert_eq!(relations[0].target.id, "Iroh.md");
        assert_eq!(relations[0].origin, RelationOrigin::Model);
        assert_eq!(relations[0].status, RelationStatus::Suggested);
    }

    #[test]
    fn prompt_contains_only_supplied_targets_and_chunk_ids() {
        let document = document();
        let allowed = vec![target("Iroh.md")];
        let request = build_relation_extraction_request(&document, &allowed, "[]", 6);
        assert!(request.user_prompt.contains("Iroh.md"));
        assert!(request.user_prompt.contains(&document.chunks[0].id));
        assert!(request.system_prompt.contains("at most 6"));
        assert_eq!(request.task, StructuredTask::ExtractRelations);
    }
}
