# TODO — Validation exhaustive des addons Elephant

> **Statut : spécification de tests, aucun test n'est considéré comme réussi tant qu'il n'a pas été exécuté avec preuves.**
>
> Audit préparatoire effectué le **6 août 2026** à partir de :
>
> - l'application `SorbetUP/ElephantNote`, branche courante `main` ;
> - le catalogue officiel `SorbetUP/Elephant-Addons`, branche `main` ;
> - les manifestes et points d'entrée des addons officiels ;
> - les modules natifs déjà présents dans `Elephant/front/src/modules`.
>
> Le lien historique `assistant/ci-tauri-pr-flow` fourni comme point de départ n'existe plus au moment de cet audit. Cette TODO doit donc être exécutée sur le commit exact de la future build Bazzite, en enregistrant son SHA.

---

## 0. Objectif et règle de preuve

Le but n'est pas de vérifier uniquement que l'addon « s'ouvre ». Chaque addon doit être testé comme un composant susceptible de :

- modifier le vault ou des fichiers auxiliaires ;
- ajouter des commandes, vues, menus, raccourcis ou paramètres ;
- conserver un état persistant ;
- lancer un sidecar ou un processus externe ;
- accéder au réseau ;
- dépendre d'un autre addon ;
- réagir aux modifications du système de fichiers ;
- entrer en conflit avec une fonctionnalité native ;
- dégrader la sauvegarde en temps réel, le drag-and-drop, les liens ou la synchronisation ;
- crasher sans faire crasher l'application entière.

### 0.1 Signification des cases

- `[ ]` : non exécuté ou preuve insuffisante.
- Une case ne peut être cochée qu'avec un résultat reproductible et une preuve conservée.
- Un test qui « passe » dans un mock mais pas sur l'exécutable final reste `[ ]`.
- Un test backend ne remplace pas un test UI réel.
- Un test UI ne remplace pas la vérification des fichiers réellement écrits sur le disque.
- Un test sur X11 ne prouve pas le fonctionnement sous Wayland/Bazzite.
- Un test sur le serveur de développement ne prouve pas le fonctionnement de l'AppImage/Flatpak final.

### 0.2 Preuves obligatoires par test

Pour chaque identifiant de test, conserver :

- SHA de l'application ;
- SHA et version de l'addon ;
- format testé : AppImage, Flatpak ou autre ;
- version de Bazzite, noyau, session Wayland, environnement de bureau ;
- architecture CPU/GPU ;
- vault de fixture utilisé ;
- étapes exactes ;
- résultat attendu ;
- résultat observé ;
- capture ou vidéo pour les interactions UI ;
- logs terminal, Tauri, renderer et addon ;
- diff récursif du vault avant/après ;
- liste des processus avant/après pour les addons avec sidecar ;
- consommation mémoire/CPU maximale quand applicable ;
- statut `PASS`, `FAIL`, `BLOCKED` ou `NOT SUPPORTED` ;
- issue GitHub associée en cas d'échec.

### 0.3 Isolation minimale

- [ ] **QA-ISO-001** — Créer un nouveau vault propre pour chaque test destructif.
- [ ] **QA-ISO-002** — Créer un nouveau profil/configuration Elephant pour chaque test d'installation, migration ou permission.
- [ ] **QA-ISO-003** — Réinitialiser les variables d'environnement, ports et processus sidecar entre deux tests.
- [ ] **QA-ISO-004** — Ne jamais réutiliser un index, cache ou stockage persistant sans que le test l'indique explicitement.
- [ ] **QA-ISO-005** — Exécuter les tests indépendamment et dans un ordre aléatoire pour détecter les dépendances cachées.
- [ ] **QA-ISO-006** — Rejouer au moins une fois tout échec pour confirmer sa reproductibilité.
- [ ] **QA-ISO-007** — Tester aussi avec un chemin de vault contenant espaces, accents, emoji et caractères Unicode.
- [ ] **QA-ISO-008** — Tester avec un vault sur disque local et, si supporté, sur un montage externe.
- [ ] **QA-ISO-009** — Vérifier qu'un addon ne lit ni n'écrit dans un autre vault ouvert précédemment.
- [ ] **QA-ISO-010** — Vérifier qu'aucun test ne dépend d'Internet sauf lorsque ce besoin est explicitement testé.

---

# 1. Socle Elephant à rejouer avant et après chaque addon

Ces tests sont le minimum demandé pour l'application elle-même. Ils doivent être exécutés :

1. sans addon ;
2. avec l'addon testé seul ;
3. avec toutes ses dépendances ;
4. avec l'ensemble des addons officiels activés ;
5. après désactivation et désinstallation de l'addon.

## 1.1 Ouverture et choix du vault

- [ ] **CORE-VAULT-001** — Lancer l'exécutable final depuis une session Bazzite Wayland.
- [ ] **CORE-VAULT-002** — Vérifier qu'aucun crash, écran blanc ou boucle de chargement ne survient au premier lancement.
- [ ] **CORE-VAULT-003** — Choisir un vault existant depuis le sélecteur natif.
- [ ] **CORE-VAULT-004** — Créer un nouveau vault depuis le sélecteur.
- [ ] **CORE-VAULT-005** — Annuler le sélecteur sans crash ni chemin invalide mémorisé.
- [ ] **CORE-VAULT-006** — Refuser un chemin sans permission et afficher une erreur utile.
- [ ] **CORE-VAULT-007** — Refuser un fichier choisi à la place d'un dossier.
- [ ] **CORE-VAULT-008** — Rouvrir automatiquement le dernier vault valide.
- [ ] **CORE-VAULT-009** — Gérer proprement un dernier vault supprimé ou déplacé hors de l'application.
- [ ] **CORE-VAULT-010** — Changer de vault sans conserver l'état, les notes, l'index ou les résultats de l'ancien vault.
- [ ] **CORE-VAULT-011** — Vérifier les permissions Flatpak/portal si une build Flatpak est produite.
- [ ] **CORE-VAULT-012** — Vérifier l'ouverture par double-clic et lancement terminal de l'AppImage si une AppImage est produite.

## 1.2 Création de fichiers et dossiers

- [ ] **CORE-FS-001** — Créer une note à la racine depuis l'UI.
- [ ] **CORE-FS-002** — Créer une note dans un dossier imbriqué depuis l'UI.
- [ ] **CORE-FS-003** — Créer un dossier à la racine depuis l'UI.
- [ ] **CORE-FS-004** — Créer un sous-dossier depuis l'UI.
- [ ] **CORE-FS-005** — Vérifier immédiatement l'existence et le contenu sur le disque.
- [ ] **CORE-FS-006** — Gérer les doublons de noms sans écrasement silencieux.
- [ ] **CORE-FS-007** — Gérer les noms avec espaces, accents, emoji et caractères autorisés.
- [ ] **CORE-FS-008** — Refuser ou normaliser clairement les noms interdits.
- [ ] **CORE-FS-009** — Renommer une note et vérifier le renommage disque.
- [ ] **CORE-FS-010** — Renommer un dossier et vérifier tous les descendants.
- [ ] **CORE-FS-011** — Déplacer une note et vérifier le chemin réel.
- [ ] **CORE-FS-012** — Déplacer un dossier et vérifier tous les descendants.
- [ ] **CORE-FS-013** — Supprimer une note sans supprimer un homonyme ailleurs.
- [ ] **CORE-FS-014** — Supprimer un dossier vide puis non vide avec confirmation correcte.
- [ ] **CORE-FS-015** — Vérifier qu'aucune action ne sort de la racine du vault via `..`, lien symbolique ou chemin absolu.

## 1.3 Synchronisation avec les modifications manuelles dans l'explorateur de fichiers

- [ ] **CORE-WATCH-001** — Créer manuellement une note dans l'explorateur : apparition automatique dans Elephant.
- [ ] **CORE-WATCH-002** — Créer manuellement un dossier : apparition automatique.
- [ ] **CORE-WATCH-003** — Modifier manuellement une note fermée : contenu correct à l'ouverture.
- [ ] **CORE-WATCH-004** — Modifier manuellement une note ouverte : mise à jour ou conflit explicite, jamais perte silencieuse.
- [ ] **CORE-WATCH-005** — Renommer manuellement une note : mise à jour sans doublon fantôme.
- [ ] **CORE-WATCH-006** — Renommer manuellement un dossier : arborescence mise à jour.
- [ ] **CORE-WATCH-007** — Déplacer manuellement une note : ancienne entrée supprimée, nouvelle entrée créée.
- [ ] **CORE-WATCH-008** — Supprimer manuellement une note ouverte : état UI sûr et explicite.
- [ ] **CORE-WATCH-009** — Supprimer manuellement un dossier : suppression récursive reflétée.
- [ ] **CORE-WATCH-010** — Effectuer une rafale de 100 créations/modifications/suppressions sans crash ni événement perdu.
- [ ] **CORE-WATCH-011** — Vérifier l'absence de boucle de watcher lorsque l'application écrit elle-même.
- [ ] **CORE-WATCH-012** — Refaire les cas avec fichiers générés par chaque addon.

## 1.4 Édition, sauvegarde en temps réel et durabilité

- [ ] **CORE-SAVE-001** — Ouvrir une note existante sans crash.
- [ ] **CORE-SAVE-002** — Saisir du texte simple à vitesse humaine.
- [ ] **CORE-SAVE-003** — Vérifier la sauvegarde automatique sur le disque sans action manuelle.
- [ ] **CORE-SAVE-004** — Vérifier que le contenu disque correspond exactement à l'éditeur après chaque pause d'autosave.
- [ ] **CORE-SAVE-005** — Changer de note immédiatement après une frappe et vérifier qu'aucun caractère n'est perdu.
- [ ] **CORE-SAVE-006** — Fermer la fenêtre immédiatement après une frappe et vérifier la durabilité.
- [ ] **CORE-SAVE-007** — Tuer le processus après la confirmation visuelle de sauvegarde et vérifier le fichier.
- [ ] **CORE-SAVE-008** — Simuler un échec d'écriture et vérifier qu'aucun faux indicateur « sauvegardé » n'est affiché.
- [ ] **CORE-SAVE-009** — Reprendre après rétablissement des permissions/disque.
- [ ] **CORE-SAVE-010** — Éditer simultanément depuis Elephant et un éditeur externe : conflit explicite et aucune perte silencieuse.
- [ ] **CORE-SAVE-011** — Tester une note vide, très courte, très longue et contenant de longues lignes.
- [ ] **CORE-SAVE-012** — Tester undo/redo au clavier et à la souris.
- [ ] **CORE-SAVE-013** — Tester sélection, couper, copier, coller, glisser-déposer de texte et IME.
- [ ] **CORE-SAVE-014** — Vérifier la stabilité du curseur pendant l'autosave et les événements de watcher.
- [ ] **CORE-SAVE-015** — Ouvrir successivement 100 notes sans crash ni fuite mémoire majeure.

## 1.5 Markdown en temps réel

- [ ] **CORE-MD-001** — Titres, gras, italique, barré, citations, listes et séparateurs.
- [ ] **CORE-MD-002** — Listes imbriquées, tâches cochables et indentation.
- [ ] **CORE-MD-003** — Liens Markdown, liens wiki et ancres.
- [ ] **CORE-MD-004** — Images relatives et absolues autorisées.
- [ ] **CORE-MD-005** — Code inline et blocs de code clôturés.
- [ ] **CORE-MD-006** — Tableaux et retours à la ligne.
- [ ] **CORE-MD-007** — Markdown incomplet ou malformé sans crash.
- [ ] **CORE-MD-008** — Passage édition/rendu sans déplacement injustifié du curseur.
- [ ] **CORE-MD-009** — Le contenu source sauvegardé reste stable après réouverture.
- [ ] **CORE-MD-010** — Les addons Wiki, Graph, Sites, AI et Code execution ne réécrivent pas involontairement le Markdown.

## 1.6 Excalidraw natif

- [ ] **CORE-EXCAL-001** — Créer un dessin Excalidraw depuis l'UI.
- [ ] **CORE-EXCAL-002** — Dessiner, saisir du texte, déplacer, redimensionner et supprimer.
- [ ] **CORE-EXCAL-003** — Sauvegarder puis rouvrir sans perte.
- [ ] **CORE-EXCAL-004** — Insérer/référencer le dessin dans une note.
- [ ] **CORE-EXCAL-005** — Vérifier les fichiers réellement créés dans le vault.
- [ ] **CORE-EXCAL-006** — Modifier un dessin existant depuis son lien.
- [ ] **CORE-EXCAL-007** — Tester thème clair/sombre et redimensionnement.
- [ ] **CORE-EXCAL-008** — Tester dessin volumineux et import d'image.
- [ ] **CORE-EXCAL-009** — Vérifier qu'aucun addon ne remplace ou duplique les commandes Excalidraw.
- [ ] **CORE-EXCAL-010** — Vérifier la synchronisation des fichiers Excalidraw via l'addon Sync.

## 1.7 Paramètres et barres UI

- [ ] **CORE-SET-001** — Ouvrir tous les panneaux de paramètres sans crash.
- [ ] **CORE-SET-002** — Modifier, sauvegarder et retrouver chaque paramètre après redémarrage.
- [ ] **CORE-SET-003** — Distinguer paramètres globaux, par vault et par addon.
- [ ] **CORE-SET-004** — Valider les valeurs et refuser les entrées invalides.
- [ ] **CORE-SET-005** — Réinitialiser un paramètre sans supprimer les autres.
- [ ] **CORE-SET-006** — Masquer la barre latérale.
- [ ] **CORE-SET-007** — Réafficher la barre latérale sans raccourci inaccessible.
- [ ] **CORE-SET-008** — Passer la barre en largeur complète/réduite.
- [ ] **CORE-SET-009** — Redimensionner et conserver la largeur après redémarrage.
- [ ] **CORE-SET-010** — Vérifier qu'une vue d'addon ne bloque pas hide/full.
- [ ] **CORE-SET-011** — Vérifier qu'un addon désactivé disparaît des réglages, commandes et barres.
- [ ] **CORE-SET-012** — Vérifier qu'aucun secret n'apparaît en clair dans les logs ou exports de réglages.

## 1.8 Drag-and-drop de fichiers et images

- [ ] **CORE-DND-001** — Déposer un fichier depuis l'explorateur dans un dossier visible de l'application.
- [ ] **CORE-DND-002** — Vérifier la copie/move attendue et le fichier réel sur disque.
- [ ] **CORE-DND-003** — Déposer plusieurs fichiers.
- [ ] **CORE-DND-004** — Gérer les collisions sans écrasement silencieux.
- [ ] **CORE-DND-005** — Déposer un gros fichier avec progression ou état explicite.
- [ ] **CORE-DND-006** — Déposer un dossier si supporté, sinon refuser proprement.
- [ ] **CORE-DND-007** — Déposer une image à une position précise dans une note.
- [ ] **CORE-DND-008** — Vérifier l'insertion exacte au curseur/caret.
- [ ] **CORE-DND-009** — Vérifier la copie dans le dossier d'assets prévu et le lien relatif correct.
- [ ] **CORE-DND-010** — Annuler puis refaire l'insertion d'image.
- [ ] **CORE-DND-011** — Rouvrir la note et vérifier le rendu de l'image.
- [ ] **CORE-DND-012** — Déplacer/supprimer l'image externe d'origine : la copie du vault doit rester valide.
- [ ] **CORE-DND-013** — Déposer un fichier générique dans une note et insérer un lien cliquable à la position voulue.
- [ ] **CORE-DND-014** — Cliquer un PDF avec addon PDF installé/activé : ouverture dans l'addon.
- [ ] **CORE-DND-015** — Cliquer un PDF sans addon PDF : ouverture par l'application système par défaut.
- [ ] **CORE-DND-016** — Addon PDF installé mais désactivé/crashé : fallback système sans blocage.
- [ ] **CORE-DND-017** — Tester images, PDF, texte, archive et type inconnu.
- [ ] **CORE-DND-018** — Vérifier que les addons Sites, Sync, OCR et Knowledge traitent correctement les nouveaux assets.

---

# 2. Contrat commun obligatoire pour chaque addon

Chaque addon officiel doit avoir une suite dédiée qui réutilise cette section.

## 2.1 Catalogue, manifeste et intégrité

- [ ] **ADDON-COMMON-001** — L'entrée du catalogue existe et pointe vers un dossier réel.
- [ ] **ADDON-COMMON-002** — `id`, `name`, `version`, `entry`, plateformes et dépendances sont cohérents entre catalogue et manifeste.
- [ ] **ADDON-COMMON-003** — La version du package téléchargé correspond au catalogue et au manifeste.
- [ ] **ADDON-COMMON-004** — Le checksum est vérifié avant installation.
- [ ] **ADDON-COMMON-005** — Un checksum faux bloque l'installation.
- [ ] **ADDON-COMMON-006** — Un manifeste invalide ou incomplet est refusé sans crash.
- [ ] **ADDON-COMMON-007** — Un point d'entrée absent est refusé.
- [ ] **ADDON-COMMON-008** — Un ID dupliqué ne produit pas deux instances.
- [ ] **ADDON-COMMON-009** — Une version inférieure/égale/supérieure est comparée correctement.
- [ ] **ADDON-COMMON-010** — Aucun addon non déclaré ne peut être chargé depuis le vault ou le cache.

## 2.2 Cycle de vie

- [ ] **ADDON-COMMON-011** — Installer sur un profil propre.
- [ ] **ADDON-COMMON-012** — Activer sans redémarrage si le runtime le promet.
- [ ] **ADDON-COMMON-013** — Redémarrer et vérifier l'activation persistante.
- [ ] **ADDON-COMMON-014** — Désactiver et vérifier la disparition immédiate des contributions UI.
- [ ] **ADDON-COMMON-015** — Réactiver sans doublon de commandes, listeners ou vues.
- [ ] **ADDON-COMMON-016** — Mettre à jour depuis la version précédente avec migration contrôlée.
- [ ] **ADDON-COMMON-017** — Interrompre une mise à jour et revenir à un état cohérent.
- [ ] **ADDON-COMMON-018** — Refuser un downgrade incompatible ou le gérer explicitement.
- [ ] **ADDON-COMMON-019** — Désinstaller sans laisser de processus, listener ou commande active.
- [ ] **ADDON-COMMON-020** — Documenter et vérifier la politique de conservation/suppression de ses données.
- [ ] **ADDON-COMMON-021** — Réinstaller et vérifier qu'aucun état corrompu n'est réutilisé.
- [ ] **ADDON-COMMON-022** — Rejouer install/enable/disable/uninstall 20 fois sans fuite ni duplication.

## 2.3 Dépendances

- [ ] **ADDON-COMMON-023** — Installer automatiquement ou demander explicitement les dépendances requises.
- [ ] **ADDON-COMMON-024** — Bloquer l'activation si une dépendance manque.
- [ ] **ADDON-COMMON-025** — Bloquer une version incompatible.
- [ ] **ADDON-COMMON-026** — Charger les dépendances dans le bon ordre.
- [ ] **ADDON-COMMON-027** — Refuser les cycles de dépendances.
- [ ] **ADDON-COMMON-028** — Empêcher la désactivation d'une dépendance utilisée ou désactiver proprement les dépendants.
- [ ] **ADDON-COMMON-029** — Afficher une erreur compréhensible et réparable.
- [ ] **ADDON-COMMON-030** — Ne jamais continuer dans un état partiellement fonctionnel silencieux.

## 2.4 Permissions et sécurité

- [ ] **ADDON-COMMON-031** — Demander uniquement les permissions déclarées.
- [ ] **ADDON-COMMON-032** — Refuser une API non autorisée.
- [ ] **ADDON-COMMON-033** — Vérifier chaque permission en succès puis en refus.
- [ ] **ADDON-COMMON-034** — Bloquer `../`, chemins absolus et traversée par lien symbolique.
- [ ] **ADDON-COMMON-035** — Limiter les écritures au vault ou aux chemins explicitement autorisés.
- [ ] **ADDON-COMMON-036** — Limiter les requêtes réseau aux domaines/ports autorisés par CSP.
- [ ] **ADDON-COMMON-037** — Ne pas exposer tokens, clés, contenu de notes ou chemins privés dans les logs.
- [ ] **ADDON-COMMON-038** — Ne pas exécuter de contenu du vault comme code sans action explicite.
- [ ] **ADDON-COMMON-039** — Résister aux manifestes, messages IPC, JSON et contenus malformés.
- [ ] **ADDON-COMMON-040** — Vérifier les permissions après mise à jour du manifeste.
- [ ] **ADDON-COMMON-041** — Vérifier l'isolation entre deux addons.
- [ ] **ADDON-COMMON-042** — Vérifier qu'un addon crashé ne peut pas contourner les permissions au redémarrage.

## 2.5 État, paramètres et migrations

- [ ] **ADDON-COMMON-043** — Valeurs par défaut exactes.
- [ ] **ADDON-COMMON-044** — Validation des types, bornes, URL, chemins et secrets.
- [ ] **ADDON-COMMON-045** — Persistance après redémarrage.
- [ ] **ADDON-COMMON-046** — Isolation par vault quand l'état est lié au vault.
- [ ] **ADDON-COMMON-047** — Migration depuis chaque version encore supportée.
- [ ] **ADDON-COMMON-048** — Récupération après état tronqué ou JSON corrompu.
- [ ] **ADDON-COMMON-049** — Réinitialisation sans casser l'addon.
- [ ] **ADDON-COMMON-050** — Suppression ou conservation documentée lors de la désinstallation.
- [ ] **ADDON-COMMON-051** — Aucun secret en clair dans le vault.
- [ ] **ADDON-COMMON-052** — Aucun état d'un vault A réutilisé dans un vault B.

## 2.6 UI, commandes et accessibilité

- [ ] **ADDON-COMMON-053** — Toutes les commandes déclarées sont enregistrées une seule fois.
- [ ] **ADDON-COMMON-054** — Toutes les vues déclarées s'ouvrent, se ferment et se restaurent.
- [ ] **ADDON-COMMON-055** — Les commandes disparaissent à la désactivation.
- [ ] **ADDON-COMMON-056** — Aucun raccourci ne remplace silencieusement un raccourci natif.
- [ ] **ADDON-COMMON-057** — Navigation clavier, focus visible, lecteur d'écran et contraste minimal.
- [ ] **ADDON-COMMON-058** — Thèmes clair/sombre.
- [ ] **ADDON-COMMON-059** — Fenêtre petite, grande, maximisée et mise à l'échelle.
- [ ] **ADDON-COMMON-060** — Hide/full de la barre fonctionne avec la vue ouverte.
- [ ] **ADDON-COMMON-061** — Les erreurs sont visibles et actionnables.
- [ ] **ADDON-COMMON-062** — Les notifications ne se dupliquent pas.
- [ ] **ADDON-COMMON-063** — Aucun écran blanc après erreur de rendu.
- [ ] **ADDON-COMMON-064** — État vide, chargement, succès, erreur et retry sont testés.

## 2.7 Watchers, concurrence et intégrité des données

- [ ] **ADDON-COMMON-065** — Réagir correctement aux créations externes.
- [ ] **ADDON-COMMON-066** — Réagir correctement aux modifications externes.
- [ ] **ADDON-COMMON-067** — Réagir correctement aux renommages/déplacements externes.
- [ ] **ADDON-COMMON-068** — Réagir correctement aux suppressions externes.
- [ ] **ADDON-COMMON-069** — Ne pas créer de boucle d'écriture avec le watcher.
- [ ] **ADDON-COMMON-070** — Supporter deux fenêtres/processus si l'application l'autorise.
- [ ] **ADDON-COMMON-071** — Utiliser des écritures atomiques pour index, métadonnées et sorties importantes.
- [ ] **ADDON-COMMON-072** — Récupérer un fichier temporaire ou interrompu.
- [ ] **ADDON-COMMON-073** — Ne jamais écraser silencieusement une modification concurrente.
- [ ] **ADDON-COMMON-074** — Ne jamais modifier une note simplement parce qu'elle a été lue/indexée.

## 2.8 Crash, limites et récupération

- [ ] **ADDON-COMMON-075** — Exception dans `activate` : application toujours utilisable.
- [ ] **ADDON-COMMON-076** — Exception pendant une commande : erreur isolée.
- [ ] **ADDON-COMMON-077** — Boucle ou tâche longue : timeout appliqué.
- [ ] **ADDON-COMMON-078** — Limite mémoire appliquée ou échec contrôlé.
- [ ] **ADDON-COMMON-079** — Limite CPU appliquée ou échec contrôlé.
- [ ] **ADDON-COMMON-080** — Annulation réelle des tâches et streams.
- [ ] **ADDON-COMMON-081** — Redémarrage de l'addon sans redémarrer toute l'application si prévu.
- [ ] **ADDON-COMMON-082** — Pas de processus orphelin après fermeture.
- [ ] **ADDON-COMMON-083** — Logs structurés avec addon, version, action et erreur.
- [ ] **ADDON-COMMON-084** — Aucun contenu sensible dans un rapport de crash.
- [ ] **ADDON-COMMON-085** — Relance après crash sans corruption de l'état.

## 2.9 Réseau et mode hors ligne

À exécuter pour tout addon ayant la permission réseau.

- [ ] **ADDON-COMMON-086** — Fonctionnement nominal sur réseau stable.
- [ ] **ADDON-COMMON-087** — Démarrage totalement hors ligne.
- [ ] **ADDON-COMMON-088** — DNS invalide, refus de connexion et timeout.
- [ ] **ADDON-COMMON-089** — TLS/certificat invalide refusé.
- [ ] **ADDON-COMMON-090** — Réponse HTTP invalide, vide, tronquée ou trop grande.
- [ ] **ADDON-COMMON-091** — Retry borné avec backoff, sans boucle infinie.
- [ ] **ADDON-COMMON-092** — Annulation pendant une requête.
- [ ] **ADDON-COMMON-093** — Proxy/VPN si officiellement supporté.
- [ ] **ADDON-COMMON-094** — Pas d'envoi réseau non sollicité au démarrage.
- [ ] **ADDON-COMMON-095** — Les données envoyées sont visibles/consenties selon le contrat fonctionnel.

## 2.10 Plateformes et sidecars

- [ ] **ADDON-COMMON-096** — Respecter la liste de plateformes du manifeste.
- [ ] **ADDON-COMMON-097** — Ne pas proposer l'installation sur une plateforme non supportée.
- [ ] **ADDON-COMMON-098** — Sidecar absent : erreur utile, application intacte.
- [ ] **ADDON-COMMON-099** — Sidecar de mauvaise architecture : erreur utile.
- [ ] **ADDON-COMMON-100** — Sidecar sans permission d'exécution : diagnostic précis.
- [ ] **ADDON-COMMON-101** — Lancement, handshake, requêtes, arrêt et relance du sidecar.
- [ ] **ADDON-COMMON-102** — Aucun port/processus orphelin.
- [ ] **ADDON-COMMON-103** — Les arguments et variables d'environnement ne divulguent pas de secret.
- [ ] **ADDON-COMMON-104** — Packaging réel du sidecar dans AppImage/Flatpak.
- [ ] **ADDON-COMMON-105** — Fonctionnement sous Wayland sans dépendance X11 implicite.

---

# 3. Inventaire des addons officiels et tests spécifiques

## 3.1 Dashboard — `elephant.dashboard` — v1.0.0

### Contrat observé

- Vue `dashboard`.
- Commande `dashboard.open`.
- Lit, cherche et peut écrire dans le vault.
- Stockage persistant.
- Intègre notes, calendrier, tâches et notes rapides.

### Modifications/effets à contrôler

- Notes rapides ou tâches créées dans le vault.
- Configuration/layout conservés dans le stockage de l'addon.
- Listeners sur notes, calendrier et fichiers.
- Contribution de vue et commande.

### TODO spécifique

- [ ] **DASH-001** — Installer et ouvrir depuis la commande puis depuis l'UI.
- [ ] **DASH-002** — Vérifier l'état vide sur un vault neuf.
- [ ] **DASH-003** — Afficher les notes attendues sans lire un autre vault.
- [ ] **DASH-004** — Afficher les événements calendrier et tâches attendus.
- [ ] **DASH-005** — Créer une note rapide et vérifier le fichier exact sur disque.
- [ ] **DASH-006** — Modifier/supprimer la note rapide et vérifier le disque.
- [ ] **DASH-007** — Vérifier la mise à jour en temps réel après création externe d'une note.
- [ ] **DASH-008** — Vérifier la mise à jour après renommage/suppression externe.
- [ ] **DASH-009** — Réorganiser les widgets si supporté et vérifier la persistance.
- [ ] **DASH-010** — Tester les données malformées provenant d'un autre module.
- [ ] **DASH-011** — Ouvrir une note depuis le dashboard au bon chemin.
- [ ] **DASH-012** — Vérifier qu'une vue Dashboard ne bloque pas la barre hide/full.
- [ ] **DASH-013** — Désactiver : aucune écriture ou mise à jour en arrière-plan.
- [ ] **DASH-014** — Désinstaller : documenter et vérifier le sort des notes créées et du layout.
- [ ] **DASH-015** — Comparer avec tout dashboard natif pour éviter commande/vue dupliquée.

---

## 3.2 AI Base — `elephant.ai` — v2.0.1

### Contrat observé

- Base commune pour génération, embeddings et streaming.
- Fournisseurs locaux Ollama et API OpenAI-compatible.
- Accès réseau à localhost et domaines explicitement autorisés.
- Peut lire, chercher et écrire dans le vault.
- Stockage de configuration.
- Limites annoncées : 256 MB, 60 % CPU, timeout 120 s.

### Modifications/effets à contrôler

- Paramètres de fournisseur et modèle.
- Secrets API.
- Éventuels contenus insérés/modifiés dans les notes.
- Connexions réseau et streams.
- API consommée par AI Chat, AI Search, AI OCR et Open Models.

### TODO spécifique

- [ ] **AI-BASE-001** — Configurer Ollama local et lister les modèles.
- [ ] **AI-BASE-002** — Générer une réponse complète avec Ollama.
- [ ] **AI-BASE-003** — Tester le streaming token par token.
- [ ] **AI-BASE-004** — Annuler un stream et vérifier l'arrêt réel de la requête.
- [ ] **AI-BASE-005** — Tester un endpoint OpenAI-compatible valide.
- [ ] **AI-BASE-006** — Tester URL, clé et modèle invalides.
- [ ] **AI-BASE-007** — Vérifier qu'une clé n'apparaît jamais dans logs, UI secondaire ou vault.
- [ ] **AI-BASE-008** — Tester les embeddings et leur dimension/format.
- [ ] **AI-BASE-009** — Changer de fournisseur/modèle pendant une session.
- [ ] **AI-BASE-010** — Redémarrer et retrouver la configuration sans fuite de secret.
- [ ] **AI-BASE-011** — Vérifier qu'Ollama local n'envoie aucune donnée au cloud.
- [ ] **AI-BASE-012** — Vérifier la CSP : domaine non autorisé bloqué.
- [ ] **AI-BASE-013** — Tester serveur absent, lent, réponse tronquée, statut 4xx/5xx.
- [ ] **AI-BASE-014** — Tester contexte vide, énorme et contenu Unicode.
- [ ] **AI-BASE-015** — Vérifier qu'aucune note n'est écrite sans action explicite.
- [ ] **AI-BASE-016** — Écrire dans une note via une action explicite et vérifier autosave/disque.
- [ ] **AI-BASE-017** — Tester quota/limites de tokens et message explicite.
- [ ] **AI-BASE-018** — Vérifier timeout, mémoire et CPU réels.
- [ ] **AI-BASE-019** — Vérifier l'API de dépendance avec chaque addon consommateur.
- [ ] **AI-BASE-020** — Désactiver pendant une requête et vérifier l'annulation/nettoyage.

---

## 3.3 AI Chat — `elephant.ai-chat` — v1.0.0

### Contrat observé

- Dépend de `elephant.ai >= 2.0.1`.
- Chat contextuel, historique et contexte de note.
- Lit/recherche le vault ; pas de permission d'écriture vault déclarée.
- Stockage persistant et vue dédiée.

### Modifications/effets à contrôler

- Historique de conversation dans le stockage addon.
- Lecture du contenu de la note/sélection.
- Appels à AI Base.
- Aucun changement de note sans permission/commande explicite fournie par un autre composant.

### TODO spécifique

- [ ] **AI-CHAT-001** — Activation bloquée sans AI Base.
- [ ] **AI-CHAT-002** — Activation avec version compatible d'AI Base.
- [ ] **AI-CHAT-003** — Ouvrir/fermer/restaurer la vue.
- [ ] **AI-CHAT-004** — Conversation sans note ouverte.
- [ ] **AI-CHAT-005** — Conversation avec note courante comme contexte.
- [ ] **AI-CHAT-006** — Conversation avec sélection précise comme contexte.
- [ ] **AI-CHAT-007** — Vérifier exactement quelles données sont envoyées au fournisseur.
- [ ] **AI-CHAT-008** — Historique conservé après redémarrage.
- [ ] **AI-CHAT-009** — Suppression d'une conversation et absence dans le stockage.
- [ ] **AI-CHAT-010** — Note renommée/supprimée pendant une conversation.
- [ ] **AI-CHAT-011** — Contexte long tronqué de manière explicite et stable.
- [ ] **AI-CHAT-012** — Stream, annulation, retry et changement de fournisseur.
- [ ] **AI-CHAT-013** — Liens/citations vers notes ouvrent le bon fichier.
- [ ] **AI-CHAT-014** — Vérifier l'absence d'écriture cachée dans le vault.
- [ ] **AI-CHAT-015** — Désactiver AI Base pendant le chat : état d'erreur récupérable.
- [ ] **AI-CHAT-016** — Désinstaller et vérifier la politique de conservation de l'historique.

---

## 3.4 AI Search — `elephant.ai-search` — v1.1.0

### Contrat observé

- Dépend de `elephant.knowledge >= 1.1.0` et `elephant.ai >= 2.0.1`.
- Recherche sémantique et recommandations.
- Lit/recherche le vault ; ne déclare pas d'écriture vault.
- Utilise stockage et vue.

### Modifications/effets à contrôler

- Requêtes d'embeddings via AI Base.
- Lecture de l'index Knowledge.
- État de recherche/recommandations dans le stockage.
- Aucun changement du contenu des notes.

### TODO spécifique

- [ ] **AI-SEARCH-001** — Activation bloquée si une dépendance manque.
- [ ] **AI-SEARCH-002** — Recherche avec index vide, absent, valide, ancien et corrompu.
- [ ] **AI-SEARCH-003** — Résultats pertinents sur fixture contrôlée avec vérité attendue.
- [ ] **AI-SEARCH-004** — Ouvrir le résultat au bon fichier.
- [ ] **AI-SEARCH-005** — Recommandations stables et explicables sur fixture fixe.
- [ ] **AI-SEARCH-006** — Mise à jour après création/modification externe puis réindexation.
- [ ] **AI-SEARCH-007** — Suppression/renommage : aucun résultat fantôme.
- [ ] **AI-SEARCH-008** — Changement de modèle d'embedding et invalidation/migration de l'index.
- [ ] **AI-SEARCH-009** — Panne fournisseur d'embeddings avec fallback/erreur contrôlée.
- [ ] **AI-SEARCH-010** — Recherche Unicode, accents, symboles et requête vide.
- [ ] **AI-SEARCH-011** — Grand vault : latence, mémoire et annulation.
- [ ] **AI-SEARCH-012** — Isolation stricte entre deux vaults.
- [ ] **AI-SEARCH-013** — Vérifier qu'aucune note n'est modifiée.
- [ ] **AI-SEARCH-014** — Désactivation de Knowledge/AI pendant une recherche.
- [ ] **AI-SEARCH-015** — Rejouer avec Sync actif et modifications concurrentes.

---

## 3.5 AI OCR — `elephant.ai-ocr` — v0.1.0

### Contrat observé

- Dépend de `elephant.ai >= 2.0.1`.
- OCR d'images et PDF scannés via le fournisseur configuré.
- Ne déclare pas d'écriture directe dans le vault.
- Lit/recherche le vault, utilise stockage, vues et commandes.

### Modifications/effets à contrôler

- Lecture des images/PDF.
- Envoi éventuel des documents au fournisseur AI.
- Résultat OCR en mémoire/stockage.
- Insertion éventuelle dans l'éditeur via API hôte, à clarifier et tester contre les permissions.

### TODO spécifique

- [ ] **AI-OCR-001** — OCR PNG, JPEG et format image supporté.
- [ ] **AI-OCR-002** — OCR PDF scanné mono-page.
- [ ] **AI-OCR-003** — OCR PDF scanné multi-page.
- [ ] **AI-OCR-004** — Image tournée, sombre, bruitée et haute résolution.
- [ ] **AI-OCR-005** — Français, anglais, accents, chiffres et mise en page.
- [ ] **AI-OCR-006** — PDF contenant déjà une couche texte.
- [ ] **AI-OCR-007** — Fichier invalide, chiffré, vide ou trop gros.
- [ ] **AI-OCR-008** — Vérifier le consentement avant envoi à un fournisseur distant.
- [ ] **AI-OCR-009** — Vérifier les octets/données réellement envoyés.
- [ ] **AI-OCR-010** — Annulation, timeout et fournisseur indisponible.
- [ ] **AI-OCR-011** — Copier le résultat sans modifier la note.
- [ ] **AI-OCR-012** — Si insertion supportée, insérer exactement au curseur et vérifier le disque.
- [ ] **AI-OCR-013** — Confirmer qu'aucune écriture vault ne contourne le manifeste.
- [ ] **AI-OCR-014** — Tester coexistence avec drag-and-drop image.
- [ ] **AI-OCR-015** — Tester coexistence avec l'addon/lecteur PDF et fallback système.

---

## 3.6 Wiki — `elephant.wiki` — v1.0.0

### Contrat observé

- Liens wiki, backlinks et mentions non liées.
- Lit, cherche et peut écrire dans le vault.
- Addon natif annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- Création éventuelle de notes manquantes.
- Modification éventuelle des liens lors de renommages.
- Index/état de backlinks dans le stockage.
- Commandes et vues de navigation.

### TODO spécifique

- [ ] **WIKI-001** — Parser `[[Note]]`.
- [ ] **WIKI-002** — Parser alias, ancre et titre contenant espaces/Unicode.
- [ ] **WIKI-003** — Ignorer correctement les pseudo-liens dans code et zones non interprétables.
- [ ] **WIKI-004** — Ouvrir un lien existant au bon fichier.
- [ ] **WIKI-005** — Gérer un lien ambigu ou plusieurs notes homonymes.
- [ ] **WIKI-006** — Créer une note manquante uniquement après action explicite.
- [ ] **WIKI-007** — Vérifier le chemin et contenu de la note créée.
- [ ] **WIKI-008** — Backlinks exacts après ajout/suppression de lien.
- [ ] **WIKI-009** — Mentions non liées exactes, sans faux positifs dans code.
- [ ] **WIKI-010** — Renommer une note depuis Elephant et vérifier la politique de mise à jour des liens.
- [ ] **WIKI-011** — Renommer depuis l'explorateur et vérifier absence de corruption.
- [ ] **WIKI-012** — Liens cycliques, auto-liens et graphes très denses.
- [ ] **WIKI-013** — Grand vault : temps d'indexation et mémoire.
- [ ] **WIKI-014** — Aucun changement de Markdown lors d'une simple lecture.
- [ ] **WIKI-015** — Conflit/coexistence avec le support wiki natif de l'éditeur.
- [ ] **WIKI-016** — Coexistence Wiki + Graph + Knowledge + Sync.
- [ ] **WIKI-017** — Vérifier réellement desktop puis matrices Android/iOS annoncées.

---

## 3.7 Graph — `elephant.graph` — v1.1.0

### Contrat observé

- Graphe interactif du vault.
- Lit/recherche le vault ; ne déclare pas d'écriture.
- Addon natif annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- Cache/layout de graphe dans le stockage.
- Vue et commandes.
- Aucun changement du contenu des notes.

### TODO spécifique

- [ ] **GRAPH-001** — Construire nœuds/arêtes depuis une fixture connue.
- [ ] **GRAPH-002** — Vérifier nombre exact de nœuds/arêtes.
- [ ] **GRAPH-003** — Notes isolées, liens cassés, alias et cycles.
- [ ] **GRAPH-004** — Zoom, pan, sélection, recentrage et filtres.
- [ ] **GRAPH-005** — Cliquer un nœud et ouvrir la bonne note.
- [ ] **GRAPH-006** — Création/modification externe reflétée après refresh prévu.
- [ ] **GRAPH-007** — Renommage/suppression sans nœud fantôme.
- [ ] **GRAPH-008** — Grand vault : limite, virtualisation, FPS, CPU et mémoire.
- [ ] **GRAPH-009** — Persistance éventuelle du layout et isolation par vault.
- [ ] **GRAPH-010** — Vérifier qu'aucune note n'est écrite.
- [ ] **GRAPH-011** — Coexistence avec le module Graph natif sans deux commandes/vues concurrentes.
- [ ] **GRAPH-012** — Coexistence avec Wiki et Knowledge.
- [ ] **GRAPH-013** — Hide/full de la barre et redimensionnement.
- [ ] **GRAPH-014** — Vérifier réellement les plateformes annoncées.

---

## 3.8 Knowledge — `elephant.knowledge` — v1.1.0

### Contrat observé

- Indexation du vault.
- Écrit l'index dans `.elephant/knowledge/index.json`.
- Rafraîchissement manuel et périodique.
- Lit, cherche et écrit dans le vault.
- Addon natif annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- Création/modification de `.elephant/knowledge/index.json`.
- Timers de rafraîchissement.
- Cache/état dans le stockage.
- Interaction avec AI Search, Wiki, Graph, Sync et watchers.

### TODO spécifique

- [ ] **KNOW-001** — Premier index sur vault neuf et chemin exact du fichier.
- [ ] **KNOW-002** — Vérifier le schéma et la version de l'index.
- [ ] **KNOW-003** — Écriture atomique : jamais d'index partiellement remplacé.
- [ ] **KNOW-004** — Rafraîchissement manuel.
- [ ] **KNOW-005** — Rafraîchissement périodique et arrêt réel à la désactivation.
- [ ] **KNOW-006** — Indexation incrémentale après création/modification/renommage/suppression.
- [ ] **KNOW-007** — Exclure `.elephant/knowledge` lui-même pour éviter la récursion.
- [ ] **KNOW-008** — Exclure ou traiter explicitement binaires, images, PDF et dossiers cachés.
- [ ] **KNOW-009** — Index corrompu/tronqué : reconstruction contrôlée.
- [ ] **KNOW-010** — Interruption en plein write : récupération.
- [ ] **KNOW-011** — Deux fenêtres/indexeurs : verrouillage ou coordination.
- [ ] **KNOW-012** — Aucun changement du contenu des notes.
- [ ] **KNOW-013** — Grand vault : durée, mémoire, CPU et annulation.
- [ ] **KNOW-014** — Isolation par vault et aucun index réutilisé.
- [ ] **KNOW-015** — Sync actif : éviter boucle et conflits sur l'index.
- [ ] **KNOW-016** — Documenter si `.elephant/knowledge` doit être synchronisé ou ignoré.
- [ ] **KNOW-017** — Désinstallation : conserver/supprimer l'index selon politique explicite.
- [ ] **KNOW-018** — Coexistence avec tout index natif existant.

---

## 3.9 Open Models — `elephant.open-models` — v1.0.0

### Contrat observé

- Dépend de `elephant.ai >= 2.0.1`.
- Catalogue, installation et sélection de modèles open source.
- Accès à registry Ollama, Hugging Face et Ollama local.
- Ne déclare pas d'écriture vault.
- Stockage persistant, commandes et vue.

### Modifications/effets à contrôler

- Téléchargements hors vault.
- Inventaire local de modèles.
- Choix du modèle actif.
- Espace disque, cache, processus Ollama et réseau.

### TODO spécifique

- [ ] **MODELS-001** — Charger les catalogues autorisés.
- [ ] **MODELS-002** — Rechercher/filtrer un modèle.
- [ ] **MODELS-003** — Installer un petit modèle et vérifier son emplacement réel.
- [ ] **MODELS-004** — Vérifier progression, taille et checksum.
- [ ] **MODELS-005** — Annuler puis reprendre un téléchargement.
- [ ] **MODELS-006** — Disque plein et espace insuffisant.
- [ ] **MODELS-007** — Réponse de catalogue malveillante/malformée.
- [ ] **MODELS-008** — Mode hors ligne avec catalogue/cache existant puis absent.
- [ ] **MODELS-009** — Sélectionner le modèle et vérifier AI Base.
- [ ] **MODELS-010** — Changer de modèle pendant une requête.
- [ ] **MODELS-011** — Supprimer un modèle et libérer réellement l'espace.
- [ ] **MODELS-012** — Ne pas supprimer un modèle utilisé sans avertissement.
- [ ] **MODELS-013** — Ollama absent, arrêté ou version incompatible.
- [ ] **MODELS-014** — Aucun fichier écrit dans le vault.
- [ ] **MODELS-015** — Aucun code de modèle exécuté par simple affichage du catalogue.
- [ ] **MODELS-016** — Désinstallation de l'addon : modèles conservés/supprimés selon politique explicite.

---

## 3.10 Codex Connection — `elephant.codex-connection` — v0.1.0

### Contrat observé

- Bridge d'agent avec sidecar `elephant-codex-server --stdio`.
- Capacités sidecar : lecture, écriture et recherche du vault.
- Desktop uniquement.
- Permission native et stockage persistant.

### Modifications/effets à contrôler

- Processus sidecar.
- Protocole stdio/IPC.
- Fichiers du vault modifiés par l'agent.
- Configuration/identifiants de connexion.
- Accès large au vault.

### TODO spécifique

- [ ] **CODEX-001** — Vérifier que le sidecar est réellement inclus dans AppImage/Flatpak Bazzite.
- [ ] **CODEX-002** — Lancer, effectuer le handshake et arrêter proprement.
- [ ] **CODEX-003** — Sidecar absent, non exécutable, mauvaise architecture et crash immédiat.
- [ ] **CODEX-004** — Lire une note autorisée.
- [ ] **CODEX-005** — Rechercher dans le vault.
- [ ] **CODEX-006** — Écrire/modifier une note avec action/consentement attendu.
- [ ] **CODEX-007** — Vérifier autosave, watcher et contenu disque après écriture agent.
- [ ] **CODEX-008** — Bloquer lecture/écriture hors vault.
- [ ] **CODEX-009** — Bloquer traversée par `..`, chemin absolu et symlink.
- [ ] **CODEX-010** — Message protocolaire invalide, trop grand, incomplet et ordre inattendu.
- [ ] **CODEX-011** — Annuler une opération longue.
- [ ] **CODEX-012** — Tuer le sidecar pendant une écriture : aucune note tronquée.
- [ ] **CODEX-013** — Redémarrer sans processus orphelin.
- [ ] **CODEX-014** — Deux instances Elephant : ports/stdio/processus isolés.
- [ ] **CODEX-015** — Aucun secret dans ligne de commande, environnement ou logs.
- [ ] **CODEX-016** — Fonctionnement sans AI Base, puisqu'aucune dépendance n'est déclarée.
- [ ] **CODEX-017** — Ne pas proposer sur Android/iOS.
- [ ] **CODEX-018** — Conflit de permissions avec Code execution et Sites.
- [ ] **CODEX-019** — Désactivation immédiate : plus aucune capacité agent.
- [ ] **CODEX-020** — Audit exhaustif de toutes les opérations que le protocole expose.

---

## 3.11 Sync — `elephant.sync` — v2.0.0

### Contrat observé

- Synchronisation avec métadonnées sous `.sync`.
- Modes dossier, Git et cloud.
- Options de chiffrement.
- Lit, cherche et écrit dans le vault.
- Réseau et API natives.
- Annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- `.sync` et éventuels journaux/états.
- Fichiers du vault poussés/tirés/supprimés.
- Credentials et clés de chiffrement.
- Processus réseau et timers.
- Conflits avec autosave, watchers et dossiers générés par addons.

### TODO spécifique

- [ ] **SYNC-001** — Initialiser chaque mode avec fixture propre.
- [ ] **SYNC-002** — Vérifier exactement les fichiers créés sous `.sync`.
- [ ] **SYNC-003** — Premier push d'un vault.
- [ ] **SYNC-004** — Premier pull vers un vault vide.
- [ ] **SYNC-005** — Création/modification/renommage/déplacement/suppression dans les deux sens.
- [ ] **SYNC-006** — Modifications externes via explorateur détectées et synchronisées.
- [ ] **SYNC-007** — Autosave en temps réel pendant une synchronisation.
- [ ] **SYNC-008** — Même note modifiée sur deux appareils : conflit explicite sans perte.
- [ ] **SYNC-009** — Conflit fichier/dossier de même nom.
- [ ] **SYNC-010** — Gros fichiers, images, PDF, Excalidraw et fichiers inconnus.
- [ ] **SYNC-011** — Liens relatifs et assets restent valides après sync.
- [ ] **SYNC-012** — Chiffrement nominal et absence de contenu en clair côté distant.
- [ ] **SYNC-013** — Mauvaise clé : aucun écrasement local.
- [ ] **SYNC-014** — Rotation de clé/migration si supportée.
- [ ] **SYNC-015** — Coupure réseau à chaque étape, reprise idempotente.
- [ ] **SYNC-016** — Pas de boucle de synchronisation due aux métadonnées `.sync`.
- [ ] **SYNC-017** — Mode dossier : source indisponible, montage retiré, permissions.
- [ ] **SYNC-018** — Mode Git : repo propre, dirty, conflit, remote absent, auth refusée.
- [ ] **SYNC-019** — Mode cloud : timeout, 4xx/5xx, quota, réponse invalide.
- [ ] **SYNC-020** — Vérifier que credentials/clés ne sont pas écrits en clair dans le vault/logs.
- [ ] **SYNC-021** — Exclusions explicites pour `.elephant/knowledge`, `.elephant-site`, caches et addon storage.
- [ ] **SYNC-022** — Tester les dossiers générés `Calendar` et `Imported/Google Keep`.
- [ ] **SYNC-023** — Deux synchronisations simultanées : verrouillage.
- [ ] **SYNC-024** — Arrêt de l'application pendant sync : récupération atomique.
- [ ] **SYNC-025** — Désactivation : aucun timer/réseau restant.
- [ ] **SYNC-026** — Désinstallation : politique de conservation de `.sync`.
- [ ] **SYNC-027** — Vérifier réellement desktop puis matrices Android/iOS annoncées.
- [ ] **SYNC-028** — Coexistence avec tout module Sync natif sans double moteur.

---

## 3.12 Calendar — `elephant.calendar` — v1.0.0

### Contrat observé

- Vue calendrier et commande d'ouverture.
- Création, ouverture et replanification de notes par date.
- Dossier cible par défaut `Calendar`.
- Lit, cherche et écrit dans le vault.
- Addon natif annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- Création/renommage/déplacement de notes dans le dossier calendrier.
- Métadonnées de dates.
- Préférences de vue.
- Conflit potentiel avec le module Calendar natif.

### TODO spécifique

- [ ] **CAL-001** — Ouvrir la vue mensuelle/hebdomadaire/journalière si disponibles.
- [ ] **CAL-002** — Créer une note à une date et vérifier chemin/nom/contenu exacts.
- [ ] **CAL-003** — Créer automatiquement le dossier `Calendar` s'il manque.
- [ ] **CAL-004** — Refuser/gérer un fichier nommé `Calendar` à la place du dossier.
- [ ] **CAL-005** — Ouvrir une note existante depuis une date.
- [ ] **CAL-006** — Deux notes pour la même date : règle explicite.
- [ ] **CAL-007** — Replanifier par drag-and-drop et vérifier le disque.
- [ ] **CAL-008** — Collision de nom lors de la replanification.
- [ ] **CAL-009** — Créer/modifier/renommer externement et rafraîchir la vue.
- [ ] **CAL-010** — Dates limites de mois/année et année bissextile.
- [ ] **CAL-011** — Fuseau Europe/Paris, changement heure d'été/hiver.
- [ ] **CAL-012** — Changement de fuseau sans déplacer incorrectement une date locale.
- [ ] **CAL-013** — Configurer un autre dossier cible et vérifier isolation.
- [ ] **CAL-014** — Template invalide ou absent.
- [ ] **CAL-015** — Aucun doublon après double-clic/commande répétée.
- [ ] **CAL-016** — Coexistence avec Dashboard et Recently Edited.
- [ ] **CAL-017** — Coexistence avec le module Calendar natif sans doubles vues/commandes.
- [ ] **CAL-018** — Sync des notes calendrier et conflits multi-appareils.
- [ ] **CAL-019** — Vérifier réellement les plateformes annoncées.

---

## 3.13 Sites — `elephant.sites` — v1.1.0

### Contrat observé

- Publication de notes en site statique.
- Sidecar `elephant-sites-server --stdio`.
- Écrit sous `.elephant-site`.
- Accès réseau, natif, desktop uniquement.

### Modifications/effets à contrôler

- Processus sidecar.
- Dossier `.elephant-site`.
- Sorties HTML/CSS/JS/assets générées.
- Réseau de publication/preview.
- Lecture des notes et assets privés.

### TODO spécifique

- [ ] **SITES-001** — Sidecar réellement packagé et exécutable sur Bazzite.
- [ ] **SITES-002** — Générer un site minimal depuis une note.
- [ ] **SITES-003** — Vérifier tous les fichiers créés sous `.elephant-site`.
- [ ] **SITES-004** — Rendu titres, listes, liens, code, tableaux et images.
- [ ] **SITES-005** — Copier/réécrire correctement les assets relatifs.
- [ ] **SITES-006** — Liens wiki et liens vers fichiers/PDF : politique explicite.
- [ ] **SITES-007** — Plusieurs notes, navigation et liens cassés.
- [ ] **SITES-008** — Preview local, ouverture navigateur et arrêt serveur.
- [ ] **SITES-009** — Rebuild incrémental après modification.
- [ ] **SITES-010** — Supprimer une page source et retirer la sortie obsolète.
- [ ] **SITES-011** — Ne jamais publier une note non sélectionnée/privée.
- [ ] **SITES-012** — Bloquer path traversal via titre, lien ou asset.
- [ ] **SITES-013** — Contenu Markdown/HTML hostile sans exécution inattendue.
- [ ] **SITES-014** — Réseau absent, timeout, auth et échec de publication.
- [ ] **SITES-015** — Interruption pendant build : sortie précédente intacte ou état explicite.
- [ ] **SITES-016** — Grand site : durée, mémoire, logs et annulation.
- [ ] **SITES-017** — Pas de processus/port orphelin.
- [ ] **SITES-018** — Sync actif : décider si `.elephant-site` est exclu ou synchronisé.
- [ ] **SITES-019** — Désinstallation : politique de conservation de la sortie.
- [ ] **SITES-020** — Ne pas proposer sur Android/iOS.
- [ ] **SITES-021** — Coexistence avec tout module Sites natif.

---

## 3.14 Code execution — `elephant.code-execution`

### Blocage observé

- **Le catalogue annonce `2.2.1`.**
- **Le manifeste annonce `2.2.0`.**
- Cette divergence doit bloquer publication/installation jusqu'à correction ou règle de résolution explicite.

### Contrat observé

- Exécution de plusieurs runtimes.
- Accès natif.
- Lit/écrit le vault et le stockage.
- Desktop uniquement.
- Limites annoncées : 512 MB, 80 % CPU, timeout 300 s.
- L'existence, le nom et le packaging du sidecar doivent être confirmés par le code/build, car le contrat catalogue/manifeste n'est pas entièrement cohérent.

### Modifications/effets à contrôler

- Processus enfants/interpréteurs.
- Répertoire de travail.
- Fichiers potentiellement écrits dans le vault.
- Sorties stdout/stderr.
- Variables d'environnement et secrets.
- État des blocs de code dans l'éditeur.

### TODO spécifique

- [ ] **CODE-001** — Corriger/clarifier la divergence catalogue `2.2.1` / manifeste `2.2.0`.
- [ ] **CODE-002** — Ajouter un test CI bloquant toute divergence future.
- [ ] **CODE-003** — Inventorier exactement chaque runtime annoncé et son binaire.
- [ ] **CODE-004** — Vérifier le packaging de chaque runtime/sidecar sur Bazzite.
- [ ] **CODE-005** — Exécuter un bloc minimal par langage supporté.
- [ ] **CODE-006** — Vérifier stdout, stderr, code de sortie et durée.
- [ ] **CODE-007** — Stopper manuellement une exécution infinie.
- [ ] **CODE-008** — Appliquer réellement timeout 300 s.
- [ ] **CODE-009** — Appliquer ou contrôler limites mémoire/CPU.
- [ ] **CODE-010** — Runtime absent ou mauvaise version : diagnostic précis.
- [ ] **CODE-011** — Gros stdout/stderr : streaming, limite et UI non bloquée.
- [ ] **CODE-012** — Entrée Unicode, binaire, vide et code syntaxiquement invalide.
- [ ] **CODE-013** — Définir/vérifier le répertoire de travail.
- [ ] **CODE-014** — Vérifier chaque fichier créé/modifié par le code.
- [ ] **CODE-015** — Bloquer sortie du vault si le sandbox le promet.
- [ ] **CODE-016** — Tester commandes destructives, fork bomb et accès réseau selon politique.
- [ ] **CODE-017** — Ne jamais injecter tokens/secrets de l'application dans l'environnement.
- [ ] **CODE-018** — Annuler en fermant la note ou désactivant l'addon.
- [ ] **CODE-019** — Pas de processus enfant orphelin après crash/fermeture.
- [ ] **CODE-020** — Exécuter uniquement après action explicite, jamais au rendu d'une note.
- [ ] **CODE-021** — Recharger une note avec résultats intégrés sans divergence DOM/crash.
- [ ] **CODE-022** — Copier/effacer output et conserver le Markdown source.
- [ ] **CODE-023** — Coexistence avec Markdown temps réel et autosave.
- [ ] **CODE-024** — Conflit de permissions avec Codex Connection.
- [ ] **CODE-025** — Ne pas proposer sur Android/iOS.
- [ ] **CODE-026** — Désinstallation pendant/entre exécutions et nettoyage complet.

---

## 3.15 Google Keep Import — `elephant.google-keep-import` — v1.0.0

### Contrat observé

- Import de Google Takeout JSON/HTML vers Markdown.
- Dossier cible par défaut `Imported/Google Keep`.
- Lit/écrit le vault et le stockage.
- Accès natif.
- Desktop uniquement.

### Modifications/effets à contrôler

- Création du dossier cible.
- Notes Markdown, pièces jointes et images importées.
- Éventuel journal d'import/état de déduplication.
- Dates, labels, checklists et métadonnées converties.

### TODO spécifique

- [ ] **KEEP-001** — Importer une fixture JSON minimale.
- [ ] **KEEP-002** — Importer une fixture HTML minimale.
- [ ] **KEEP-003** — Confirmer si archive Takeout directe est supportée ; tester ou refuser clairement.
- [ ] **KEEP-004** — Vérifier le dossier cible exact et sa création.
- [ ] **KEEP-005** — Notes texte, titre vide, note vide et note longue.
- [ ] **KEEP-006** — Checklists cochées/non cochées et ordre.
- [ ] **KEEP-007** — Labels, couleurs, épinglé, archivé et corbeille selon politique documentée.
- [ ] **KEEP-008** — Dates de création/modification et fuseaux.
- [ ] **KEEP-009** — Images, audio, dessins et autres pièces jointes disponibles.
- [ ] **KEEP-010** — Liens relatifs valides après déplacement du vault.
- [ ] **KEEP-011** — Unicode, emoji, RTL et encodage invalide.
- [ ] **KEEP-012** — Noms identiques/collisions sans écrasement.
- [ ] **KEEP-013** — Réimport identique : idempotence ou doublons explicitement gérés.
- [ ] **KEEP-014** — Import partiellement déjà réalisé.
- [ ] **KEEP-015** — JSON/HTML invalide ou tronqué avec rapport d'erreur.
- [ ] **KEEP-016** — Interruption au milieu : rollback ou reprise cohérente.
- [ ] **KEEP-017** — Bloquer path traversal dans noms/pièces jointes.
- [ ] **KEEP-018** — Ne jamais modifier les fichiers Takeout source.
- [ ] **KEEP-019** — Produire un résumé : importés, ignorés, erreurs et chemins.
- [ ] **KEEP-020** — Vérifier chaque sortie sur disque puis ouverture dans Elephant.
- [ ] **KEEP-021** — Sync immédiat du dossier importé sans perte/boucle.
- [ ] **KEEP-022** — Désinstallation : aucun impact sur les notes importées.
- [ ] **KEEP-023** — Ne pas proposer sur Android/iOS.

---

## 3.16 Recently Edited — `elephant.recently-edited` — v0.1.0

### Contrat observé

- Vue des notes récemment modifiées.
- Lit/recherche le vault ; ne déclare pas d'écriture vault.
- Stockage persistant.
- Annoncé sur desktop, Android et iOS.

### Modifications/effets à contrôler

- Liste/cache et préférences dans le stockage.
- Listener sur événements d'édition et/ou mtimes.
- Contribution de vue/commande.
- Aucun changement des notes.

### TODO spécifique

- [ ] **RECENT-001** — État vide sur vault neuf.
- [ ] **RECENT-002** — Éditer une note dans Elephant : apparition immédiate.
- [ ] **RECENT-003** — Éditer une note depuis l'explorateur/éditeur externe : règle observée correcte.
- [ ] **RECENT-004** — Ordre exact par date/événement sur fixture contrôlée.
- [ ] **RECENT-005** — Égalité de timestamps et précision du système de fichiers.
- [ ] **RECENT-006** — Fuseau et changement d'heure sans ordre incohérent.
- [ ] **RECENT-007** — Ouvrir un item au bon fichier.
- [ ] **RECENT-008** — Renommer/déplacer : item mis à jour, pas de doublon.
- [ ] **RECENT-009** — Supprimer : item retiré ou marqué explicitement.
- [ ] **RECENT-010** — Exclure dossiers, fichiers cachés, métadonnées et binaires non-notes.
- [ ] **RECENT-011** — Limite configurable et persistance.
- [ ] **RECENT-012** — Grand vault et rafale d'éditions : latence/mémoire.
- [ ] **RECENT-013** — Aucun fichier de note modifié.
- [ ] **RECENT-014** — Coexistence avec Dashboard et Calendar.
- [ ] **RECENT-015** — Désactivation : listener arrêté.
- [ ] **RECENT-016** — Vérifier réellement les plateformes annoncées.

---

# 4. Matrices d'interactions obligatoires

## 4.1 Addons dépendants de la pile AI

- [ ] **MATRIX-AI-001** — AI Base seul.
- [ ] **MATRIX-AI-002** — AI Base + AI Chat.
- [ ] **MATRIX-AI-003** — AI Base + Knowledge + AI Search.
- [ ] **MATRIX-AI-004** — AI Base + AI OCR.
- [ ] **MATRIX-AI-005** — AI Base + Open Models.
- [ ] **MATRIX-AI-006** — Tous les addons AI ensemble.
- [ ] **MATRIX-AI-007** — Désactiver AI Base alors que chaque dépendant est ouvert.
- [ ] **MATRIX-AI-008** — Mettre à jour AI Base avec dépendants installés.
- [ ] **MATRIX-AI-009** — Changer fournisseur/modèle et vérifier chaque dépendant.
- [ ] **MATRIX-AI-010** — Vérifier qu'une erreur fournisseur ne cascade pas en crash global.

## 4.2 Wiki, Graph et Knowledge

- [ ] **MATRIX-KG-001** — Même fixture produit liens/backlinks/nœuds cohérents.
- [ ] **MATRIX-KG-002** — Renommage depuis Elephant.
- [ ] **MATRIX-KG-003** — Renommage externe.
- [ ] **MATRIX-KG-004** — Suppression et restauration.
- [ ] **MATRIX-KG-005** — Rafale de modifications.
- [ ] **MATRIX-KG-006** — Aucun double indexeur natif/addon.
- [ ] **MATRIX-KG-007** — Pas de boucle entre mise à jour de l'index et Sync.
- [ ] **MATRIX-KG-008** — Résultats AI Search cohérents après changement de graphe.

## 4.3 Fichiers, assets, PDF et publication

- [ ] **MATRIX-FILE-001** — Drop image → lien Markdown → Knowledge → Sync → Sites.
- [ ] **MATRIX-FILE-002** — Drop PDF → lien → lecteur PDF addon si présent.
- [ ] **MATRIX-FILE-003** — Lecteur PDF absent/désactivé/crashé → fallback système.
- [ ] **MATRIX-FILE-004** — OCR d'un PDF déposé sans modifier le fichier source.
- [ ] **MATRIX-FILE-005** — Site publié avec images/fichiers liés.
- [ ] **MATRIX-FILE-006** — Déplacement externe d'asset et comportement des liens.
- [ ] **MATRIX-FILE-007** — Sync de l'asset pendant qu'une note est sauvegardée.
- [ ] **MATRIX-FILE-008** — Excalidraw lié et publié/synchronisé selon politique.

## 4.4 Addons écrivant des dossiers spéciaux

Vérifier la propriété, le schéma, l'exclusion/inclusion Sync et le nettoyage pour :

- [ ] **MATRIX-PATH-001** — `.elephant/knowledge/index.json` — Knowledge.
- [ ] **MATRIX-PATH-002** — `.sync` — Sync.
- [ ] **MATRIX-PATH-003** — `Calendar` — Calendar.
- [ ] **MATRIX-PATH-004** — `.elephant-site` — Sites.
- [ ] **MATRIX-PATH-005** — `Imported/Google Keep` — Google Keep Import.
- [ ] **MATRIX-PATH-006** — Dossier d'assets de l'application.
- [ ] **MATRIX-PATH-007** — Stockage persistant hors vault de chaque addon.
- [ ] **MATRIX-PATH-008** — Collision si l'utilisateur crée manuellement un fichier à la place d'un dossier spécial.
- [ ] **MATRIX-PATH-009** — Collision entre deux versions d'un addon.
- [ ] **MATRIX-PATH-010** — Sauvegarde/restauration de ces chemins.

## 4.5 Fonctions natives portant le même domaine

Le front contient déjà des modules natifs pour plusieurs domaines. Pour chacun, décider et prouver l'un des modèles suivants :

1. l'addon est un wrapper déclaratif autour du module natif ;
2. l'addon remplace le module natif ;
3. les deux coexistent avec responsabilités distinctes.

Aucun domaine ne doit charger deux moteurs, deux watchers ou deux commandes identiques.

- [ ] **MATRIX-NATIVE-001** — AI natif vs `elephant.ai`.
- [ ] **MATRIX-NATIVE-002** — Calendar natif vs `elephant.calendar`.
- [ ] **MATRIX-NATIVE-003** — Graph natif vs `elephant.graph`.
- [ ] **MATRIX-NATIVE-004** — Knowledge natif vs `elephant.knowledge`.
- [ ] **MATRIX-NATIVE-005** — Sync natif vs `elephant.sync`.
- [ ] **MATRIX-NATIVE-006** — Sites natif vs `elephant.sites`.
- [ ] **MATRIX-NATIVE-007** — Support wiki éditeur vs `elephant.wiki`.
- [ ] **MATRIX-NATIVE-008** — Code blocks éditeur vs `elephant.code-execution`.
- [ ] **MATRIX-NATIVE-009** — Aucun paramètre fantôme après désactivation de l'addon.
- [ ] **MATRIX-NATIVE-010** — Aucun comportement natif essentiel ne disparaît avec l'addon si ce n'est pas intentionnel.

## 4.6 Tous addons activés

- [ ] **MATRIX-ALL-001** — Installation complète depuis profil neuf.
- [ ] **MATRIX-ALL-002** — Résolution correcte de toutes les dépendances.
- [ ] **MATRIX-ALL-003** — Premier lancement sans crash/écran blanc.
- [ ] **MATRIX-ALL-004** — Inventaire exact des commandes, vues, paramètres et raccourcis.
- [ ] **MATRIX-ALL-005** — Aucun doublon.
- [ ] **MATRIX-ALL-006** — Rejouer tout le socle `CORE-*`.
- [ ] **MATRIX-ALL-007** — Mesurer temps de lancement, mémoire idle et CPU idle.
- [ ] **MATRIX-ALL-008** — Rafale de modifications externes.
- [ ] **MATRIX-ALL-009** — Déconnexion réseau.
- [ ] **MATRIX-ALL-010** — Crash d'un sidecar pendant une sauvegarde.
- [ ] **MATRIX-ALL-011** — Fermeture : aucun processus orphelin.
- [ ] **MATRIX-ALL-012** — Redémarrage : aucun état corrompu.
- [ ] **MATRIX-ALL-013** — Désactiver chaque addon un par un.
- [ ] **MATRIX-ALL-014** — Désinstaller chaque addon un par un.
- [ ] **MATRIX-ALL-015** — Vault final identique aux changements explicitement demandés, sans fichier parasite inconnu.

---

# 5. Matrice plateformes

## 5.1 Bazzite desktop — priorité de preuve

Tous les addons desktop doivent être testés sur l'exécutable final, pas seulement en développement.

- [ ] **PLAT-BAZ-001** — Bazzite stable, session Wayland réelle.
- [ ] **PLAT-BAZ-002** — AppImage final.
- [ ] **PLAT-BAZ-003** — Flatpak final si distribué.
- [ ] **PLAT-BAZ-004** — Portals fichiers et ouverture application par défaut.
- [ ] **PLAT-BAZ-005** — Drag-and-drop depuis Dolphin/Nautilus selon environnement ciblé.
- [ ] **PLAT-BAZ-006** — Accès aux vaults hors home selon sandbox/permissions.
- [ ] **PLAT-BAZ-007** — Sidecars exécutables et bibliothèques embarquées.
- [ ] **PLAT-BAZ-008** — NVIDIA/AMD/Intel sans dépendance GPU obligatoire non déclarée.
- [ ] **PLAT-BAZ-009** — Mise à l'échelle 100/125/150/200 %.
- [ ] **PLAT-BAZ-010** — Thèmes clair/sombre et décorations Wayland.
- [ ] **PLAT-BAZ-011** — Ouverture PDF par défaut via portail.
- [ ] **PLAT-BAZ-012** — Logs exploitables dans le terminal et le renderer.

## 5.2 Plateformes annoncées mais non prouvées par Bazzite

Ces cases ne peuvent pas être cochées par un test Linux desktop.

- [ ] **PLAT-MOBILE-001** — Wiki Android.
- [ ] **PLAT-MOBILE-002** — Wiki iOS.
- [ ] **PLAT-MOBILE-003** — Graph Android.
- [ ] **PLAT-MOBILE-004** — Graph iOS.
- [ ] **PLAT-MOBILE-005** — Knowledge Android.
- [ ] **PLAT-MOBILE-006** — Knowledge iOS.
- [ ] **PLAT-MOBILE-007** — Sync Android.
- [ ] **PLAT-MOBILE-008** — Sync iOS.
- [ ] **PLAT-MOBILE-009** — Calendar Android.
- [ ] **PLAT-MOBILE-010** — Calendar iOS.
- [ ] **PLAT-MOBILE-011** — Recently Edited Android.
- [ ] **PLAT-MOBILE-012** — Recently Edited iOS.
- [ ] **PLAT-MOBILE-013** — Vérifier que Codex, Sites, Code execution et Keep Import ne sont pas proposés sur mobile.

---

# 6. Automatisation minimale attendue

Chaque test manuel critique doit être doublé autant que possible par un test automatisé réel, tout en conservant la preuve manuelle de l'exécutable final.

- [ ] **AUTO-001** — Un fichier de spec indépendant par addon.
- [ ] **AUTO-002** — Fixtures de vault versionnées et immuables.
- [ ] **AUTO-003** — Profil temporaire indépendant par test.
- [ ] **AUTO-004** — Tests du catalogue/manifeste en CI.
- [ ] **AUTO-005** — Test bloquant la divergence de version Code execution.
- [ ] **AUTO-006** — Tests d'installation/activation/désactivation/désinstallation.
- [ ] **AUTO-007** — Tests de permissions négatifs.
- [ ] **AUTO-008** — Tests de dépendances négatifs.
- [ ] **AUTO-009** — Tests filesystem/watchers avec vraies opérations disque.
- [ ] **AUTO-010** — Tests autosave avec lecture réelle du fichier.
- [ ] **AUTO-011** — Tests drag-and-drop UI sur binaire packagé.
- [ ] **AUTO-012** — Tests sidecars sur binaire packagé.
- [ ] **AUTO-013** — Tests de crash et récupération.
- [ ] **AUTO-014** — Captures/logs/artifacts uniquement en cas d'échec ou preuve finale bornée.
- [ ] **AUTO-015** — Rétention courte et compression pour limiter le stockage GitHub Actions.
- [ ] **AUTO-016** — Aucun test « always pass », aucun `continue-on-error` pour les gates de preuve.
- [ ] **AUTO-017** — Pas de mock pour les parcours qui prétendent prouver l'exécutable final.
- [ ] **AUTO-018** — Matrice Bazzite/Wayland exécutée dans un environnement aussi proche que possible, puis validation sur machine Bazzite réelle.
- [ ] **AUTO-019** — Rapport machine lisible JSON/JUnit plus résumé Markdown léger.
- [ ] **AUTO-020** — Toute régression crée un artefact de reproduction minimal.

### Nommage recommandé des specs

```text
tests/addons/
  common/
    lifecycle.spec.*
    dependencies.spec.*
    permissions.spec.*
    filesystem.spec.*
    crash-isolation.spec.*
  dashboard.spec.*
  ai-base.spec.*
  ai-chat.spec.*
  ai-search.spec.*
  ai-ocr.spec.*
  wiki.spec.*
  graph.spec.*
  knowledge.spec.*
  open-models.spec.*
  codex-connection.spec.*
  sync.spec.*
  calendar.spec.*
  sites.spec.*
  code-execution.spec.*
  google-keep-import.spec.*
  recently-edited.spec.*
  all-addons.spec.*
```

---

# 7. Fiche de résultat obligatoire par addon

Copier cette fiche dans le dossier de preuves de chaque addon.

```markdown
## Addon

- ID :
- Version catalogue :
- Version manifeste :
- SHA package :
- SHA application :
- Plateforme :
- Format :
- Vault fixture :
- Dépendances :
- Permissions déclarées :
- Permissions réellement utilisées :
- Vues ajoutées :
- Commandes ajoutées :
- Paramètres ajoutés :
- Fichiers créés :
- Fichiers modifiés :
- Fichiers supprimés :
- Stockage hors vault :
- Processus/sidecars :
- Domaines réseau contactés :
- Nettoyage à la désactivation :
- Nettoyage à la désinstallation :

## Test

- ID :
- Préconditions :
- Étapes :
- Attendu :
- Observé :
- Statut :
- Logs :
- Capture/vidéo :
- Diff vault :
- Processus avant/après :
- Issue :
```

---

# 8. Critères de sortie « PROVEN » pour un addon

Un addon ne peut pas être déclaré **PROVEN** tant que toutes les conditions suivantes ne sont pas réunies :

- [ ] **EXIT-001** — Tous ses tests spécifiques sont `PASS` sur la build finale ciblée.
- [ ] **EXIT-002** — Tous les tests communs applicables sont `PASS`.
- [ ] **EXIT-003** — Le socle `CORE-*` passe avant et après activation.
- [ ] **EXIT-004** — Toutes ses dépendances et incompatibilités sont testées.
- [ ] **EXIT-005** — Toutes les permissions sont exercées en succès et en refus.
- [ ] **EXIT-006** — Tous les chemins/fichiers modifiés sont inventoriés.
- [ ] **EXIT-007** — Aucun changement hors contrat.
- [ ] **EXIT-008** — Aucun crash de l'application, écran blanc ou perte de données.
- [ ] **EXIT-009** — Aucun processus, listener, timer ou port orphelin.
- [ ] **EXIT-010** — Désactivation et désinstallation sont propres.
- [ ] **EXIT-011** — Mode hors ligne et erreurs réseau sont contrôlés si applicable.
- [ ] **EXIT-012** — Sidecar packagé et contrôlé si applicable.
- [ ] **EXIT-013** — Compatibilité Bazzite Wayland démontrée avec preuves.
- [ ] **EXIT-014** — Les plateformes annoncées ont chacune une preuve distincte ou sont retirées du manifeste.
- [ ] **EXIT-015** — Les conflits avec les modules natifs sont résolus et testés.
- [ ] **EXIT-016** — Le catalogue, le manifeste et le package ont exactement la même version.
- [ ] **EXIT-017** — Les preuves sont attachées et reproductibles.
- [ ] **EXIT-018** — Les tests ne reposent pas uniquement sur des mocks.
- [ ] **EXIT-019** — Les échecs intermittents sont à zéro sur plusieurs répétitions.
- [ ] **EXIT-020** — Une réinstallation propre reproduit le même résultat.

---

# 9. Bloqueurs déjà identifiés pendant l'inventaire

- [ ] **BLOCK-001** — Le lien historique `assistant/ci-tauri-pr-flow` n'est plus une branche disponible : figer le SHA exact de la build à certifier.
- [ ] **BLOCK-002** — Corriger la divergence `elephant.code-execution` : catalogue `2.2.1`, manifeste `2.2.0`.
- [ ] **BLOCK-003** — Confirmer le mécanisme/checksum réel du catalogue avant de considérer l'installation sûre.
- [ ] **BLOCK-004** — Confirmer le packaging réel des sidecars Codex, Sites et Code execution dans les formats Linux distribués.
- [ ] **BLOCK-005** — Décider la relation entre addons et modules natifs AI, Calendar, Graph, Knowledge, Sync et Sites.
- [ ] **BLOCK-006** — Définir la politique Sync pour `.elephant/knowledge`, `.elephant-site`, `.sync` et les stockages addon.
- [ ] **BLOCK-007** — Définir le contrat exact de l'addon PDF, absent de l'inventaire officiel actuel mais requis par le parcours de fichiers PDF.
- [ ] **BLOCK-008** — Définir le fallback Linux/portal pour ouvrir un PDF avec l'application système.
- [ ] **BLOCK-009** — Confirmer comment AI OCR insère un résultat sans permission `vault.write`, ou retirer cette attente.
- [ ] **BLOCK-010** — Confirmer le sandbox réel et les garanties de sécurité de Code execution.
- [ ] **BLOCK-011** — Confirmer la politique de conservation des données à la désinstallation pour les 16 addons.
- [ ] **BLOCK-012** — Exécuter les validations sur la vraie build Bazzite finale avant toute déclaration de fonctionnement.
