# TODO — UX core et fiabilité générale

## Objectif

Même avec une architecture addon avancée, Elephant reste d'abord un éditeur/vault. Les fonctions de base doivent être rapides, prédictibles et récupérables.

## P0 — Autosave durable

État dirty/saving/saved/error fondé sur l'écriture réellement durable. Flush sur close/background lorsque possible ; write atomique et recovery journal pour éviter fichier tronqué.

## P0 — File watcher reconciliation

Création/modification externe met à jour arbre et onglets sans reload global. Si une note ouverte change hors app, détecter le conflit et ne jamais écraser silencieusement.

## P0 — Startup/recovery

Après crash : restaurer vault, onglets et dernier contenu durable. Un addon défaillant ne bloque pas le startup core ; mode safe/recovery explicite si nécessaire.

## P0 — Error UX

Erreur actionnable : opération, cible, cause compréhensible, retry/reveal/log details. Pas de toast éphémère unique lorsque des données ne sont pas sauvées.

## P1 — Sidebar

Grande arborescence virtualisée, expansion sans ouvrir, keyboard, drag/drop, edge gesture mobile, état par vault. Recently Edited doit être réactif via addon/API.

## P1 — Tabs/navigation

Back/forward, reopen closed, rename/move sans doublon de path, protection perte, file handlers. Deep links via resource handle stable.

## P1 — Settings

Recherche globale, sections addon cohérentes, reset par setting/section, scope global/vault/device visible, export sans secrets.

## P1 — Offline-first

Aucune fonction core de note ne dépend du réseau. Addons cloud indiquent offline/queue sans ralentir l'éditeur.

## Validation

Scénarios indépendants create/open/edit/autosave/external edit/rename/move/drop/restart/crash/low-disk. Mesurer latency et vérifier absence de perte de contenu.