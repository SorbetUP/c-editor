# TODO — Runtime éditeur, blocs et extensions

## Problème

Code Execution doit actuellement observer les mutations DOM pour réinstaller sa toolbar après repaint Rust. C'est le symptôme d'un lifecycle de bloc insuffisamment exposé.

## P0 — Block lifecycle API

Le runtime Rust publie `mount/update/unmount` pour des descriptors stables : block ID, kind, language/metadata, surface handle et revision. Les addons enregistrent une extension et reçoivent ces callbacks, sans scanner le DOM.

## P0 — Block identity

IDs stables ou réconciliables, nécessaires pour citations, annotations, tasks, Anki et outputs d'exécution. L'identité ne dépend pas de l'index DOM.

## P0 — Mutation API

Edits addon passent par opérations editor structurées ou document transaction, avec undo/redo, selection mapping et autosave cohérents. Le DOM n'est jamais source de vérité.

## P1 — Inline/decorations

Décorations, diagnostics, links, badges et annotations comme contributions virtualisées, avec limites et possibilité de désactiver une extension lente.

## P1 — Paste/drop

Pipeline priority/canHandle/preview/transaction. Images/fichiers utilisent Assets/File Handler APIs, pas des paths ad hoc.

## P1 — Commands/keymaps

Keybindings namespaced, conflits visibles et overridables. Slash commands utilisent le command registry avec contexte editor.

## P1 — Performance

Budget de temps par extension, diagnostics slow callbacks, aucun full-document scan synchrone à chaque frappe. Le runtime Rust reste canonique pour le texte.

## Validation

- Code Execution sans MutationObserver de réparation.
- Bloc référencé retrouvé après édition autour de lui.
- Undo annule une mutation addon comme une mutation native.
- Drag/drop image/fichier cohérent desktop/mobile.
- Extension défaillante isolée et désactivable.