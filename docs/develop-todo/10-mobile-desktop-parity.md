# 10 — Parité mobile / desktop

## Principe

La parité signifie mêmes **semantics** pour les primitives core, pas nécessairement même UI ni même backend OS.

## P0 workflows à garantir partout

1. choisir/créer un vault ;
2. créer dossier/note ;
3. détecter création/modification externe lorsque l'OS le permet ;
4. éditer et autosave ;
5. rename/move/delete ;
6. importer/drop/share image/fichier ;
7. ouvrir un fichier via handler interne ou app système ;
8. settings persistants ;
9. addon enable/disable + permissions ;
10. navigation/sidebar fiable.

## Storage permissions mobile

Abstraire SAF/document providers/security-scoped bookmarks derrière VaultHandle. L'UI ne doit pas manipuler un path non valide comme s'il était un filesystem desktop.

## Lifecycle

Background/foreground, process death, rotation/config changes et low-memory doivent être testés. Flush editor et checkpoints jobs lors des hooks disponibles, sans supposer qu'un callback final sera toujours reçu.

## Sidebar gestures

Edge swipe avec zone de déclenchement stable, mouvement 1:1 avec doigt, velocity threshold, cancel/reverse naturels. Le geste ne doit pas entrer en conflit avec navigation editor horizontale sans arbitration explicite.

## Keyboard / safe areas

Toolbar editor doit rester accessible avec IME ouvert. Utiliser viewport/safe-area primitives communes. Tester claviers Samsung/Gboard et hardware keyboard au niveau comportemental.

## Permissions UX

Demander caméra uniquement au moment où une feature caméra est utilisée. Pour choisir un vault, demander stockage/document access approprié, pas permissions sans rapport.

## Performance

Budgets distincts mobile : startup, mémoire idle, scrolling file tree, editor typing, graph. Dégrader LOD/preview, pas correctness.

## Acceptance matrix

Android/iOS/Desktop pour chaque workflow P0. Un `unsupported` documenté est acceptable pour une capability addon réellement indisponible ; un core workflow cassé ne l'est pas.