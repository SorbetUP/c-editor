use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

use super::operation::utf16_to_byte;
use super::{EditError, Operation, Transaction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParagraphBoundaryCommand {
  InsertParagraph,
}

#[derive(Clone, Copy)]
enum Destination {
  Document {
    parent: NodeId,
    paragraph_index: usize,
  },
  List {
    list: NodeId,
    item_index: usize,
    checked: Option<bool>,
  },
}

struct BoundaryPath {
  paragraph: NodeId,
  top_wrapper: NodeId,
  top_index: usize,
  destination: Destination,
}

impl ParagraphBoundaryCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    match self {
      Self::InsertParagraph => build_insert_paragraph(document, selection),
    }
  }
}

fn build_insert_paragraph(
  document: &Document,
  selection: Selection,
) -> Result<Transaction, EditError> {
  let caret = selection.caret().ok_or(EditError::NonCollapsedSelection)?;
  let value = text_value(document, caret.node)?;
  utf16_to_byte(value, caret.node, caret.offset_utf16)?;
  let length = value.encode_utf16().count() as u32;
  if caret.offset_utf16 != 0 && caret.offset_utf16 != length {
    return Err(EditError::UnsupportedStructure(caret.node));
  }
  let path = boundary_path(document, caret.node)?;
  if caret.offset_utf16 == 0 {
    build_at_start(document, selection, caret, path)
  } else {
    build_at_end(document, selection, path)
  }
}

fn build_at_start(
  document: &Document,
  selection: Selection,
  caret: SelectionPoint,
  path: BoundaryPath,
) -> Result<Transaction, EditError> {
  let paragraph_node = document
    .node(path.paragraph)
    .ok_or(EditError::NodeNotFound(path.paragraph))?;
  let moving = paragraph_node.children[path.top_index..].to_vec();
  let mut next_id = document.next_available_id().0;
  let new_item =
    matches!(path.destination, Destination::List { .. }).then(|| take_id(&mut next_id));
  let new_paragraph = take_id(&mut next_id);
  let empty_text = (path.top_index == 0).then(|| take_id(&mut next_id));
  let mut operations = insert_destination(path.destination, new_item, new_paragraph);
  operations.extend(
    moving
      .into_iter()
      .enumerate()
      .map(|(index, node)| Operation::MoveNode {
        node,
        new_parent: new_paragraph,
        new_index: index,
      }),
  );
  if let Some(empty_text) = empty_text {
    operations.push(Operation::InsertNode {
      parent: path.paragraph,
      index: 0,
      node: text_node(empty_text, String::new()),
    });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

fn build_at_end(
  document: &Document,
  selection: Selection,
  path: BoundaryPath,
) -> Result<Transaction, EditError> {
  let paragraph_node = document
    .node(path.paragraph)
    .ok_or(EditError::NodeNotFound(path.paragraph))?;
  let following = paragraph_node.children[path.top_index + 1..].to_vec();
  let mut next_id = document.next_available_id().0;
  let new_item =
    matches!(path.destination, Destination::List { .. }).then(|| take_id(&mut next_id));
  let new_paragraph = take_id(&mut next_id);
  let empty_text = following.is_empty().then(|| take_id(&mut next_id));
  let target = if let Some(first) = following.first().copied() {
    first_text_descendant(document, first)?
  } else {
    empty_text.expect("empty paragraph must allocate a text node")
  };

  let mut operations = insert_destination(path.destination, new_item, new_paragraph);
  operations.extend(
    following
      .into_iter()
      .enumerate()
      .map(|(index, node)| Operation::MoveNode {
        node,
        new_parent: new_paragraph,
        new_index: index,
      }),
  );
  if let Some(empty_text) = empty_text {
    operations.push(Operation::InsertNode {
      parent: new_paragraph,
      index: 0,
      node: text_node(empty_text, String::new()),
    });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: target,
      offset_utf16: 0,
    }),
  })
}

fn insert_destination(
  destination: Destination,
  new_item: Option<NodeId>,
  new_paragraph: NodeId,
) -> Vec<Operation> {
  match destination {
    Destination::Document {
      parent,
      paragraph_index,
    } => vec![Operation::InsertNode {
      parent,
      index: paragraph_index + 1,
      node: Node::new(new_paragraph, NodeKind::Block(BlockKind::Paragraph), None),
    }],
    Destination::List {
      list,
      item_index,
      checked,
    } => {
      let new_item = new_item.expect("list destination must allocate an item");
      vec![
        Operation::InsertNode {
          parent: list,
          index: item_index + 1,
          node: Node::new(
            new_item,
            NodeKind::Block(BlockKind::ListItem {
              checked: checked.map(|_| false),
            }),
            None,
          ),
        },
        Operation::InsertNode {
          parent: new_item,
          index: 0,
          node: Node::new(new_paragraph, NodeKind::Block(BlockKind::Paragraph), None),
        },
      ]
    }
  }
}

fn boundary_path(document: &Document, text: NodeId) -> Result<BoundaryPath, EditError> {
  text_value(document, text)?;
  let mut current = text;
  let mut top_wrapper = None;
  let paragraph = loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    let parent = node
      .parent
      .ok_or(EditError::UnsupportedStructure(current))?;
    let parent_node = document
      .node(parent)
      .ok_or(EditError::NodeNotFound(parent))?;
    match &parent_node.kind {
      NodeKind::Inline(_) => {
        if parent_node.children.as_slice() != [current] {
          return Err(EditError::UnsupportedStructure(parent));
        }
        top_wrapper = Some(parent);
        current = parent;
      }
      NodeKind::Block(BlockKind::Paragraph | BlockKind::Heading { .. } | BlockKind::BlockQuote) => {
        break parent
      }
      _ => return Err(EditError::UnsupportedStructure(parent)),
    }
  };
  let top_wrapper = top_wrapper.ok_or(EditError::UnsupportedStructure(text))?;
  let top_index = document
    .child_index(paragraph, top_wrapper)
    .ok_or(EditError::UnsupportedStructure(top_wrapper))?;
  let paragraph_node = document
    .node(paragraph)
    .ok_or(EditError::NodeNotFound(paragraph))?;
  let parent = paragraph_node
    .parent
    .ok_or(EditError::UnsupportedStructure(paragraph))?;
  let parent_node = document
    .node(parent)
    .ok_or(EditError::NodeNotFound(parent))?;
  let destination = if parent == document.root {
    Destination::Document {
      parent,
      paragraph_index: document
        .child_index(parent, paragraph)
        .ok_or(EditError::UnsupportedStructure(paragraph))?,
    }
  } else if let NodeKind::Block(BlockKind::ListItem { checked }) = &parent_node.kind {
    if parent_node.children.as_slice() != [paragraph] {
      return Err(EditError::UnsupportedStructure(parent));
    }
    let list = parent_node
      .parent
      .ok_or(EditError::UnsupportedStructure(parent))?;
    if !matches!(
      document.node(list).map(|node| &node.kind),
      Some(NodeKind::Block(BlockKind::List { .. }))
    ) {
      return Err(EditError::UnsupportedStructure(list));
    }
    Destination::List {
      list,
      item_index: document
        .child_index(list, parent)
        .ok_or(EditError::UnsupportedStructure(parent))?,
      checked: *checked,
    }
  } else {
    return Err(EditError::UnsupportedStructure(parent));
  };

  Ok(BoundaryPath {
    paragraph,
    top_wrapper,
    top_index,
    destination,
  })
}

fn first_text_descendant(document: &Document, root: NodeId) -> Result<NodeId, EditError> {
  let mut stack = vec![root];
  while let Some(current) = stack.pop() {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if matches!(node.kind, NodeKind::Inline(InlineKind::Text { .. })) {
      return Ok(current);
    }
    stack.extend(node.children.iter().rev().copied());
  }
  Err(EditError::UnsupportedStructure(root))
}

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  match &document
    .node(node)
    .ok_or(EditError::NodeNotFound(node))?
    .kind
  {
    NodeKind::Inline(InlineKind::Text { value }) => Ok(value),
    _ => Err(EditError::NotTextNode(node)),
  }
}

fn text_node(id: NodeId, value: String) -> Node {
  Node::new(id, NodeKind::Inline(InlineKind::Text { value }), None)
}

fn take_id(next_id: &mut u64) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  id
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn marked_text(document: &Document) -> NodeId {
    let block = document.children(document.root).next().unwrap().id;
    let wrapper = document.children(block).nth(1).unwrap().id;
    first_text_descendant(document, wrapper).unwrap()
  }

  #[test]
  fn splits_before_a_mark_without_leaving_an_empty_wrapper() {
    let mut document = parse_markdown("before**bold**after");
    let text = marked_text(&document);
    let inverse = ParagraphBoundaryCommand::InsertParagraph
      .build(
        &document,
        Selection::collapsed(SelectionPoint {
          node: text,
          offset_utf16: 0,
        }),
      )
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "before\n\n**bold**after");

    inverse.apply(&mut document).unwrap();
    assert_eq!(to_markdown(&document), "before**bold**after");
  }

  #[test]
  fn splits_after_a_mark_and_moves_following_content() {
    let mut document = parse_markdown("before**bold**after");
    let text = marked_text(&document);
    ParagraphBoundaryCommand::InsertParagraph
      .build(
        &document,
        Selection::collapsed(SelectionPoint {
          node: text,
          offset_utf16: 4,
        }),
      )
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "before**bold**\n\nafter");
  }

  #[test]
  fn splits_a_marked_list_item_at_its_start() {
    let mut document = parse_markdown("- before**bold**after");
    let list = document.children(document.root).next().unwrap().id;
    let item = document.children(list).next().unwrap().id;
    let paragraph = document.children(item).next().unwrap().id;
    let wrapper = document.children(paragraph).nth(1).unwrap().id;
    let text = first_text_descendant(&document, wrapper).unwrap();
    ParagraphBoundaryCommand::InsertParagraph
      .build(
        &document,
        Selection::collapsed(SelectionPoint {
          node: text,
          offset_utf16: 0,
        }),
      )
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(to_markdown(&document), "- before\n- **bold**after");
  }
}
