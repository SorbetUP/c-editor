# TODO — Framework pour intégrations externes

## Objectif

Mail, GitHub, calendriers, cloud drives, Jira/Linear, Teams/Slack et futurs connecteurs partagent OAuth, cursor sync, webhooks, rate limits, offline, provenance et external writes. Le host doit fournir ces primitives sans connaître les domaines métier.

## P0 — Account/connection resource

Handle de connexion avec provider, account label, scopes, status, last sync et error class. Les tokens restent dans Secret Store.

## P0 — OAuth/deep-link callback

Flow PKCE/state, browser opener, callback loopback/deep-link sécurisé, cancellation, timeout et revoke. Aucun addon ne lance un serveur arbitraire sans permission.

## P0 — Cursor sync

Jobs durables avec cursor opaque, paging, dedup external IDs/revisions, backoff et deletion/tombstone. Le host fournit l'infrastructure ; l'adapter traduit l'API provider.

## P0 — External write proposals

Send mail, create event, comment issue, merge PR, etc. passent par Proposal Service. Preview montre compte, cible et payload significatif.

## P1 — Webhook ingress

Optionnel, surtout pour déploiements serveur/enterprise : signature, replay protection, event dedup et queue durable. Desktop local reste capable de polling si webhook impossible.

## P1 — Offline cache

Objets externes gardent provenance, revision et stale status. Un cache stale n'est pas présenté comme état courant sans indicateur.

## P1 — Rate limits

Budget et Retry-After centralisés ; scheduler réduit fréquence au lieu de laisser chaque addon spammer l'API.

## Validation

- Token révoqué -> reconnect state, pas boucle d'erreurs.
- Rejouer page/webhook ne duplique pas objets.
- External write jamais exécuté avant policy requise.
- Aucun token dans vault/log/sync.
- Connecteur offline ne bloque pas startup/editor.