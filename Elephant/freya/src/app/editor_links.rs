//! Markdown link resolution and editor actions.
//!
//! Link parsing is kept outside the paragraph renderer so the filesystem
//! boundary can be tested without depending on Freya layout or hit testing.

use muya_core::{
    model::{InlineKind, NodeId, NodeKind},
    Document,
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LinkTarget {
    Internal {
        relative_path: PathBuf,
        fragment: Option<String>,
    },
    External(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MarkdownLink {
    pub(crate) label: String,
    pub(crate) destination: String,
}

pub(crate) fn collect_links(document: &Document) -> Vec<MarkdownLink> {
    let mut links = Vec::new();
    collect_links_from(document, document.root, &mut links);
    links
}

fn collect_links_from(document: &Document, parent: NodeId, links: &mut Vec<MarkdownLink>) {
    for node in document.children(parent) {
        match &node.kind {
            NodeKind::Inline(InlineKind::Link { destination, .. }) => {
                links.push(MarkdownLink {
                    label: inline_text(document, node.id),
                    destination: destination.clone(),
                });
                collect_links_from(document, node.id, links);
            }
            NodeKind::Inline(InlineKind::AutoLink { destination }) => {
                links.push(MarkdownLink {
                    label: destination.clone(),
                    destination: destination.clone(),
                });
                collect_links_from(document, node.id, links);
            }
            _ => collect_links_from(document, node.id, links),
        }
    }
}

fn inline_text(document: &Document, parent: NodeId) -> String {
    let mut text = String::new();
    for node in document.children(parent) {
        match &node.kind {
            NodeKind::Inline(InlineKind::Text { value }) => text.push_str(value),
            NodeKind::Inline(InlineKind::Escaped { value }) => text.push(*value),
            NodeKind::Inline(InlineKind::CodeSpan { code }) => text.push_str(code),
            NodeKind::Inline(InlineKind::SoftBreak | InlineKind::HardBreak) => text.push('\n'),
            _ => text.push_str(&inline_text(document, node.id)),
        }
    }
    text
}

pub(crate) fn resolve(
    root: &Path,
    source_path: &Path,
    destination: &str,
) -> Result<LinkTarget, String> {
    let destination = destination.trim();
    if destination.is_empty() {
        return Err("Link destination is empty".to_owned());
    }
    if is_external(destination) {
        return Ok(LinkTarget::External(destination.to_owned()));
    }

    let (path_part, fragment) = destination
        .split_once('#')
        .map_or((destination, None), |(path, fragment)| {
            (path, (!fragment.is_empty()).then(|| fragment.to_owned()))
        });
    let source_path = canonicalize_inside(root, source_path)?;
    let target = if path_part.is_empty() {
        source_path
    } else {
        let relative = safe_relative_path(path_part)?;
        source_path.parent().unwrap_or(root).join(relative)
    };
    let target = if !target.exists() && target.extension().is_none() {
        target.with_extension("md")
    } else {
        target
    };
    let target = canonicalize_inside(root, &target)?;
    let root = fs::canonicalize(root).map_err(|error| format!("Vault is unavailable: {error}"))?;
    let relative_path = target
        .strip_prefix(root)
        .map_err(|_| "Link target escapes the active vault".to_owned())?
        .to_path_buf();
    Ok(LinkTarget::Internal {
        relative_path,
        fragment,
    })
}

fn is_external(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("mailto:")
        || value.starts_with("ftp://")
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(format!("Refusing unsafe link destination: {value}"));
    }
    Ok(path.to_path_buf())
}

fn canonicalize_inside(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|error| format!("Vault is unavailable: {error}"))?;
    let path = fs::canonicalize(path)
        .map_err(|error| format!("Link target is unavailable at {}: {error}", path.display()))?;
    if !path.starts_with(&root) {
        return Err(format!(
            "Link target escapes the active vault: {}",
            path.display()
        ));
    }
    Ok(path)
}

pub(crate) fn activate(mut state: freya::prelude::State<super::ShellState>, destination: &str) {
    let (root, source) = {
        let shell = state.read();
        let root = shell.vault.as_ref().map(|vault| vault.root().to_path_buf());
        let source = shell
            .editor
            .as_ref()
            .and_then(|editor| editor.path())
            .map(Path::to_path_buf);
        (root, source)
    };
    let Some(root) = root else {
        state.write().error = Some("Cannot open link without an active vault".to_owned());
        return;
    };
    let Some(source) = source else {
        state.write().error = Some("Cannot resolve link without an open note".to_owned());
        return;
    };
    match resolve(&root, &source, destination) {
        Ok(LinkTarget::Internal {
            relative_path,
            fragment,
        }) => {
            let relative = relative_path.to_string_lossy().replace('\\', "/");
            if root.join(&relative_path).is_dir() {
                state.write().open_directory(relative.clone());
            } else {
                state.write().open_note_path(&relative);
            }
            if let Some(fragment) = fragment {
                let result = state
                    .write()
                    .editor
                    .as_mut()
                    .ok_or_else(|| "Cannot scroll link anchor without an open note".to_owned())
                    .and_then(|editor| editor.scroll_to_fragment(&fragment));
                if let Err(error) = result {
                    eprintln!(
                        "[freya][editor] action=link-anchor-failure fragment={fragment} error={error}"
                    );
                    state.write().error = Some(error);
                }
            }
            eprintln!("[freya][editor] action=link-open path={relative}");
        }
        Ok(LinkTarget::External(destination)) => match open_external(&destination) {
            Ok(()) => {
                eprintln!("[freya][editor] action=link-external-open url={destination}");
                state.write().error = None;
            }
            Err(error) => {
                eprintln!("[freya][editor] action=link-external-failure error={error}");
                state.write().error = Some(error);
            }
        },
        Err(error) => {
            eprintln!(
                "[freya][editor] action=link-failure destination={destination} error={error}"
            );
            state.write().error = Some(error);
        }
    }
}

fn open_external(destination: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(destination).status();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(destination).status();
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", "start", "", destination])
        .status();
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let result: Result<std::process::ExitStatus, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "opening external links is unsupported on this platform",
    ));
    let status = result.map_err(|error| format!("Unable to open external link: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("External link opener exited with {status}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-link-domain-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("Notes")).unwrap();
        fs::write(root.join("Notes/Alpha.md"), "# Alpha\n").unwrap();
        fs::write(root.join("Notes/Beta.md"), "# Beta\n").unwrap();
        let source = root.join("Notes/Alpha.md");
        (root, source)
    }

    #[test]
    fn resolves_relative_note_and_fragment_inside_vault() {
        let (root, source) = fixture();
        assert_eq!(
            resolve(&root, &source, "Beta.md#section").unwrap(),
            LinkTarget::Internal {
                relative_path: PathBuf::from("Notes/Beta.md"),
                fragment: Some("section".to_owned()),
            }
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_paths_outside_vault_and_preserves_external_targets() {
        let (root, source) = fixture();
        assert!(resolve(&root, &source, "../outside.md").is_err());
        assert_eq!(
            resolve(&root, &source, "https://example.test").unwrap(),
            LinkTarget::External("https://example.test".to_owned())
        );
        let _ = fs::remove_dir_all(root);
    }
}
