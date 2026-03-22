- [x] Reproduire le bug sur une note neuve créée depuis le composer.
- [x] Isoler la différence entre persistance locale UI et persistance réelle dans le vault.
- [x] Vérifier le contrat `PUT /api/note` côté API locale.
- [x] Identifier la cause racine dans le calcul du titre d'une note brouillon.
- [x] Corriger le calcul du titre pour les notes sans `path`.
- [x] Rejouer la création d'une note avec un titre unique et vérifier le fichier écrit.

## Reproduction

- Ouvrir l'éditeur web avec l'API locale active.
- Créer une note neuve depuis le composer.
- Remplacer `# Nouvelle note` par un titre unique.
- Fermer le modal via le bouton d'envoi.
- Observer que l'UI montre la nouvelle note, mais que le vault n'ajoute pas un nouveau fichier.

## Cause racine

- Une note neuve démarrait avec `title: "Nouvelle note"` dans l'état front.
- Au moment du save, `getNoteTitle(note)` priorisait encore `note.title` au lieu du heading réel du contenu.
- La requête `PUT /api/note` envoyait donc toujours le titre placeholder pour une note sans `path`.
- L'API locale régénérait alors `nouvelle-note.md` et réécrivait ce fichier au lieu d'en créer un nouveau.

## Correctif

- Prioriser le heading Markdown réel dans `getNoteTitle(...)`.
- Initialiser une note brouillon avec `title: ""` pour éviter un placeholder persistant dans l'état.

## Validation

- Repro relancée avec un titre unique `Saved Fresh ...`.
- Le fichier `/tmp/elephantnote-vault-test/Notes/saved-fresh-....md` est bien créé.
- `activeNoteId` devient le slug réel du fichier sauvegardé.

## Régression à surveiller

- Renommage d'une note existante: le titre UI doit suivre le heading, sans renommer le fichier si `path` existe déjà.
- Brouillon vide: l'UI doit continuer à afficher `Nouvelle note` tant qu'aucun vrai titre n'est saisi.
