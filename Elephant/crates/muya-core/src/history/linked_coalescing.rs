use crate::edit::{Operation, Transaction};
use crate::model::{Document, InlineKind, InlineMarkKind, NodeId, NodeKind};
use crate::selection::SelectionPoint;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LinkedGroup {
  group: u64,
  mark: InlineMarkKind,
}

pub(super) fn should_coalesce(
  document: &Document,
  transaction: &Transaction,
  previous: Option<&Transaction>,
) -> bool {
  let Some(previous) = previous else {
    return false;
  };
  let mut existing = groups_at_point(document, transaction.selection_before.anchor);
  for group in groups_at_point(document, transaction.selection_before.focus) {
    if !existing.contains(&group) {
      existing.push(group);
    }
  }
  if existing.is_empty() {
    return false;
  }

  let inserted = inserted_groups(transaction);
  existing.iter().any(|current| {
    inserted.iter().any(|candidate| {
      candidate.group != current.group
        && candidate.mark != current.mark
        && previous_removes_group(document, previous, current.group)
    })
  })
}

fn groups_at_point(document: &Document, point: SelectionPoint) -> Vec<LinkedGroup> {
  let mut output = Vec::new();
  let mut current = Some(point.node);
  while let Some(node_id) = current {
    let Some(node) = document.node(node_id) else {
      break;
    };
    if let NodeKind::Inline(InlineKind::MarkFragment { mark, group, .. }) = &node.kind {
      let linked = LinkedGroup {
        group: *group,
        mark: *mark,
      };
      if !output.contains(&linked) {
        output.push(linked);
      }
    }
    current = node.parent;
  }
  output
}

fn inserted_groups(transaction: &Transaction) -> Vec<LinkedGroup> {
  let mut output = Vec::new();
  for operation in &transaction.operations {
    match operation {
      Operation::InsertNode { node, .. } => append_node_group(node, &mut output),
      Operation::InsertSubtree { subtree, .. } => {
        for node in &subtree.nodes {
          append_node_group(node, &mut output);
        }
      }
      _ => {}
    }
  }
  output
}

fn append_node_group(node: &crate::model::Node, output: &mut Vec<LinkedGroup>) {
  let NodeKind::Inline(InlineKind::MarkFragment { mark, group, .. }) = &node.kind else {
    return;
  };
  let linked = LinkedGroup {
    group: *group,
    mark: *mark,
  };
  if !output.contains(&linked) {
    output.push(linked);
  }
}

fn previous_removes_group(document: &Document, previous: &Transaction, group: u64) -> bool {
  previous.operations.iter().any(|operation| {
    let Operation::RemoveNode { node } = operation else {
      return false;
    };
    subtree_contains_group(document, *node, group)
  })
}

fn subtree_contains_group(document: &Document, root: NodeId, group: u64) -> bool {
  let Some(node) = document.node(root) else {
    return false;
  };
  matches!(
    &node.kind,
    NodeKind::Inline(InlineKind::MarkFragment {
      group: candidate, ..
    }) if *candidate == group
  ) || node
    .children
    .iter()
    .any(|child| subtree_contains_group(document, *child, group))
}
