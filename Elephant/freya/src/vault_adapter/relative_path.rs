use std::path::Path;

use super::{AdapterError, AdapterResult};

/// Strict boundary used before calling the committed backend helpers.
///
/// `entries::normalize_relative_path` in the committed backend removes parent
/// components. Removing `..` changes the requested target, so Freya rejects it
/// before any filesystem operation instead.
pub(super) fn validate(path: &str) -> AdapterResult<String> {
    if path.contains('\0') {
        return Err(AdapterError::new("Vault paths cannot contain NUL bytes."));
    }

    let normalized = path.replace('\\', "/");
    if Path::new(&normalized).is_absolute()
        || normalized.starts_with('/')
        || has_windows_drive_prefix(&normalized)
    {
        return Err(AdapterError::new(format!(
            "Vault paths must be relative: {normalized}"
        )));
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err(AdapterError::new(format!(
                "Parent path components are not allowed: {normalized}"
            )));
        }
        parts.push(part);
    }
    Ok(parts.join("/"))
}

fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_targets_that_committed_normalization_would_rewrite() {
        assert!(validate("../outside.md").is_err());
        assert!(validate("notes/../../outside.md").is_err());
        assert!(validate("/absolute.md").is_err());
        assert!(validate("C:\\absolute.md").is_err());
        assert!(validate("nul\0byte.md").is_err());
    }

    #[test]
    fn normalizes_only_equivalent_relative_spelling() {
        assert_eq!(validate("./notes\\today.md").unwrap(), "notes/today.md");
    }
}
