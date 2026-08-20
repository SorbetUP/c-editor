# Canonical Note upstream

`SorbetUP/Note` is the canonical development repository for this Muya-compatible Rust editor core.

This public-tree copy exists so Elephant/Freya can build and test from a normal checkout without credentials for the private Note submodule. The core was transplanted into Note from this exact tree at Elephant commit `5dab1d75171121e686c05542271f27495d86d1ca`; no engine source changes have diverged since that transplant at the time this marker was added.

Development rule:

1. implement parser/serializer/editing/history/selection/protocol changes in `SorbetUP/Note/crates/muya-core` first;
2. run Note core + graphical CI there;
3. mirror the exact core source into this directory;
4. advance `Elephant/feature-incoming/Note` to the same Note commit;
5. run Elephant's Freya integration/differential suites.

Do not implement editor-core behavior only in this mirror.
