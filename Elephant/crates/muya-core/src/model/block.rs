use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
  Default,
  Left,
  Center,
  Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListKind {
  Unordered,
  Ordered,
  Task,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontMatterStyle {
  Yaml,
  Toml,
  Json,
  Semicolon,
}

impl FrontMatterStyle {
  pub fn delimiters(self) -> (&'static str, &'static str) {
    match self {
      Self::Yaml => ("---", "---"),
      Self::Toml => ("+++", "+++"),
      Self::Json => ("{", "}"),
      Self::Semicolon => (";;;", ";;;"),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockKind {
  Paragraph,
  Heading {
    level: u8,
  },
  BlockQuote,
  List {
    kind: ListKind,
    start: Option<u64>,
  },
  ListItem {
    checked: Option<bool>,
  },
  Table,
  TableRow,
  TableCell {
    alignment: Alignment,
    header: bool,
  },
  CodeBlock {
    language: Option<String>,
    fenced: bool,
  },
  ThematicBreak,
  HtmlBlock,
  MathBlock,
  FrontMatter {
    style: FrontMatterStyle,
  },
  FootnoteDefinition {
    label: String,
  },
  ReferenceDefinition {
    label: String,
  },
  Diagram {
    language: String,
  },
}
