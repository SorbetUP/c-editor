# 07 — Settings, connections et secrets

## Direction

Settings doit être un shell extensible : le host gère navigation/recherche/persistence et les addons contribuent des schemas/sections. Les credentials ne doivent jamais être de simples champs JSON persistés avec le reste.

## Settings schema

Chaque réglage déclare type, default, validation, scope (`global`, `vault`, `workspace`, `device`), sensitivity, restart requirement et migration version.

## Persistence

- écriture atomique ;
- migrations versionnées ;
- unknown fields conservés ou gérés explicitement lors downgrade ;
- aucun état UI transitoire mélangé aux settings durables ;
- chargement des préférences terminé avant de considérer le renderer prêt lorsque le comportement en dépend.

## Secrets

API host secure store retournant `SecretRef`. Un addon peut demander l'usage d'un secret accordé sans en obtenir une copie persistante dans sa config lorsque l'architecture le permet.

## Connections

Settings > Connections commun : accounts/providers, scopes, health, last sync, reconnect/logout. Calendar/GitHub/mail/Codex peuvent contribuer leurs détails sans refaire le shell OAuth.

## Search

Recherche globale sur labels, descriptions et keywords des sections core/addons. Résultat navigue directement vers le contrôle et le met en évidence.

## UX

- sections stables ;
- dirty/unsaved state seulement si une sauvegarde explicite est nécessaire ;
- erreurs de validation inline ;
- reset par setting/section ;
- paramètres avancés repliables ;
- mobile navigation sans panneaux trop larges.

## Acceptance

Migration settings, corrupted file recovery, secret absent des logs/export, addon disabled avec settings conservés, vault switch, recherche d'un setting addon et OAuth revoked/reconnect.