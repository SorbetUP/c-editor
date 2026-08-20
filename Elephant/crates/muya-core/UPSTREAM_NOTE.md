# Canonical Note upstream

`SorbetUP/Note` is the canonical development repository for this Muya-compatible Rust editor core.

This public-tree copy exists so Elephant/Freya can build and test from a normal checkout without credentials for the private Note submodule.

Pinned canonical snapshot:

- repository: `SorbetUP/Note`
- commit: `ce1fd7aa5391e0561f514ec31b79ba873f764e05`
- required Elephant gitlink target: `ce1fd7aa5391e0561f514ec31b79ba873f764e05`
- `crates/muya-core/src` Git tree: `282cb8f63051a311264ce568919a5a7f9f47589a`
- `crates/muya-core/data` Git tree: `7f09e2bd5ba5a1c014429a9f5e2ceeea9e452062`

Development rule:

1. implement parser/serializer/editing/history/selection/protocol changes in `SorbetUP/Note/crates/muya-core` first;
2. run Note core + graphical CI there;
3. mirror the exact core `src` and `data` trees into this directory;
4. advance `Elephant/feature-incoming/Note` to the same Note commit;
5. run Elephant's Freya integration/differential suites.

The `freya-note-provenance` workflow enforces the pinned Git identities above without requiring access to the private submodule. Do not implement editor-core behavior only in this mirror.
