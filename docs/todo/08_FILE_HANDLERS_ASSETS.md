# TODO — File handlers, assets et ouverture de documents

## Objectif

Elephant reste générique : si un addon déclare gérer `.pdf`, `.c`, `.mmd`, audio, etc., il devient handler. Sans handler, l'app ouvre via l'application système lorsque cela est sûr/supporté.

## P0 — Handler registry

Contribution `file.handlers` : extensions/MIME, priority, view/edit capabilities, platforms, open/create/import, preview optionnelle. Résolution déterministe et choix utilisateur si plusieurs handlers sont équivalents.

## P0 — Fallback système

Si aucun handler : `openExternal` via API host permissionnée. Ne pas hardcoder Chrome/PDF. Les fichiers inconnus restent visibles dans le vault.

## P0 — Asset API

`assets.import/read/copy/resolveUrl/metadata` avec handles et scope. Drag/drop dans une note crée asset + insertion atomique ; un échec ne laisse pas d'orphelin silencieux.

## P0 — Path safety

Canonicalization, symlink checks, hidden-dir policy et ownership. Aucun path absolu hors scope sans grant explicite.

## P1 — Large files

Streaming/range read, taille connue avant chargement, cancellation. PDF/video/archive ne doivent pas être chargés entièrement en base64 dans le renderer.

## P1 — External links

Distinguer import dans vault et lien vers fichier externe. Le second utilise bookmark/handle OS lorsque possible et gère disparition/permission perdue.

## P1 — Context actions

Open with, reveal, copy path/link, change default handler. Defaults par type et sync configurable.

## Validation

- Installer/désinstaller handler change la résolution proprement.
- `.c` ouvert par addon déclaré ou app système.
- Drop gros fichier borné en mémoire.
- Symlink traversal rejeté.
- Désinstaller PDF addon n'empêche pas l'ouverture externe d'un PDF.