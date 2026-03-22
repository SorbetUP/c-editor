# 🔧 C Editor Engine Architecture

Ce dossier contient tous les moteurs C spécialisés créés comme librairies séparées pour le projet C-Editor. Chaque moteur est conçu pour être modulaire, réutilisable et intégrable dans différentes interfaces (TUI, GUI).

## 📋 Moteurs Disponibles

### 1. **Moteur Curseur** (`cursor/`)
- **Rôle** : Gestion avancée du curseur et navigation
- **Fonctionnalités** :
  - Navigation par mots (Ctrl+Left/Right)  
  - Détection de formatage Markdown
  - Gestion intelligente des sauts de ligne
  - Support des parenthèses et crochets
  - Duplication de lignes et indentation automatique
- **Statut** : ✅ Complet et testé
- **Librairie** : `libcursor.a`

### 2. **Moteur Markdown** (`markdown/`)
- **Rôle** : Parsing et génération Markdown avancé
- **Fonctionnalités** :
  - Parser inline styles (**bold**, *italic*, ==highlight==)
  - Support des tableaux et images
  - Validation de structure Markdown
  - Auto-formatage des listes et titres
  - Génération JSON à partir de Markdown
- **Statut** : ✅ Complet et testé
- **Librairie** : `libmarkdown.a`

### 3. **Moteur Éditeur** (`editor/`)
- **Rôle** : API principale pour l'édition de documents
- **Fonctionnalités** :
  - Structure Document unifiée
  - Interface ABI stable
  - Intégration avec les autres moteurs
- **Statut** : ✅ Complet et testé
- **Librairie** : `libeditor.a`

### 4. **Moteur de Recherche** (`search_engine/`) 🆕
- **Rôle** : Recherche avancée avec support futurs embeddings
- **Fonctionnalités** :
  - Indexation de documents avec hash table
  - Recherche floue (fuzzy search) avec score de pertinence
  - Support futur pour embeddings ML et recherche sémantique
  - Recherche regex (placeholder)
  - Mise en surbrillance des résultats
- **Statut** : ✅ Architecture complète, implémentation de base
- **Librairie** : `libsearch_engine.a`

### 5. **Moteur de Rendu** (`render_engine/`) ✅
- **Rôle** : Rendu web léger sans technologies web lourdes
- **Fonctionnalités** :
  - Rendu DOM-like sans WebKit/Chromium
  - Support multi-backend (macOS/Core Graphics, iOS, Android/Canvas, Linux/X11/Wayland, Windows/Direct2D/GDI)
  - Layout engine basique avec gestion d'éléments
  - Rendu de texte, images, rectangles avec transparence
  - Backend software pour compatibilité universelle
  - API unifiée cross-platform avec détection automatique
- **Statut** : ✅ Implémentation complète pour toutes les plateformes
- **Librairie** : `librender_engine.a` (87KB)

### 6. **Moteur de Sauvegarde** (`backup_engine/`) 🆕
- **Rôle** : Sauvegarde automatique intelligente
- **Fonctionnalités** :
  - Stratégies multiples (immediate, timed, idle, smart)
  - Support multi-destination (local, réseau, cloud)
  - Compression et chiffrement optionnels
  - Versioning Git-like avec historique
  - Restauration sélective par timestamp/version
- **Statut** : 📋 Interface complète, implémentation à venir
- **Librairie** : `libbackup_engine.a`

### 7. **Moteur Cryptographique** (`crypto_engine/`) 🆕
- **Rôle** : Chiffrement et hachage sécurisé pour le cloud
- **Fonctionnalités** :
  - Chiffrement AES-256-GCM, ChaCha20-Poly1305
  - Hachage SHA-256, SHA-512, BLAKE2B
  - Dérivation de clés PBKDF2, Scrypt, Argon2
  - Notes sécurisées pour stockage cloud non-sécurisé
  - Génération de mots de passe forts
- **Statut** : ✅ Démo fonctionnelle, expansion prévue avec vraies libs crypto
- **Librairie** : `libcrypto_engine.a`

### 8. **Moteur Vault Core** (`vault_core/`) 🆕
- **Rôle** : Définition et validation du vault utilisateur
- **Fonctionnalités** :
  - Validation d'un chemin absolu de vault
  - Sauvegarde du chemin racine courant
  - Détection simple des fichiers note Markdown
- **Statut** : ✅ Fondation compilée et testée
- **Librairie** : `libvault_core.a`

### 9. **Moteur Link Engine** (`link_engine/`) 🆕
- **Rôle** : Détection des liens entre notes
- **Fonctionnalités** :
  - Extraction de wikilinks `[[Note]]`
  - Base pour backlinks et graphe de notes
- **Statut** : ✅ Fondation compilée et testée
- **Librairie** : `liblink_engine.a`

### 10. **Moteur Privacy Engine** (`privacy_engine/`) 🆕
- **Rôle** : Confidentialité légère des previews
- **Fonctionnalités** :
  - Détection des notes sensibles via `#credentials`
  - Masquage des previews/listes
- **Statut** : ✅ Fondation compilée et testée
- **Librairie** : `libprivacy_engine.a`

### 11. **Moteur Sync Engine** (`sync_engine/`) 🆕
- **Rôle** : Configuration de sync et arbitrage latest-wins
- **Fonctionnalités** :
  - Modes `local`, `web`, `sync`
  - Validation endpoint distant
  - Résolution simple des versions les plus récentes
- **Statut** : ✅ Fondation compilée et testée
- **Librairie** : `libsync_engine.a`

### 12. **Moteur Render Extensions** (`render_ext/`) 🆕
- **Rôle** : Catalogue des extensions Markdown rendables
- **Fonctionnalités** :
  - Détection de blocs fenced `mermaid`, `markmap`, `katex`, `graphviz`, `echarts`, `flashcard`
  - Base pour futur rendu enrichi
- **Statut** : ✅ Fondation compilée et testée
- **Librairie** : `librender_ext.a`

## 🏗️ Architecture

```
c-editor/engines/
├── cursor/           # Navigation et gestion curseur
├── markdown/         # Parsing et génération Markdown  
├── editor/          # API principale édition
├── search_engine/   # Recherche avancée + ML
├── render_engine/   # Rendu léger sans web tech
├── backup_engine/   # Sauvegarde automatique
├── crypto_engine/   # Chiffrement pour cloud
├── vault_core/      # Sélection et validation du vault
├── link_engine/     # Liens entre notes
├── privacy_engine/  # Notes sensibles / masquage
├── sync_engine/     # Sync latest-wins et modes
├── render_ext/      # Extensions Markdown enrichies
└── Makefile         # Build system unifié
```

## 🚀 Compilation

```bash
# Compiler tous les moteurs
make engines

# Compiler un moteur spécifique
make search_engine
make crypto_engine

# Nettoyer tout
make clean

# Statut de tous les moteurs
make status
```

## 🎯 Utilisation Future

Ces moteurs sont conçus pour être intégrés dans :

1. **TUI Editor** - Interface terminal avancée
2. **GUI Editor** - Interface graphique (Flutter/native)
3. **Web Assembly** - Version web du projet
4. **API Server** - Service de traitement de documents

## 🔐 Sécurité

Le moteur cryptographique permet de :
- Sauvegarder des notes chiffrées sur des services non-sécurisés
- Hasher les titres pour l'identification sans révéler le contenu
- Utiliser des services cloud publics en toute sécurité

## 🔍 Recherche Avancée

Le moteur de recherche prépare le terrain pour :
- Intégration de modèles d'embedding (sentence-transformers)
- Recherche sémantique intelligente
- Classification automatique de documents
- Recommandations de contenu similaire

## 📦 Extensibilité

Chaque moteur peut être étendu indépendamment :
- Ajout de nouveaux algorithmes crypto
- Support de nouveaux formats dans le moteur Markdown
- Nouveaux backends de rendu
- Stratégies de backup personnalisées

Cette architecture modulaire garantit la maintenabilité et permet l'évolution progressive de chaque composant selon les besoins.
