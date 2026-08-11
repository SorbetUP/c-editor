# TODO — Contrats desktop/mobile et backends natifs

## Objectif

Éviter deux produits divergents. Les APIs produit sont communes ; chaque plateforme annonce support et fournit un adapter natif.

## P0 — Capability matrix runtime

`platform.capabilities` annonce file picker, filesystem scope, camera, microphone, notifications, background execution, external opener, native sidecars, local models et secure storage. L'UI s'adapte à ce contrat.

## P0 — Handles plateforme

Android/iOS ne simulent pas un path POSIX desktop. Utiliser document-tree/bookmark handles persistants, revalidation permission au démarrage et erreurs actionnables.

## P0 — Permission timing

Camera/micro/storage seulement lors de l'action qui l'exige. Un addon ne déclenche pas une permission au startup sans raison utilisateur visible.

## P0 — Background limits

Jobs connaissent les contraintes OS. Si une opération longue ne peut rester active, checkpoint + reprise explicite plutôt qu'un faux status running.

## P1 — Addon packaging

Manifest décrit plateformes/architectures. UI n'affiche pas `Install` pour package impossible ; elle peut expliquer l'indisponibilité.

## P1 — Interaction mobile

Sidebar edge gesture, keyboard avoidance, sheets/dialogs, alternatives au drag/drop et touch targets font partie des acceptance scenarios.

## P1 — Local models

Sélection selon RAM/backend device, pas de téléchargement d'un modèle incompatible. Thermal/battery constraints visibles si pertinentes.

## Validation

- Vault Android survit au restart.
- Pas de permission caméra au démarrage hors besoin.
- Même API documents via handles plateforme.
- Addon desktop-only échoue proprement sur mobile.
- Create/save/drop/sidebar testés dans l'app mobile réelle.