# TODO — Secrets, permissions et sécurité addon

## Problème

Providers IA, mail, calendrier, GitHub, data et futurs packs enterprise nécessitent des credentials. Les stocker dans settings JSON/addon storage facilite fuite dans logs, export, sync ou prompt.

## P0 — Secret store

API `secrets` avec clés opaques par addon/account. Backend : Keychain/Secret Service/Credential Manager lorsque disponible, fallback chiffré documenté sinon. Les settings conservent seulement un `secretRef`.

## P0 — Permission manifest

Permissions fines : documents read/write par scope, assets, network hosts, external opener, native service, camera/microphone, notifications, jobs, secrets, execution, webview. Installation affiche le différentiel de permissions.

## P0 — Runtime grants

Les permissions sensibles peuvent être accordées à l'usage/workspace. Un grant a scope, expiration et provenance. Révocation immédiate invalide sessions/handlers concernés.

## P0 — Network broker

HTTPS, DNS rebinding/public-address checks, redirects bornés, host allowlist, limites temps/taille et erreurs TLS visibles. Localhost, s'il est réellement requis, utilise une permission distincte.

## P0 — Web content

HTML externe/sites/embedded apps ont origine dédiée, CSP et aucun bridge Tauri/addon par défaut. `allow-scripts + allow-same-origin` exige une justification de menace et une isolation supplémentaire.

## P1 — Prompt/data boundary

Marquer note/mail/web/OCR récupéré comme `untrusted_content`. Ce contenu n'accorde aucun droit à l'agent et ne peut modifier la policy.

## P1 — Logging sanitization

Redaction centralisée Authorization, cookies, tokens, URLs signées et secret refs. Tests injectant un faux secret pour prouver son absence des logs/artifacts.

## Validation

- Export settings/vault sans secret provider.
- Révocation grant effective immédiatement.
- Un addon sans network ne contourne pas via Tauri/fetch.
- Un site preview ne voit pas `window.__TAURI__`.
- Prompt injection dans une note ne peut autoriser un external write.