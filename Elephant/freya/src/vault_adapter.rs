//! Typed Freya adapter for the existing Elephant vault contract.
//!
//! This module is deliberately an adapter, not a second filesystem
//! implementation. The CRUD and listing operations below call the production
//! Tauri helpers included with `#[path]`. The only local filesystem reads are
//! metadata enrichment for fields that the current `entries.rs` response does
//! not expose yet (`tags` and an Excalidraw image preview).
//!
//! Provenance:
//! - `Elephant/backend/tauri/src/vault/entries.rs`: path validation,
//!   directory pagination, entry summaries, note/folder CRUD, rename, move,
//!   delete-to-trash, restore and empty-trash behavior.
//! - `Elephant/backend/tauri/src/vault/types.rs`: `VaultDescriptor`, vault
//!   identity and schema types.
//! - `Elephant/backend/tauri/src/vault_layout.rs`: hidden vault roots,
//!   internal directories and visible-path rules.
//! - `Elephant/frontend/app/stores/vaultStore.js`: `path`, `filename`,
//!   `title`, `type`/`kind`, `noteCount`, `excerpt`, `tags`, `updatedAt`,
//!   pagination semantics and move/delete path behavior.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;
use std::fs;
use std::path::Path;

// `entries.rs` retains its production `crate::vault_layout` import. The host
// Freya crate should expose the same path module at its crate root. Keeping
// this declaration here also makes this file usable as a small standalone
// crate root in focused adapter tests.
#[path = "../../backend/tauri/src/vault_layout.rs"]
pub mod vault_layout;

// `entries.rs` expects these modules as siblings. `config.rs` cannot be
// included directly because it owns Tauri's AppHandle-facing registry; these
// two pure helpers are the only functions used by entries/metadata.
mod config {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now_string() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    }

    pub fn basename(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Personal")
            .to_string()
    }
}

#[path = "../../backend/tauri/src/vault/entries.rs"]
pub(crate) mod entries;
#[path = "../../backend/tauri/src/vault/metadata.rs"]
pub(crate) mod metadata;
#[path = "../../backend/tauri/src/fts.rs"]
pub(crate) mod production_fts;
#[path = "../../backend/tauri/src/vault/types.rs"]
pub(crate) mod types;

use entries as production_entries;
use types::VaultDescriptor;

pub const DEFAULT_PAGE_SIZE: usize = 120;
pub const MAX_PAGE_SIZE: usize = 500;

/// Exact source locations used by this adapter.
pub const PROVENANCE: &[&str] = &[
    "Elephant/backend/tauri/src/vault/entries.rs::list_directory_page",
    "Elephant/backend/tauri/src/vault/entries.rs::create_note",
    "Elephant/backend/tauri/src/vault/entries.rs::create_folder",
    "Elephant/backend/tauri/src/vault/entries.rs::rename_entry",
    "Elephant/backend/tauri/src/vault/entries.rs::move_entry",
    "Elephant/backend/tauri/src/vault/entries.rs::delete_entry",
    "Elephant/backend/tauri/src/vault/entries.rs::list_trash",
    "Elephant/backend/tauri/src/vault/entries.rs::restore_trash",
    "Elephant/backend/tauri/src/vault/entries.rs::empty_trash",
    "Elephant/backend/tauri/src/vault/types.rs::VaultDescriptor",
    "Elephant/backend/tauri/src/vault_layout.rs::is_visible_vault_path",
    "Elephant/backend/tauri/src/vault_layout.rs::hidden_root",
    "Elephant/backend/tauri/src/vault_layout.rs::hidden_dir",
    "Elephant/frontend/app/stores/vaultStore.js::activeEntries",
    "Elephant/frontend/app/stores/vaultStore.js::moveEntry",
    "Elephant/frontend/app/stores/vaultStore.js::deleteEntry",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterError(String);

impl AdapterError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AdapterError {}

impl From<String> for AdapterError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<std::io::Error> for AdapterError {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for AdapterError {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

pub type AdapterResult<T> = Result<T, AdapterError>;

/// Values used by the web store's `type`/`kind` contract without discarding
/// future backend values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Note,
    Folder,
    Drawing,
    File,
    Other(String),
}

impl EntryKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Note => "note",
            Self::Folder => "folder",
            Self::Drawing => "drawing",
            Self::File => "file",
            Self::Other(value) => value,
        }
    }
}

impl From<String> for EntryKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "note" => Self::Note,
            "folder" => Self::Folder,
            "drawing" => Self::Drawing,
            "file" => Self::File,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for EntryKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EntryKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildPreview {
    pub title: String,
    #[serde(rename = "type")]
    pub entry_type: EntryKind,
    pub kind: EntryKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultEntry {
    pub path: String,
    pub filename: String,
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub entry_type: EntryKind,
    pub kind: EntryKind,
    pub is_directory: bool,
    pub note_count: usize,
    pub excerpt: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub updated_at: String,
    pub children_preview: Vec<ChildPreview>,
    pub drawing_preview: Option<String>,
    pub full_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    pub relative_path: String,
    pub offset: usize,
    pub limit: usize,
    pub include_preview: bool,
}

impl PageRequest {
    pub fn new(relative_path: impl Into<String>) -> Self {
        Self {
            relative_path: relative_path.into(),
            offset: 0,
            limit: DEFAULT_PAGE_SIZE,
            include_preview: true,
        }
    }

    pub fn with_window(mut self, offset: usize, limit: usize) -> Self {
        self.offset = offset;
        self.limit = limit;
        self
    }

    pub fn without_preview(mut self) -> Self {
        self.include_preview = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultPage {
    pub entries: Vec<VaultEntry>,
    pub relative_path: String,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub next_offset: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub trash_path: String,
    pub restore_token: String,
    pub original_path: String,
    pub deleted_at: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResult {
    pub deleted: bool,
    pub path: String,
    pub original_path: String,
    pub trash_path: String,
    pub restore_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored: bool,
    pub path: String,
    pub original_path: String,
    pub trash_path: String,
    #[serde(default)]
    pub cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyTrashResult {
    pub emptied: bool,
    pub count: usize,
    pub removed: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct VaultAdapter {
    descriptor: VaultDescriptor,
}

impl VaultAdapter {
    pub fn open(root: impl AsRef<Path>) -> AdapterResult<Self> {
        let root = fs::canonicalize(root.as_ref()).map_err(|error| {
            AdapterError::new(format!(
                "Unable to open vault {}: {error}",
                root.as_ref().display()
            ))
        })?;
        if !root.is_dir() {
            return Err(AdapterError::new(format!(
                "Vault path is not a directory: {}",
                root.display()
            )));
        }

        let name = config::basename(&root);
        Ok(Self {
            descriptor: VaultDescriptor {
                id: types::slug_id(&name),
                name,
                path: root.to_string_lossy().replace('\\', "/"),
                icon: String::new(),
                last_opened_at: config::now_string(),
                enabled: true,
            },
        })
    }

    pub fn from_descriptor(descriptor: VaultDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn descriptor(&self) -> &VaultDescriptor {
        &self.descriptor
    }

    pub fn root(&self) -> &Path {
        Path::new(&self.descriptor.path)
    }

    pub fn list(&self, request: PageRequest) -> AdapterResult<VaultPage> {
        let relative_path = self.validate_visible_path(&request.relative_path, true)?;
        let limit = request.limit.clamp(1, MAX_PAGE_SIZE);
        let raw_entries = production_entries::list_directory_page(
            &self.descriptor,
            &relative_path,
            request.offset,
            Some(limit.saturating_add(1)),
            request.include_preview,
        )?;
        let has_more = raw_entries.len() > limit;
        let entries = raw_entries
            .into_iter()
            .take(limit)
            .map(|value| self.entry_from_value(value))
            .collect::<AdapterResult<Vec<_>>>()?;

        Ok(VaultPage {
            entries,
            relative_path,
            offset: request.offset,
            limit,
            has_more,
            next_offset: has_more.then_some(request.offset.saturating_add(limit)),
        })
    }

    pub fn list_directory(&self, relative_path: impl Into<String>) -> AdapterResult<VaultPage> {
        self.list(PageRequest::new(relative_path))
    }

    pub fn find_entry(&self, relative_path: &str) -> AdapterResult<VaultEntry> {
        let relative_path = self.validate_visible_path(relative_path, false)?;
        let parent = parent_relative_path(&relative_path);
        let mut offset = 0;
        loop {
            let page = self.list(
                PageRequest::new(parent)
                    .with_window(offset, MAX_PAGE_SIZE)
                    .without_preview(),
            )?;
            if let Some(entry) = page
                .entries
                .into_iter()
                .find(|entry| entry.path == relative_path)
            {
                return Ok(entry);
            }
            let Some(next_offset) = page.next_offset else {
                break;
            };
            offset = next_offset;
        }
        Err(AdapterError::new(format!(
            "Search result is no longer present: {relative_path}"
        )))
    }

    pub fn rebuild_search_index(&self) -> AdapterResult<production_fts::IndexRefreshStatus> {
        let index = production_fts::FtsIndex::open(self.root())
            .map_err(|error| AdapterError::new(format!("Unable to open search index: {error}")))?;
        index
            .rebuild_from_files(&self.descriptor.id, self.root())
            .map_err(|error| AdapterError::new(format!("Unable to rebuild search index: {error}")))
    }

    pub fn search_index(
        &self,
        query: &str,
        limit: usize,
    ) -> AdapterResult<Vec<production_fts::Hit>> {
        let index = production_fts::FtsIndex::open(self.root())
            .map_err(|error| AdapterError::new(format!("Unable to open search index: {error}")))?;
        index
            .search(query, limit.clamp(1, MAX_PAGE_SIZE))
            .map_err(|error| AdapterError::new(format!("Unable to query search index: {error}")))
    }

    pub fn create_note(
        &self,
        relative_path: Option<String>,
        filename: Option<String>,
        title: Option<String>,
    ) -> AdapterResult<VaultEntry> {
        let relative_path = self.validate_optional_directory(relative_path)?;
        self.ensure_internal_layout_safe()?;
        let value =
            production_entries::create_note(&self.descriptor, relative_path, filename, title)?;
        self.entry_from_value(value)
    }

    pub fn create_folder(&self, relative_path: Option<String>) -> AdapterResult<VaultEntry> {
        let relative_path = self.validate_optional_entry_path(relative_path)?;
        self.ensure_internal_layout_safe()?;
        let value = production_entries::create_folder(&self.descriptor, relative_path)?;
        self.entry_from_value(value)
    }

    pub fn rename(&self, relative_path: impl AsRef<str>, title: String) -> AdapterResult<()> {
        let relative_path = self.validate_visible_path(relative_path.as_ref(), false)?;
        self.ensure_internal_layout_safe()?;
        production_entries::rename_entry(&self.descriptor, relative_path, title)?;
        Ok(())
    }

    /// Returns `false` for the same no-op/self-move cases rejected by
    /// `vaultStore.moveEntry`; otherwise it calls the production helper and
    /// returns `true` after the filesystem operation succeeds.
    pub fn move_entry(
        &self,
        relative_path: impl AsRef<str>,
        target_directory_path: Option<String>,
    ) -> AdapterResult<bool> {
        let relative_path = self.validate_visible_path(relative_path.as_ref(), false)?;
        let target_directory_path = target_directory_path
            .map(|path| self.validate_visible_path(&path, true))
            .transpose()?;
        let target = target_directory_path.as_deref().unwrap_or("");
        if target == relative_path || target.starts_with(&format!("{relative_path}/")) {
            return Ok(false);
        }
        if parent_relative_path(&relative_path) == target {
            return Ok(false);
        }
        self.ensure_internal_layout_safe()?;
        production_entries::move_entry(&self.descriptor, relative_path, target_directory_path)?;
        Ok(true)
    }

    pub fn delete(&self, relative_path: impl AsRef<str>) -> AdapterResult<DeleteResult> {
        let relative_path = self.validate_visible_path(relative_path.as_ref(), false)?;
        self.ensure_internal_layout_safe()?;
        let value = production_entries::delete_entry(&self.descriptor, relative_path)?;
        serde_json::from_value(value).map_err(AdapterError::from)
    }

    pub fn list_trash(&self) -> AdapterResult<Vec<TrashEntry>> {
        production_entries::list_trash(&self.descriptor)?
            .into_iter()
            .map(|value| serde_json::from_value(value).map_err(AdapterError::from))
            .collect()
    }

    pub fn restore_trash(&self, trash_path: impl AsRef<str>) -> AdapterResult<RestoreResult> {
        self.ensure_internal_layout_safe()?;
        let trash_path = production_entries::validate_relative_path(trash_path.as_ref())?;
        let value = production_entries::restore_trash(&self.descriptor, trash_path)?;
        serde_json::from_value(value).map_err(AdapterError::from)
    }

    pub fn empty_trash(&self) -> AdapterResult<EmptyTrashResult> {
        self.ensure_internal_layout_safe()?;
        let value = production_entries::empty_trash(&self.descriptor)?;
        serde_json::from_value(value).map_err(AdapterError::from)
    }

    fn validate_optional_directory(
        &self,
        relative_path: Option<String>,
    ) -> AdapterResult<Option<String>> {
        relative_path
            .map(|path| self.validate_visible_path(&path, true))
            .transpose()
    }

    fn validate_optional_entry_path(
        &self,
        relative_path: Option<String>,
    ) -> AdapterResult<Option<String>> {
        relative_path
            .map(|path| self.validate_visible_path(&path, false))
            .transpose()
    }

    fn validate_visible_path(&self, path: &str, allow_empty: bool) -> AdapterResult<String> {
        let normalized = production_entries::validate_relative_path(path)?;
        if !allow_empty && normalized.is_empty() {
            return Err(AdapterError::new("A visible vault entry path is required."));
        }
        if !vault_layout::is_visible_vault_path(&normalized)
            || normalized
                .split('/')
                .any(production_entries::is_ignored_entry)
        {
            return Err(AdapterError::new(format!(
                "Refusing hidden or internal vault path: {normalized}"
            )));
        }
        self.ensure_no_symlink_components(&normalized)?;
        Ok(normalized)
    }

    fn ensure_no_symlink_components(&self, relative_path: &str) -> AdapterResult<()> {
        let mut current = self.root().to_path_buf();
        for component in relative_path.split('/').filter(|part| !part.is_empty()) {
            current.push(component);
            let metadata = match fs::symlink_metadata(&current) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(error.into()),
            };
            if metadata.file_type().is_symlink() {
                return Err(AdapterError::new(format!(
                    "Refusing to follow a symlinked vault path: {}",
                    current.display()
                )));
            }
        }
        Ok(())
    }

    fn ensure_internal_layout_safe(&self) -> AdapterResult<()> {
        for path in [
            vault_layout::hidden_root(self.root()),
            vault_layout::hidden_dir(self.root(), vault_layout::TRASH_DIR),
        ] {
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                if metadata.file_type().is_symlink() {
                    return Err(AdapterError::new(format!(
                        "Refusing to use a symlinked internal vault path: {}",
                        path.display()
                    )));
                }
            }
        }
        Ok(())
    }

    fn entry_from_value(&self, value: Value) -> AdapterResult<VaultEntry> {
        let raw: RawEntry = serde_json::from_value(value)?;
        let path = required(raw.path, "entry path")?;
        let name = raw
            .name
            .or(raw.filename)
            .unwrap_or_else(|| filename_from_path(&path));
        let is_directory = raw.is_directory.unwrap_or(false);
        let type_name = raw
            .entry_type
            .or(raw.kind)
            .unwrap_or_else(|| if is_directory { "folder" } else { "file" }.to_string());
        let entry_type = if is_directory {
            EntryKind::Folder
        } else {
            EntryKind::from(type_name)
        };
        let title = raw
            .title
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| title_from_filename(&name));
        let preview = raw.preview.unwrap_or_default();
        let excerpt = raw.excerpt.unwrap_or_else(|| preview.clone());
        let tags = match raw.tags.as_ref() {
            Some(value) => tags_from_value(value),
            None => self.read_tags(&path),
        };
        let drawing_preview = raw
            .drawing_preview
            .filter(|preview| !preview.trim().is_empty())
            .or_else(|| drawing_preview_for(&path, &preview, &excerpt));
        let children_preview = raw
            .children_preview
            .into_iter()
            .map(|child| {
                let child_type = EntryKind::from(
                    child
                        .entry_type
                        .or(child.kind)
                        .unwrap_or_else(|| "file".to_string()),
                );
                ChildPreview {
                    title: child.title.unwrap_or_else(|| "Untitled".to_string()),
                    entry_type: child_type.clone(),
                    kind: child_type,
                }
            })
            .collect();

        Ok(VaultEntry {
            path: path.clone(),
            filename: name.clone(),
            name,
            title,
            entry_type: entry_type.clone(),
            kind: entry_type,
            is_directory,
            note_count: raw.note_count.unwrap_or(0),
            excerpt,
            preview,
            tags,
            updated_at: raw.updated_at.unwrap_or_default(),
            children_preview,
            drawing_preview,
            full_path: raw
                .full_path
                .unwrap_or_else(|| self.root().join(&path).to_string_lossy().into_owned()),
        })
    }

    fn read_tags(&self, relative_path: &str) -> Vec<String> {
        if !relative_path.to_ascii_lowercase().ends_with(".md") {
            return Vec::new();
        }
        fs::read_to_string(self.root().join(relative_path))
            .ok()
            .map(|markdown| frontmatter_tags(&markdown))
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEntry {
    path: Option<String>,
    name: Option<String>,
    filename: Option<String>,
    title: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    kind: Option<String>,
    is_directory: Option<bool>,
    note_count: Option<usize>,
    excerpt: Option<String>,
    preview: Option<String>,
    tags: Option<Value>,
    updated_at: Option<String>,
    #[serde(default)]
    children_preview: Vec<RawChildPreview>,
    drawing_preview: Option<String>,
    full_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawChildPreview {
    title: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    kind: Option<String>,
}

fn required(value: Option<String>, field: &str) -> AdapterResult<String> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AdapterError::new(format!("Production entry is missing {field}.")))
}

fn filename_from_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn title_from_filename(filename: &str) -> String {
    filename
        .strip_suffix(".md")
        .or_else(|| filename.strip_suffix(".excalidraw"))
        .or_else(|| filename.strip_suffix(".excalidraw.png"))
        .unwrap_or(filename)
        .to_string()
}

fn parent_relative_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

fn tags_from_value(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect(),
        Value::String(value) => value
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn frontmatter_tags(markdown: &str) -> Vec<String> {
    let mut lines = markdown.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }

    let mut tags = Vec::new();
    let mut reading_list = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if reading_list {
            if let Some(value) = trimmed.strip_prefix('-') {
                let value = value.trim().trim_matches(['"', '\'']);
                if !value.is_empty() {
                    tags.push(value.to_string());
                }
                continue;
            }
            reading_list = false;
        }
        let Some(value) = trimmed.strip_prefix("tags:") else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            reading_list = true;
        } else if value.starts_with('[') && value.ends_with(']') {
            tags.extend(
                value[1..value.len() - 1]
                    .split(',')
                    .map(str::trim)
                    .map(|tag| tag.trim_matches(['"', '\'']))
                    .filter(|tag| !tag.is_empty())
                    .map(str::to_string),
            );
        } else {
            let value = value.trim_matches(['"', '\'']);
            if !value.is_empty() {
                tags.push(value.to_string());
            }
        }
    }
    tags
}

fn drawing_preview_for(path: &str, preview: &str, excerpt: &str) -> Option<String> {
    let lower_path = path.to_ascii_lowercase();
    if lower_path.ends_with(".excalidraw.png") {
        return Some(path.to_string());
    }
    if lower_path.ends_with(".excalidraw") {
        return Some(format!("{path}.png"));
    }
    drawing_path_from_markdown(preview).or_else(|| drawing_path_from_markdown(excerpt))
}

fn drawing_path_from_markdown(markdown: &str) -> Option<String> {
    let marker = ".assets/";
    let start = markdown.find(marker)?;
    let mut begin = start;
    while begin > 0 {
        let previous = markdown.as_bytes()[begin - 1] as char;
        if matches!(previous, '.' | '/' | '\\') {
            begin -= 1;
        } else {
            break;
        }
    }
    let tail = &markdown[begin..];
    let end = tail
        .find(|character: char| character.is_whitespace() || matches!(character, ')' | ']' | '"'))
        .unwrap_or(tail.len());
    let candidate = tail[..end].trim();
    if candidate.to_ascii_lowercase().ends_with(".png") {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-vault-adapter-{name}-{stamp}"));
        fs::create_dir_all(&root).expect("create fixture root");
        root
    }

    #[test]
    fn preserves_production_entry_shape_and_page_window() {
        let root = temp_root("shape");
        fs::create_dir_all(root.join("Projects")).expect("create folder");
        fs::write(
            root.join("Projects/Plan.md"),
            "---\ntitle: Plan\ntype: drawing\ntags: [work, roadmap]\n---\n\n# Plan\n\n![Excalidraw: Plan](../.assets/Plan.png)\n",
        )
        .expect("write note");
        fs::write(root.join("Alpha.md"), "# Alpha\n\nBody").expect("write alpha");
        let adapter = VaultAdapter::open(&root).expect("open adapter");

        let page = adapter
            .list(PageRequest::new("").with_window(0, 1))
            .expect("list page");
        assert_eq!(page.entries.len(), 1);
        assert!(page.has_more);
        assert_eq!(page.next_offset, Some(1));

        let nested = adapter
            .list_directory("Projects")
            .expect("list nested folder");
        let entry = &nested.entries[0];
        assert_eq!(entry.path, "Projects/Plan.md");
        assert_eq!(entry.filename, "Plan.md");
        assert_eq!(entry.name, "Plan.md");
        assert_eq!(entry.title, "Plan");
        assert_eq!(entry.entry_type, EntryKind::Drawing);
        assert_eq!(entry.kind, EntryKind::Drawing);
        assert_eq!(entry.tags, vec!["work", "roadmap"]);
        assert!(entry.excerpt.contains("Excalidraw"));
        assert_eq!(
            entry.drawing_preview.as_deref(),
            Some("../.assets/Plan.png")
        );
        assert!(!entry.updated_at.is_empty());

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn delegates_crud_and_trash_to_production_helpers() {
        let root = temp_root("crud");
        let adapter = VaultAdapter::open(&root).expect("open adapter");
        let folder = adapter
            .create_folder(Some("Projects".to_string()))
            .expect("create folder");
        assert_eq!(folder.entry_type, EntryKind::Folder);
        let note = adapter
            .create_note(
                Some("Projects".to_string()),
                Some("Plan.md".to_string()),
                Some("Plan".to_string()),
            )
            .expect("create note");
        assert_eq!(note.path, "Projects/Plan.md");
        assert_eq!(
            fs::read_to_string(root.join(&note.path)).unwrap(),
            "# Plan\n"
        );

        adapter
            .rename(&note.path, "Renamed".to_string())
            .expect("rename");
        assert!(adapter
            .move_entry("Projects/Renamed.md", None)
            .expect("move"));
        let deleted = adapter.delete("Renamed.md").expect("delete");
        assert!(deleted.deleted);
        let trash = adapter.list_trash().expect("list trash");
        assert_eq!(trash.len(), 1);
        let restored = adapter
            .restore_trash(&trash[0].trash_path)
            .expect("restore");
        assert!(restored.restored);
        assert!(root.join("Renamed.md").is_file());
        let emptied = adapter.empty_trash().expect("empty trash");
        assert!(emptied.emptied);
        assert_eq!(emptied.count, 0);

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn rejects_hidden_and_symlinked_paths_before_mutation() {
        let root = temp_root("safety");
        fs::create_dir_all(root.join(".assets")).expect("hidden directory");
        fs::write(root.join("Visible.md"), "visible").expect("visible note");
        let adapter = VaultAdapter::open(&root).expect("open adapter");
        assert!(adapter.list_directory(".assets").is_err());
        assert!(adapter.delete(".assets").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = temp_root("safety-outside");
            symlink(&outside, root.join("linked")).expect("symlink");
            assert!(adapter
                .create_note(
                    Some("linked".to_string()),
                    Some("Escape.md".to_string()),
                    Some("Escape".to_string()),
                )
                .is_err());
            assert!(!outside.join("Escape.md").exists());
            fs::remove_dir_all(outside).expect("remove outside");
        }

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn keeps_unknown_backend_kinds_instead_of_collapsing_them() {
        let kind = EntryKind::from("calendar-event".to_string());
        assert_eq!(kind.as_str(), "calendar-event");
        let value = serde_json::to_value(&kind).expect("serialize kind");
        assert_eq!(value, Value::String("calendar-event".to_string()));
    }
}
