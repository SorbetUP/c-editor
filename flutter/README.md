# Flutter Note Editor - v1.1.0 App Container

Application Flutter multiplateforme pour l'édition de notes, utilisant le core C validé pour le traitement des documents.

## 🎯 Objectif

Cette application sert de "container" intelligent qui:
- ✅ Fournit une interface utilisateur native sur toutes les plateformes
- ✅ Gère les opérations d'I/O et le stockage multiplateforme  
- ✅ Intègre le core C validé via FFI (natif) et WASM (web)
- ✅ Offre une expérience utilisateur cohérente et performante

## 🏗️ Architecture

### Séparation des responsabilités
- **Core C**: Parsing markdown, conversion JSON, canonicalisation
- **Flutter App**: Interface utilisateur, stockage, navigation, I/O
- **Platform Bindings**: FFI (desktop/mobile) + WASM (web)

### Stack technique
- **Flutter 3.35+** - Framework UI multiplateforme
- **Riverpod** - State management avec code generation
- **go_router** - Routing déclaratif type-safe
- **Material Design 3** - Thème sombre (#202124)
- **i18n** - Support français/anglais

## 🚀 Installation rapide

```bash
# 1. Installer Flutter (si pas déjà fait)
brew install --cask flutter

# 2. Setup du projet
cd flutter
chmod +x scripts/setup.sh
./scripts/setup.sh

# 3. Lancer l'app
flutter run -d chrome    # Web
flutter run -d macos     # Desktop macOS
```

## 📱 Plateformes supportées

| Plateforme | Statut | Integration | Notes |
|------------|--------|-------------|-------|
| Web | ✅ | WASM | Production ready |
| macOS | ✅ | FFI | Production ready |
| Windows | ✅ | FFI | Production ready |
| Linux | ✅ | FFI | Production ready |
| iOS | 🔄 | FFI | En développement |
| Android | 🔄 | FFI | En développement |

## 🎨 Fonctionnalités

### Interface utilisateur
- **Thème sombre** (#202124) Material Design 3
- **Navigation fluide** avec go_router
- **Responsive design** adaptatif
- **Internationalisation** français/anglais

### Gestion des notes
- **Liste des notes** avec aperçu et métadonnées
- **Visualiseur riche** avec support spans, images, tableaux
- **Import/Export** Markdown et JSON
- **Stockage multiplateforme** (filesystem/app dir/IndexedDB)

### Intégration C Core
- **API unifiée** EditorApi abstrait
- **Binding automatique** selon la plateforme
- **Gestion d'erreurs** robuste avec EditorResult<T>
- **Performance optimale** grâce au core C validé

## 📋 Structure du projet

```
flutter/
├── lib/
│   ├── core/                    # Logique core application
│   │   ├── routing/            # Configuration go_router  
│   │   ├── state/              # State management Riverpod
│   │   ├── storage/            # Stockage multiplateforme
│   │   └── editor/             # Intégration core C
│   ├── features/               # Modules par fonctionnalité
│   │   ├── home/              # Écran liste des notes
│   │   ├── viewer/            # Visualiseur de documents
│   │   └── settings/          # Paramètres application
│   ├── models/                # Modèles de données
│   ├── shared/                # Composants partagés
│   │   ├── theme/             # Thème Material Design
│   │   ├── i18n/              # Internationalisation
│   │   └── widgets/           # Widgets réutilisables
│   ├── ffi/                   # Bindings FFI (desktop/mobile)
│   ├── wasm/                  # Bindings WASM (web)
│   └── main.dart              # Point d'entrée
├── scripts/                   # Scripts de build et setup
├── ARCHITECTURE.md            # Documentation architecture
└── README.md                  # Ce fichier
```

## 🔧 Développement

### State Management avec Riverpod
```dart
@riverpod
class Notes extends _$Notes {
  @override
  FutureOr<List<String>> build() async {
    final storage = ref.read(storageServiceProvider);
    return await storage.listNotes();
  }
}
```

### Intégration Platform Editor
```dart
final editor = PlatformEditor.instance;
await editor.initialize();

final result = await editor.parseMarkdown(markdown);
if (result.isSuccess) {
  final document = result.data!;
  // Utiliser le document...
}
```

### Thématisation
```dart
MaterialApp.router(
  theme: AppTheme.darkTheme(), // #202124 background
  routerConfig: AppRouter.router,
);
```

## 🧪 Tests et qualité

```bash
# Tests unitaires
flutter test

# Analyse statique
flutter analyze

# Tests d'intégration
flutter drive --target=test_driver/app.dart

# Coverage
flutter test --coverage
```

## 📦 Build et déploiement

```bash
# Web
flutter build web --release

# Desktop macOS
flutter build macos --release

# Desktop Windows
flutter build windows --release

# Desktop Linux  
flutter build linux --release
```

## 🎯 Roadmap v1.1.x

- [ ] **Tests E2E** complets avec golden tests
- [ ] **CI/CD** automatisé avec validation multiplateforme
- [ ] **Performance** optimisations (lazy loading, virtualization)
- [ ] **Mobile** finalisation iOS/Android
- [ ] **Plugins** architecture extensible

## 📄 Livrables v1.1.0

✅ **App container Flutter** multiplateforme fonctionnel  
✅ **Intégration core C** via FFI et WASM  
✅ **Interface utilisateur** Material Design 3  
✅ **Stockage multiplateforme** avec abstraction  
✅ **Routing et navigation** type-safe  
✅ **Internationalisation** français/anglais  
✅ **Documentation** architecture et utilisation

## 🤝 Contribution

1. Respecter les guidelines Dart/Flutter
2. Utiliser Riverpod pour le state management
3. Écrire des tests pour les nouvelles fonctionnalités
4. Assurer la compatibilité multiplateforme
5. Documenter les changements d'API

---

**Status**: ✅ v1.1.0-app-shell **COMPLETE**  
**Intégration**: Core C v1.0.1-tests validé  
**Plateformes**: Web, macOS, Windows, Linux ready