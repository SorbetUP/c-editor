use crate::edit::{EditError, Operation, Transaction, Utf16Range};
use crate::model::{BlockKind, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsertImage {
  pub source: String,
  pub alt: String,
  pub title: Option<String>,
}

impl InsertImage {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let (text, start, end, use_selected_text) = match selection.ordered_same_node() {
      Some((text, start, end)) => (text, start, end, true),
      None => (
        selection.focus.node,
        selection.focus.offset_utf16,
        selection.focus.offset_utf16,
        false,
      ),
    };
    if forbidden_context(document, text) {
      return Ok(noop(selection));
    }
    let value = text_value(document, text)?;
    let start_byte = utf16_to_byte(value, text, start)?;
    let end_byte = utf16_to_byte(value, text, end)?;
    let selected = use_selected_text.then_some(&value[start_byte..end_byte]);
    let alt = match selected.filter(|value| !value.is_empty()) {
      Some(value) => value.to_string(),
      None if self.alt.is_empty() => infer_alt(&self.source),
      None => self.alt,
    };
    let source = normalize_source(&self.source);
    let title = self.title.filter(|value| !value.is_empty());
    if matches!(
      document.node(text).map(|node| &node.kind),
      Some(NodeKind::Inline(InlineKind::CodeSpan { .. }))
    ) {
      let inserted = image_markdown(&source, &alt, title.as_deref());
      let alt_start = start + 2;
      return Ok(Transaction {
        operations: vec![Operation::ReplaceText {
          node: text,
          range: Utf16Range::new(start, end),
          inserted,
        }],
        selection_before: selection,
        selection_after: Selection {
          anchor: SelectionPoint {
            node: text,
            offset_utf16: alt_start,
          },
          focus: SelectionPoint {
            node: text,
            offset_utf16: alt_start + alt.encode_utf16().count() as u32,
          },
        },
      });
    }
    let parent = document
      .node(text)
      .and_then(|node| node.parent)
      .ok_or(EditError::UnsupportedStructure(text))?;
    let index = document
      .child_index(parent, text)
      .ok_or(EditError::UnsupportedStructure(text))?;
    let suffix = value[end_byte..].to_string();
    let full_length = value.encode_utf16().count() as u32;
    let image = document.next_available_id();
    let tail = NodeId(image.0.saturating_add(1));

    Ok(Transaction {
      operations: vec![
        Operation::ReplaceText {
          node: text,
          range: Utf16Range::new(start, full_length),
          inserted: String::new(),
        },
        Operation::InsertNode {
          parent,
          index: index + 1,
          node: Node::new(
            image,
            NodeKind::Inline(InlineKind::Image { source, title, alt }),
            None,
          ),
        },
        Operation::InsertNode {
          parent,
          index: index + 2,
          node: Node::new(
            tail,
            NodeKind::Inline(InlineKind::Text { value: suffix }),
            None,
          ),
        },
      ],
      selection_before: selection,
      selection_after: Selection::collapsed(SelectionPoint {
        node: tail,
        offset_utf16: 0,
      }),
    })
  }
}

fn text_value(document: &Document, node: NodeId) -> Result<&str, EditError> {
  match document.node(node).map(|node| &node.kind) {
    Some(NodeKind::Inline(InlineKind::Text { value })) => Ok(value),
    Some(NodeKind::Inline(InlineKind::CodeSpan { code })) => Ok(code),
    Some(_) => Err(EditError::NotTextNode(node)),
    None => Err(EditError::NodeNotFound(node)),
  }
}

fn image_markdown(source: &str, alt: &str, title: Option<&str>) -> String {
  match title {
    Some(title) => format!(r#"![{alt}]({source} "{title}")"#),
    None => format!("![{alt}]({source})"),
  }
}

fn forbidden_context(document: &Document, mut node: NodeId) -> bool {
  while let Some(current) = document.node(node) {
    if matches!(
      current.kind,
      NodeKind::Block(BlockKind::CodeBlock { .. } | BlockKind::ThematicBreak)
    ) {
      return true;
    }
    let Some(parent) = current.parent else {
      break;
    };
    node = parent;
  }
  false
}

fn infer_alt(source: &str) -> String {
  let filename = source
    .rsplit(|character| character == '/' || character == '\\')
    .next()
    .unwrap_or(source);
  let Some((stem, extension)) = filename.rsplit_once('.') else {
    return String::new();
  };
  if stem.is_empty()
    || extension.is_empty()
    || !extension
      .chars()
      .all(|character| character.is_ascii_lowercase())
  {
    return String::new();
  }
  stem.to_string()
}

pub(super) fn normalize_source(source: &str) -> String {
  if source.starts_with("data:") {
    return source.to_string();
  }
  let encoded_spaces = source.replace(' ', "%20");
  if source.contains("://") {
    encoded_spaces
  } else {
    encoded_spaces.replace('#', "%23")
  }
}

fn utf16_to_byte(value: &str, node: NodeId, target: u32) -> Result<usize, EditError> {
  let mut utf16 = 0u32;
  for (byte, character) in value.char_indices() {
    if utf16 == target {
      return Ok(byte);
    }
    utf16 = utf16.saturating_add(character.len_utf16() as u32);
    if utf16 > target {
      return Err(EditError::InvalidUtf16Boundary {
        node,
        offset: target,
      });
    }
  }
  if utf16 == target {
    Ok(value.len())
  } else {
    Err(EditError::RangeOutOfBounds {
      node,
      start: target,
      end: target,
    })
  }
}

fn noop(selection: Selection) -> Transaction {
  Transaction {
    operations: Vec::new(),
    selection_before: selection,
    selection_after: selection,
  }
}
