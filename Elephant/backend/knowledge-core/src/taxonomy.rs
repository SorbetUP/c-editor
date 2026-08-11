use serde::{Deserialize, Serialize};

const MAX_TAG_KEY_LENGTH: usize = 160;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalTag {
    pub id: String,
    pub canonical_name: String,
    pub display_name: String,
    pub parent_id: Option<String>,
    pub description: String,
    pub status: TagStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TagStatus {
    Active,
    Merged,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagAlias {
    pub normalized_alias: String,
    pub tag_id: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTagCandidate {
    pub display_name: String,
    pub parent_id: Option<String>,
    pub description: String,
}

impl CanonicalTag {
    pub fn new(
        display_name: impl Into<String>,
        parent_id: Option<String>,
        description: impl Into<String>,
    ) -> Result<Self, String> {
        let display_name = clean_display_name(&display_name.into())?;
        let canonical_name = canonical_tag_key(&display_name)?;
        let id = stable_tag_id(&canonical_name, parent_id.as_deref());
        Ok(Self {
            id,
            canonical_name,
            display_name,
            parent_id,
            description: description.into().trim().to_string(),
            status: TagStatus::Active,
        })
    }
}

impl TagAlias {
    pub fn new(
        alias: impl AsRef<str>,
        tag_id: impl Into<String>,
        language: Option<String>,
    ) -> Result<Self, String> {
        Ok(Self {
            normalized_alias: normalize_alias(alias.as_ref())?,
            tag_id: tag_id.into(),
            language: language
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty()),
        })
    }
}

pub fn normalize_alias(value: &str) -> Result<String, String> {
    let normalized = normalize_words(value);
    validate_key(&normalized, "Tag alias")?;
    Ok(normalized)
}

pub fn canonical_tag_key(value: &str) -> Result<String, String> {
    let segments = value
        .split(['/', '>'])
        .map(normalize_words)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let canonical = segments.join("/");
    validate_key(&canonical, "Canonical tag")?;
    Ok(canonical)
}

pub fn clean_display_name(value: &str) -> Result<String, String> {
    let display = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    validate_key(&display, "Tag display name")?;
    Ok(display)
}

pub fn stable_tag_id(canonical_name: &str, parent_id: Option<&str>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"tag");
    hasher.update(&[0]);
    if let Some(parent_id) = parent_id {
        hasher.update(parent_id.as_bytes());
    }
    hasher.update(&[0]);
    hasher.update(canonical_name.as_bytes());
    let hex = hasher.finalize().to_hex().to_string();
    format!("tag-{}", &hex[..24])
}

fn normalize_words(value: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for character in value.trim().chars() {
        if character.is_alphanumeric() || matches!(character, '-' | '_' | '+' | '#' | '.') {
            if pending_space && !output.is_empty() {
                output.push(' ');
            }
            for lower in character.to_lowercase() {
                output.push(lower);
            }
            pending_space = false;
        } else if character.is_whitespace() || matches!(character, ':' | ';' | ',' | '|') {
            pending_space = true;
        }
    }
    output.trim().to_string()
}

fn validate_key(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} cannot be empty."));
    }
    if value.chars().count() > MAX_TAG_KEY_LENGTH {
        return Err(format!("{field} exceeds {MAX_TAG_KEY_LENGTH} characters."));
    }
    if value.starts_with('.') || value.contains("..") {
        return Err(format!("{field} contains a reserved path segment."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_case_spacing_and_hierarchy_separators() {
        assert_eq!(
            canonical_tag_key("  Programming Languages > Rust  ").unwrap(),
            "programming languages/rust"
        );
        assert_eq!(
            normalize_alias("Artificial   Intelligence").unwrap(),
            "artificial intelligence"
        );
    }

    #[test]
    fn keeps_distinct_languages_until_an_alias_is_explicitly_added() {
        assert_ne!(
            normalize_alias("AI").unwrap(),
            normalize_alias("Intelligence artificielle").unwrap()
        );
        let tag = CanonicalTag::new("Intelligence artificielle", None, "").unwrap();
        let alias =
            TagAlias::new("Artificial Intelligence", tag.id.clone(), Some("en".into())).unwrap();
        assert_eq!(alias.tag_id, tag.id);
        assert_eq!(alias.normalized_alias, "artificial intelligence");
    }

    #[test]
    fn stable_ids_change_when_parent_changes() {
        let first = stable_tag_id("rust", Some("tag-programming"));
        let second = stable_tag_id("rust", Some("tag-materials"));
        assert_ne!(first, second);
        assert_eq!(first, stable_tag_id("rust", Some("tag-programming")));
    }

    #[test]
    fn rejects_empty_and_reserved_values() {
        assert!(canonical_tag_key("   ").is_err());
        assert!(canonical_tag_key("../secret").is_err());
    }
}
