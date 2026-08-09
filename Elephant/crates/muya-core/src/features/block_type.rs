use crate::edit::{EditError, Operation, Transaction};
use crate::model::{BlockKind, Document, ListKind, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::block_type_list::set_list_kind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockTypeCommand {
  ToggleBlockQuote,
  ToggleCodeBlock,
  SetListKind(ListKind),
}

impl BlockTypeCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    match self {
      Self::ToggleBlockQuote => toggle_simple(
        document,
        selection,
        BlockKind::BlockQuote,
        BlockKind::Paragraph,
      ),
      Self::ToggleCodeBlock => toggle_code(document, selection),
      Self::SetListKind(kind) => set_list_kind(document, selection, kind),
    }
  }
}

fn toggle_simple(
  document: &Document,
  selection: Selection,
  target: BlockKind,
  fallback: BlockKind,
) -> Result<Transaction, EditError> {
  let block = text_block(document, selection.focus.node)?;
  let current = &document
    .node(block)
    .ok_or(EditError::NodeNotFound(block))?
    .kind;
  let kind = if current == &NodeKind::Block(target.clone()) {
    fallback
  } else if matches!(current, NodeKind::Block(BlockKind::Paragraph)) {
    target
  } else {
    return Err(EditError::UnsupportedStructure(block));
  };
  set_kind(selection, block, kind, selection)
}

fn toggle_code(document: &Document, selection: Selection) -> Result<Transaction, EditError> {
  let block = text_block(document, selection.focus.node)?;
  let node = document.node(block).ok_or(EditError::NodeNotFound(block))?;
  let (kind, offset) = match node.kind {
    NodeKind::Block(BlockKind::CodeBlock { .. }) => (
      BlockKind::Paragraph,
      text_length(document, selection.focus.node),
    ),
    NodeKind::Block(BlockKind::Paragraph) => (
      BlockKind::CodeBlock {
        language: None,
        fenced: true,
      },
      0,
    ),
    _ => return Err(EditError::UnsupportedStructure(block)),
  };
  let after = Selection::collapsed(SelectionPoint {
    node: selection.focus.node,
    offset_utf16: offset,
  });
  set_kind(selection, block, kind, after)
}

fn set_kind(
  before: Selection,
  block: NodeId,
  kind: BlockKind,
  after: Selection,
) -> Result<Transaction, EditError> {
  Ok(Transaction {
    operations: vec![Operation::SetBlockKind { node: block, kind }],
    selection_before: before,
    selection_after: after,
  })
}

pub(super) fn text_block(document: &Document, mut node: NodeId) -> Result<NodeId, EditError> {
  loop {
    let current = document.node(node).ok_or(EditError::NodeNotFound(node))?;
    if matches!(
      current.kind,
      NodeKind::Block(BlockKind::Paragraph | BlockKind::BlockQuote | BlockKind::CodeBlock { .. })
    ) {
      return Ok(node);
    }
    node = current
      .parent
      .ok_or(EditError::UnsupportedStructure(node))?;
  }
}

fn text_length(document: &Document, node: NodeId) -> u32 {
  match document.node(node).map(|node| &node.kind) {
    Some(NodeKind::Inline(crate::model::InlineKind::Text { value })) => {
      value.encode_utf16().count() as u32
    }
    _ => 0,
  }
}
