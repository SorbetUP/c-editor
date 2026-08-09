# BUG: Tauri affiche le mode source CodeMirror au lieu de Muya Rust

## Résumé

La capture montre l’éditeur source CodeMirror avec le Markdown brut (`# dddd`). Le mode source masquait donc Muya Rust et donnait l’impression que le rendu Markdown ne fonctionnait pas.

## Cause

`editorWithTabs` montait `sourceCode.vue` lorsque la préférence transitoire `sourceCode` était vraie. Le composant Rust était alors démonté.

## Correction

- suppression du montage de `sourceCode.vue` dans l’éditeur ;
- montage permanent de `RustMuyaRuntimeEditor` ;
- neutralisation de l’activation du mode source par les anciennes préférences, l’IPC et les raccourcis ;
- suppression de la commande « source-code-mode » de la palette ;
- ajout des indicateurs d’acceptation `rustEditorPresent` et `codeMirrorPresent` dans `readState`.

## Vérification

- suite unitaire app : 168 fichiers passés, 3 207 tests passés, 27 fichiers ignorés ;
- lint ciblé et `git diff --check` passés ;
- le scénario Tauri a observé `sourceCode: false`, `rustEditorPresent: true` et `codeMirrorPresent: false` après ouverture de note.

## Risque restant

Le scénario Tauri complet a ensuite rencontré un timeout indépendant lors de la fermeture du panneau de recherche (`waitUntilGone`), après le contrôle Rust/CodeMirror. Il faut le rejouer séparément avant de qualifier une build release complète.
