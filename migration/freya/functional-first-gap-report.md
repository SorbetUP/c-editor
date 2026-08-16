# Freya vanilla — rapport des gaps fonctionnels (functional-first)

Date d’audit : 2026-08-16  
Branche inspectée : 'nsb/freya-native-migration'  
Révision inspectée : '188f5ff8b'  
Périmètre : Freya native vanilla uniquement, sans installation, route, service ni runtime d’addon.

## Règle de lecture

Ce rapport sépare le comportement présent dans le code ou ciblé par un test local de la preuve d’un parcours utilisateur complet.

- **PARTIALLY PROVEN** : un chemin Freya et/ou une assertion ciblée existe, mais il manque au moins une preuve de fenêtre native, d’état disque, de redémarrage, d’erreur ou de validation sur la révision finale.
- **NOT PROVEN** : le parcours complet n’est pas démontré par un artefact exécutable et vérifiable.
- **BLOCKED** : une condition concrète empêche la preuve demandée.
- Un test présent dans l’arbre de travail mais non exécuté pendant cet audit ne constitue pas une preuve finale.
- La comparaison pixel et la parité visuelle ne sont pas utilisées pour promouvoir un parcours fonctionnel au statut prouvé.
- Les changements locaux préexistants n’ont pas été modifiés. Aucun code ni test n’a été exécuté ou édité pour produire ce rapport.

## Synthèse

Les briques vanilla visibles dans Freya couvrent déjà une partie des actions de base : choix de vault, création de note et de dossier, navigation de bibliothèque, ouverture/édition Muya, préférences d’apparence/navigation et recherche.

Les gaps qui restent fonctionnellement importants sont :

1. relier ces briques à un vrai parcours de fenêtre Freya lancé depuis un profil et un vault propres ;
2. vérifier les effets persistants après fermeture et redémarrage du processus ;
3. prouver les chemins d’erreur et leur visibilité ;
4. prouver les changements externes du vault et le rafraîchissement de l’interface ;
5. distinguer les interactions simulées par le harnais de test des interactions clavier/souris/clipboard réelles ;
6. valider l’acceptance desktop Freya et le paquet distribué, sans inclure les addons dans le verdict vanilla.

## Parcours élémentaires

| Petit parcours vanilla | État actuel | Gap fonctionnel réel | Preuve attendue |
|---|---|---|---|
| Démarrer avec un vault existant | PARTIALLY PROVEN | 'ELEPHANT_FREYA_VAULT' et le picker existent, mais le choix utilisateur, l’activation d’un second vault, le profil propre et la reprise après redémarrage ne sont pas prouvés sur la révision finale. | Fenêtre Freya lancée avec un profil vide ; sélectionner un vault existant ; afficher son contenu ; fermer ; relancer avec le même profil ; retrouver le vault actif. Capturer stdout/stderr, le chemin actif et le fichier de registre. |
| Refuser un chemin de vault invalide | PARTIALLY PROVEN | Une assertion ciblée existe pour conserver le picker visible, mais le rendu de l’erreur et l’absence d’état partiellement ouvert dans le processus réel ne sont pas démontrés. | Fournir un chemin inexistant/non lisible ; observer un message d’erreur stable, le picker toujours utilisable et aucun vault actif ; vérifier le log d’erreur sans contenu privé. |
| Basculer entre deux vaults et retirer une entrée | PARTIALLY PROVEN | 'vault_registry.rs' et 'vault_registry_freya_testing.rs' couvrent le registre au niveau du harnais, mais l’effet sur la fenêtre, le vault affiché et la persistance après relance n’est pas prouvé. | Activer Vault A puis Vault B ; vérifier le contenu affiché ; retirer A ; relancer ; vérifier que le registre ne contient plus A et que B reste actif. |
| Créer une note depuis le menu Create | PARTIALLY PROVEN | Le menu expose 'Note' et une assertion vérifie la création d’un markdown, mais le parcours natif nommage → édition → sauvegarde → réouverture n’est pas validé de bout en bout. | Cliquer sur le menu réel, créer une note, saisir un titre et du contenu ; vérifier le fichier markdown, la liste, la réouverture et le contenu après redémarrage. |
| Ouvrir, renommer et supprimer une note | PARTIALLY PROVEN | Les actions de carte et l’erreur d’ouverture sont ciblées dans 'library_settings_freya_testing.rs'; il manque la preuve d’une action réelle sur une fenêtre propre et la vérification systématique du disque après chaque étape. | Pour une note fixture : ouvrir, renommer, fermer, vérifier le chemin et le contenu ; supprimer, vérifier l’absence du fichier et le comportement de la corbeille/erreur. |
| Éditer une note Muya et autosauvegarder | PARTIALLY PROVEN | Les modules 'editor_view.rs', 'editor_keyboard_freya_testing.rs' et 'editor_lifecycle_freya_testing.rs' ciblent les transitions Muya, mais l’écriture disque et le redémarrage final ne sont pas prouvés sur une application Freya propre. | Saisir un contenu synthétique ; attendre la politique d’autosave ; vérifier le markdown sur disque ; fermer immédiatement puis relancer ; vérifier le même AST/contenu et l’absence de perte. |
| Fermer une note sale et signaler une erreur d’écriture | PARTIALLY PROVEN | Les chemins de flush et d’erreur existent dans les tests ciblés, mais il manque une défaillance d’écriture provoquée dans le processus réel, visible pour l’utilisateur et présente dans les logs. | Rendre le fichier non inscriptible ou simuler un échec contrôlé ; fermer ; conserver l’éditeur sale ; afficher l’erreur ; vérifier request id, cause et nettoyage dans le log. |
| Utiliser les raccourcis clavier Muya | PARTIALLY PROVEN | Entrée, fusion/séparation, historique, sélection UTF-16, formatage, tableaux, tâches et IME sont ciblés par le harnais ; le clavier système et le focus de la fenêtre native ne sont pas prouvés. | Dans la fenêtre Freya, envoyer les vrais événements clavier ; vérifier caret/sélection, markdown produit, undo/redo, IME et persistance. Le résultat attendu doit être contrôlé à la fois dans l’UI et sur disque. |
| Coller du texte riche ou une image | NOT PROVEN | Le test de clipboard utilise le provider Freya et couvre du markup riche, mais ce n’est pas une preuve du clipboard système ni du chemin de drop d’un fichier/image. | Coller depuis le clipboard système puis déposer une image/fichier ; vérifier le nœud Muya, l’asset créé, le lien markdown, l’affichage et le rechargement après redémarrage. |
| Créer un dossier à la racine puis dans le dossier courant | PARTIALLY PROVEN | 'library_create_folder_freya_testing.rs' vérifie le ciblage racine/nested et l’absence de doublon ; l’interaction native, le rafraîchissement externe et la suppression/renommage complets restent à prouver. | Créer les deux dossiers dans une fenêtre propre ; vérifier les répertoires sur disque, l’arbre, la liste et le résultat après redémarrage. |
| Naviguer dans un dossier, revenir, changer la vue et gérer un dossier vide | PARTIALLY PROVEN | Les états grid/list, nested, empty et back sont couverts par 'library_settings_freya_testing.rs', mais pas comme une session utilisateur native finale avec historique et contenu fixture indépendant. | Ouvrir un dossier nested, passer grid/list, revenir avec le bouton et le raccourci attendu ; vérifier route, contenu et historique après un redémarrage. |
| Déplacer une note ou un dossier par glisser-déposer | PARTIALLY PROVEN | Une assertion vérifie le déplacement du fichier réel après drag ; la trajectoire/pointer réelle, le refus d’une cible invalide et le rafraîchissement de toutes les vues ne sont pas démontrés. | Glisser une entrée vers un dossier valide puis une cible invalide ; vérifier les chemins source/cible, l’arbre, la liste, l’état d’erreur et la récupération sans doublon. |
| Voir un changement externe du vault | NOT PROVEN | Le scan initial est exercé, mais un watcher/refresh fiable après création, renommage ou suppression hors de Freya n’est pas démontré. | Modifier le vault depuis un second processus ; observer l’apparition, le renommage puis la disparition sans relancer l’application ; conserver log d’événement et état disque. |
| Ouvrir la barre latérale, le rail et l’arbre de dossiers | PARTIALLY PROVEN | Les routes et contrôles existent, mais la géométrie réellement utilisable, le focus clavier, le scroll et le post-état après drag ne sont pas validés dans la fenêtre native. | Cliquer et utiliser les raccourcis dans une fenêtre réelle ; vérifier la cible active, l’arbre, la visibilité après scroll et le retour à l’état initial. |
| Rechercher une note puis ouvrir le résultat | PARTIALLY PROVEN | 'search_runtime_freya_testing.rs' cible le runtime de recherche, mais la requête live, le résultat correspondant, l’ouverture de la note, l’état vide et l’état erreur ne sont pas prouvés ensemble sur un index propre. | Indexer un fixture connu ; saisir une requête ; vérifier les résultats et le nombre attendu ; ouvrir le résultat ; tester aucun résultat, index vide et erreur de lecture ; vérifier les logs. |
| Changer le thème et la visibilité de navigation | PARTIALLY PROVEN | 'settings_effects_freya_testing.rs' cible les effets et la restauration ; il manque la preuve dans la fenêtre native et la confirmation que toutes les surfaces vanilla utilisent la préférence restaurée. | Modifier le thème et le rail ; fermer et relancer avec le même 'ELEPHANT_FREYA_PROFILE' ; vérifier UI, préférence canonique sur disque et état restauré. |
| Modifier une préférence d’éditeur ou de bibliothèque | PARTIALLY PROVEN | 'library_settings_freya_testing.rs' cible une préférence canonique et un JSON malformé ; la couverture de chaque contrôle vanilla, du focus, de l’annulation et de la restauration après redémarrage n’est pas complète. | Pour chaque contrôle vanilla : valeur avant/après dans l’UI, JSON canonique, redémarrage, valeur restaurée ; pour JSON invalide : surface d’erreur visible et application utilisable. |
| Ouvrir/fermer Settings et revenir à la bibliothèque | PARTIALLY PROVEN | Un test de cycle modal existe, mais le sélecteur de fermeture et le comportement Escape/focus doivent être prouvés sur la route produite, avec conservation de la route et des changements. | Ouvrir Settings, naviguer dans les sections vanilla, fermer par bouton puis Escape ; vérifier bibliothèque, focus, route et préférences. |
| Restaurer ou vider la corbeille du vault | NOT PROVEN | Les contrôles sont présents dans 'settings_vault.rs', mais aucun parcours complet de suppression → corbeille → restauration/vidage avec état disque et erreur n’est établi ici. | Supprimer une note fixture ; vérifier la corbeille ; restaurer et rouvrir ; répéter puis vider ; vérifier les fichiers, l’UI, les erreurs et le redémarrage. |
| Ouvrir le menu Create et choisir Drawing | PARTIALLY PROVEN | La présence de l’entrée 'Drawing' est ciblée, mais le cycle création → sauvegarde → réouverture → suppression n’est pas inclus dans une preuve vanilla complète de la même session. | Créer un dessin, enregistrer, fermer, relancer, rouvrir puis supprimer ; vérifier l’artefact réel et l’absence de résidu. |

## Preuves minimales à exécuter

Les commandes ci-dessous sont les commandes attendues pour produire les preuves ; elles n’ont pas été exécutées pendant cet audit.

### Vérification Freya ciblée

~~~bash
cargo test --manifest-path Elephant/freya/Cargo.toml --test freya_shell_acceptance -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test vault_registry_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test create_folder_note_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test library_create_folder_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test library_settings_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test sidebar_folder_navigation_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_keyboard_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_rich_interactions_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_lifecycle_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test search_runtime_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test settings_effects_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test settings_modal_lifecycle_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test drawing_freya_testing -- --test-threads=1 --nocapture
~~~

Puis la vérification cargo globale :

~~~bash
pnpm freya:check
pnpm freya:test
~~~

Ces commandes vérifient le code et le harnais Freya ; elles ne suffisent pas seules à prouver le parcours utilisateur.

### Preuve Freya native avec profil et vault propres

Préparer un vault fixture synthétique, puis lancer deux fois le même profil :

~~~bash
ELEPHANT_FREYA_PROFILE=/private/tmp/elephant-freya-vanilla-profile \
ELEPHANT_FREYA_VAULT=/private/tmp/elephant-freya-vanilla-vault \
cargo run --manifest-path Elephant/freya/Cargo.toml
~~~

La première exécution doit produire les actions et les artefacts attendus ; la seconde doit démontrer la reprise. Conserver :

- capture de la fenêtre ou vidéo courte des actions ;
- stdout/stderr Freya ;
- JSON de profil/registre ;
- arborescence du vault avant/après ;
- contenu markdown ou asset créé ;
- trace d’erreur pour au moins un chemin invalide.

### Acceptance desktop de l’application réelle

À exécuter depuis une révision propre, avec un fixture sans addon installé et avec les artefacts de logs conservés :

~~~bash
pnpm test:desktop:acceptance
pnpm test:desktop:acceptance:packaged
~~~

Le verdict vanilla doit être extrait des actions dossiers, notes, Muya, recherche, navigation et settings. La suite officielle des addons est hors périmètre et ne doit pas être utilisée pour déclarer Freya vanilla fonctionnelle.

## Conditions de sortie du statut PARTIALLY PROVEN

Un petit parcours peut passer à **PROVEN** seulement lorsque la même révision fournit :

1. une assertion ciblée qui échoue sur la régression visée et passe après correction ;
2. une exécution dans la fenêtre Freya native avec profil et vault propres ;
3. une vérification de l’effet réel sur disque ou dans le registre ;
4. une vérification après fermeture/redémarrage lorsque le parcours est persistant ;
5. une preuve du chemin d’erreur correspondant ;
6. les logs et artefacts conservés, avec leur chemin exact ;
7. pour la livraison desktop, l’acceptance non packagée et packagée quand le parcours le requiert.

À ce stade, aucun parcours de ce rapport ne doit être annoncé comme entièrement prouvé sur la seule présence des tests ou sur une comparaison pixel.

## Mise à jour de l’audit fonctionnel — 2026-08-16

Après récupération de `origin` et correction des interactions, la branche
`nsb/freya-native-migration` a été rejouée sur un vault temporaire réel. Les
parcours Freya couvrent maintenant les dossiers (ouverture, expansion,
renommage, suppression et drag-and-drop), l’éditeur Muya (titre, tags, pin,
undo/redo, mise en forme inline et flush), la recherche depuis les surfaces
principales, l’historique Back/Forward, le resize du rail et les effets
Settings avec persistance canonique.

Preuves exécutées :

```text
Freya integration loop (toutes les suites sauf differential_freya_capture) : PASS
pnpm freya:check : PASS (warnings Rust existants, aucun error)
cargo test --manifest-path Elephant/freya/Cargo.toml --lib -- --test-threads=1 : 143 passed
pnpm test:unit : 174 files passed, 27 skipped; 3239 tests passed, 171 skipped
node build/scripts/verify-agent-governance.mjs : PASS
pnpm test:vue-to-freya : PASS
differential_freya_capture avec fixture officiel : manifest status=passed,
  14 actions et 120 frames Freya capturées en 1280x684
```

L’artefact de capture est conservé sous
`/private/tmp/codex-freya-capture.sXQZGF/output/` (`manifest.json`,
`run-log.json`, 120 PNG et `runtime-error.log`). Il prouve le chemin de
production Freya, l’état et les hashes du vault ; il ne prouve pas encore
l’égalité Tauri/Freya.

Le statut global reste **PARTIALLY PROVEN**. L’acceptance desktop Tauri réelle
a été exécutée mais échoue encore sur `.elephant-physical-code-run` avec
`Create menu is incomplete` (`excalidrawLogo.visible: false`) avant le parcours
complet. Les preuves fenêtre Freya native, acceptance packagée, restart
inter-runtime et comparaison visuelle stricte restent à établir.

## Mise à jour fonctionnelle — 2026-08-16 (suite)

Le parcours code est maintenant implémenté avant son habillage : le bloc garde
son éditeur Muya éditable et délègue Copy/Run au service officiel versionné,
avec sortie, code de sortie et erreur visibles. Le sélecteur de coffres utilise
également le chemin de pointer press mesuré lorsque le texte ne remonte pas
vers la ligne accessible ; le changement de coffre et la persistance du
registre sont couverts par un test réel.

Preuves supplémentaires :

```text
cargo test --manifest-path Elephant/freya/Cargo.toml --test code_execution_freya_testing -- --test-threads=1 : 1 passed
cargo test --manifest-path Elephant/freya/Cargo.toml --test vault_registry_freya_testing -- --test-threads=1 : 1 passed
Freya integration loop (toutes les suites sauf differential_freya_capture) : PASS
pnpm test:desktop:acceptance : PASS sur le build Tauri dev final, 1216 logs
```

La compatibilité `muya-js` du paquet officiel épinglé est appliquée par
`build/scripts/sync-elephant-addons.mjs` avec une garde de source exacte ; elle
est donc rejouée en CI fraîche et ne dépend pas du cache local ignoré.

La capture native Tauri a produit des écrans réels, mais son parcours
différentiel reste **NOT PROVEN** : après les actions de recherche et de
navigation, le runner n’a pas trouvé `[data-testid="muya-runtime-editor"]` au
moment d’éditer la note et a bloqué les actions suivantes. Le parcours Freya
correspondant passe avec 14 actions et ses PNG sont conservés sous
`/private/tmp/codex-freya-differential-final/freya/`. Cela ne permet pas encore
de déclarer l’égalité pixel ou la parité complète Tauri/Freya.
