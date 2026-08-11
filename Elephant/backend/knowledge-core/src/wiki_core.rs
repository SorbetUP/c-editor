use crate::extraction::{StructuredModelRequest, StructuredTask};
use crate::model::{DocumentSnapshot, KnowledgeChunk};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiDraftStatus {
    Proposed,
    Accepted,
    Rejected,
    Outdated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiSourceChunk {
    pub document_path: String,
    pub document_title: String,
    pub chunk_id: String,
    pub heading: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiSynthesis {
    pub title: String,
    pub summary: Vec<WikiClaim>,
    #[serde(default)]
    pub sections: Vec<WikiSection>,
    #[serde(default)]
    pub related_wikis: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiSection {
    pub heading: String,
    #[serde(default)]
    pub claims: Vec<WikiClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiClaim {
    pub text: String,
    #[serde(default)]
    pub citation_chunk_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiCitation {
    pub key: String,
    pub document_path: String,
    pub document_title: String,
    pub chunk_id: String,
    pub heading: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderedWiki {
    pub title: String,
    pub slug: String,
    pub markdown: String,
    pub citations: Vec<WikiCitation>,
    pub source_paths: Vec<String>,
    pub source_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WikiDraft {
    pub id: String,
    pub topic: String,
    pub title: String,
    pub slug: String,
    pub markdown: String,
    pub citations: Vec<WikiCitation>,
    pub source_paths: Vec<String>,
    pub source_hash: String,
    pub model_id: String,
    pub status: WikiDraftStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn collect_wiki_sources(documents: &[DocumentSnapshot]) -> Vec<WikiSourceChunk> {
    let mut sources = Vec::new();
    for document in documents {
        let headings = document
            .sections
            .iter()
            .map(|section| (section.id.as_str(), section.heading.as_str()))
            .collect::<HashMap<_, _>>();
        for chunk in &document.chunks {
            sources.push(source_from_chunk(document, chunk, &headings));
        }
    }
    sources
}

fn source_from_chunk(
    document: &DocumentSnapshot,
    chunk: &KnowledgeChunk,
    headings: &HashMap<&str, &str>,
) -> WikiSourceChunk {
    WikiSourceChunk {
        document_path: document.relative_path.clone(),
        document_title: document.title.clone(),
        chunk_id: chunk.id.clone(),
        heading: headings
            .get(chunk.section_id.as_str())
            .copied()
            .unwrap_or(document.title.as_str())
            .to_string(),
        start_offset: chunk.start_offset,
        end_offset: chunk.end_offset,
        text: chunk.text.clone(),
    }
}

pub fn build_wiki_synthesis_request(
    topic: &str,
    requested_title: Option<&str>,
    sources: &[WikiSourceChunk],
    max_sections: usize,
) -> StructuredModelRequest {
    let source_text = sources
        .iter()
        .map(|source| {
            format!(
                "<source chunk_id=\"{}\" document=\"{}\" heading=\"{}\">\n{}\n</source>",
                source.chunk_id, source.document_path, source.heading, source.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    StructuredModelRequest {
        task: StructuredTask::WriteWikiSection,
        system_prompt: format!(
            "You synthesize a cited local wiki from supplied note chunks. Return only data matching the requested JSON schema. Every factual claim must cite one or more supplied chunk IDs. Never cite a chunk that was not supplied. Do not repeat the same idea across sections. Return no more than {max_sections} sections. related_wikis contains only short concept names suitable for wikilinks, never file paths."
        ),
        user_prompt: format!(
            "Topic: {}\nRequested title: {}\n\nSources:\n{}",
            topic.trim(),
            requested_title.unwrap_or("").trim(),
            source_text
        ),
        json_schema_name: "elephantnote_wiki_synthesis_v1".into(),
        max_output_tokens: 6_144,
    }
}

impl WikiSynthesis {
    pub fn validate(&self, sources: &[WikiSourceChunk], max_sections: usize) -> WikiValidation {
        let mut errors = Vec::new();
        if self.title.trim().is_empty() {
            errors.push("Wiki title cannot be empty.".into());
        }
        if self.title.chars().count() > 200 {
            errors.push("Wiki title exceeds 200 characters.".into());
        }
        if self.summary.is_empty() {
            errors.push("Wiki summary must contain at least one cited claim.".into());
        }
        if self.sections.len() > max_sections {
            errors.push(format!(
                "Wiki contains {} sections; the configured maximum is {max_sections}.",
                self.sections.len()
            ));
        }

        let allowed_chunks = sources
            .iter()
            .map(|source| source.chunk_id.as_str())
            .collect::<HashSet<_>>();
        let mut headings = HashSet::new();
        validate_claims("Summary", &self.summary, &allowed_chunks, &mut errors);
        for (index, section) in self.sections.iter().enumerate() {
            let heading = section.heading.trim();
            if heading.is_empty() {
                errors.push(format!("Wiki section {index} has an empty heading."));
            }
            let normalized = heading.to_lowercase();
            if !headings.insert(normalized) {
                errors.push(format!("Wiki section heading is duplicated: {heading}."));
            }
            if section.claims.is_empty() {
                errors.push(format!("Wiki section `{heading}` contains no claims."));
            }
            validate_claims(heading, &section.claims, &allowed_chunks, &mut errors);
        }

        let mut related = HashSet::new();
        for value in &self.related_wikis {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                errors.push("Related wiki names cannot be empty.".into());
            }
            if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
                errors.push(format!("Invalid related wiki name: {trimmed}."));
            }
            if !related.insert(trimmed.to_lowercase()) {
                errors.push(format!("Related wiki is duplicated: {trimmed}."));
            }
        }

        WikiValidation {
            valid: errors.is_empty(),
            errors,
        }
    }
}

fn validate_claims(
    section: &str,
    claims: &[WikiClaim],
    allowed_chunks: &HashSet<&str>,
    errors: &mut Vec<String>,
) {
    let mut seen = HashSet::new();
    for (index, claim) in claims.iter().enumerate() {
        let text = claim.text.trim();
        if text.is_empty() {
            errors.push(format!("Claim {index} in `{section}` is empty."));
        }
        if !seen.insert(text.to_lowercase()) {
            errors.push(format!("Duplicate claim in `{section}`: {text}."));
        }
        if claim.citation_chunk_ids.is_empty() {
            errors.push(format!(
                "Claim {index} in `{section}` must cite at least one source chunk."
            ));
        }
        for chunk_id in &claim.citation_chunk_ids {
            if !allowed_chunks.contains(chunk_id.as_str()) {
                errors.push(format!(
                    "Claim {index} in `{section}` cites an unknown chunk: {chunk_id}."
                ));
            }
        }
    }
}

pub fn parse_and_render_wiki(
    response_json: &str,
    topic: &str,
    sources: &[WikiSourceChunk],
    max_sections: usize,
) -> Result<RenderedWiki, String> {
    let synthesis: WikiSynthesis = serde_json::from_str(response_json)
        .map_err(|error| format!("Invalid wiki synthesis JSON: {error}"))?;
    let validation = synthesis.validate(sources, max_sections);
    if !validation.valid {
        return Err(validation.errors.join(" "));
    }
    render_wiki(&synthesis, topic, sources)
}

pub fn render_wiki(
    synthesis: &WikiSynthesis,
    topic: &str,
    sources: &[WikiSourceChunk],
) -> Result<RenderedWiki, String> {
    let source_by_id = sources
        .iter()
        .map(|source| (source.chunk_id.as_str(), source))
        .collect::<HashMap<_, _>>();
    let mut citation_numbers = BTreeMap::<String, usize>::new();
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str("generated: true\n");
    markdown.push_str("generator: elephantnote-knowledge-core\n");
    markdown.push_str(&format!("topic: {}\n", yaml_scalar(topic)));
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", synthesis.title.trim()));

    render_claims(
        &mut markdown,
        &synthesis.summary,
        &mut citation_numbers,
        &source_by_id,
    )?;
    for section in &synthesis.sections {
        markdown.push_str(&format!("\n## {}\n\n", section.heading.trim()));
        render_claims(
            &mut markdown,
            &section.claims,
            &mut citation_numbers,
            &source_by_id,
        )?;
    }

    if !synthesis.related_wikis.is_empty() {
        markdown.push_str("\n## Related wikis\n\n");
        for related in &synthesis.related_wikis {
            markdown.push_str(&format!("- [[{}]]\n", related.trim()));
        }
    }

    let mut citations = Vec::new();
    if !citation_numbers.is_empty() {
        markdown.push_str("\n## Sources\n\n");
        for (chunk_id, number) in &citation_numbers {
            let source = source_by_id
                .get(chunk_id.as_str())
                .ok_or_else(|| format!("Missing source chunk while rendering: {chunk_id}"))?;
            let key = format!("source-{number}");
            let anchor = if source.heading.trim().is_empty() {
                String::new()
            } else {
                format!("#{}", source.heading.trim())
            };
            markdown.push_str(&format!(
                "[^{}]: [[{}{}|{} — {}]] (bytes {}–{})\n",
                key,
                source.document_path,
                anchor,
                source.document_title,
                source.heading,
                source.start_offset,
                source.end_offset
            ));
            citations.push(WikiCitation {
                key,
                document_path: source.document_path.clone(),
                document_title: source.document_title.clone(),
                chunk_id: source.chunk_id.clone(),
                heading: source.heading.clone(),
                start_offset: source.start_offset,
                end_offset: source.end_offset,
            });
        }
    }

    let slug = slugify(&synthesis.title);
    let mut source_paths = sources
        .iter()
        .map(|source| source.document_path.clone())
        .collect::<Vec<_>>();
    source_paths.sort();
    source_paths.dedup();
    let source_hash = source_hash(sources);
    Ok(RenderedWiki {
        title: synthesis.title.trim().to_string(),
        slug,
        markdown,
        citations,
        source_paths,
        source_hash,
    })
}

fn render_claims(
    markdown: &mut String,
    claims: &[WikiClaim],
    citation_numbers: &mut BTreeMap<String, usize>,
    source_by_id: &HashMap<&str, &WikiSourceChunk>,
) -> Result<(), String> {
    for claim in claims {
        let mut references = Vec::new();
        for chunk_id in &claim.citation_chunk_ids {
            if !source_by_id.contains_key(chunk_id.as_str()) {
                return Err(format!("Unknown source chunk while rendering: {chunk_id}"));
            }
            let next = citation_numbers.len() + 1;
            let number = *citation_numbers.entry(chunk_id.clone()).or_insert(next);
            references.push(format!("[^source-{number}]"));
        }
        references.sort();
        references.dedup();
        markdown.push_str(claim.text.trim());
        markdown.push(' ');
        markdown.push_str(&references.join(""));
        markdown.push_str("\n\n");
    }
    Ok(())
}

pub fn wiki_draft_from_rendered(
    topic: &str,
    rendered: RenderedWiki,
    model_id: &str,
    timestamp: i64,
) -> WikiDraft {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"wiki-draft");
    hasher.update(&[0]);
    hasher.update(topic.as_bytes());
    hasher.update(&[0]);
    hasher.update(rendered.source_hash.as_bytes());
    hasher.update(&[0]);
    hasher.update(rendered.markdown.as_bytes());
    let hex = hasher.finalize().to_hex().to_string();
    WikiDraft {
        id: format!("wiki-{}", &hex[..24]),
        topic: topic.trim().to_string(),
        title: rendered.title,
        slug: rendered.slug,
        markdown: rendered.markdown,
        citations: rendered.citations,
        source_paths: rendered.source_paths,
        source_hash: rendered.source_hash,
        model_id: model_id.to_string(),
        status: WikiDraftStatus::Proposed,
        created_at: timestamp,
        updated_at: timestamp,
    }
}

pub fn source_hash(sources: &[WikiSourceChunk]) -> String {
    let mut sorted = sources.to_vec();
    sorted.sort_by(|left, right| left.chunk_id.cmp(&right.chunk_id));
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"wiki-sources");
    for source in sorted {
        hasher.update(&[0]);
        hasher.update(source.document_path.as_bytes());
        hasher.update(&[0]);
        hasher.update(source.chunk_id.as_bytes());
        hasher.update(&[0]);
        hasher.update(source.text.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for character in value.trim().chars() {
        if character.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            for lower in character.to_lowercase() {
                slug.push(lower);
            }
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "wiki".into()
    } else {
        slug.chars().take(120).collect()
    }
}

fn yaml_scalar(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::analyze_markdown;

    fn sources() -> Vec<WikiSourceChunk> {
        let documents = vec![
            analyze_markdown(
                "Notes/Iroh.md",
                "# Iroh\n\nIroh attempts direct peer-to-peer connections and can use relays.",
                1,
            ),
            analyze_markdown(
                "Notes/Sync.md",
                "# Sync\n\nElephantNote uses Iroh as its synchronization transport.",
                1,
            ),
        ];
        collect_wiki_sources(&documents)
    }

    #[test]
    fn rejects_uncited_or_unknown_claims() {
        let sources = sources();
        let synthesis = WikiSynthesis {
            title: "Iroh".into(),
            summary: vec![WikiClaim {
                text: "Unsupported claim".into(),
                citation_chunk_ids: vec!["missing".into()],
            }],
            sections: Vec::new(),
            related_wikis: Vec::new(),
        };
        let validation = synthesis.validate(&sources, 8);
        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("unknown chunk")));
    }

    #[test]
    fn renders_markdown_and_footnotes_from_structured_claims() {
        let sources = sources();
        let synthesis = WikiSynthesis {
            title: "Iroh synchronization".into(),
            summary: vec![WikiClaim {
                text: "Iroh supports direct peer-to-peer connectivity with relay fallback.".into(),
                citation_chunk_ids: vec![sources[0].chunk_id.clone()],
            }],
            sections: vec![WikiSection {
                heading: "Use in ElephantNote".into(),
                claims: vec![WikiClaim {
                    text: "ElephantNote uses Iroh as its synchronization transport.".into(),
                    citation_chunk_ids: vec![sources[1].chunk_id.clone()],
                }],
            }],
            related_wikis: vec!["Peer-to-peer networking".into()],
        };
        let rendered = render_wiki(&synthesis, "Iroh", &sources).unwrap();
        assert!(rendered.markdown.contains("generated: true"));
        assert!(rendered.markdown.contains("[^source-1]"));
        assert!(rendered.markdown.contains("[[Notes/Iroh.md#Iroh"));
        assert!(rendered.markdown.contains("[[Peer-to-peer networking]]"));
        assert_eq!(rendered.citations.len(), 2);
    }

    #[test]
    fn output_is_stable_for_same_sources_and_synthesis() {
        let sources = sources();
        let synthesis = WikiSynthesis {
            title: "Iroh".into(),
            summary: vec![WikiClaim {
                text: "Iroh provides connectivity.".into(),
                citation_chunk_ids: vec![sources[0].chunk_id.clone()],
            }],
            sections: Vec::new(),
            related_wikis: Vec::new(),
        };
        let first = render_wiki(&synthesis, "Iroh", &sources).unwrap();
        let second = render_wiki(&synthesis, "Iroh", &sources).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            slugify("  Iroh & Synchronisation  "),
            "iroh-synchronisation"
        );
    }
}
