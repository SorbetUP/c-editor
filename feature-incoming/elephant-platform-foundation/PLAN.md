# PLAN - Elephant Platform Foundation

## Objectif

Transformer ElephantNote en base de travail pour une application de notes moderne, locale d'abord, multi-interfaces, avec vault sur dossier, sync reseau, UI type Google Keep/Blinko, confidentialite legere, liens entre notes, moteur Markdown enrichi et futures integrations LLM.

## Contexte repo

- Le coeur actuel est en C dans `engines/`.
- La demo principale exposee est `web/site/docs/index.html` avec `engines/editor` + `engines/markdown` + `engines/cursor` en WASM.
- Il existe deja des sous-libs de support: recherche, vault, fichier, crypto, rendu.
- Il n'existe pas encore de fondation claire pour:
  - selection d'un vault utilisateur sur dossier local
  - graphe de notes / liens explicites entre notes
  - sync reseau simple type latest-wins
  - confidentialite legere des notes sensibles
  - catalogue d'extensions Markdown rendables

## Hypotheses

- Le depot doit rester pilote par un coeur C modulaire.
- Les interfaces desktop/web/mobile peuvent utiliser des couches adaptees, mais doivent partager les memes concepts de donnees.
- Le mode "desktop local" reste la cible prioritaire avant les apps mobiles.
- La sync MVP peut etre "latest updated wins" avant gestion avancee de conflits.
- L'integration Blinko ne sera pas un copier-coller; on reprend des idees et des composants cibles par phases.

## Scope

### Inclus dans cette fondation

- Definition d'une architecture cible.
- Decoupage en nouvelles sous-libs C.
- Squelettes compiles + tests unitaires de base.
- Documentation de roadmap de portage.

### Exclu de cette premiere passe

- UI finale complete type Google Keep/Blinko.
- Sync multi-device complete de production.
- Apps Android/iPhone livrees.
- Excalidraw integre et editeur graphique complet.
- LLM tab complet.

## Plan technique

### Phase 1 - Fondation coeur

- [ ] Ajouter `engines/vault_core`
  - Gestion du chemin du vault
  - Validation du dossier
  - Politique de fichiers notes / assets
- [ ] Ajouter `engines/link_engine`
  - Extraction des liens `[[Note]]`
  - Base pour references entre notes
- [ ] Ajouter `engines/privacy_engine`
  - Detection `#credentials`
  - Masquage preview/liste
- [ ] Ajouter `engines/sync_engine`
  - Config de mode `local | web | sync`
  - Endpoint reseau distant
  - Resolution latest-wins basique
- [ ] Ajouter `engines/render_ext`
  - Registre d'extensions Markdown supportees
  - Detection fenced blocks: mermaid, markmap, katex, graphviz, echarts, flashcard

### Phase 2 - Donnees et vault local

- [ ] Brancher `vault_core` au shell desktop
- [ ] Ajouter selection de dossier vault dans l'app
- [ ] Sauver le chemin du vault dans la config app
- [ ] Introduire un index local des notes
- [ ] Introduire format de metadonnees de note et d'attachments

### Phase 3 - UI notes moderne

- [ ] Construire une interface sobre type cartes / grille / liste
- [ ] Ajouter vue "notes", vue "keep-like", vue "detail"
- [ ] Ajouter edition plein ecran
- [ ] Ajouter references et backlinks
- [ ] Ajouter preview masquee pour notes sensibles

### Phase 4 - Local server + web companion

- [ ] Ajouter serveur local desktop
- [ ] Exposer l'interface web sur port configurable
- [ ] Fournir API locale pour notes / vault / sync / settings
- [ ] Connecter la web app a cette API

### Phase 5 - Sync reseau

- [ ] Discovery / configuration d'un pair distant
- [ ] Push/pull latest-wins
- [ ] Tests de sync locale reseau
- [ ] Comptes multiples par serveur

### Phase 6 - Import / drawing / mobile / speech

- [ ] Reprendre le menu d'import Blinko et ses flows utiles
- [ ] Ajouter importeurs (Markdown, ZIP, Google Keep)
- [ ] Integrer un module dessin type Excalidraw ou equivalent
- [ ] Ajouter apps Android/iPhone
- [ ] Evaluer reprise Speech-to-Text depuis Handy

### Phase 7 - IA

- [ ] Definir le contrat LLM commun
- [ ] Providers locaux/distants
- [ ] RAG sur vault
- [ ] UX IA separee du noyau notes

## Checklist

- [ ] Ecrire et maintenir les API des nouveaux modules C
- [ ] Ajouter tests unitaires sur chaque nouveau module
- [ ] Integrer les nouveaux modules dans `engines/Makefile`
- [ ] Documenter l'architecture cible dans `docs/`
- [ ] Maintenir la demo web existante fonctionnelle

## Tests / Validation

- [ ] `make -C engines/vault_core test`
- [ ] `make -C engines/link_engine test`
- [ ] `make -C engines/privacy_engine test`
- [ ] `make -C engines/sync_engine test`
- [ ] `make -C engines/render_ext test`
- [ ] `make -C engines test`

## Risques

- Risque majeur de dispersion si on tente UI desktop/web/mobile + sync + IA en une seule phase.
- Risque de couplage excessif entre moteur C et UI si les contrats ne sont pas figes maintenant.
- Risque de dette si on integre des features Blinko directement sans les adapter au modele ElephantNote.

## Rollout

- Etape 1: fondation C + doc + tests
- Etape 2: vault local desktop
- Etape 3: UI notes moderne
- Etape 4: web companion + sync
- Etape 5: mobile + IA

## Estimation

- Fondations C + branchement build/tests: moyen
- Vault local + UI desktop propre: dure
- Web local + sync LAN + comptes multiples: tres_dure
- Mobile + speech-to-text + drawing + IA: extreme
