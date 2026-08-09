use crate::edit::{EditError, Operation, Transaction};
use crate::model::{Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::image_insert::normalize_source;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageCommand {
  Replace {
    image: NodeId,
    source: String,
    alt: String,
    title: Option<String>,
  },
  Delete {
    image: NodeId,
  },
}

impl ImageCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    match self {
      Self::Replace {
        image,
        source,
        alt,
        title,
      } => replace_image(document, selection, image, source, alt, title),
      Self::Delete { image } => delete_image(document, selection, image),
    }
  }
}

fn replace_image(
  document: &Document,
  selection: Selection,
  image: NodeId,
  source: String,
  alt: String,
  title: Option<String>,
) -> Result<Transaction, EditError> {
  image_kind(document, image)?;
  let (parent, index) = location(document, image)?;
  let replacement = Node::new(
    image,
    NodeKind::Inline(InlineKind::Image {
      source: normalize_source(&source),
      title: title.filter(|value| !value.is_empty()),
      alt,
    }),
    None,
  );
  Ok(Transaction {
    operations: vec![
      Operation::RemoveNode { node: image },
      Operation::InsertNode {
        parent,
        index,
        node: replacement,
      },
    ],
    selection_before: selection,
    selection_after: selection,
  })
}

fn delete_image(
  document: &Document,
  selection: Selection,
  image: NodeId,
) -> Result<Transaction, EditError> {
  image_kind(document, image)?;
  let (parent, index) = location(document, image)?;
  let siblings = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?
    .children
    .clone();

  if let Some(point) = preceding_text(document, &siblings[..index]) {
    return Ok(remove_with_selection(selection, image, point));
  }
  if let Some(point) = following_text(document, &siblings[index + 1..]) {
    return Ok(remove_with_selection(selection, image, point));
  }

  let text = document.next_available_id();
  Ok(Transaction {
    operations: vec![
      Operation::RemoveNode { node: image },
      Operation::InsertNode {
        parent,
        index,
        node: Node::new(
          text,
          NodeKind::Inline(InlineKind::Text {
            value: String::new(),
          }),
          None,
        ),
      },
    ],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    }),
  })
}

fn image_kind(document: &Document, image: NodeId) -> Result<(), EditError> {
  match document.node(image).map(|node| &node.kind) {
    Some(NodeKind::Inline(InlineKind::Image { .. })) => Ok(()),
    Some(_) => Err(EditError::UnsupportedStructure(image)),
    None => Err(EditError::NodeNotFound(image)),
  }
}

fn location(document: &Document, image: NodeId) -> Result<(NodeId, usize), EditError> {
  let parent = document
    .node(image)
    .and_then(|node| node.parent)
    .ok_or(EditError::UnsupportedStructure(image))?;
  let index = document
    .child_index(parent, image)
    .ok_or(EditError::UnsupportedStructure(image))?;
  Ok((parent, index))
}

fn preceding_text(document: &Document, siblings: &[NodeId]) -> Option<SelectionPoint> {
  siblings.iter().rev().find_map(|node| {
    let NodeKind::Inline(InlineKind::Text { value }) = &document.node(*node)?.kind else {
      return None;
    };
    Some(SelectionPoint {
      node: *node,
      offset_utf16: value.encode_utf16().count() as u32,
    })
  })
}

fn following_text(document: &Document, siblings: &[NodeId]) -> Option<SelectionPoint> {
  siblings.iter().find_map(|node| {
    matches!(
      document.node(*node).map(|node| &node.kind),
      Some(NodeKind::Inline(InlineKind::Text { .. }))
    )
    .then_some(SelectionPoint {
      node: *node,
      offset_utf16: 0,
    })
  })
}

fn remove_with_selection(
  selection: Selection,
  image: NodeId,
  point: SelectionPoint,
) -> Transaction {
  Transaction {
    operations: vec![Operation::RemoveNode { node: image }],
    selection_before: selection,
    selection_after: Selection::collapsed(point),
  }
}
