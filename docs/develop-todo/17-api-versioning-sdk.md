# 17 — API versioning et SDK addons

## Problème

Sans contrat SDK explicite, les addons finissent par importer des helpers internes ou dépendre de détails renderer/Tauri. Chaque refactor host devient alors une migration manuelle du catalogue.

## SDK

Publier un package/type definitions canonique contenant : manifest schema, resource client/provider interfaces, capability descriptors, permissions, UI contribution types, jobs/events, settings, file handles et error types.

## Versioning

- host API semver ou version de protocole explicite ;
- resources versionnées indépendamment lorsque nécessaire ;
- additive changes préférées ;
- deprecated avec fenêtre documentée ;
- incompatibilité détectée avant activation.

## Conformance

Fournir un test kit qu'un addon peut exécuter : manifest validation, lifecycle, cleanup, permission checks, resource schema, cancellation, platform declarations.

## Dev environment

- mock host contract pour unit tests ;
- integration harness avec vrai host headless lorsque possible ;
- fixture vault ;
- package/sign/validate commands ;
- logs corrélés.

## Documentation

Docs générées depuis schemas/types + exemples minimaux. Chaque API sensible documente side effects, threading/lifecycle, platform differences et error cases.

## Compatibility matrix

Catalog indique versions app compatibles. Une app ancienne ne doit pas installer un addon incompatible uniquement parce que le ZIP peut être extrait.

## Acceptance

Compiler/tester plusieurs addons officiels uniquement contre SDK public ; détecter imports internes dans CI ; addon N-1 compatible selon policy ; erreur propre sur version trop récente.