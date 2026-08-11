# 09 — File handlers, Open With et drag/drop

## Direction

Le core ne doit pas contenir un traitement spécial « PDF addon ». Il doit fournir un registry générique de handlers par type de fichier.

## File handler registry

Un addon déclare extensions/MIME + capabilities : preview/edit/extract/convert. Le host choisit le handler préféré selon compatibilité + préférence utilisateur. Si aucun handler interne n'existe, ouvrir avec l'application système.

## MIME

Ne pas faire confiance uniquement à l'extension. Utiliser MIME/sniffing borné pour sécurité et sélectionner le bon handler. Un mismatch est signalé.

## Open With

Context menu : handler Elephant compatibles + `System default`. Permettre définir préférence par type. Un addon désinstallé retire proprement sa préférence ou déclenche fallback système.

## Drag vers file tree

- drop fichier externe dans dossier => copie/import atomique ;
- collision => rename/replace/cancel policy ;
- progression pour gros fichiers ;
- permissions plateforme respectées.

## Drag/paste dans note

Image => attachment + Markdown image au caret. Fichier => attachment/lien cliquable. Drop d'une note interne => wiki/file link selon action. Le caret/drop position est calculé par l'éditeur.

## Mobile share

Android/iOS share target doit mapper vers le même Import/Attachment API, pas un code path différent sans tests.

## Security

Path traversal, symlinks, fichiers spéciaux, énorme input et MIME actif. Aucun handler addon ne reçoit le path global s'il peut travailler via handle scoped.

## Acceptance

PDF sans addon -> system app ; handler installé -> choix interne ; `.c` avec handler custom ; drag image au milieu d'une note ; drag gros fichier ; collision ; mobile share ; uninstall handler ; fichier extension trompeuse.