# 08 — Éditeur / Muya : fiabilité et parité

## Invariant

Le contenu sauvegardé sur disque doit être la représentation attendue du document après chaque opération d'édition. Le rendu riche ne doit jamais être une seconde source de vérité divergente.

## Muya decomposition / Rust migration

Conserver l'objectif de parité de comportement avant optimisation. Pour chaque composant migré : test de caractérisation de l'implémentation actuelle, nouvelle implémentation, comparaison sur corpus Markdown, puis remplacement. Éviter les gros rewrites impossibles à bisecter.

## Document transaction

Chaque edit produit un modèle/version. Autosave écrit la version courante ; si une réponse de save correspond à une version plus ancienne, elle ne doit pas marquer la nouvelle version comme sauvegardée.

## Autosave

- debounce court mais borné ;
- flush on blur/route/vault close/app lifecycle lorsque possible ;
- indication d'erreur de save visible ;
- retry sans dupliquer/revenir en arrière ;
- crash recovery journal pour edits non flushés.

## DOM / model divergence

Les outputs de code blocks, widgets et addons ne doivent pas muter arbitrairement le DOM géré par l'éditeur. Utiliser des decorations/embeds avec lifecycle explicite. Ajouter des tests contre `DOM diverged`, recursive update et stack overflow.

## Markdown temps réel

Corpus de caractérisation : headings, lists imbriquées, code fences, tables, links/wiki links, images, blockquotes, task lists, footnotes si supportées, Unicode, long lines. Tester aller-retour parse/render/serialize sans corruption.

## Slash / quick insert

Command registry commun avec capability contributions. `/` ouvre rapidement ; clavier et tactile ; recherche ; action insère une transaction editor, pas du DOM brut.

## Attachments

Drop/paste image : copier dans `.assets` via Attachment API, puis insérer au caret exact. Drop fichier : lien avec handler/Open With. Undo doit retirer le lien ; suppression physique de l'asset orphelin doit être gérée avec prudence, pas immédiatement si encore référencé.

## Executable code blocks

Output hors contenu Markdown ou sérialisation explicitement définie. Run/Stop ne doit jamais déclencher une reparse destructive du document entier.

## Performance

Mesurer input latency p95, ouverture gros documents, memory after repeated open/close, incremental parse cost. Virtualiser uniquement les parties qui préservent correctement selection/caret/IME.

## Acceptance

- typing + autosave + kill/restart ;
- IME/composition ;
- undo/redo après image drop ;
- code output volumineux ;
- markdown round-trip corpus ;
- note très longue ;
- changement de note pendant save ;
- aucun lost update.