# Freya vanilla acceptance — gaps de preuve

Date de l’audit : 2026-08-16  
Branche : nsb/freya-native-migration  
HEAD observé : 188f5ff8b  
Périmètre : dossiers, notes, éditeur/Muya et settings. Ce document ne modifie
aucun code et ne transforme pas un test headless en preuve de parcours réel.

## Lecture du statut

- PROVEN : un scénario réel de test local possède une assertion explicite sur
  l’effet UI, l’état ou le fichier concerné.
- PARTIALLY PROVEN : la frontière Freya Testing est couverte, mais il manque
  au moins le processus réel, le redémarrage, le packaging, le parcours Tauri
  correspondant ou la comparaison visuelle.
- NOT PROVEN : aucune preuve suffisante n’est disponible pour le parcours
  complet sur le commit final.
- BLOCKED : une dépendance ou un échec concret empêche la preuve.

Le checkout était sale pendant l’audit : plusieurs fichiers Freya et tests
étaient modifiés ou non suivis. Une réussite observée avant commit ne constitue
donc pas une preuve du commit final.

## PROVEN

### Frontière Freya Testing — preuves ciblées existantes

Les suites suivantes contiennent des parcours avec fixture disque, arbre
d’accessibilité et assertions sur l’effet réel :

- Dossier : library_create_folder_freya_testing.rs vérifie Create → Folder à
  la racine puis dans un dossier imbriqué et contrôle les répertoires créés.
- Notes et dossiers : library_settings_freya_testing.rs couvre ouverture,
  historique retour, renommage, suppression, navigation imbriquée, déplacement
  par drag, pagination après scroll et visibilité d’un dossier dans le sidebar.
- Création de note : freya_shell_acceptance.rs vérifie Create → Note, le
  fichier Markdown sur disque et l’ouverture de l’éditeur.
- Muya : editor_keyboard_freya_testing.rs couvre Enter, Delete, Backspace,
  Ctrl/Cmd+End, undo/redo et sélection inter-nœuds ;
  editor_rich_interactions_freya_testing.rs couvre formatage, tâches,
  tableaux, clipboard et IME ; editor_lifecycle_freya_testing.rs couvre
  autosave, fermeture, erreur d’écriture, scroll et sauvegarde.
- Settings : settings_effects_freya_testing.rs vérifie thème et visibilité
  du rail avec persistance ; library_settings_freya_testing.rs vérifie la
  préférence autosave, la restauration au redémarrage logique et l’erreur de
  JSON invalide.

Cette preuve est limitée à freya-testing et à des fixtures temporaires. Elle
ne prouve ni l’application Freya packagée ni la parité Tauri/Freya.

## PARTIALLY PROVEN

### Dossiers

Les effets locaux principaux sont couverts : création, découverte, renommage,
suppression, déplacement et navigation imbriquée. Les gaps restent :

- parcours dans une fenêtre Freya réelle et non seulement TestingRunner ;
- modification externe du système de fichiers puis rafraîchissement/watch réel ;
- preuve de mouvement drag-over/drop avec les mêmes frames que Tauri ;
- comparaison du contenu, des erreurs et de la géométrie sur un checkout propre.

Commande ciblée :

~~~sh
cargo test --manifest-path Elephant/freya/Cargo.toml --test library_create_folder_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test library_settings_freya_testing -- --test-threads=1 --nocapture
~~~

### Notes

La création, l’ouverture, l’édition, l’autosave et plusieurs effets disque sont
assertés par les tests Freya. Il manque encore une preuve complète et propre
du parcours suivant : créer → nommer → éditer → autosave → arrêter le
processus → relancer le processus → vérifier le même contenu et les mêmes
états, puis exécuter le parcours équivalent dans Tauri.

Commandes ciblées :

~~~sh
cargo test --manifest-path Elephant/freya/Cargo.toml --test freya_shell_acceptance -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_lifecycle_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_keyboard_freya_testing -- --test-threads=1 --nocapture
~~~

### Muya / éditeur

Les tests couvrent désormais des chemins Muya substantiels, y compris IME,
clipboard simulé et historique. Ils ne prouvent pas encore :

- le clipboard de la fenêtre native réelle ;
- le drop de fichiers/images et la copie dans .assets ;
- les images, wiki-links, slash menu et toutes les structures Markdown ;
- la sélection, le caret et le scroll visuellement comparés à Muya/Tauri ;
- l’équivalence des erreurs et du redémarrage dans les deux runtimes.

Le test de clipboard peut être exécuté séparément pour identifier un éventuel
blocage de compilation/provider :

~~~sh
cargo test --manifest-path Elephant/freya/Cargo.toml --test editor_rich_interactions_freya_testing -- --test-threads=1 --nocapture
~~~

### Settings

Les préférences Appearance/Editor et certaines erreurs de persistance ont une
preuve headless. Le checkout contient aussi des chemins Freya pour le registre
de vaults et les add-ons, mais la couverture complète manque encore pour :

- tous les contrôles de chaque section et leur comportement clavier/souris ;
- renommage/icône et toutes les transitions du registre de vaults ;
- installation, désactivation, désinstallation, réinstallation et nettoyage
  runtime d’un add-on réel ;
- providers IA, sync et autres settings qui nécessitent un runtime externe ;
- rendu statique et motion parity du panneau avec le panneau Tauri.

Commandes ciblées :

~~~sh
cargo test --manifest-path Elephant/freya/Cargo.toml --test settings_effects_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test vault_registry_freya_testing -- --test-threads=1 --nocapture
cargo test --manifest-path Elephant/freya/Cargo.toml --test native_workspace_navigation_freya_testing -- --test-threads=1 --nocapture
pnpm test:official-addons:e2e
~~~

## NOT PROVEN

### Acceptance réelle et packaging

Aucune preuve actuelle ne permet d’affirmer que ces parcours passent sur un
commit propre et final dans l’application Freya distribuable. Les commandes à
exécuter avant toute conclusion sont :

~~~sh
pnpm freya:check
pnpm freya:test
pnpm test:desktop:acceptance
pnpm test:desktop:acceptance:packaged
~~~

La suite Freya complète doit être exécutée avec les suites d’intégration
isolées, comme dans .github/workflows/freya-native.yml, afin d’identifier
précisément une suite en échec :

~~~sh
for file in Elephant/freya/tests/*.rs; do
  suite="$(basename "$file" .rs)"
  [ "$suite" = "differential_freya_capture" ] && continue
  cargo test --manifest-path Elephant/freya/Cargo.toml --test "$suite" -- --test-threads=1
done
~~~

### Comparaison Tauri / Freya

Valider d’abord le scénario :

~~~sh
node --input-type=module -e 'import { loadScenario } from "./tools/freya-differential/lib/scenario.mjs"; await loadScenario("migration/freya/differential-scenarios.json"); console.log("scenario-valid")'
~~~

Puis exécuter le parcours orchestré avec le runner Tauri WebDriver du workflow
et le captureur Freya :

~~~sh
pnpm --dir tools/tauri-wdio-embedded --ignore-workspace install --frozen-lockfile
cd Elephant/backend/tauri
cargo tauri build --debug --no-bundle --features acceptance-wdio --config tauri.wdio.conf.json
cd ../..

root="$PWD/test-results/freya-differential-orchestrated"
rm -rf "$root"
mkdir -p "$root"
node tools/freya-differential/orchestrate.mjs \
  --tauri-command 'pnpm --dir tools/tauri-wdio-embedded --ignore-workspace exec wdio run wdio.conf.mjs' \
  --freya-command 'cargo test --manifest-path Elephant/freya/Cargo.toml --test differential_freya_capture -- --test-threads=1 --nocapture' \
  --scenario migration/freya/differential-scenarios.json \
  --output "$root" \
  --timeout-ms 600000 \
  --threshold 0.12 \
  --max-different-ratio 0.10
~~~

Le diagnostic pixel strict doit ensuite être exécuté séparément :

~~~sh
node tools/freya-differential/compare.mjs \
  "$root/comparison/tauri" \
  "$root/comparison/freya" \
  --reference-label tauri \
  --candidate-label freya \
  --threshold 0 \
  --max-different-ratio 0 \
  --report "$root/strict-comparison-report.json" \
  --diff-dir "$root/strict-diffs"
~~~

Les conditions minimales sont :

- orchestration-report.json.status == "ok" ;
- strict-comparison-report.json.status sans pixel-mismatch ;
- mêmes actions, frames, dimensions, états et fichiers persistés.

Un test fonctionnel vert ne compense pas un pixel diff en échec.

## BLOCKED

### Comparaison pixel stricte connue en échec

Un artefact officiel disponible pendant cet audit, sous
/private/tmp/ci-31868047552/test-results/freya-differential-orchestrated/orchestration-report.json,
rapporte :

- status: "mismatch" ;
- 14 actions comparées sur 14 ;
- 120 frames comparées sur 120 ;
- 120 frames en échec pixel ;
- maxDifferentRatio: 0 et threshold: 0.

Cet artefact prouve que la comparaison pixel stricte n’est pas verte pour cette
exécution. Il ne faut pas écrire « CI verte » tant qu’un nouveau rapport ne
montre pas explicitement le statut de succès.

### Gate CI à vérifier

Dans .github/workflows/freya-native.yml, le code capture le résultat du
comparateur strict dans strict_compare_status, mais la valeur n’est pas
utilisée pour faire échouer le job. Le workflow vérifie ensuite principalement
report.status issu de la gate perceptuelle threshold 0.12 / max ratio 0.10.

Pour une preuve stricte locale, le contrôle doit au minimum être équivalent à :

~~~sh
strict_compare_status=$?
test "$strict_compare_status" -eq 0
~~~

Sans cette vérification et sans lecture du JSON strict, un job peut être
fonctionnellement accepté alors que la comparaison pixel stricte échoue.

## Conclusion

Les parcours dossiers, notes, Muya et settings disposent maintenant de
preuves headless ciblées, mais la qualification globale reste
PARTIALLY PROVEN. Les preuves manquantes sont le commit propre final, le
processus Freya/Tauri réel, le packaging, le redémarrage complet, les erreurs
et la comparaison visuelle/motion. La comparaison pixel connue étant en
mismatch, aucune affirmation de CI verte ou de parité visuelle complète
n’est justifiée.

## Nouvelle exécution contrôlée — 2026-08-16

L’acceptance desktop Tauri a été relancée depuis la branche de migration avec
le runner réel, Vite et la fenêtre native. Le résultat reste **BLOCKED / NOT
PROVEN** : le runner atteint `renderer:ready`, puis échoue sur
`.elephant-physical-code-run` avec `Create menu is incomplete`
(`excalidrawLogo.visible: false`) avant la capture des checkpoints suivants.

Artefacts de cette exécution :

- `test-results/acceptance/latest-tauri.log` ;
- `test-results/acceptance/latest.json`.

En parallèle, les suites Freya fonctionnelles hors différentiel sont passées,
et la capture Freya officielle a produit 14 actions et 120 frames avec état et
hashes du vault : `/private/tmp/codex-freya-capture.sXQZGF/output/`. Cette
preuve ne remplace pas le parcours Tauri manquant et ne permet pas de déclarer
la CI ou la parité visuelle vertes.
