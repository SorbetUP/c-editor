use serde::{Deserialize, Serialize};

use super::{BlockKind, InlineKind, InlineSyntax, NodeId, SourceRange};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "layer", content = "value", rename_all = "snake_case")]
pub enum NodeKind {
  Document,
  Block(BlockKind),
  Inline(InlineKind),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
  pub id: NodeId,
  pub parent: Option<NodeId>,
  pub children: Vec<NodeId>,
  pub kind: NodeKind,
  pub source: Option<SourceRange>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub inline_syntax: Option<InlineSyntax>,
}

impl Node {
  pub fn new(id: NodeId, kind: NodeKind, source: Option<SourceRange>) -> Self {
    Self {
      id,
      parent: None,
      children: Vec::new(),
      kind,
      source,
      inline_syntax: None,
    }
  }

  pub fn with_inline_syntax(mut self, syntax: InlineSyntax) -> Self {
    self.inline_syntax = Some(syntax);
    self
  }
}
