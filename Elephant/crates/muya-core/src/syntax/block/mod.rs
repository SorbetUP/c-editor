pub mod blockquote;
pub mod fenced_code;
pub mod front_matter;
pub mod heading;
pub mod list;
pub mod paragraph;
pub mod table;
pub mod thematic_break;

use super::{FeatureSpec, SyntaxLayer};

pub static FEATURES: &[FeatureSpec] = &[
  FeatureSpec {
    id: "frontmatter",
    layer: SyntaxLayer::Block,
    precedence: 10,
    fallback: false,
  },
  FeatureSpec {
    id: "fenced_code",
    layer: SyntaxLayer::Block,
    precedence: 20,
    fallback: false,
  },
  FeatureSpec {
    id: "indented_code",
    layer: SyntaxLayer::Block,
    precedence: 30,
    fallback: false,
  },
  FeatureSpec {
    id: "heading_atx",
    layer: SyntaxLayer::Block,
    precedence: 40,
    fallback: false,
  },
  FeatureSpec {
    id: "heading_setext",
    layer: SyntaxLayer::Block,
    precedence: 50,
    fallback: false,
  },
  FeatureSpec {
    id: "thematic_break",
    layer: SyntaxLayer::Block,
    precedence: 60,
    fallback: false,
  },
  FeatureSpec {
    id: "blockquote",
    layer: SyntaxLayer::Block,
    precedence: 70,
    fallback: false,
  },
  FeatureSpec {
    id: "task_list",
    layer: SyntaxLayer::Block,
    precedence: 80,
    fallback: false,
  },
  FeatureSpec {
    id: "ordered_list",
    layer: SyntaxLayer::Block,
    precedence: 90,
    fallback: false,
  },
  FeatureSpec {
    id: "unordered_list",
    layer: SyntaxLayer::Block,
    precedence: 100,
    fallback: false,
  },
  FeatureSpec {
    id: "table",
    layer: SyntaxLayer::Block,
    precedence: 110,
    fallback: false,
  },
  FeatureSpec {
    id: "html_block",
    layer: SyntaxLayer::Block,
    precedence: 120,
    fallback: false,
  },
  FeatureSpec {
    id: "math_block",
    layer: SyntaxLayer::Block,
    precedence: 130,
    fallback: false,
  },
  FeatureSpec {
    id: "diagram",
    layer: SyntaxLayer::Block,
    precedence: 140,
    fallback: false,
  },
  FeatureSpec {
    id: "footnote_definition",
    layer: SyntaxLayer::Block,
    precedence: 150,
    fallback: false,
  },
  FeatureSpec {
    id: "reference_definition",
    layer: SyntaxLayer::Block,
    precedence: 160,
    fallback: false,
  },
  FeatureSpec {
    id: "paragraph",
    layer: SyntaxLayer::Block,
    precedence: u16::MAX,
    fallback: true,
  },
];
