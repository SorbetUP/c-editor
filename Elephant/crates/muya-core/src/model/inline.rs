use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InlineMarkKind {
  Emphasis,
  Strong,
  Strike,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkFragmentEdge {
  Start,
  Middle,
  End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceStyle {
  Full,
  Collapsed,
  Shortcut,
}

/// Source-shape metadata for semantic inline nodes.
///
/// Freya and other renderers should only need to understand `InlineKind`.
/// Markdown-specific spelling that matters for round-tripping lives here so a
/// reference link remains a normal link and a bare autolink remains a normal
/// clickable link at the rendering layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineSyntax {
  Reference {
    reference: String,
    style: ReferenceStyle,
  },
  BareAutoLink {
    text: String,
  },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineKind {
  Text {
    value: String,
  },
  Escaped {
    value: char,
  },
  Emphasis,
  Strong,
  Strike,
  MarkFragment {
    mark: InlineMarkKind,
    group: u64,
    edge: MarkFragmentEdge,
  },
  CodeSpan {
    code: String,
  },
  Link {
    destination: String,
    title: Option<String>,
  },
  Image {
    source: String,
    title: Option<String>,
    alt: String,
  },
  AutoLink {
    destination: String,
  },
  InlineHtml {
    raw: String,
  },
  InlineMath {
    source: String,
  },
  Emoji {
    shortcode: String,
    value: String,
  },
  Superscript,
  Subscript,
  FootnoteReference {
    label: String,
  },
  SoftBreak,
  HardBreak,
}
