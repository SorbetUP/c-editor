pub mod code_span;
pub mod emphasis;
pub mod escape;
pub mod line_break;
pub mod link;
pub mod text;

use super::{FeatureSpec, SyntaxLayer};

pub static FEATURES: &[FeatureSpec] = &[
  FeatureSpec {
    id: "escape",
    layer: SyntaxLayer::Inline,
    precedence: 10,
    fallback: false,
  },
  FeatureSpec {
    id: "code_span",
    layer: SyntaxLayer::Inline,
    precedence: 20,
    fallback: false,
  },
  FeatureSpec {
    id: "image",
    layer: SyntaxLayer::Inline,
    precedence: 30,
    fallback: false,
  },
  FeatureSpec {
    id: "link",
    layer: SyntaxLayer::Inline,
    precedence: 40,
    fallback: false,
  },
  FeatureSpec {
    id: "reference_image",
    layer: SyntaxLayer::Inline,
    precedence: 50,
    fallback: false,
  },
  FeatureSpec {
    id: "reference_link",
    layer: SyntaxLayer::Inline,
    precedence: 60,
    fallback: false,
  },
  FeatureSpec {
    id: "autolink",
    layer: SyntaxLayer::Inline,
    precedence: 70,
    fallback: false,
  },
  FeatureSpec {
    id: "strong",
    layer: SyntaxLayer::Inline,
    precedence: 80,
    fallback: false,
  },
  FeatureSpec {
    id: "emphasis",
    layer: SyntaxLayer::Inline,
    precedence: 90,
    fallback: false,
  },
  FeatureSpec {
    id: "strikethrough",
    layer: SyntaxLayer::Inline,
    precedence: 100,
    fallback: false,
  },
  FeatureSpec {
    id: "inline_math",
    layer: SyntaxLayer::Inline,
    precedence: 110,
    fallback: false,
  },
  FeatureSpec {
    id: "inline_html",
    layer: SyntaxLayer::Inline,
    precedence: 120,
    fallback: false,
  },
  FeatureSpec {
    id: "emoji",
    layer: SyntaxLayer::Inline,
    precedence: 130,
    fallback: false,
  },
  FeatureSpec {
    id: "superscript",
    layer: SyntaxLayer::Inline,
    precedence: 140,
    fallback: false,
  },
  FeatureSpec {
    id: "subscript",
    layer: SyntaxLayer::Inline,
    precedence: 150,
    fallback: false,
  },
  FeatureSpec {
    id: "footnote_reference",
    layer: SyntaxLayer::Inline,
    precedence: 160,
    fallback: false,
  },
  FeatureSpec {
    id: "hard_break",
    layer: SyntaxLayer::Inline,
    precedence: 170,
    fallback: false,
  },
  FeatureSpec {
    id: "soft_break",
    layer: SyntaxLayer::Inline,
    precedence: 180,
    fallback: false,
  },
  FeatureSpec {
    id: "text",
    layer: SyntaxLayer::Inline,
    precedence: u16::MAX,
    fallback: true,
  },
];
