# Flutter App Container - Architecture v1.1.0

## Vue d'ensemble

L'application Flutter fournit une interface utilisateur multiplateforme pour l'éditeur de notes basé sur le core C validé. Elle sert de "container" gérant l'I/O, le stockage, et l'interface utilisateur, tandis que le core C gère l'analyse et la conversion des documents.

## Architecture

### Structure des dossiers

```
flutter/
├── lib/
│   ├── core/                    # Fonctionnalités core
│   │   ├── routing/            # Configuration go_router
│   │   ├── state/              # State management Riverpod
│   │   ├── storage/            # Abstraction stockage multiplateforme
│   │   └── editor/             # Interface vers C core
│   ├── features/               # Fonctionnalités par écran
│   │   ├── home/              # Écran d'accueil + liste notes
│   │   ├── viewer/            # Visualiseur de documents
│   │   └── settings/          # Paramètres
│   ├── shared/                # Composants partagés
│   │   ├── theme/             # Thème Material Design 3
│   │   ├── i18n/              # Internationalisation
│   │   └── widgets/           # Widgets réutilisables
│   ├── models/                # Modèles de données
│   ├── ffi/                   # Bindings FFI (desktop/mobile)
│   ├── wasm/                  # Bindings WASM (web)
│   └── main.dart              # Point d'entrée
```

### Couches architecturales

1. **Présentation** (UI/Widgets)
   - Screens: HomeScreen, ViewerScreen, SettingsScreen
   - Widgets réutilisables: DocumentRenderer, NoteCard
   - Thème sombre (#202124) Material Design 3

2. **Logique métier** (Providers/State)
   - Riverpod pour state management
   - AsyncNotifier pour operations async
   - Separation of concerns claire

3. **Données** (Storage/API)
   - StorageService abstrait (Desktop/Mobile/Web)
   - PlatformEditor (FFI/WASM selon plateforme)
   - Configuration persistante

4. **Core C** (Intégration native)
   - FFI pour desktop/mobile
   - WASM pour web
   - API uniforme via EditorApi

## Fonctionnalités implémentées

### ✅ Routing (go_router)
- `/home` - Liste des notes
- `/viewer?note=path` - Visualiseur de document
- `/settings` - Paramètres
- Navigation type-safe avec query parameters

### ✅ State Management (Riverpod)
- `appStateProvider` - Configuration app
- `notesProvider` - Liste des notes
- `noteDetailsProvider` - Détails note individuelle
- `uiStateProvider` - État UI (loading, errors, snackbars)

### ✅ Stockage multiplateforme
- **Desktop**: Système de fichiers avec atomic writes
- **Mobile**: App documents directory
- **Web**: IndexedDB via SharedPreferences

### ✅ Thème
- Dark theme avec background #202124
- Material Design 3 compliance
- Typography cohérente
- Color scheme adaptatif

### ✅ Internationalisation
- Support Français/Anglais
- AppLocalizations avec delegate
- Locale par défaut configurable

### ✅ Intégration C Core
- EditorApi abstrait uniforme
- FfiEditorApi pour FFI (natif)
- WasmEditorApi pour WASM (web)
- PlatformEditor factory automatique

## Modèles de données

### Document
Représentation complète d'un document avec:
- `elements[]` - Éléments de contenu (text, image, table)
- `meta` - Métadonnées (titre, dates, auteur)
- `styles[]` - Classes de style (highlight, underline)

### AppConfig
Configuration persistante:
- Préférences UI (police, taille, locale)
- Chemins de stockage
- Paramètres performance

## Intégration C Core

### Interface unifiée (EditorApi)
```dart
abstract class EditorApi {
  Future<EditorResult<Document>> parseMarkdown(String markdown);
  Future<EditorResult<String>> exportToMarkdown(Document doc);
  Future<EditorResult<String>> exportToJson(Document doc);
  Future<EditorResult<Document>> simulateEditor(List<String> chars);
}
```

### Implémentations spécifiques
- **FFI**: Communication directe avec libeditor.{a,so,dll}
- **WASM**: Interface JavaScript avec module WASM
- **Sélection automatique** selon la plateforme

## Tests et validation

### Structure de test
- Unit tests pour modèles et providers
- Widget tests pour composants UI
- Integration tests pour flows complets
- Golden tests pour validation visuelle

### Scripts de build
- `scripts/setup.sh` - Installation et configuration
- Support multiplateforme (web, desktop, mobile)
- CI/CD ready avec validation automatique

## Points d'extension

### Nouvelles fonctionnalités
1. **Synchronisation** - Cloud sync via providers
2. **Plugins** - Architecture extensible
3. **Collaboration** - Real-time editing
4. **Themes** - Multiple color schemes

### Optimisations
1. **Performance** - Lazy loading, virtualization
2. **Offline** - Cache intelligent
3. **Accessibilité** - Screen readers, navigation
4. **Analytics** - Usage tracking anonyme

## Standards de code

- **Dart**: Effective Dart guidelines
- **Architecture**: Clean Architecture + MVVM
- **State**: Riverpod best practices
- **UI**: Material Design 3 specifications
- **I18n**: ARB files pour traductions
- **Tests**: 80%+ coverage minimale

L'application est conçue comme un shell robuste et extensible autour du core C validé, respectant les principes de séparation des responsabilités et de multiplateforme first.