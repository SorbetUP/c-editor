#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


path = Path("Elephant/freya/src/app/drawing_canvas.rs")
text = path.read_text()
text = replace_once(
    text,
    '#[path = "drawing_text_edit.rs"]\nmod drawing_text_edit;\n',
    '#[path = "drawing_text_edit.rs"]\nmod drawing_text_edit;\n#[path = "drawing_selection_controls.rs"]\nmod drawing_selection_controls;\n',
    "selection controls module",
)
text = replace_once(
    text,
    '        .children(primitives)\n        .maybe_child(editing_index.map(|index| {',
    '        .children(primitives)\n        .child(drawing_selection_controls::selection_controls(state))\n        .maybe_child(editing_index.map(|index| {',
    "selection controls child",
)
path.write_text(text)
