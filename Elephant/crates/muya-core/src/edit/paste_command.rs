use crate::model::{BlockKind, Document, InlineKind, NodeId, NodeKind};
use crate::selection::Selection;

use super::{
  paste::build_paste_markdown, paste_nested::build_nested_paste,
  paste_nested_structured::build_nested_structured_paste, Command, EditError, Transaction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InlineWrapper {
  Emphasis,
  Strong,
  Strike,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasteCommand {
  markdown: String,
}

impl PasteCommand {
  pub fn new(markdown: impl Into<String>) -> Self {
    Self {
      markdown: markdown.into(),
    }
  }

  pub fn build(&self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    if let Some(transaction) = build_matching_inline_paste(document, selection, &self.markdown)? {
      return Ok(transaction);
    }
    if let Some(transaction) = build_nested_structured_paste(document, selection, &self.markdown)? {
      return Ok(transaction);
    }
    if let Some(transaction) = build_nested_paste(document, selection, &self.markdown)? {
      return Ok(transaction);
    }
    build_paste_markdown(document, selection, &self.markdown)
  }
}

fn build_matching_inline_paste(
  document: &Document,
  selection: Selection,
  markdown: &str,
) -> Result<Option<Transaction>, EditError> {
  let Some((text, _, _)) = selection.ordered_same_node() else {
    return Ok(None);
  };
  let Some(target_wrappers) = target_wrapper_stack(document, text) else {
    return Ok(None);
  };
  if target_wrappers.is_empty() {
    return Ok(None);
  }

  let fragment = crate::parse_markdown(markdown);
  let Some((pasted_wrappers, pasted_text)) = single_wrapped_text(&fragment) else {
    return Ok(None);
  };
  if target_wrappers != pasted_wrappers {
    return Ok(None);
  }

  Command::InsertText(pasted_text)
    .build(document, selection)
    .map(Some)
}

fn target_wrapper_stack(document: &Document, text: NodeId) -> Option<Vec<InlineWrapper>> {
  if !matches!(
    document.node(text)?.kind,
    NodeKind::Inline(InlineKind::Text { .. })
  ) {
    return None;
  }
  let mut current = text;
  let mut wrappers = Vec::new();
  loop {
    let parent = document.node(current)?.parent?;
    let parent_node = document.node(parent)?;
    match &parent_node.kind {
      NodeKind::Inline(kind) => {
        let wrapper = wrapper_kind(kind)?;
        if parent_node.children.len() != 1 {
          return None;
        }
        wrappers.push(wrapper);
        current = parent;
      }
      NodeKind::Block(BlockKind::Paragraph) => {
        if parent_node.parent != Some(document.root) {
          return None;
        }
        wrappers.reverse();
        return Some(wrappers);
      }
      _ => return None,
    }
  }
}

fn single_wrapped_text(document: &Document) -> Option<(Vec<InlineWrapper>, String)> {
  let blocks = document.children(document.root).collect::<Vec<_>>();
  if blocks.len() != 1 || !matches!(blocks[0].kind, NodeKind::Block(BlockKind::Paragraph)) {
    return None;
  }
  let mut current = blocks[0].id;
  let mut wrappers = Vec::new();
  loop {
    let node = document.node(current)?;
    if node.children.len() != 1 {
      return None;
    }
    let child = document.node(node.children[0])?;
    match &child.kind {
      NodeKind::Inline(InlineKind::Text { value }) => {
        return Some((wrappers, value.clone()));
      }
      NodeKind::Inline(kind) => {
        wrappers.push(wrapper_kind(kind)?);
        current = child.id;
      }
      _ => return None,
    }
  }
}

fn wrapper_kind(kind: &InlineKind) -> Option<InlineWrapper> {
  match kind {
    InlineKind::Emphasis => Some(InlineWrapper::Emphasis),
    InlineKind::Strong => Some(InlineWrapper::Strong),
    InlineKind::Strike => Some(InlineWrapper::Strike),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::selection::SelectionPoint;

  fn text_node(document: &Document, root: NodeId) -> Option<NodeId> {
    let node = document.node(root)?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
      return Some(root);
    }
    node
      .children
      .iter()
      .find_map(|child| text_node(document, *child))
  }

  #[test]
  fn pastes_matching_markdown_inside_an_existing_inline_mark() {
    let mut document = crate::parse_markdown("**pasted**");
    let text = text_node(&document, document.root).expect("strong text node");
    let length = match &document.node(text).unwrap().kind {
      NodeKind::Inline(InlineKind::Text { value }) => value.encode_utf16().count() as u32,
      _ => unreachable!(),
    };
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: length,
    });

    let transaction = PasteCommand::new("**pasted**")
      .build(&document, selection)
      .expect("paste inside matching strong mark");
    transaction.apply(&mut document).expect("apply paste");

    assert_eq!(crate::to_markdown(&document), "**pastedpasted**");
  }
}
