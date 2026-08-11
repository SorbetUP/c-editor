# 03 — Permissions, proposals et sécurité

## Objectif

Un addon ou agent ne doit jamais obtenir un droit parce qu'il sait appeler une commande IPC. Les droits doivent être exprimés, accordés et vérifiés au niveau host.

## Permission model

Scopes composables :

- vault read/write par dossier/resource ;
- network par domains ou connector ;
- process/execute ;
- clipboard/camera/share ;
- external account scopes ;
- notifications ;
- secret access via handles ;
- UI contributions sans accès data implicite.

## Grants

Distinguer permission demandée dans manifest et grant effectif. Un update d'addon demandant un nouveau scope ne reçoit pas automatiquement ce scope.

## Proposal service

Le host doit proposer une primitive commune pour les mutations à review :

`createProposal -> preview -> approve/reject -> validatePrecondition -> apply -> audit`.

Champs : target resource id, target version, operation, payload/diff, requesting addon/run, provenance, risk class, expiration et idempotency key.

## Pourquoi host-owned

Si chaque addon réimplémente sa confirmation, un agent peut contourner une UI ou les semantics divergent. Un service central permet une policy uniforme pour notes, Wiki, Calendar, mail, GitHub, Code Execution et Automation.

## Prompt injection / untrusted content

Le host ne tente pas de « comprendre » les prompts, mais garantit qu'un résultat venant d'un document/web/mail n'augmente jamais les permissions. Les capabilities autorisées sont calculées hors modèle.

## Audit

Conserver un journal borné des mutations sensibles : qui/addon/run, quoi, cible, résultat. Redacter contenu/secrets selon policy. L'audit ne doit pas devenir une copie illimitée des données privées.

## Acceptance

- path traversal et symlink escape ;
- addon update avec nouvelle permission ;
- proposal stale après modification cible ;
- double approval idempotent ;
- revoked permission pendant run ;
- agent tool list filtrée ;
- external write impossible sans grant ;
- secrets absents des logs/audit.