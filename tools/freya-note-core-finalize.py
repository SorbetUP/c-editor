from pathlib import Path

CORE_MOD = Path("Elephant/crates/muya-core/src/edit/mod.rs")
CORE_COMMAND = Path("Elephant/crates/muya-core/src/edit/command.rs")
PASTE_NESTED = Path("Elephant/crates/muya-core/src/edit/paste_nested.rs")
PASTE_STRUCTURED = Path("Elephant/crates/muya-core/src/edit/paste_nested_structured.rs")
EDITOR = Path("Elephant/freya/src/editor.rs")
CROSS = Path("Elephant/crates/muya-core/src/edit/cross_selection.rs")


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text()
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    path.write_text(text.replace(old, new, 1))


if not CROSS.is_file():
    raise SystemExit("cross_selection.rs is missing from the vendored Muya core")

replace_once(
    CORE_MOD,
    "mod code_span;\nmod command;",
    "mod code_span;\nmod command;\nmod cross_selection;",
    "cross-selection module wiring",
)

replace_once(
    CORE_COMMAND,
    """  let (start, end, parent, start_index, end_index) =
    ordered_sibling_text_selection(document, selection)?;""",
    """  let (start, end, parent, start_index, end_index) =
    match ordered_sibling_text_selection(document, selection) {
      Ok(ordered) => ordered,
      Err(EditError::CrossNodeSelection) => {
        return super::cross_selection::build_cross_wrapper_replace(document, selection, inserted);
      }
      Err(error) => return Err(error),
    };""",
    "cross-wrapper replacement dispatch",
)

for path in (PASTE_NESTED, PASTE_STRUCTURED):
    replace_once(
        path,
        """  let mut node = Node::new(id, kind, None);
  node.parent = parent;
  nodes.push(node);""",
        """  let mut node = Node::new(id, kind, None);
  node.parent = parent;
  node.inline_syntax = source_node.inline_syntax.clone();
  nodes.push(node);""",
        f"lossless paste clone in {path.name}",
    )

replace_once(
    EDITOR,
    """    pub fn paste_markdown(
        &mut self,
        markdown: impl Into<String>,
    ) -> Result<EditorUpdate, EditorError> {
        let markdown = markdown.into();
        match self.dispatch(EditorAction::PasteMarkdown(markdown.clone())) {
            Err(EditorError::Edit(EditError::CrossNodeSelection)) => {
                self.dispatch(EditorAction::InsertText(String::new()))?;
                self.dispatch(EditorAction::PasteMarkdown(markdown))
            }
            result => result,
        }
    }""",
    """    pub fn paste_markdown(
        &mut self,
        markdown: impl Into<String>,
    ) -> Result<EditorUpdate, EditorError> {
        self.dispatch(EditorAction::PasteMarkdown(markdown.into()))
    }""",
    "remove Elephant cross-node paste fallback",
)

forbidden = [
    "Err(EditorError::Edit(EditError::CrossNodeSelection))",
    "self.dispatch(EditorAction::InsertText(String::new()))?;\n                self.dispatch(EditorAction::PasteMarkdown(markdown))",
]
editor = EDITOR.read_text()
leaked = [marker for marker in forbidden if marker in editor]
if leaked:
    raise SystemExit(f"adapter fallback remains after cutover: {leaked}")

required = [
    "mod cross_selection;",
    "super::cross_selection::build_cross_wrapper_replace",
    "node.inline_syntax = source_node.inline_syntax.clone();",
]
joined = "\n".join(
    path.read_text() for path in (CORE_MOD, CORE_COMMAND, PASTE_NESTED, PASTE_STRUCTURED)
)
missing = [marker for marker in required if marker not in joined]
if missing:
    raise SystemExit(f"Muya cutover incomplete: {missing}")
