use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoteDocument {
    pub markdown: String,
    pub metadata: NoteMetadata,
}

#[derive(Debug)]
pub enum NoteError {
    EmptyPath,
    InvalidTaskLine,
}

impl NoteDocument {
    pub fn from_markdown(markdown: impl Into<String>) -> Self {
        let markdown = markdown.into();
        let title = markdown
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .unwrap_or("Untitled")
            .trim()
            .to_owned();
        let tags = crate::parse_tags(&markdown);
        Self {
            markdown,
            metadata: NoteMetadata {
                title,
                tags,
                created_at: None,
            },
        }
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        let mut lines = self.markdown.lines();
        let replacement = format!("# {title}");
        self.markdown = if matches!(lines.next(), Some(line) if line.starts_with("# ")) {
            std::iter::once(replacement.as_str())
                .chain(lines)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            format!("{replacement}\n\n{}", self.markdown)
        };
        self.metadata.title = title;
    }
}
