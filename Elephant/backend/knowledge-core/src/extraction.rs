use crate::model::DocumentSnapshot;
use crate::taxonomy::{canonical_tag_key, NewTagCandidate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const DEFAULT_MAX_TAGS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaggingExtraction {
    pub suggested_title: Option<String>,
    #[serde(default)]
    pub tags: Vec<TagSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagSuggestion {
    pub proposed_name: String,
    pub existing_tag_id: Option<String>,
    pub new_tag: Option<NewTagCandidate>,
    pub confidence: f32,
    #[serde(default)]
    pub evidence_chunk_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractionValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl TaggingExtraction {
    pub fn validate(&self, document: &DocumentSnapshot) -> ExtractionValidation {
        self.validate_with_limit(document, DEFAULT_MAX_TAGS)
    }

    pub fn validate_with_limit(
        &self,
        document: &DocumentSnapshot,
        max_tags: usize,
    ) -> ExtractionValidation {
        let mut errors = Vec::new();
        if self.tags.len() > max_tags {
            errors.push(format!(
                "Model returned {} tags; the configured maximum is {max_tags}.",
                self.tags.len()
            ));
        }

        if let Some(title) = &self.suggested_title {
            let trimmed = title.trim();
            if trimmed.is_empty() {
                errors.push("Suggested title cannot be empty.".into());
            }
            if trimmed.chars().count() > 200 {
                errors.push("Suggested title exceeds 200 characters.".into());
            }
        }

        let valid_chunks = document
            .chunks
            .iter()
            .map(|chunk| chunk.id.as_str())
            .collect::<HashSet<_>>();
        let mut seen_targets = HashSet::new();

        for (index, tag) in self.tags.iter().enumerate() {
            let label = format!("Tag suggestion {index}");
            let proposed_name = tag.proposed_name.trim();
            if proposed_name.is_empty() {
                errors.push(format!("{label} has an empty proposed name."));
            }
            if canonical_tag_key(proposed_name).is_err() {
                errors.push(format!("{label} has an invalid proposed name."));
            }
            if !(0.0..=1.0).contains(&tag.confidence) || !tag.confidence.is_finite() {
                errors.push(format!("{label} confidence must be between 0 and 1."));
            }
            if tag.reason.trim().is_empty() {
                errors.push(format!("{label} must explain why the tag applies."));
            }
            if tag.evidence_chunk_ids.is_empty() {
                errors.push(format!("{label} must cite at least one evidence chunk."));
            }
            for chunk_id in &tag.evidence_chunk_ids {
                if !valid_chunks.contains(chunk_id.as_str()) {
                    errors.push(format!(
                        "{label} references an unknown evidence chunk: {chunk_id}."
                    ));
                }
            }

            match (&tag.existing_tag_id, &tag.new_tag) {
                (Some(_), Some(_)) => errors.push(format!(
                    "{label} cannot target an existing tag and create a new tag simultaneously."
                )),
                (None, None) => errors.push(format!(
                    "{label} must target an existing tag or provide a new tag candidate."
                )),
                _ => {}
            }

            let target_key = tag
                .existing_tag_id
                .clone()
                .unwrap_or_else(|| format!("new:{proposed_name}"));
            if !seen_targets.insert(target_key) {
                errors.push(format!("{label} duplicates another tag suggestion."));
            }
        }

        ExtractionValidation {
            valid: errors.is_empty(),
            errors,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredModelRequest {
    pub task: StructuredTask,
    pub system_prompt: String,
    pub user_prompt: String,
    pub json_schema_name: String,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StructuredTask {
    SuggestTitleAndTags,
    ConsolidateTaxonomy,
    ExtractRelations,
    BuildWikiOutline,
    WriteWikiSection,
    ValidateWikiCitations,
}

pub fn build_tagging_request(
    document: &DocumentSnapshot,
    known_taxonomy_json: &str,
    max_tags: usize,
) -> StructuredModelRequest {
    let chunks = document
        .chunks
        .iter()
        .map(|chunk| format!("<chunk id=\"{}\">\n{}\n</chunk>", chunk.id, chunk.text))
        .collect::<Vec<_>>()
        .join("\n\n");

    StructuredModelRequest {
        task: StructuredTask::SuggestTitleAndTags,
        system_prompt: format!(
            "You classify one Markdown note for a local knowledge system. Return only data matching the requested JSON schema. Prefer existing canonical tags. Propose a new tag only when no existing tag is semantically equivalent. Return at most {max_tags} useful tags. Every tag must cite one or more supplied chunk IDs. Do not infer facts absent from the chunks."
        ),
        user_prompt: format!(
            "Document path: {}\nCurrent title: {}\n\nKnown canonical taxonomy:\n{}\n\nDocument chunks:\n{}",
            document.relative_path, document.title, known_taxonomy_json, chunks
        ),
        json_schema_name: "elephantnote_tagging_extraction_v1".into(),
        max_output_tokens: 2_048,
    }
}

pub fn parse_tagging_response(
    json: &str,
    document: &DocumentSnapshot,
    max_tags: usize,
) -> Result<TaggingExtraction, String> {
    let output: TaggingExtraction =
        serde_json::from_str(json).map_err(|error| format!("Invalid tagging JSON: {error}"))?;
    let validation = output.validate_with_limit(document, max_tags);
    if !validation.valid {
        return Err(validation.errors.join(" "));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;

    fn document() -> DocumentSnapshot {
        analyze_markdown(
            "Notes/Iroh.md",
            "# Iroh\n\nIroh provides peer-to-peer networking and relay fallback.",
            1,
        )
    }

    #[test]
    fn accepts_evidence_backed_existing_tag() {
        let document = document();
        let output = TaggingExtraction {
            suggested_title: Some("Iroh networking".into()),
            tags: vec![TagSuggestion {
                proposed_name: "Iroh".into(),
                existing_tag_id: Some("tag-iroh".into()),
                new_tag: None,
                confidence: 0.98,
                evidence_chunk_ids: vec![document.chunks[0].id.clone()],
                reason: "The note directly describes Iroh.".into(),
            }],
        };
        assert!(output.validate(&document).valid);
    }

    #[test]
    fn rejects_hallucinated_chunk_and_ambiguous_target() {
        let document = document();
        let output = TaggingExtraction {
            suggested_title: None,
            tags: vec![TagSuggestion {
                proposed_name: "Networking".into(),
                existing_tag_id: Some("tag-networking".into()),
                new_tag: Some(NewTagCandidate {
                    display_name: "Networking".into(),
                    parent_id: None,
                    description: String::new(),
                }),
                confidence: 0.8,
                evidence_chunk_ids: vec!["missing".into()],
                reason: "Relevant".into(),
            }],
        };
        let validation = output.validate(&document);
        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("unknown evidence")));
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("simultaneously")));
    }

    #[test]
    fn parses_only_valid_structured_json() {
        let document = document();
        let json = format!(
            r#"{{"suggested_title":"Iroh networking","tags":[{{"proposed_name":"Iroh","existing_tag_id":"tag-iroh","new_tag":null,"confidence":0.95,"evidence_chunk_ids":["{}"],"reason":"Direct subject"}}]}}"#,
            document.chunks[0].id
        );
        assert!(parse_tagging_response(&json, &document, 8).is_ok());
        assert!(parse_tagging_response("not-json", &document, 8).is_err());
    }

    #[test]
    fn request_includes_exact_chunk_identifiers() {
        let document = document();
        let request = build_tagging_request(&document, "[]", 5);
        assert!(request.user_prompt.contains(&document.chunks[0].id));
        assert!(request.system_prompt.contains("at most 5"));
        assert_eq!(request.task, StructuredTask::SuggestTitleAndTags);
    }
}
