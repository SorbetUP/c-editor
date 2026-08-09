use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownDocument {
  pub frontmatter: Value,
  pub blocks: Vec<MarkdownBlock>,
  pub outline: Vec<MarkdownHeading>,
  pub links: Vec<MarkdownLink>,
  pub images: Vec<MarkdownImage>,
  pub tasks: Vec<MarkdownTask>,
  pub plain_text: String,
  pub html: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownBlock {
  pub kind: String,
  pub raw: String,
  pub text: String,
  pub level: Option<u8>,
  pub language: Option<String>,
  pub checked: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownHeading {
  pub level: u8,
  pub title: String,
  pub slug: String,
  pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownLink {
  pub label: String,
  pub url: String,
  pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownImage {
  pub alt: String,
  pub url: String,
  pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownTask {
  pub text: String,
  pub checked: bool,
  pub line: usize,
}

impl MarkdownBlock {
  pub fn new(kind: impl Into<String>, raw: impl Into<String>, text: impl Into<String>, line_level: Option<u8>) -> Self {
    Self {
      kind: kind.into(),
      raw: raw.into(),
      text: text.into(),
      level: line_level,
      language: None,
      checked: None,
    }
  }
}
