# Citation cross-note insertion

- Request: Corriger le parcours sélection → citation retenue → insertion dans une autre note, avec le cas sans sélection.
- Level: high
- Date: 2026-08-09
- Slug: note-citation-cross-note

## Summary

- [x] Le bouton/icône de citation doit apparaître après une sélection réelle dans l’éditeur.
- [x] La citation doit rester disponible après navigation vers une autre note.
- [x] La citation doit s’insérer à la dernière ligne si aucune sélection n’existe dans la note cible.

## Repro Steps

- [x] Ouvrir `Alpha note`.
- [x] Sélectionner `Visible alpha body line.`.
- [x] Activer l’icône de citation.
- [x] Ouvrir `Beta project` sans sélectionner de texte.
- [x] Cliquer l’icône de citation retenue.

## Environment

- [x] Checkout courant `develop`, application Electron/Tauri renderer, macOS.
- [x] Log utilisateur fourni : boucle de résolution d’assets Excalidraw ; il ne contient pas encore l’échec de citation ciblé.

## Observed vs Expected

- Observed: après navigation, le curseur effondré restauré au début de la note cible était utilisé comme position d’insertion ; la citation pouvait être écrite avant le front matter. Le seul test UI précédent vérifiait seulement la présence du bouton après sélection.
- Expected: le bouton de sélection et l’icône retenue suivent le parcours de production complet, puis la citation est persistée dans la note cible au dernier emplacement lorsque la sélection cible est absente.

## Hypotheses

- [x] Le comportement cross-note n’est pas couvert par l’acceptance UI, donc une régression de buffer, de sélection restaurée ou de mise à jour du fichier peut passer inaperçue.
- [x] Le runtime de citation doit utiliser le fichier courant après navigation et ne pas traiter un curseur effondré restauré comme une sélection d’insertion.

## Investigation Plan

- [x] Tracer `selectionchange`/`mouseup`, création du bouton, `copyCitation`, `renderPalette`, navigation et `pasteCitation`.
- [x] Reproduire le parcours avec le vrai Electron renderer et vérifier le Markdown persisté.

## Fix Plan

- [x] Corriger uniquement le chemin production si la reproduction échoue.
- [x] Distinguer une sélection réelle d’un curseur effondré ; sans sélection, utiliser la fin du Markdown courant.
- [x] Renforcer le test UI sur sélection, bouton disponible, buffer retenu, navigation, insertion sans sélection et dernière ligne.

## Regression Tests

- [x] `tests/app/e2e/ui-feature-regressions.spec.js` couvre le parcours complet cross-note.
- [x] Les tests unitaires existants couvrent la restauration de sélection, la construction du Markdown et l’insertion au curseur.
- [x] Preuve rouge : `/tmp/elephant-citation-red-3.log`.
- [x] Preuve verte UI : `/tmp/elephant-citation-ui-final-4.log`.
- [x] Preuve verte unit : `/tmp/elephant-citation-unit-final.log`.

## Release Notes

- [x] Aucun bouton de citation ajouté dans la toolbar ; le contrôle reste une action flottante dédiée.

## Risks

- [ ] La validation packaged/Tauri et les autres plateformes restent hors de ce correctif ciblé.

## Rollout

- [ ] Aucun commit ni push demandé ou effectué.
