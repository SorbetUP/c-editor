use crate::edit::{EditError, Operation, Transaction};
use crate::model::{Alignment, BlockKind, Document, InlineKind, Node, NodeId, NodeKind};
use crate::selection::{Selection, SelectionPoint};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableCommand {
  InsertRowAfter,
  DeleteRow,
  InsertColumnAfter,
  DeleteColumn,
}

#[derive(Clone, Copy, Debug)]
struct TableContext {
  table: NodeId,
  row: NodeId,
  cell: NodeId,
  row_index: usize,
  column_index: usize,
  row_count: usize,
  column_count: usize,
}

impl TableCommand {
  pub fn build(self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    let context = table_context(document, selection)?;
    match self {
      Self::InsertRowAfter => build_insert_row_after(document, selection, context),
      Self::DeleteRow => build_delete_row(document, selection, context),
      Self::InsertColumnAfter => build_insert_column_after(document, selection, context),
      Self::DeleteColumn => build_delete_column(document, selection, context),
    }
  }
}

fn build_insert_row_after(
  document: &Document,
  selection: Selection,
  context: TableContext,
) -> Result<Transaction, EditError> {
  let alignments = table_alignments(document, context.table, context.column_count)?;
  let mut next_id = document.next_available_id().0;
  let row = take_id(&mut next_id);
  let mut operations = vec![Operation::InsertNode {
    parent: context.table,
    index: context.row_index + 1,
    node: Node::new(row, NodeKind::Block(BlockKind::TableRow), None),
  }];
  let mut first_text = None;

  for (column, alignment) in alignments.into_iter().enumerate() {
    let cell = take_id(&mut next_id);
    let text = take_id(&mut next_id);
    first_text.get_or_insert(text);
    operations.extend([
      Operation::InsertNode {
        parent: row,
        index: column,
        node: Node::new(
          cell,
          NodeKind::Block(BlockKind::TableCell {
            alignment,
            header: false,
          }),
          None,
        ),
      },
      Operation::InsertNode {
        parent: cell,
        index: 0,
        node: Node::new(
          text,
          NodeKind::Inline(InlineKind::Text {
            value: String::new(),
          }),
          None,
        ),
      },
    ]);
  }

  let caret = SelectionPoint {
    node: first_text.ok_or(EditError::UnsupportedStructure(context.table))?,
    offset_utf16: 0,
  };
  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(caret),
  })
}

fn build_delete_row(
  document: &Document,
  selection: Selection,
  context: TableContext,
) -> Result<Transaction, EditError> {
  if context.row_index == 0 {
    return Err(EditError::UnsupportedStructure(context.row));
  }

  let target_row_index = if context.row_index + 1 < context.row_count {
    context.row_index + 1
  } else {
    context.row_index - 1
  };
  let target_row = document
    .node(context.table)
    .and_then(|table| table.children.get(target_row_index))
    .copied()
    .ok_or(EditError::UnsupportedStructure(context.table))?;
  let target_cell = document
    .node(target_row)
    .and_then(|row| {
      row
        .children
        .get(context.column_index.min(context.column_count - 1))
    })
    .copied()
    .ok_or(EditError::UnsupportedStructure(target_row))?;
  let target_text = first_text_descendant(document, target_cell)?;

  Ok(Transaction {
    operations: vec![Operation::RemoveNode { node: context.row }],
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: target_text,
      offset_utf16: 0,
    }),
  })
}

fn build_insert_column_after(
  document: &Document,
  selection: Selection,
  context: TableContext,
) -> Result<Transaction, EditError> {
  let insert_index = context.column_index + 1;
  let rows = document
    .node(context.table)
    .ok_or(EditError::NodeNotFound(context.table))?
    .children
    .clone();
  let mut next_id = document.next_available_id().0;
  let mut operations = Vec::with_capacity(rows.len() * 2);
  let mut selected_text = None;

  for (row_index, row) in rows.into_iter().enumerate() {
    let cell = take_id(&mut next_id);
    let text = take_id(&mut next_id);
    if row_index == context.row_index {
      selected_text = Some(text);
    }
    operations.extend([
      Operation::InsertNode {
        parent: row,
        index: insert_index,
        node: Node::new(
          cell,
          NodeKind::Block(BlockKind::TableCell {
            alignment: Alignment::Default,
            header: row_index == 0,
          }),
          None,
        ),
      },
      Operation::InsertNode {
        parent: cell,
        index: 0,
        node: Node::new(
          text,
          NodeKind::Inline(InlineKind::Text {
            value: String::new(),
          }),
          None,
        ),
      },
    ]);
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: selected_text.ok_or(EditError::UnsupportedStructure(context.table))?,
      offset_utf16: 0,
    }),
  })
}

fn build_delete_column(
  document: &Document,
  selection: Selection,
  context: TableContext,
) -> Result<Transaction, EditError> {
  if context.column_count <= 1 {
    return Err(EditError::UnsupportedStructure(context.table));
  }

  let target_column = if context.column_index + 1 < context.column_count {
    context.column_index + 1
  } else {
    context.column_index - 1
  };
  let rows = document
    .node(context.table)
    .ok_or(EditError::NodeNotFound(context.table))?
    .children
    .clone();
  let selected_row = rows
    .get(context.row_index)
    .copied()
    .ok_or(EditError::UnsupportedStructure(context.row))?;
  let selected_target_cell = document
    .node(selected_row)
    .and_then(|row| row.children.get(target_column))
    .copied()
    .ok_or(EditError::UnsupportedStructure(selected_row))?;
  let target_text = first_text_descendant(document, selected_target_cell)?;

  let mut operations = Vec::with_capacity(rows.len());
  for row in rows {
    let cell = document
      .node(row)
      .and_then(|node| node.children.get(context.column_index))
      .copied()
      .ok_or(EditError::UnsupportedStructure(row))?;
    operations.push(Operation::RemoveNode { node: cell });
  }

  Ok(Transaction {
    operations,
    selection_before: selection,
    selection_after: Selection::collapsed(SelectionPoint {
      node: target_text,
      offset_utf16: 0,
    }),
  })
}

fn table_context(document: &Document, selection: Selection) -> Result<TableContext, EditError> {
  let caret = selection.caret().ok_or(EditError::NonCollapsedSelection)?;
  let cell = ancestor_matching(document, caret.node, |kind| {
    matches!(kind, NodeKind::Block(BlockKind::TableCell { .. }))
  })?;
  let row = document
    .node(cell)
    .and_then(|node| node.parent)
    .ok_or(EditError::UnsupportedStructure(cell))?;
  if !matches!(
    document.node(row).map(|node| &node.kind),
    Some(NodeKind::Block(BlockKind::TableRow))
  ) {
    return Err(EditError::UnsupportedStructure(row));
  }
  let table = document
    .node(row)
    .and_then(|node| node.parent)
    .ok_or(EditError::UnsupportedStructure(row))?;
  if !matches!(
    document.node(table).map(|node| &node.kind),
    Some(NodeKind::Block(BlockKind::Table))
  ) {
    return Err(EditError::UnsupportedStructure(table));
  }

  let table_node = document.node(table).ok_or(EditError::NodeNotFound(table))?;
  let row_index = document
    .child_index(table, row)
    .ok_or(EditError::UnsupportedStructure(row))?;
  let row_node = document.node(row).ok_or(EditError::NodeNotFound(row))?;
  let column_index = document
    .child_index(row, cell)
    .ok_or(EditError::UnsupportedStructure(cell))?;
  let column_count = row_node.children.len();
  if column_count == 0
    || table_node.children.iter().any(|candidate| {
      document
        .node(*candidate)
        .is_none_or(|candidate| candidate.children.len() != column_count)
    })
  {
    return Err(EditError::UnsupportedStructure(table));
  }

  Ok(TableContext {
    table,
    row,
    cell,
    row_index,
    column_index,
    row_count: table_node.children.len(),
    column_count,
  })
}

fn table_alignments(
  document: &Document,
  table: NodeId,
  column_count: usize,
) -> Result<Vec<Alignment>, EditError> {
  let header = document
    .node(table)
    .and_then(|table| table.children.first())
    .copied()
    .ok_or(EditError::UnsupportedStructure(table))?;
  let header_node = document
    .node(header)
    .ok_or(EditError::NodeNotFound(header))?;
  if header_node.children.len() != column_count {
    return Err(EditError::UnsupportedStructure(header));
  }
  header_node
    .children
    .iter()
    .map(|cell| match document.node(*cell).map(|node| &node.kind) {
      Some(NodeKind::Block(BlockKind::TableCell { alignment, .. })) => Ok(*alignment),
      _ => Err(EditError::UnsupportedStructure(*cell)),
    })
    .collect()
}

fn ancestor_matching(
  document: &Document,
  start: NodeId,
  predicate: impl Fn(&NodeKind) -> bool,
) -> Result<NodeId, EditError> {
  let mut current = start;
  loop {
    let node = document
      .node(current)
      .ok_or(EditError::NodeNotFound(current))?;
    if predicate(&node.kind) {
      return Ok(current);
    }
    current = node.parent.ok_or(EditError::UnsupportedStructure(start))?;
  }
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

fn take_id(next_id: &mut u64) -> NodeId {
  let id = NodeId(*next_id);
  *next_id = next_id.saturating_add(1);
  id
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{parse_markdown, to_markdown};

  fn cell_text(document: &Document, table: NodeId, row: usize, column: usize) -> NodeId {
    let row = document.children(table).nth(row).unwrap().id;
    let cell = document.children(row).nth(column).unwrap().id;
    first_text_descendant(document, cell).unwrap()
  }

  #[test]
  fn inserts_and_undoes_a_body_row() {
    let mut document = parse_markdown("| A | B |\n| :--- | ---: |\n| one | two |");
    let table = document.children(document.root).next().unwrap().id;
    let text = cell_text(&document, table, 1, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = TableCommand::InsertRowAfter
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert_eq!(document.children(table).count(), 3);
    assert_eq!(
      to_markdown(&document),
      "| A   | B   |\n|:--- | ---:|\n| one | two |\n|     |     |"
    );

    inverse.apply(&mut document).unwrap();
    assert_eq!(document.children(table).count(), 2);
  }

  #[test]
  fn deletes_and_restores_a_body_row() {
    let mut document = parse_markdown("| A | B |\n| --- | --- |\n| one | two |\n| three | four |");
    let table = document.children(document.root).next().unwrap().id;
    let row = document.children(table).nth(1).unwrap().id;
    let text = cell_text(&document, table, 1, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = TableCommand::DeleteRow
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert!(document.node(row).is_none());
    assert_eq!(document.children(table).count(), 2);
    assert_eq!(
      to_markdown(&document),
      "| A     | B    |\n| ----- | ---- |\n| three | four |"
    );

    inverse.apply(&mut document).unwrap();
    assert_eq!(document.children(table).nth(1).unwrap().id, row);
  }

  #[test]
  fn inserts_a_column_across_all_rows_with_default_alignment() {
    let mut document = parse_markdown("| A | B |\n| :--- | ---: |\n| one | two |");
    let table = document.children(document.root).next().unwrap().id;
    let text = cell_text(&document, table, 1, 0);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = TableCommand::InsertColumnAfter
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    for row in document.children(table) {
      assert_eq!(document.children(row.id).count(), 3);
    }
    let header = document.children(table).next().unwrap().id;
    let inserted_header = document.children(header).nth(1).unwrap();
    assert!(matches!(
      inserted_header.kind,
      NodeKind::Block(BlockKind::TableCell {
        alignment: Alignment::Default,
        header: true
      })
    ));
    assert_eq!(
      to_markdown(&document),
      "| A   |     | B   |\n|:--- | --- | ---:|\n| one |     | two |"
    );

    inverse.apply(&mut document).unwrap();
    for row in document.children(table) {
      assert_eq!(document.children(row.id).count(), 2);
    }
  }

  #[test]
  fn deletes_and_restores_a_column() {
    let mut document = parse_markdown("| A | B | C |\n| --- | --- | --- |\n| one | two | three |");
    let table = document.children(document.root).next().unwrap().id;
    let removed_header = {
      let header = document.children(table).next().unwrap().id;
      document.children(header).nth(1).unwrap().id
    };
    let text = cell_text(&document, table, 1, 1);
    let selection = Selection::collapsed(SelectionPoint {
      node: text,
      offset_utf16: 0,
    });

    let inverse = TableCommand::DeleteColumn
      .build(&document, selection)
      .unwrap()
      .apply(&mut document)
      .unwrap();
    assert!(document.node(removed_header).is_none());
    assert_eq!(
      to_markdown(&document),
      "| A   | C     |\n| --- | ----- |\n| one | three |"
    );

    inverse.apply(&mut document).unwrap();
    assert_eq!(
      document.node(removed_header).unwrap().parent.is_some(),
      true
    );
    for row in document.children(table) {
      assert_eq!(document.children(row.id).count(), 3);
    }
  }
}
