from pathlib import Path

path = Path("Elephant/freya/tests/editor_image_markdown_freya_testing.rs")
if not path.exists():
    raise SystemExit("finalizer did not create the image graphical test")
text = path.read_text()
anchor = "use elephant_freya::app::app_with_vault;\n"
if "use freya::prelude::*;" not in text:
    if anchor not in text:
        raise SystemExit("image graphical test import anchor missing")
    text = text.replace(anchor, anchor + "use freya::prelude::*;\n", 1)
path.write_text(text)
