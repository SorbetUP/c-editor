use muya_core::{
  model::{BlockKind, InlineKind, InlineSyntax, NodeKind},
  parse_markdown, to_markdown,
};

#[test]
fn mixed_tauri_document_keeps_structure_after_save_and_reparse() {
  let markdown = r#"> # Quoted heading
>
> - quoted item
>   continuation
>
>   second paragraph
>   - nested

- root item
  continuation with **bold**

  > nested quote
- sibling

    indented code
    second line

```mermaid
graph TD
  A-->B
```

[^source]: first footnote paragraph

    second footnote paragraph

    - footnote nested item

[one]: https://one.example
[two]: https://two.example "Two"

A [reference][two] and https://example.com and <dev@example.com> with [^source]."#;

  let document = parse_markdown(markdown);
  let saved = to_markdown(&document);
  let reparsed = parse_markdown(&saved);

  assert_eq!(to_markdown(&reparsed), saved);
  assert!(reparsed.nodes.values().any(|node| matches!(
    node.kind,
    NodeKind::Block(BlockKind::Diagram { ref language }) if language == "mermaid"
  )));
  assert!(reparsed.nodes.values().any(|node| matches!(
    node.kind,
    NodeKind::Block(BlockKind::CodeBlock { fenced: false, .. })
  )));
  assert!(reparsed.nodes.values().any(|node| matches!(
    node.kind,
    NodeKind::Block(BlockKind::BlockQuote)
  )));
  assert!(reparsed.nodes.values().any(|node| {
    matches!(node.kind, NodeKind::Inline(InlineKind::Link { .. }))
      && matches!(node.inline_syntax, Some(InlineSyntax::Reference { .. }))
  }));
  assert!(reparsed.nodes.values().any(|node| {
    matches!(node.kind, NodeKind::Inline(InlineKind::Link { .. }))
      && matches!(node.inline_syntax, Some(InlineSyntax::BareAutoLink { .. }))
  }));
  assert!(reparsed.nodes.values().any(|node| matches!(
    node.kind,
    NodeKind::Inline(InlineKind::FootnoteReference { .. })
  )));

  let footnote = reparsed
    .nodes
    .values()
    .find(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::FootnoteDefinition { .. })
    ))
    .expect("structured footnote definition");
  assert!(reparsed.children(footnote.id).any(|node| matches!(
    node.kind,
    NodeKind::Block(BlockKind::List { .. })
  )));

  let definitions = reparsed
    .nodes
    .values()
    .filter(|node| matches!(
      node.kind,
      NodeKind::Block(BlockKind::ReferenceDefinition { .. })
    ))
    .count();
  assert_eq!(definitions, 2);
}

#[test]
fn list_item_children_survive_serialization_as_distinct_blocks() {
  let markdown = "- first\n  continuation\n\n  second paragraph\n\n  > quote\n  > continued\n- sibling";
  let document = parse_markdown(markdown);
  let saved = to_markdown(&document);
  let reparsed = parse_markdown(&saved);

  let list = reparsed
    .children(reparsed.root)
    .find(|node| matches!(node.kind, NodeKind::Block(BlockKind::List { .. })))
    .expect("root list");
  let first = reparsed.children(list.id).next().expect("first item");
  let children = reparsed.children(first.id).collect::<Vec<_>>();
  assert!(children.len() >= 3, "list blocks collapsed during save: {saved}");
  assert!(children.iter().any(|node| matches!(
    node.kind,
    NodeKind::Block(BlockKind::BlockQuote)
  )));
}

#[test]
fn adjacent_reference_definitions_stay_adjacent_and_resolve() {
  let markdown = "[one]: https://one.example\n[two]: https://two.example \"Two\"\n\n[go][two]";
  let document = parse_markdown(markdown);
  assert_eq!(to_markdown(&document), markdown);

  assert!(document.nodes.values().any(|node| matches!(
    (&node.kind, &node.inline_syntax),
    (
      NodeKind::Inline(InlineKind::Link {
        destination,
        title: Some(title),
      }),
      Some(InlineSyntax::Reference { .. })
    ) if destination == "https://two.example" && title == "Two"
  )));
}
